//! Compact console domain types, shared limits, and one-based occupancy storage.
//!
//! The checked xSAR deal remains the owner of its fixed card arrays. Recorded
//! locations store only a checked flat tableau index or a pile marker; they do
//! not copy card identities. Mutable occupancy is stored separately from the
//! original deal, without authorising guest input or implementing stock movement.

use std::{fmt, io, path::PathBuf};

use thiserror::Error;

pub use xsar::games::pyramid::{STOCK_LEN, TABLEAU_LEN};

use crate::utility::{row_column_from_location, tableau_index};


/// Default offline search budget; an explicit CLI time limit overrides this value.
///
/// Preserve the established two-minute allowance, independently of app policy.
pub const DEFAULT_SEARCH_DURATION: std::time::Duration = std::time::Duration::from_secs(120);


/// Number of rows in a standard 28-card Pyramid tableau.
pub const TABLEAU_ROWS: usize = 7;


/// Character width of a compact tableau cell, including separation whitespace.
pub const TABLEAU_CELL_WIDTH: usize = 14;


/// Maximum CSV input size in bytes, bounding the console's input allocation.
pub const MAX_SOLUTION_BYTES: usize = 65_536;


/// Console safety cap on recorded operations, independent of game rules.
///
/// This permits recordings longer than the accepted 48-step example while
/// bounding sequence bookkeeping and the retained operation collection.
pub const MAX_SOLUTION_RECORDS: usize = 4096;


/// Exit code used for invalid or missing command-line arguments.
pub const EXIT_USAGE_ERROR: u8 = 2;


/// A checked tableau location stored as one flat index in `0..TABLEAU_LEN`.
///
/// The private byte retains the 28-position domain without storing redundant
/// row and column integers. External recording coordinates remain one-based.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableauLocation {
    /// Validated flat index in the original row-major tableau array.
    index: u8,
}


impl TableauLocation {
    /// Construct a location from a flat index, rejecting indices outside 0..28.
    pub fn from_index(index: usize) -> Option<Self> {
        if index >= TABLEAU_LEN {
            return None;
        }

        u8::try_from(index).ok().map(|index| Self { index })
    }


    /// Return the original row-major array index without changing its identity.
    pub const fn index(self) -> usize {
        self.index as usize
    }
}


/// A fixed tableau position or one of the two currently visible DRAW piles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Location {
    /// A position whose one-based row and column are validated before storage.
    Tableau(TableauLocation),
    /// The currently visible DRAW pile on the left of the tableau.
    DL,
    /// The currently visible DRAW pile on the right of the tableau.
    DR,
}


impl fmt::Display for Location {
    /// Print the CSV's one-based `row|column`, `DL`, or `DR` location notation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tableau(_) => {
                let (row, column) = row_column_from_location(*self).ok_or(fmt::Error)?;
                write!(formatter, "{row}|{column}")
            }
            Self::DL => formatter.write_str("DL"),
            Self::DR => formatter.write_str("DR"),
        }
    }
}


/// An owned 28-element triangular array with checked one-based access.
///
/// Rows and columns use one-based coordinates: row N contains columns 1..=N.
/// The position model uses this array for occupancy only; original card
/// identities remain in the separately owned, validated xSAR deal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriangularMatrix<T> {
    /// Sole fixed backing array, retaining its original row-major cell order.
    data: [T; TABLEAU_LEN],
}


impl<T> TriangularMatrix<T> {
    /// Move a complete fixed array into storage without allocating or cloning.
    pub const fn new(data: [T; TABLEAU_LEN]) -> Self {
        Self { data }
    }


    /// Borrow the element at `1 <= column <= row <= 7`, or return `None`.
    pub fn get(&self, row: usize, column: usize) -> Option<&T> {
        tableau_index(row, column).and_then(|index| self.data.get(index))
    }


    /// Mutably borrow a valid cell, rejecting bounds before index arithmetic.
    ///
    /// Returns `None` unless `1 <= column <= row <= 7`. Invalid and extreme
    /// coordinates leave every backing element unchanged.
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut T> {
        tableau_index(row, column).and_then(|index| self.data.get_mut(index))
    }


