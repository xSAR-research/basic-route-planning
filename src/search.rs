//! Bounded, deterministic offline shortest-route search over the agreed rules.
//!
//! A* uses the rules module's admissible heuristic and conservative pruning.
//! Dijkstra disables both, providing a reference search on manageable deals.
//! Card identities remain in the borrowed complete deal. Immutable predecessor
//! records keep reopened states from changing previously discovered routes.
//!
//! The memory limit covers a conservative estimate of collection capacities and
//! route reconstruction, with an additional per-record cap. It is not an OS RSS
//! guarantee: allocator bookkeeping, stack storage and the caller's deal are not
//! measured. Every search-owned growing collection uses fallible reservation.

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    mem::size_of,
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
};

use xsar::games::pyramid::CompleteDeal;

use crate::{
    parameters::{
        MAX_SUCCESSORS, RecordedAction, SearchLimits, SearchMethod, SearchOutcome, SearchState,
        SearchStats, SearchStop, TABLEAU_LEN,
    },
    rules,
};


/// Conservative aggregate bytes allowed per immutable record before growth.
///
/// This caps retained records independently of unique states, including reopened
/// states and stale frontier entries. Capacity-based accounting below provides
/// a second guard and reserves room for the reconstructed action collection.
const MEMORY_BYTES_PER_RECORD: usize = 1024;


/// Smallest useful collection reservation, in elements rather than bytes.
const MIN_COLLECTION_CAPACITY: usize = 4;


/// Allowance for hash bucket rounding, control bytes and spare table capacity.
///
/// The standard map exposes usable element capacity rather than allocated bytes.
/// This deliberately generous factor is an estimate, not an allocator contract.
const HASH_MEMORY_FACTOR: usize = 4;


/// One immutable history record, retained even after its state is reopened.
#[derive(Clone, Copy, Debug)]
struct Node {
    /// Rule state reached by the operation stored in this record.
    state: SearchState,
    /// Earlier arena record on this particular route, absent at the root.
    parent: Option<usize>,
    /// Operation from the parent, absent only for the initial state.
    action: Option<RecordedAction>,
    /// Unit-cost operations from the initial state to this record.
    depth: usize,
}


/// A heap entry ordered by lowest f, highest g, then earliest insertion.
#[derive(Clone, Copy, Debug)]
struct FrontierEntry {
    /// Immutable predecessor-arena index, also the stable insertion serial.
    node: usize,
    /// A* estimated total cost, or Dijkstra's known cost.
    priority: usize,
    /// Known unit-cost route length used to break equal-priority ties.
    depth: usize,
}


impl PartialEq for FrontierEntry {
    /// Compare exactly the fields participating in the total heap ordering.
    fn eq(&self, other: &Self) -> bool {
        (self.priority, self.depth, self.node) == (other.priority, other.depth, other.node)
    }
}


impl Eq for FrontierEntry {}


impl Ord for FrontierEntry {
    /// Reverse f and insertion order because BinaryHeap pops the greatest item.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.depth.cmp(&other.depth))
            .then_with(|| other.node.cmp(&self.node))
    }
}


impl PartialOrd for FrontierEntry {
    /// Delegate partial ordering to the deterministic total ordering.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


/// Allocation-free graph interface, permitting independent engine regression tests.
trait SearchGraph {
    /// Whether this state has reached the graph's success condition.
    fn is_goal(&self, state: SearchState) -> bool;

    /// Generate every legal unit-cost edge without allocating a temporary vector.
    fn successors(
        &self,
        state: SearchState,
    ) -> [Option<(RecordedAction, SearchState)>; MAX_SUCCESSORS];

    /// Return a lower bound on additional unit-cost operations to the goal.
    fn heuristic(&self, state: SearchState) -> u32;

    /// Prove that this state cannot reach a goal, conservatively returning false.
    fn impossible(&self, state: SearchState) -> bool;
}


/// Rule adapter borrowing the sole immutable source of card identities.
struct PyramidGraph<'a> {
    /// Validated original deal, shared by all states without cloning its cards.
    deal: &'a CompleteDeal,
    /// Per-original-tableau-card eligible complements, excluding covering pairs.
    complement_masks: [u64; TABLEAU_LEN],
}


