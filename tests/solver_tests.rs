//! Independent route legality and bounded solver checks through public APIs.
//!
//! The two licensed upstream decks retain their expected optimal logical costs.
//! The accepted recording is independently simulated against the original deal;
//! no test sends guest input or treats a search limit as unsolvability.

use std::{fs, path::PathBuf, sync::atomic::AtomicBool, time::Duration};

use basic_route_planning::{
    input::load_inputs,
    parameters::{
        Location, MAX_SOLUTION_RECORDS, RecordedAction, SearchLimits, SearchMethod, SearchOutcome,
        SearchStop, TableauLocation,
    },
    replay::replay,
    search::solve,
};
use xsar::games::pyramid::CompleteDeal;


/// Locate a package-owned input without relying on the current working directory.
fn package_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}


/// Decode one complete synthetic upstream deck through the authoritative codec.
fn upstream_deal(cost: usize) -> CompleteDeal {
    let path = package_path(&format!("tests/fixtures/upstream-{cost}.json"));
    let file = fs::File::open(path).expect("open licensed upstream fixture");
    xsar::pyramid_json::read_from(file).expect("decode complete upstream fixture")
}


/// Construct a checked tableau location from a known zero-based test index.
fn at(index: usize) -> Location {
    Location::Tableau(TableauLocation::from_index(index).expect("valid test index"))
}


/// Require a bounded search to prove a solution, preserving diagnostics on refusal.
fn solved_actions(deal: &CompleteDeal, method: SearchMethod) -> Box<[RecordedAction]> {
    match solve(
        deal,
        SearchLimits::default(),
        &AtomicBool::new(false),
        method,
    ) {
        SearchOutcome::Solved { actions, .. } => actions,
        other => panic!("expected an optimal solution: {other:?}"),
    }
}


/// Handwritten all-tableau route gives the 15-cost fixture an independent oracle.
fn minimal_tableau_route() -> Vec<RecordedAction> {
    let pairs = [
        (22, 23),
        (24, 25),
        (26, 27),
        (16, 17),
        (18, 19),
        (20, 21),
        (12, 13),
        (14, 15),
        (10, 11),
        (6, 7),
        (8, 9),
        (4, 5),
        (2, 3),
    ];
    let mut actions: Vec<_> = pairs
        .into_iter()
        .map(|(left, right)| RecordedAction::Match(at(left), at(right)))
        .collect();
    actions.push(RecordedAction::King(at(1)));
    actions.push(RecordedAction::King(at(0)));
    actions
}


/// Replay every accepted CSV action through explicit piles, retaining source bytes.
#[test]
fn accepted_recording_is_a_legal_complete_tableau_clear() {
    let deal_path = package_path("Evidence/pyramid-recon-20261018-v6-01.json");
    let csv_path = package_path("Evidence/pyramid-recon-20261018-v6-01-solver.csv");
    let json_before = fs::read(&deal_path).expect("read accepted deal");
    let csv_before = fs::read(&csv_path).expect("read accepted CSV");
    let input = load_inputs(&deal_path, Some(&csv_path)).expect("load matched evidence");
    let original = input.deal.clone();
    let actions: Vec<_> = input
        .solution
        .expect("read recorded solution")
        .steps
        .iter()
        .map(|step| step.action)
        .collect();
    let trace = replay(&input.deal, &actions, true).expect("replay accepted route");

    assert_eq!(trace.len(), 48);
    assert_eq!(trace.first().unwrap().dl_before, Some(28));
    assert_eq!(trace.first().unwrap().dr_before, None);
    assert_eq!(trace.first().unwrap().first, Some(26));
    assert_eq!(trace.first().unwrap().second, Some(27));
    assert_eq!(trace.last().unwrap().remaining_tableau, 0);
    assert_eq!(trace.last().unwrap().recycles, 1);
    assert_eq!(input.deal, original);
    assert_eq!(fs::read(deal_path).unwrap(), json_before);
    assert_eq!(fs::read(csv_path).unwrap(), csv_before);
}