    /// Borrow the complete fixed array in its original row-major order.
    pub const fn as_array(&self) -> &[T; TABLEAU_LEN] {
        &self.data
    }
}


/// Whether a fixed original position is known to contain a card.
///
/// Card identities stay in the immutable xSAR deal. Unknown occupancy is distinct
/// from empty and remains blocking until the position is explicitly established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Occupancy {
    /// Presence is uncertain; it must not be treated as an empty position.
    Unknown,
    /// A card is known to remain in this fixed original position.
    Occupied,
    /// The position is known to be empty following an established removal.
    Empty,
}


/// Mutable occupancy associated with one immutable original Pyramid deal.
///
/// Fixed tableau coordinates and original stock ordinals retain their identity.
/// Stock occupancy does not establish DRAW pile ordering, tops, or recycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyramidPosition {
    /// Mutable presence knowledge for the 28 original tableau positions.
    pub(crate) tableau: TriangularMatrix<Occupancy>,
    /// Mutable presence knowledge for the 24 original stock ordinals.
    pub(crate) stock: [Occupancy; STOCK_LEN],
}


/// A refused checked occupancy access or removal, without a partial mutation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PositionError {
    /// The supplied one-based coordinates are outside the triangular tableau.
    #[error(
        "invalid tableau location {row}|{column}; expected 1 <= column <= row <= {TABLEAU_ROWS}"
    )]
    InvalidTableau {
        /// Rejected one-based row supplied by the caller.
        row: usize,
        /// Rejected one-based column supplied by the caller.
        column: usize,
    },
    /// At least one direct covering position remains occupied or unknown.
    #[error("tableau location {row}|{column} is covered")]
    Covered {
        /// One-based row of the blocked tableau position.
        row: usize,
        /// One-based column of the blocked tableau position.
        column: usize,
    },
    /// Removal requires known occupancy, but this position is empty or unknown.
    #[error("tableau location {row}|{column} is not known to be occupied")]
    TableauNotOccupied {
        /// One-based row of the refused tableau position.
        row: usize,
        /// One-based column of the refused tableau position.
        column: usize,
    },
    /// The supplied ordinal is outside the 24-card original stock array.
    #[error("invalid stock ordinal {ordinal}; expected an ordinal in 0..{STOCK_LEN}")]
    InvalidStock {
        /// Rejected zero-based original stock ordinal supplied by the caller.
        ordinal: usize,
    },
    /// Removal requires known occupancy, but this stock position is empty or unknown.
    #[error("stock ordinal {ordinal} is not known to be occupied")]
    StockNotOccupied {
        /// Zero-based original stock ordinal of the refused position.
        ordinal: usize,
    },
}


/// A recorded operation, retaining card selection order for two-card matches.
///
/// These values describe the manual recording; they do not establish that an
/// operation is legal against a changing stock, waste, or tableau state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordedAction {
    /// Select the first card and then the second card for a sum-of-13 match.
    Match(Location, Location),
    /// Remove the King at the recorded location with a single selection.
    King(Location),
    /// Press the D key to advance the DRAW piles.
    Move,
    /// Press the D key to recycle; kept distinct for recording clarity.
    Recycle,
}


impl RecordedAction {
    /// Return the canonical uppercase action label used in the CSV.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Match(_, _) => "MATCH",
            Self::King(_) => "KING",
            Self::Move => "MOVE",
            Self::Recycle => "RECYCLE",
        }
    }


    /// Return CARD1 and CARD2 without reordering the recorded selections.
    pub const fn locations(self) -> (Option<Location>, Option<Location>) {
        match self {
            Self::Match(first, second) => (Some(first), Some(second)),
            Self::King(location) => (Some(location), None),
            Self::Move | Self::Recycle => (None, None),
        }
    }


    /// Return a human-readable description without claiming route legality.
    pub const fn operation(self) -> &'static str {
        match self {
            Self::Match(_, _) => "Match CARD1 then CARD2 (sum 13)",
            Self::King(_) => "Remove the King at CARD1",
            Self::Move => "Press D",
            Self::Recycle => "Press D to recycle",
        }
    }
}