impl SearchGraph for PyramidGraph<'_> {
    /// Tableau clearance is the agreed board-level goal.
    fn is_goal(&self, state: SearchState) -> bool {
        state.is_goal()
    }


    /// Delegate all legal move generation to the pure transition model.
    fn successors(
        &self,
        state: SearchState,
    ) -> [Option<(RecordedAction, SearchState)>; MAX_SUCCESSORS] {
        rules::successors(self.deal, state)
    }


    /// Use the complementary-rank lower bound on remaining removal operations.
    fn heuristic(&self, state: SearchState) -> u32 {
        rules::heuristic(self.deal, state)
    }


    /// Delegate conservative missing-complement detection to the rules module.
    fn impossible(&self, state: SearchState) -> bool {
        rules::impossible_with_masks(state, &self.complement_masks)
    }
}


/// Owned bounded search collections and counters for one synchronous invocation.
struct Engine<'a> {
    /// Immutable state and predecessor records, including reopened histories.
    nodes: Vec<Node>,
    /// Pending records; stale entries are skipped after a lower cost is found.
    frontier: BinaryHeap<FrontierEntry>,
    /// Lowest discovered unit cost for each distinct rule state.
    best: HashMap<SearchState, usize>,
    /// Caller-supplied resource limits, unchanged throughout the invocation.
    limits: SearchLimits,
    /// Caller-owned flag, observed before work and between successors.
    cancel: &'a AtomicBool,
    /// Start of this invocation's elapsed-time budget.
    started: Instant,
    /// Accumulated counters, with elapsed time refreshed when reporting.
    stats: SearchStats,
}


impl<'a> Engine<'a> {
    /// Create empty collections without performing a heap allocation.
    fn new(limits: SearchLimits, cancel: &'a AtomicBool) -> Self {
        Self {
            nodes: Vec::new(),
            frontier: BinaryHeap::new(),
            best: HashMap::new(),
            limits,
            cancel,
            started: Instant::now(),
            stats: SearchStats {
                expanded: 0,
                generated: 0,
                states: 0,
                peak_frontier: 0,
                elapsed: std::time::Duration::ZERO,
            },
        }
    }


    /// Give cancellation precedence and stop once the elapsed budget is spent.
    fn poll(&self) -> Result<(), SearchStop> {
        if self.cancel.load(AtomicOrdering::Relaxed) {
            return Err(SearchStop::Cancelled);
        }

        if self.started.elapsed() >= self.limits.max_duration {
            return Err(SearchStop::TimeLimit);
        }

        Ok(())
    }


    /// Copy counters and measure elapsed time at the reported stopping point.
    fn stats(&self) -> SearchStats {
        SearchStats {
            expanded: self.stats.expanded,
            generated: self.stats.generated,
            states: self.nodes.len(),
            peak_frontier: self.stats.peak_frontier,
            elapsed: self.started.elapsed(),
        }
    }


    /// Report a budget, cancellation, allocation or invariant stop without proof.
    fn stopped(&self, reason: SearchStop) -> SearchOutcome {
        SearchOutcome::Stopped {
            reason,
            stats: self.stats(),
        }
    }