/// Both adapted upstream fixtures retain their published minimum logical costs.
#[test]
fn astar_matches_upstream_15_and_44_operation_fixtures() {
    for expected in [15, 44] {
        let deal = upstream_deal(expected);
        let actions = solved_actions(&deal, SearchMethod::AStar);
        assert_eq!(actions.len(), expected);
        let trace = replay(&deal, &actions, true).expect("independently validate A* route");
        assert_eq!(trace.last().unwrap().remaining_tableau, 0);
    }
}


/// Zero-heuristic Dijkstra agrees with A* on the small minimum-tableau fixture.
#[test]
fn dijkstra_and_astar_agree_on_minimal_fixture() {
    let deal = upstream_deal(15);
    let astar = solved_actions(&deal, SearchMethod::AStar);
    let dijkstra = solved_actions(&deal, SearchMethod::Dijkstra);

    assert_eq!(astar.len(), 15);
    assert_eq!(dijkstra.len(), astar.len());
    replay(&deal, &dijkstra, true).expect("independent Dijkstra replay");
    let handwritten = minimal_tableau_route();
    assert_eq!(handwritten.len(), 15);
    replay(&deal, &handwritten, true).expect("independent handwritten clear");
}


/// Empty DR, wrong ranks, covered cards and repeated selections refuse in place.
#[test]
fn replay_refuses_unavailable_same_card_and_wrong_rank_actions() {
    let deal = upstream_deal(15);
    for (action, expected_reason) in [
        (RecordedAction::King(Location::DR), "DR is empty"),
        (RecordedAction::King(at(0)), "covered"),
        (RecordedAction::King(at(21)), "value 13"),
        (RecordedAction::Match(at(22), at(22)), "distinct"),
        (RecordedAction::Match(at(21), at(22)), "sum to 13"),
        (RecordedAction::Recycle, "empty DL"),
    ] {
        let error = replay(&deal, &[action], false).expect_err("refuse illegal action");
        assert_eq!(error.step, 1);
        assert!(error.reason.contains(expected_reason), "{error}");
    }
}


/// MATCH cannot use a newly uncovered parent as its simultaneous second selection.
#[test]
fn replay_checks_both_match_selections_before_either_removal() {
    let deal = upstream_deal(15);
    let prefix = [
        RecordedAction::Match(at(22), at(23)),
        RecordedAction::Match(at(24), at(25)),
        RecordedAction::Match(at(26), at(27)),
        RecordedAction::Match(at(16), at(17)),
        RecordedAction::Match(at(18), at(19)),
    ];
    let mut actions = prefix.to_vec();
    actions.push(RecordedAction::Match(at(21), at(15)));
    let error = replay(&deal, &actions, false).expect_err("parent still covered at selection");
    assert_eq!(error.step, 6);
    assert!(error.reason.contains("covered"));
}


/// Removal cannot reuse a former tableau card, even when its coordinate stays valid.
#[test]
fn replay_refuses_reusing_a_removed_tableau_card() {
    let deal = upstream_deal(15);
    let actions = [
        RecordedAction::Match(at(22), at(23)),
        RecordedAction::Match(at(22), at(23)),
    ];
    let error = replay(&deal, &actions, false).expect_err("card already removed");
    assert_eq!(error.step, 2);
    assert!(error.reason.contains("already been removed"));
}


/// Waste turnover restores first-encounter order and the third recycle is refused.
#[test]
fn replay_preserves_turnover_order_and_two_recycle_limit() {
    let deal = upstream_deal(15);
    let mut actions = vec![RecordedAction::Move; 24];
    actions.push(RecordedAction::Recycle);
    let trace = replay(&deal, &actions, false).expect("first turnover");
    assert_eq!(trace[0].first, Some(28));
    assert_eq!(trace[0].dl_after, Some(29));
    assert_eq!(trace[0].dr_after, Some(28));
    assert_eq!(trace[23].dl_after, None);
    assert_eq!(trace[23].dr_after, Some(51));
    assert_eq!(trace[24].dl_after, Some(28));
    assert_eq!(trace[24].dr_after, None);
    assert_eq!(trace[24].recycles, 1);

    actions.extend([RecordedAction::Move; 24]);
    actions.push(RecordedAction::Recycle);
    let trace = replay(&deal, &actions, false).expect("second turnover");
    assert_eq!(trace.last().unwrap().dl_after, Some(28));
    assert_eq!(trace.last().unwrap().recycles, 2);
    actions.extend([RecordedAction::Move; 24]);
    actions.push(RecordedAction::Recycle);
    let error = replay(&deal, &actions, false).expect_err("third turnover refused");
    assert_eq!(error.step, actions.len());
    assert!(error.reason.contains("two-recycle"));
}