/// One structurally validated CSV row, identified by its play-order sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SolutionStep {
    /// One-based sequence number, independent of the row's original file order.
    pub sequence: u16,
    /// Parsed action with its required, validated fixed locations.
    pub action: RecordedAction,
}


/// A bounded recording stored in sequence order with no unused vector capacity.
#[derive(Debug, Eq, PartialEq)]
pub struct RecordedSolution {
    /// Validated operation rows ordered by their contiguous sequence numbers.
    pub steps: Box<[SolutionStep]>,
    /// Whether sequence ordering differs from the rows' original file order.
    pub reordered: bool,
}


/// Validated console request; filesystem paths retain their original OS spelling.
#[derive(Debug, Eq, PartialEq)]
pub enum CliAction {
    /// Display command-line usage without opening files.
    Help,
    /// Display the package version without opening files.
    Version,
    /// Calculate a minimum-cost route under the pinned offline rules profile.
    Solve {
        /// Complete immutable deal document.
        deal_path: PathBuf,
        /// Optional recorded comparison, independently replayed before search.
        solution_path: Option<PathBuf>,
        /// Explicit finite search limits.
        limits: SearchLimits,
    },
    /// Read the original deal and optional manual solution before presentation.
    Inspect {
        /// Explicit path to an xSAR Pyramid JSON document.
        deal_path: PathBuf,
        /// Optional explicit path to a recorded solution CSV.
        solution_path: Option<PathBuf>,
    },
}


/// Both validated inputs retained together until console presentation completes.
#[derive(Debug)]
pub struct ConsoleData {
    /// Sole owner of the immutable 28-card tableau and 24-card stock arrays.
    pub deal: xsar::games::pyramid::CompleteDeal,
    /// Mutable presence knowledge, kept separate from original card identities.
    pub position: PyramidPosition,
    /// Compact parsed actions, when a solution path was supplied.
    pub solution: Option<RecordedSolution>,
}


/// An invalid command-line request, reported with exit code two and usage.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ArgumentError(
    /// Human-readable reason why the supplied command-line request was refused.
    pub String,
);


/// Contextual file-loading failures; no partial input is presented on failure.
#[derive(Debug, Error)]
pub enum InputError {
    /// An explicit input path could not be opened for reading.
    #[error("failed to open '{path}': {source}")]
    Open {
        /// Input path supplied by the caller.
        path: PathBuf,
        /// Original operating-system open error.
        #[source]
        source: io::Error,
    },
    /// xSAR refused the complete deal or encountered a bounded read failure.
    #[error("failed to parse Pyramid deal from '{path}': {source}")]
    Deal {
        /// Input path supplied by the caller.
        path: PathBuf,
        /// Original checked codec error.
        #[source]
        source: xsar::pyramid_json::JsonError,
    },
    /// The manual CSV could not be read or structurally validated.
    #[error("failed to parse solution CSV from '{path}': {source}")]
    Solution {
        /// Input path supplied by the caller.
        path: PathBuf,
        /// Bounded CSV or recording-contract failure.
        #[source]
        source: SolutionError,
    },
}