    /// Reserve space for one history record, frontier entry and optional new key.
    ///
    /// Capacity growth is estimated before allocation and checked afterwards.
    /// A standard-library capacity rounding change may therefore stop conservatively
    /// after a reservation; the memory estimate is not a hard RSS or peak guarantee.
    fn reserve_insert(&mut self, new_state: bool) -> Result<(), SearchStop> {
        self.poll()?;
        if self.nodes.len() >= self.limits.max_states {
            return Err(SearchStop::StateLimit);
        }

        let record_cap = self.limits.max_memory_bytes / MEMORY_BYTES_PER_RECORD;
        if self.nodes.len() >= record_cap {
            return Err(SearchStop::MemoryLimit);
        }

        let node_target = growth_target(self.nodes.len(), self.nodes.capacity(), record_cap);
        let frontier_target =
            growth_target(self.frontier.len(), self.frontier.capacity(), record_cap);
        let map_target = if new_state {
            growth_target(self.best.len(), self.best.capacity(), record_cap)
        } else {
            self.best.capacity()
        };
        let map_estimate = if map_target > self.best.capacity() {
            map_target.saturating_mul(2).saturating_add(1)
        } else {
            self.best.capacity()
        };

        if estimated_bytes(node_target, frontier_target, map_estimate)
            > self.limits.max_memory_bytes
        {
            return Err(SearchStop::MemoryLimit);
        }

        if node_target > self.nodes.capacity() {
            self.nodes
                .try_reserve_exact(node_target.saturating_sub(self.nodes.len()))
                .map_err(|_| SearchStop::AllocationFailure)?;
        }

        if frontier_target > self.frontier.capacity() {
            self.frontier
                .try_reserve_exact(frontier_target.saturating_sub(self.frontier.len()))
                .map_err(|_| SearchStop::AllocationFailure)?;
        }

        if map_target > self.best.capacity() {
            self.best
                .try_reserve(map_target.saturating_sub(self.best.len()))
                .map_err(|_| SearchStop::AllocationFailure)?;
        }

        if estimated_bytes(
            self.nodes.capacity(),
            self.frontier.capacity(),
            self.best.capacity(),
        ) > self.limits.max_memory_bytes
        {
            return Err(SearchStop::MemoryLimit);
        }

        self.poll()
    }


    /// Insert a previously checked improvement without invalidating old parents.
    fn insert(&mut self, node: Node, heuristic: u32) -> Result<(), SearchStop> {
        let new_state = !self.best.contains_key(&node.state);
        self.reserve_insert(new_state)?;
        let node_index = self.nodes.len();
        let priority = node.depth.saturating_add(heuristic as usize);
        self.best.insert(node.state, node.depth);
        self.nodes.push(node);
        self.frontier.push(FrontierEntry {
            node: node_index,
            priority,
            depth: node.depth,
        });
        self.stats.peak_frontier = self.stats.peak_frontier.max(self.frontier.len());
        Ok(())
    }


    /// Reconstruct a bounded route through immutable, strictly earlier records.
    ///
    /// Each parent must precede its child and have depth exactly one lower.
    /// Missing or inconsistent records produce InternalInvariant, never a route.
    fn route(&self, goal_index: usize) -> Result<Box<[RecordedAction]>, SearchStop> {
        self.poll()?;
        let goal = self
            .nodes
            .get(goal_index)
            .ok_or(SearchStop::InternalInvariant)?;
        let mut actions = Vec::new();
        actions
            .try_reserve_exact(goal.depth)
            .map_err(|_| SearchStop::AllocationFailure)?;
        let extra_action_capacity = actions.capacity().saturating_sub(self.nodes.capacity());
        let allocated_bytes = estimated_bytes(
            self.nodes.capacity(),
            self.frontier.capacity(),
            self.best.capacity(),
        )
        .saturating_add(extra_action_capacity.saturating_mul(size_of::<RecordedAction>()));
        if allocated_bytes > self.limits.max_memory_bytes {
            return Err(SearchStop::MemoryLimit);
        }

        let mut index = goal_index;
        let mut depth = goal.depth;
        loop {
            self.poll()?;
            let node = self.nodes.get(index).ok_or(SearchStop::InternalInvariant)?;
            if node.depth != depth {
                return Err(SearchStop::InternalInvariant);
            }

            match (node.parent, node.action) {
                (Some(parent), Some(action)) if parent < index && depth > 0 => {
                    actions.push(action);
                    index = parent;
                    depth -= 1;
                }
                (None, None) if depth == 0 => break,
                _ => return Err(SearchStop::InternalInvariant),
            }
        }

        if actions.len() != goal.depth {
            return Err(SearchStop::InternalInvariant);
        }

        actions.reverse();
        Ok(actions.into_boxed_slice())
    }
}


/// Compute geometric growth bounded by the conservative retained-record cap.
fn growth_target(length: usize, capacity: usize, maximum: usize) -> usize {
    if length < capacity {
        capacity
    } else {
        capacity
            .saturating_mul(2)
            .max(MIN_COLLECTION_CAPACITY)
            .min(maximum)
    }
}