/// Removing pile tops reveals their previous/next cards and an empty waste cannot recycle.
#[test]
fn replay_removes_stock_and_waste_without_inventing_cards() {
    let deal = upstream_deal(15);
    let mut actions = Vec::new();
    for _ in 0..5 {
        actions.push(RecordedAction::Move);
        actions.push(RecordedAction::Match(Location::DL, Location::DR));
    }
    actions.push(RecordedAction::King(Location::DL));
    actions.extend([RecordedAction::Move; 6]);
    actions.extend([RecordedAction::Match(Location::DL, Location::DR); 6]);
    actions.push(RecordedAction::King(Location::DL));
    let trace = replay(&deal, &actions, false).expect("remove all 24 stock identities");

    assert_eq!(trace[1].first, Some(29));
    assert_eq!(trace[1].second, Some(28));
    assert_eq!(trace[1].dl_after, Some(30));
    assert_eq!(trace[1].dr_after, None);
    assert_eq!(trace.last().unwrap().dl_after, None);
    assert_eq!(trace.last().unwrap().dr_after, None);
    assert_eq!(trace.last().unwrap().remaining_tableau, 28);
    actions.push(RecordedAction::Recycle);
    let error = replay(&deal, &actions, false).expect_err("nothing remains to turn over");
    assert_eq!(error.step, actions.len());
    assert!(error.reason.contains("nonempty DR"));
}


/// A draw at exhausted stock refuses, and incomplete routes do not claim success.
#[test]
fn replay_distinguishes_exhausted_draw_partial_route_and_bounded_trace() {
    let deal = upstream_deal(15);
    let draws = vec![RecordedAction::Move; 25];
    let error = replay(&deal, &draws, false).expect_err("stock exhausted");
    assert_eq!(error.step, 25);
    assert!(error.reason.contains("nonempty DL"));
    assert!(replay(&deal, &[], false).unwrap().is_empty());
    assert!(replay(&deal, &[], true).is_err());
    assert!(replay(&deal, &[RecordedAction::Move], true).is_err());
    let excessive = vec![RecordedAction::Move; MAX_SOLUTION_RECORDS + 1];
    let error = replay(&deal, &excessive, false).expect_err("trace cap");
    assert_eq!(error.step, 0);
    assert!(error.reason.contains("exceeds"));
}


/// A pre-set atomic cancellation flag yields interruption rather than impossibility.
#[test]
fn cancellation_is_reported_without_an_unsolvable_claim() {
    let deal = upstream_deal(15);
    assert!(matches!(
        solve(
            &deal,
            SearchLimits::default(),
            &AtomicBool::new(true),
            SearchMethod::AStar,
        ),
        SearchOutcome::Stopped {
            reason: SearchStop::Cancelled,
            ..
        }
    ));
}


/// Explicit zero time and expansion budgets remain distinct interrupted outcomes.
#[test]
fn search_reports_resource_exhaustion_as_interruption() {
    let deal = upstream_deal(15);
    for limits in [
        SearchLimits {
            max_expansions: 0,
            ..SearchLimits::default()
        },
        SearchLimits {
            max_duration: Duration::ZERO,
            ..SearchLimits::default()
        },
    ] {
        assert!(matches!(
            solve(&deal, limits, &AtomicBool::new(false), SearchMethod::AStar),
            SearchOutcome::Stopped { .. }
        ));
    }
}