/// Output or independent route-validation failure from the offline console.
#[derive(Debug, Error)]
pub enum ConsoleError {
    /// Report writing failed, including stdout devices and downstream pipes.
    #[error("failed to write to stdout: {0}")]
    Output(#[from] io::Error),
    /// A provided or calculated route failed the independent replay model.
    #[error("independent replay failed: {0}")]
    Replay(#[from] ReplayError),
}


/// CSV parsing and structural recording errors, separate from gameplay rules.
#[derive(Debug, Error)]
pub enum SolutionError {
    /// Reading the bounded input stream failed.
    #[error("solution CSV read failed: {0}")]
    Read(#[from] io::Error),
    /// Input exceeded the console-specific byte cap.
    #[error("solution CSV exceeds {MAX_SOLUTION_BYTES} bytes")]
    TooLarge,
    /// CSV decoding failed, including invalid UTF-8 or unequal field counts.
    #[error("invalid solution CSV: {0}")]
    Csv(#[from] csv::Error),
    /// The named columns or their order did not match the recording contract.
    #[error("expected CSV header SEQUENCE,ACTION,CARD1,CARD2")]
    Header,
    /// A source record failed sequence, action, field or coordinate checks.
    #[error("CSV line {line}: {reason}")]
    Record {
        /// One-based physical line at which the record starts.
        line: u64,
        /// Specific bounded structural failure description.
        reason: String,
    },
    /// The header was present but there were no recorded actions.
    #[error("solution CSV contains no recorded actions")]
    Empty,
    /// The recording exceeded the documented operation cap.
    #[error("solution CSV exceeds {MAX_SOLUTION_RECORDS} recorded actions")]
    TooMany,
}


/// Number of original card identities retained by the complete deal.
pub const DECK_LEN: usize = TABLEAU_LEN + STOCK_LEN;


/// Presence mask for all 52 original card identities.
pub const FULL_DECK_MASK: u64 = (1_u64 << DECK_LEN) - 1;


/// Presence mask for the 28 original tableau identities.
pub const TABLEAU_MASK: u64 = (1_u64 << TABLEAU_LEN) - 1;


/// Recycles permitted by the pinned upstream rules: three stock passes total.
pub const MAX_RECYCLES: u8 = 2;


/// Numeric total required to remove a pair, or the value of a removable King.
pub const PYRAMID_MATCH_TOTAL: u8 = 13;


/// Stack capacity exceeding nine visible cards' 36 pairs, nine Kings and one D.
pub const MAX_SUCCESSORS: usize = 64;


/// Canonical complete-knowledge search position; card identities remain in xSAR.
///
/// Bits 0..28 describe tableau presence and 28..52 original stock presence.
/// The stock cursor separates waste ordinals below it from undrawn ordinals.
/// Fields are internal so public callers cannot construct invalid cursors/masks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SearchState {
    /// Presence of each original card; bits above 51 are always zero.
    pub(crate) remaining: u64,
    /// First remaining undrawn stock ordinal, or 24 for exhausted stock.
    pub(crate) stock_cursor: u8,
    /// Completed waste-to-stock recycles, in 0..=2.
    pub(crate) recycles: u8,
}


/// A refused pure transition; its supplied state remains unchanged.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RuleError {
    /// A location cannot identify a card in this state.
    #[error("invalid card location")]
    InvalidLocation,
    /// A card is absent, covered, or selected twice.
    #[error("card unavailable, covered, or selected twice")]
    Unavailable,
    /// A King is not rank 13 or a pair does not total 13.
    #[error("King must be 13 and a pair must total 13")]
    WrongRank,
    /// Recycle was requested while undrawn stock remains.
    #[error("cannot recycle while stock remains")]
    StockNotEmpty,
    /// There is no waste to recycle or no stock to draw.
    #[error("requested DRAW pile is empty")]
    EmptyWaste,
    /// Two recycles have already been consumed.
    #[error("the two-recycle limit has been reached")]
    RecycleLimit,
}


/// Explicit resource limits for one synchronous search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchLimits {
    /// Maximum retained state records, including reopened paths.
    pub max_states: usize,
    /// Maximum expanded non-stale frontier entries.
    pub max_expansions: usize,
    /// Conservative collection-storage allowance in bytes, excluding process RSS.
    pub max_memory_bytes: usize,
    /// Cooperative wall-clock deadline measured from the start of search.
    pub max_duration: std::time::Duration,
}


impl Default for SearchLimits {
    /// Use bounded workstation defaults; callers may supply smaller/larger limits.
    fn default() -> Self {
        Self {
            max_states: 2_000_000,
            max_expansions: 2_000_000,
            max_memory_bytes: 512 * 1024 * 1024,
            max_duration: DEFAULT_SEARCH_DURATION,
        }
    }
}


/// Search ordering; Dijkstra independently disables heuristic guidance/pruning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMethod {
    /// Upstream A* with an admissible rank-count lower bound.
    AStar,
    /// Zero-heuristic reference minimum-cost search for manageable fixtures.
    Dijkstra,
}


/// Work actually completed by one bounded search.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SearchStats {
    /// States whose outgoing actions were examined.
    pub expanded: usize,
    /// Legal successor transitions generated, including previously seen states.
    pub generated: usize,
    /// Retained predecessor records, including any reopened paths.
    pub states: usize,
    /// Largest number of frontier entries retained at once.
    pub peak_frontier: usize,
    /// Elapsed synchronous search time, excluding input and report rendering.
    pub elapsed: std::time::Duration,
}