/// Estimate allocated collections plus enough actions for the longest arena path.
///
/// Saturating arithmetic treats an unrepresentable estimate as maximally large.
fn estimated_bytes(nodes: usize, frontier: usize, states: usize) -> usize {
    nodes
        .saturating_mul(size_of::<Node>().saturating_add(size_of::<RecordedAction>()))
        .saturating_add(frontier.saturating_mul(size_of::<FrontierEntry>()))
        .saturating_add(
            states
                .saturating_mul(size_of::<(SearchState, usize)>().saturating_add(1))
                .saturating_mul(HASH_MEMORY_FACTOR),
        )
}


/// Search a pure unit-cost graph, returning proof only after an uncut search.
fn search_graph(
    graph: &impl SearchGraph,
    initial: SearchState,
    limits: SearchLimits,
    cancel: &AtomicBool,
    method: SearchMethod,
) -> SearchOutcome {
    let mut engine = Engine::new(limits, cancel);
    if let Err(reason) = engine.poll() {
        return engine.stopped(reason);
    }

    let use_astar = matches!(method, SearchMethod::AStar);
    if graph.is_goal(initial) {
        return SearchOutcome::Solved {
            actions: Box::new([]),
            stats: engine.stats(),
        };
    }

    if use_astar && graph.impossible(initial) {
        return SearchOutcome::Unsolvable {
            stats: engine.stats(),
        };
    }

    let heuristic = if use_astar {
        graph.heuristic(initial)
    } else {
        0
    };
    if let Err(reason) = engine.insert(
        Node {
            state: initial,
            parent: None,
            action: None,
            depth: 0,
        },
        heuristic,
    ) {
        return engine.stopped(reason);
    }

    loop {
        if let Err(reason) = engine.poll() {
            return engine.stopped(reason);
        }

        let Some(entry) = engine.frontier.pop() else {
            return SearchOutcome::Unsolvable {
                stats: engine.stats(),
            };
        };
        let Some(node) = engine.nodes.get(entry.node).copied() else {
            return engine.stopped(SearchStop::InternalInvariant);
        };
        if engine.best.get(&node.state) != Some(&node.depth) {
            continue;
        }

        if graph.is_goal(node.state) {
            return match engine.route(entry.node) {
                Ok(actions) => SearchOutcome::Solved {
                    actions,
                    stats: engine.stats(),
                },
                Err(reason) => engine.stopped(reason),
            };
        }

        if engine.stats.expanded >= engine.limits.max_expansions {
            return engine.stopped(SearchStop::ExpansionLimit);
        }

        engine.stats.expanded += 1;
        for (action, next) in graph.successors(node.state).into_iter().flatten() {
            if let Err(reason) = engine.poll() {
                return engine.stopped(reason);
            }

            engine.stats.generated = engine.stats.generated.saturating_add(1);
            if use_astar && graph.impossible(next) {
                continue;
            }

            let Some(depth) = node.depth.checked_add(1) else {
                return engine.stopped(SearchStop::InternalInvariant);
            };
            if engine.best.get(&next).is_some_and(|known| *known <= depth) {
                continue;
            }

            let heuristic = if use_astar { graph.heuristic(next) } else { 0 };
            if let Err(reason) = engine.insert(
                Node {
                    state: next,
                    parent: Some(entry.node),
                    action: Some(action),
                    depth,
                },
                heuristic,
            ) {
                return engine.stopped(reason);
            }
        }
    }
}


/// Find a minimum logical-operation route from a complete deal's initial state.
///
/// Every MATCH, KING, MOVE and RECYCLE costs one. A* uses the agreed admissible
/// heuristic and safe pruning; Dijkstra disables both to serve as a reference.
/// Cancellation is checked before allocation, between expansions and successors,
/// and during reconstruction. The caller may set the atomic flag from any thread.
///
/// A solved result is optimal for this model. Unsolvable means the eligible
/// frontier was exhausted or A* proved an initial missing-complement condition.
/// Resource cutoffs, cancellation and allocation failure are always Stopped;
/// no partial route is returned or claimed to prove that a deal is unsolvable.
///
/// Memory is a conservative collection-capacity estimate, not measured RSS.
/// It includes predecessor history, stale heap entries, hash capacity and route
/// output, plus a 1,024-byte-per-record cap. Limits may therefore stop earlier
/// than a precise allocator measurement would. This function performs no I/O,
/// guest operations, file changes or background work.
pub fn solve(
    deal: &CompleteDeal,
    limits: SearchLimits,
    cancel: &AtomicBool,
    method: SearchMethod,
) -> SearchOutcome {
    search_graph(
        &PyramidGraph {
            deal,
            complement_masks: rules::complement_masks(deal),
        },
        SearchState::initial(),
        limits,
        cancel,
        method,
    )
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;


    /// Tiny graph with optional misleading but admissible estimates and pruning.
    struct Graph {
        /// Directed unit-cost edges, each carrying a distinctive action.
        edges: Vec<(u64, u64, RecordedAction)>,
        /// Optional heuristic values by state number.
        estimates: Vec<(u64, u32)>,
        /// State number considered a goal.
        goal: u64,
        /// States to label impossible, for checking reference-search independence.
        pruned: Vec<u64>,
    }


    impl SearchGraph for Graph {
        /// Match the synthetic goal without interpreting Pyramid card bits.
        fn is_goal(&self, state: SearchState) -> bool {
            state.remaining == self.goal
        }


        /// Return synthetic edges in their explicit stable insertion order.
        fn successors(
            &self,
            state: SearchState,
        ) -> [Option<(RecordedAction, SearchState)>; MAX_SUCCESSORS] {
            let mut successors = [None; MAX_SUCCESSORS];
            for (slot, (_, next, action)) in successors.iter_mut().zip(
                self.edges
                    .iter()
                    .filter(|(from, _, _)| *from == state.remaining),
            ) {
                *slot = Some((*action, state_number(*next)));
            }

            successors
        }


        /// Return an explicitly supplied synthetic estimate or zero.
        fn heuristic(&self, state: SearchState) -> u32 {
            self.estimates
                .iter()
                .find_map(|(key, value)| (*key == state.remaining).then_some(*value))
                .unwrap_or(0)
        }


        /// Apply the test's pruning labels, even if intentionally wrong.
        fn impossible(&self, state: SearchState) -> bool {
            self.pruned.contains(&state.remaining)
        }
    }


    /// Create a synthetic search key without exercising the Pyramid rules.
    fn state_number(remaining: u64) -> SearchState {
        SearchState {
            remaining,
            stock_cursor: 0,
            recycles: 0,
        }
    }


    /// Give small graphs ample limits so only the property under test can stop.
    fn generous_limits() -> SearchLimits {
        SearchLimits {
            max_states: 1000,
            max_expansions: 1000,
            max_memory_bytes: 16 * 1024 * 1024,
            max_duration: Duration::from_secs(10),
        }
    }


    /// Build one unambiguous edge, optionally followed by an unreachable goal.
    fn single_edge(goal: u64) -> Graph {
        Graph {
            edges: vec![(1, 2, RecordedAction::Move)],
            estimates: Vec::new(),
            goal,
            pruned: Vec::new(),
        }
    }


    /// Extract a solved route and fail the test on any other search outcome.
    fn solved_actions(outcome: SearchOutcome) -> Box<[RecordedAction]> {
        match outcome {
            SearchOutcome::Solved { actions, .. } => actions,
            other => panic!("expected a solved graph, received {other:?}"),
        }
    }


    /// A pre-set cancellation flag takes precedence over every zero budget.
    #[test]
    fn pre_cancel_is_reported_before_zero_limits() {
        let limits = SearchLimits {
            max_states: 0,
            max_expansions: 0,
            max_memory_bytes: 0,
            max_duration: Duration::ZERO,
        };
        let result = search_graph(
            &single_edge(2),
            state_number(1),
            limits,
            &AtomicBool::new(true),
            SearchMethod::AStar,
        );
        assert!(matches!(
            result,
            SearchOutcome::Stopped {
                reason: SearchStop::Cancelled,
                ..
            }
        ));
    }


    /// Each zero budget stops distinctly, never manufacturing an unsolvable proof.
    #[test]
    fn zero_resource_limits_have_distinct_outcomes() {
        for (kind, reason) in [
            (0, SearchStop::StateLimit),
            (1, SearchStop::ExpansionLimit),
            (2, SearchStop::MemoryLimit),
            (3, SearchStop::TimeLimit),
        ] {
            let mut limits = generous_limits();
            match kind {
                0 => limits.max_states = 0,
                1 => limits.max_expansions = 0,
                2 => limits.max_memory_bytes = 0,
                _ => limits.max_duration = Duration::ZERO,
            }

            let result = search_graph(
                &single_edge(2),
                state_number(1),
                limits,
                &AtomicBool::new(false),
                SearchMethod::AStar,
            );
            assert!(
                matches!(result, SearchOutcome::Stopped { reason: actual, .. } if actual == reason)
            );
        }
    }


    /// A retained-record cutoff reached during insertion remains a stopped search.
    #[test]
    fn state_limit_during_successor_insertion_is_not_unsolvable() {
        let mut limits = generous_limits();
        limits.max_states = 1;
        let result = search_graph(
            &single_edge(2),
            state_number(1),
            limits,
            &AtomicBool::new(false),
            SearchMethod::AStar,
        );
        assert!(
            matches!(result, SearchOutcome::Stopped { reason: SearchStop::StateLimit, stats } if stats.states == 1)
        );
    }


    /// Only complete frontier exhaustion yields an unsolvable reference result.
    #[test]
    fn exhausted_frontier_is_proven_unsolvable() {
        let result = search_graph(
            &single_edge(3),
            state_number(1),
            generous_limits(),
            &AtomicBool::new(false),
            SearchMethod::Dijkstra,
        );
        assert!(
            matches!(result, SearchOutcome::Unsolvable { stats } if stats.expanded == 2 && stats.states == 2)
        );
    }


    /// Dijkstra ignores even intentionally false pruning and excessive estimates.
    #[test]
    fn reference_disables_pruning_and_heuristic() {
        let mut graph = single_edge(2);
        graph.pruned = vec![1, 2];
        graph.estimates = vec![(1, u32::MAX), (2, u32::MAX)];
        assert_eq!(
            &*solved_actions(search_graph(
                &graph,
                state_number(1),
                generous_limits(),
                &AtomicBool::new(false),
                SearchMethod::Dijkstra,
            )),
            &[RecordedAction::Move],
        );
    }


    /// A* and reference search agree on a graph requiring lower-cost reopening.
    #[test]
    fn reopening_retains_immutable_predecessor_chains() {
        let graph = Graph {
            edges: vec![
                (1, 2, RecordedAction::Move),
                (1, 5, RecordedAction::King(crate::parameters::Location::DL)),
                (2, 3, RecordedAction::Move),
                (3, 4, RecordedAction::Move),
                (5, 4, RecordedAction::Recycle),
                (4, 6, RecordedAction::Move),
                (6, 7, RecordedAction::Move),
                (7, 8, RecordedAction::Move),
                (8, 9, RecordedAction::Move),
            ],
            estimates: vec![(5, 4)],
            goal: 9,
            pruned: Vec::new(),
        };
        let expected = vec![
            RecordedAction::King(crate::parameters::Location::DL),
            RecordedAction::Recycle,
            RecordedAction::Move,
            RecordedAction::Move,
            RecordedAction::Move,
            RecordedAction::Move,
        ];
        for method in [SearchMethod::AStar, SearchMethod::Dijkstra] {
            let actions = solved_actions(search_graph(
                &graph,
                state_number(1),
                generous_limits(),
                &AtomicBool::new(false),
                method,
            ));
            assert_eq!(&*actions, expected.as_slice());
        }
    }


    /// Equal priorities retain successor order instead of depending on hash order.
    #[test]
    fn equal_cost_ties_are_deterministic() {
        let graph = Graph {
            edges: vec![
                (1, 2, RecordedAction::Move),
                (1, 3, RecordedAction::Recycle),
                (2, 4, RecordedAction::Move),
                (3, 4, RecordedAction::Recycle),
            ],
            estimates: Vec::new(),
            goal: 4,
            pruned: Vec::new(),
        };
        for _ in 0..3 {
            let route = solved_actions(search_graph(
                &graph,
                state_number(1),
                generous_limits(),
                &AtomicBool::new(false),
                SearchMethod::AStar,
            ));
            assert_eq!(&*route, &[RecordedAction::Move, RecordedAction::Move]);
        }
    }
}