/// Reason search stopped without proving either a solution or impossibility.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SearchStop {
    /// Retained state/predecessor record limit reached.
    #[error("state limit")]
    StateLimit,
    /// Expanded-state limit reached.
    #[error("expansion limit")]
    ExpansionLimit,
    /// Conservative collection-memory allowance reached.
    #[error("memory limit")]
    MemoryLimit,
    /// Cooperative wall-clock deadline reached.
    #[error("time limit")]
    TimeLimit,
    /// The caller's atomic cancellation flag was set.
    #[error("cancelled")]
    Cancelled,
    /// A checked collection allocation failed.
    #[error("allocation failure")]
    AllocationFailure,
    /// A checked internal state/predecessor invariant failed.
    #[error("internal search invariant failure")]
    InternalInvariant,
}


/// Outcome preserving the distinction between proof and bounded interruption.
#[derive(Debug, Eq, PartialEq)]
pub enum SearchOutcome {
    /// A minimum-unit-cost path under the explicitly stated upstream profile.
    Solved {
        /// Typed actions from the original deal through tableau clearance.
        actions: Box<[RecordedAction]>,
        /// Actual work/time counters.
        stats: SearchStats,
    },
    /// Exhaustive search or sound impossible-state proof found no route.
    Unsolvable {
        /// Actual work/time counters.
        stats: SearchStats,
    },
    /// No claim of unsolvability or optimality follows from this interruption.
    Stopped {
        /// Specific bound/cancellation/allocation reason.
        reason: SearchStop,
        /// Actual work/time counters.
        stats: SearchStats,
    },
}


/// One independently replayed action with original identity indices for printing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayStep {
    /// Action retaining the exact ordered tableau/pile locations.
    pub action: RecordedAction,
    /// First selected card, or the drawn stock card, as original identity 0..52.
    pub first: Option<u8>,
    /// Second selected card for a pair, otherwise absent.
    pub second: Option<u8>,
    /// Visible left top before the action.
    pub dl_before: Option<u8>,
    /// Visible right top before the action.
    pub dr_before: Option<u8>,
    /// Visible left top after the action.
    pub dl_after: Option<u8>,
    /// Visible right top after the action.
    pub dr_after: Option<u8>,
    /// Number of occupied tableau slots after the action.
    pub remaining_tableau: u8,
    /// Recycles used after the action.
    pub recycles: u8,
}


/// Independent replay refusal with the first failing one-based action number.
#[derive(Debug, Error)]
#[error("route step {step}: {reason}")]
pub struct ReplayError {
    /// One-based action number, or zero for an invalid initial/final contract.
    pub step: usize,
    /// Specific failure from the independent model.
    pub reason: String,
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Keep the complete occupancy model to exactly one byte per original slot.
    #[test]
    fn position_occupancy_storage_is_compact() {
        assert_eq!(std::mem::size_of::<Occupancy>(), 1);
        assert_eq!(
            std::mem::size_of::<PyramidPosition>(),
            TABLEAU_LEN + STOCK_LEN
        );
    }


    /// Exercise all checked flat locations and reject extreme flat indices.
    #[test]
    fn tableau_location_indices_are_checked() {
        for index in 0..TABLEAU_LEN {
            assert_eq!(
                TableauLocation::from_index(index).map(TableauLocation::index),
                Some(index)
            );
        }

        assert_eq!(TableauLocation::from_index(TABLEAU_LEN), None);
        assert_eq!(TableauLocation::from_index(usize::MAX), None);
    }


    /// Move non-Copy values into the matrix and mutate them without cloning.
    #[test]
    fn triangular_matrix_owns_non_copy_elements() {
        let data: [String; TABLEAU_LEN] = std::array::from_fn(|index| format!("card {index}"));
        let original_first_buffer = data.first().unwrap().as_ptr();
        let mut matrix = TriangularMatrix::new(data);

        assert_eq!(matrix.get(1, 1).map(String::as_str), Some("card 0"));
        assert_eq!(matrix.get(3, 2).map(String::as_str), Some("card 4"));
        assert_eq!(matrix.get(7, 7).map(String::as_str), Some("card 27"));
        assert_eq!(
            matrix.get(1, 1).map(|value| value.as_ptr()),
            Some(original_first_buffer)
        );
        assert!(std::ptr::eq(
            matrix.get(3, 2).unwrap(),
            matrix.as_array().get(4).unwrap()
        ));
        matrix.get_mut(3, 2).unwrap().push_str(" updated");
        assert_eq!(matrix.get(3, 2).map(String::as_str), Some("card 4 updated"));
        assert_eq!(matrix.get(0, 1), None);
        assert_eq!(matrix.get(3, 4), None);
        assert_eq!(matrix.get(7, 0), None);
        assert_eq!(matrix.get(8, 1), None);
        assert_eq!(matrix.get(usize::MAX, 1), None);
        assert_eq!(matrix.get(1, usize::MAX), None);
        assert_eq!(matrix.get(usize::MAX, usize::MAX), None);
    }


    /// Write and retrieve every distinct cell, independently checking fixed order.
    #[test]
    fn triangular_matrix_writes_and_reads_all_distinct_values() {
        let mut matrix = TriangularMatrix::new([0_usize; TABLEAU_LEN]);
        let mut ordinal = 0;
        for row in 1..=TABLEAU_ROWS {
            for column in 1..=row {
                *matrix.get_mut(row, column).unwrap() = ordinal * 17 + 5;
                ordinal += 1;
            }
        }

        assert_eq!(ordinal, TABLEAU_LEN);
        assert_eq!(
            matrix.as_array(),
            &std::array::from_fn(|index| index * 17 + 5)
        );
        let mut ordinal = 0;
        for row in 1..=TABLEAU_ROWS {
            for column in 1..=row {
                assert_eq!(matrix.get(row, column), Some(&(ordinal * 17 + 5)));
                ordinal += 1;
            }
        }

        assert_eq!(ordinal, TABLEAU_LEN);
    }


    /// Invalid mutable access, including extreme inputs, cannot change any cell.
    #[test]
    fn triangular_matrix_rejects_invalid_mutation_without_side_effects() {
        let mut matrix = TriangularMatrix::new(std::array::from_fn(|index| index * 17 + 5));
        let before = *matrix.as_array();
        for (row, column) in [
            (0, 0),
            (0, 1),
            (1, 0),
            (3, 4),
            (7, 8),
            (8, 1),
            (usize::MAX, 1),
            (1, usize::MAX),
            (usize::MAX, usize::MAX),
        ] {
            assert!(matrix.get_mut(row, column).is_none());
            assert_eq!(matrix.as_array(), &before);
        }
    }


    /// Preserve the exact uppercase pile and one-based coordinate CSV notation.
    #[test]
    fn location_display_uses_recording_notation() {
        assert_eq!(Location::DL.to_string(), "DL");
        assert_eq!(Location::DR.to_string(), "DR");
        assert_eq!(
            Location::Tableau(TableauLocation::from_index(0).unwrap()).to_string(),
            "1|1"
        );
        assert_eq!(
            Location::Tableau(TableauLocation::from_index(27).unwrap()).to_string(),
            "7|7"
        );
    }


    /// Keep recorded selection order and action-specific optional locations intact.
    #[test]
    fn recorded_action_fields_follow_csv_contract() {
        let pair = RecordedAction::Match(Location::DR, Location::DL);
        assert_eq!(pair.name(), "MATCH");
        assert_eq!(pair.locations(), (Some(Location::DR), Some(Location::DL)));
        assert_eq!(
            RecordedAction::King(Location::DL).locations(),
            (Some(Location::DL), None)
        );
        assert_eq!(RecordedAction::King(Location::DL).name(), "KING");
        assert_eq!(RecordedAction::Move.locations(), (None, None));
        assert_eq!(RecordedAction::Move.name(), "MOVE");
        assert_eq!(RecordedAction::Recycle.locations(), (None, None));
        assert_eq!(RecordedAction::Recycle.name(), "RECYCLE");
        assert_eq!(RecordedAction::Move.operation(), "Press D");
        assert_eq!(RecordedAction::Recycle.operation(), "Press D to recycle");
    }
}
