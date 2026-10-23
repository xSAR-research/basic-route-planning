//! Offline presentation of an immutable Pyramid deal and manual solution recording.
//!
//! Inputs are bounded and fully validated before presentation. The console retains
//! the original xSAR fixed arrays and compact typed actions, without live capture,
//! guest capture or guest input. Optional A* search prints an independently
//! replayed route under the explicitly stated pinned upstream rules.

use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
    sync::atomic::AtomicBool,
    time::Duration,
};

use xsar::games::{
    cards::Suit,
    pyramid::{CompleteDeal, ProvenanceKind, Restoration},
};

use basic_route_planning::{
    input,
    parameters::{
        ArgumentError, CliAction, ConsoleData, ConsoleError, EXIT_USAGE_ERROR, Occupancy,
        PyramidPosition, RecordedAction, RecordedSolution, ReplayStep, STOCK_LEN, SearchLimits,
        SearchMethod, SearchOutcome, SearchStats, TABLEAU_CELL_WIDTH, TABLEAU_LEN, TABLEAU_ROWS,
    },
    replay, rules, search,
    utility::tableau_index,
};


/// Parse an unsigned decimal bound without signs, whitespace or overflow.
///
/// # Errors
/// Refuses absent/non-ASCII decimal values and values exceeding usize.
fn parse_bound(value: Option<OsString>, option: &str) -> Result<usize, ArgumentError> {
    let value = value.ok_or_else(|| ArgumentError(format!("missing value after {option}")))?;
    let text = value
        .to_str()
        .ok_or_else(|| ArgumentError(format!("invalid value for {option}")))?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArgumentError(format!(
            "{option} requires an unsigned decimal integer"
        )));
    }
    text.parse()
        .map_err(|_| ArgumentError(format!("{option} value is too large")))
}


/// Parse display or bounded solving while preserving exact filesystem path bytes.
///
/// # Errors
/// Refuses missing/duplicate/unknown options and limits without explicit --solve.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<CliAction, ArgumentError> {
    let mut args = args.into_iter();
    let _program = args.next();
    let first = args
        .next()
        .ok_or_else(|| ArgumentError("missing deal JSON file path".to_owned()))?;
    if first == "--help" || first == "-h" || first == "--version" || first == "-V" {
        if args.next().is_some() {
            return Err(ArgumentError("unexpected extra argument".to_owned()));
        }
        return Ok(if first == "--help" || first == "-h" {
            CliAction::Help
        } else {
            CliAction::Version
        });
    }
    if first.to_string_lossy().starts_with('-') {
        return Err(ArgumentError(format!(
            "unexpected option '{}'; expected deal JSON file path",
            first.to_string_lossy()
        )));
    }
    let deal_path = PathBuf::from(first);
    let mut solution_path = None;
    let mut solve = false;
    let mut limits = SearchLimits::default();
    let mut seen = 0_u8;
    while let Some(option) = args.next() {
        if option == "--solution" {
            if solution_path.is_some() {
                return Err(ArgumentError("duplicate --solution".to_owned()));
            }
            solution_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                ArgumentError("missing solution CSV file path after --solution".to_owned())
            })?));
        } else if option == "--solve" {
            if solve {
                return Err(ArgumentError("duplicate --solve".to_owned()));
            }
            solve = true;
        } else {
            let option = option
                .to_str()
                .ok_or_else(|| ArgumentError("invalid option spelling".to_owned()))?;
            let bit = match option {
                "--max-states" => 1,
                "--max-expansions" => 2,
                "--max-memory-mib" => 4,
                "--max-seconds" => 8,
                _ => {
                    return Err(ArgumentError(format!(
                        "unexpected extra argument '{option}'"
                    )));
                }
            };
            if seen & bit != 0 {
                return Err(ArgumentError(format!("duplicate {option}")));
            }
            seen |= bit;
            let value = parse_bound(args.next(), option)?;
            match option {
                "--max-states" => limits.max_states = value,
                "--max-expansions" => limits.max_expansions = value,
                "--max-memory-mib" => {
                    limits.max_memory_bytes = value
                        .checked_mul(1024 * 1024)
                        .ok_or_else(|| ArgumentError("memory bound overflows bytes".to_owned()))?
                }
                "--max-seconds" => {
                    limits.max_duration =
                        Duration::from_secs(u64::try_from(value).map_err(|_| {
                            ArgumentError("time bound overflows seconds".to_owned())
                        })?)
                }
                _ => return Err(ArgumentError("invalid search limit".to_owned())),
            }
        }
    }
    if solve {
        Ok(CliAction::Solve {
            deal_path,
            solution_path,
            limits,
        })
    } else if seen != 0 {
        Err(ArgumentError("search limits require --solve".to_owned()))
    } else {
        Ok(CliAction::Inspect {
            deal_path,
            solution_path,
        })
    }
}


/// Write the supported offline invocation and options to the supplied writer.
///
/// # Errors
/// Propagates writer failures, including a closed downstream output pipe.
fn print_usage(mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "Usage: basic-route-planning <DEAL_JSON_PATH> [--solution <SOLUTION_CSV_PATH>] [--solve]\n\
         \n\
         Display saved inputs, or calculate an independently replayed offline route.\n\
         \n\
         Options:\n\
           --solution <SOLUTION_CSV_PATH>  Read the manual comparison recording\n\
           --solve                         Calculate a derived route using A*\n\
           --max-states <N>                 Retained records (default 2000000)\n\
           --max-expansions <N>             Expanded states (default 2000000)\n\
           --max-memory-mib <N>             Collection allowance (default 512 MiB)\n\
           --max-seconds <N>                Search deadline (default 120 seconds)\n\
           -h, --help                      Print usage information and exit\n\
           -V, --version                   Print the package version and exit"
    )
}


/// Return the shared codec's canonical provenance label for a checked variant.
const fn provenance_wire_str(kind: ProvenanceKind) -> &'static str {
    match kind {
        ProvenanceKind::SyntheticFixture => "synthetic_fixture",
        ProvenanceKind::ManualFixture => "manual_fixture",
        ProvenanceKind::ScreenRecognition => "screen_recognition",
    }
}


/// Return the shared codec's canonical restoration label without asserting proof.
const fn restoration_wire_str(restoration: Restoration) -> &'static str {
    match restoration {
        Restoration::Verified => "verified",
        Restoration::NotApplicable => "not_applicable",
    }
}


/// Return a portable ASCII suit abbreviation for compact tableau output.
const fn suit_code(suit: Suit) -> char {
    match suit {
        Suit::Clubs => 'C',
        Suit::Diamonds => 'D',
        Suit::Hearts => 'H',
        Suit::Spades => 'S',
    }
}


/// Print original metadata, a checked one-based tableau and all stock identities.
///
/// # Errors
/// Propagates writer failures; rejects any internal mismatch in tableau dimensions.
fn print_deal(deal: &CompleteDeal, mut writer: impl Write) -> io::Result<()> {
    let metadata = deal.metadata();
    writeln!(writer, "Saved original deal")?;
    writeln!(writer, "Deal ID:     {}", metadata.deal_id())?;
    writeln!(
        writer,
        "Provenance:  {}",
        provenance_wire_str(metadata.provenance())
    )?;
    writeln!(writer, "Producer:    {}", metadata.producer())?;
    writeln!(writer, "Source:      {}", metadata.source())?;
    writeln!(
        writer,
        "Evidence ID: {}",
        metadata.evidence_id().unwrap_or("none")
    )?;
    writeln!(
        writer,
        "Restoration: {} (producer-declared)",
        restoration_wire_str(metadata.restoration())
    )?;
    writeln!(
        writer,
        "\nTableau ({TABLEAU_ROWS} rows, {TABLEAU_LEN} cards):"
    )?;
    writeln!(
        writer,
        "Fixed row|column = rank+suit(value); A=1, J=11, Q=12, K=13"
    )?;
    writeln!(writer, "Suits: C=clubs, D=diamonds, H=hearts, S=spades\n")?;

    for row in 1..=TABLEAU_ROWS {
        write!(
            writer,
            "{:indent$}",
            "",
            indent = (TABLEAU_ROWS - row) * TABLEAU_CELL_WIDTH / 2
        )?;
        for column in 1..=row {
            let card = tableau_index(row, column)
                .and_then(|index| deal.tableau().get(index))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "checked tableau dimensions disagree",
                    )
                })?;
            let label = format!(
                "{row}|{column}={}{}({})",
                card.rank(),
                suit_code(card.suit()),
                card.rank().value()
            );
            write!(writer, "{label:<width$}", width = TABLEAU_CELL_WIDTH)?;
        }
        writeln!(writer)?;
    }

    writeln!(
        writer,
        "\nStock DRAW stack ({STOCK_LEN} cards, original encounter order):"
    )?;
    writeln!(
        writer,
        "Encounter 1 is initial DL before D; DR starts empty."
    )?;
    for (ordinal, card) in deal.stock().iter().enumerate() {
        writeln!(
            writer,
            "  [{:02}] {card} (value: {}, suit: {})",
            ordinal + 1,
            card.rank().value(),
            card.suit().as_str()
        )?;
    }
    Ok(())
}


/// Print typed actions in validated sequence order without reconstructing piles.
///
/// # Errors
/// Propagates writer failures; no source file or recorded action is modified.
fn print_solution(solution: &RecordedSolution, mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "\nRecorded solution ({} recorded actions):",
        solution.steps.len()
    )?;
    writeln!(
        writer,
        "DL = DRAW PILE LEFT; DR = DRAW PILE RIGHT. Tableau locations are fixed."
    )?;
    writeln!(
        writer,
        "Structure checked; gameplay legality and optimality are not evaluated.\n"
    )?;
    writeln!(writer, "Sequence  Action   Card 1   Card 2   Operation")?;
    for step in &solution.steps {
        let (first, second) = step.action.locations();
        let first = first.map_or_else(|| "-".to_owned(), |location| location.to_string());
        let second = second.map_or_else(|| "-".to_owned(), |location| location.to_string());
        writeln!(
            writer,
            "{:>8}  {:<7}  {:<6}   {:<6}   {}",
            step.sequence,
            step.action.name(),
            first,
            second,
            step.action.operation()
        )?;
    }
    Ok(())
}


/// Count occupied, empty and unknown slots without allocating or changing state.
fn occupancy_counts(slots: &[Occupancy]) -> (usize, usize, usize) {
    slots
        .iter()
        .fold((0, 0, 0), |(occupied, empty, unknown), slot| match slot {
            Occupancy::Occupied => (occupied + 1, empty, unknown),
            Occupancy::Empty => (occupied, empty + 1, unknown),
            Occupancy::Unknown => (occupied, empty, unknown + 1),
        })
}


/// Show occupancy and geometrically accessible slots in the initial saved layout.
///
/// # Errors
/// Propagates writer failures or an internal coordinate/model mismatch.
fn print_position(position: &PyramidPosition, mut writer: impl Write) -> io::Result<()> {
    let (occupied, empty, unknown) = occupancy_counts(position.tableau().as_array());
    writeln!(
        writer,
        "\nPosition model: tableau {occupied} occupied, {empty} empty, {unknown} unknown."
    )?;
    let (occupied, empty, unknown) = occupancy_counts(position.stock());
    writeln!(
        writer,
        "DRAW occupancy: {occupied} occupied, {empty} empty, {unknown} unknown."
    )?;
    write!(writer, "Accessible tableau positions (covering geometry):")?;
    for row in 1..=TABLEAU_ROWS {
        for column in 1..=row {
            if position
                .tableau_accessible(row, column)
                .map_err(io::Error::other)?
            {
                write!(writer, " {row}|{column}")?;
            }
        }
    }
    writeln!(writer)
}


/// Render both retained inputs after their complete validation has succeeded.
///
/// # Errors
/// Propagates report-write failures or an internal tableau-dimension mismatch.
fn print_inputs(data: &ConsoleData, mut writer: impl Write) -> io::Result<()> {
    print_deal(&data.deal, &mut writer)?;
    print_position(&data.position, &mut writer)?;
    if let Some(solution) = &data.solution {
        print_solution(solution, writer)?;
    }
    Ok(())
}


/// Format an original card identity without storing a second card inventory.
///
/// # Errors
/// Refuses invalid internal identity indices instead of inventing a card label.
fn route_card(deal: &CompleteDeal, index: Option<u8>) -> io::Result<String> {
    match index {
        None => Ok("-".to_owned()),
        Some(index) => {
            let card = rules::card_at(deal, index).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "route identity outside original deal",
                )
            })?;
            Ok(format!(
                "{}{}({})",
                card.rank(),
                suit_code(card.suit()),
                card.rank().value()
            ))
        }
    }
}


/// Print independently replayed actions with before-selected and after-pile cards.
///
/// # Errors
/// Propagates writer failures and invalid internal identity indices.
fn print_derived(
    deal: &CompleteDeal,
    trace: &[ReplayStep],
    mut writer: impl Write,
) -> io::Result<()> {
    writeln!(
        writer,
        "\nDerived solution ({} logical actions):",
        trace.len()
    )?;
    writeln!(
        writer,
        "Proof: optimal under the stated rules; independent route replay PASS."
    )?;
    writeln!(
        writer,
        "Follow from the saved original board: DL encounter 1, DR empty, Solver off."
    )?;
    writeln!(
        writer,
        "MATCH selects CARD1 then CARD2; KING selects CARD1; MOVE/RECYCLE press D."
    )?;
    writeln!(
        writer,
        "Cards show rank+suit(value); pile columns show tops AFTER each action.\n"
    )?;
    writeln!(
        writer,
        "Sequence Action  CARD1              CARD2              DL after   DR after   Tableau"
    )?;
    for (index, step) in trace.iter().enumerate() {
        let (first, second) = step.action.locations();
        let first = match first {
            Some(location) => format!("{location}={}", route_card(deal, step.first)?),
            None if step.action == RecordedAction::Move => {
                format!("DL={}", route_card(deal, step.first)?)
            }
            None => "-".to_owned(),
        };
        let second = match second {
            Some(location) => format!("{location}={}", route_card(deal, step.second)?),
            None => "-".to_owned(),
        };
        writeln!(
            writer,
            "{:>8} {:<7} {:<18} {:<18} {:<10} {:<10} {:>7}",
            index + 1,
            step.action.name(),
            first,
            second,
            route_card(deal, step.dl_after)?,
            route_card(deal, step.dr_after)?,
            step.remaining_tableau
        )?;
    }
    writeln!(
        writer,
        "Final tableau: 0 cards; objective is this board's clearance."
    )
}


/// Print measured search counters without claiming process-wide RSS measurement.
///
/// # Errors
/// Propagates report-writing failures.
fn print_search_stats(stats: SearchStats, mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "Search: {} expanded, {} generated, {} retained, peak frontier {}, {:.3}s",
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier,
        stats.elapsed.as_secs_f64()
    )
}


/// Calculate and independently replay a route, returning a distinct outcome code.
///
/// # Errors
/// Refuses an illegal supplied/calculated route or a failed output write.
fn solve_console(
    data: &ConsoleData,
    limits: SearchLimits,
    mut writer: impl Write,
) -> Result<u8, ConsoleError> {
    if let Some(solution) = &data.solution {
        let actions: Vec<_> = solution.steps.iter().map(|step| step.action).collect();
        let _ = replay::replay(&data.deal, &actions, true)?;
    }
    print_inputs(data, &mut writer)?;
    writeln!(
        writer,
        "\nRules: mchung94/pyramid-solver c6dde73; both DL/DR playable; two recycles; preserved encounter order; tableau-clear goal."
    )?;
    writeln!(
        writer,
        "Objective: one unit per MATCH, KING, MOVE or RECYCLE (pairs use two ordered clicks)."
    )?;
    writeln!(
        writer,
        "Limits: {} records, {} expansions, {} MiB collection allowance, {} seconds.",
        limits.max_states,
        limits.max_expansions,
        limits.max_memory_bytes / (1024 * 1024),
        limits.max_duration.as_secs()
    )?;
    if let Some(solution) = &data.solution {
        writeln!(
            writer,
            "Recorded comparison: independent legal replay PASS; {} logical actions; tableau cleared.",
            solution.steps.len()
        )?;
    }
    writer.flush()?;
    match search::solve(
        &data.deal,
        limits,
        &AtomicBool::new(false),
        SearchMethod::AStar,
    ) {
        SearchOutcome::Solved { actions, stats } => {
            let trace = replay::replay(&data.deal, &actions, true)?;
            print_derived(&data.deal, &trace, &mut writer)?;
            print_search_stats(stats, &mut writer)?;
            if let Some(recorded) = &data.solution {
                writeln!(
                    writer,
                    "Comparison: derived {} versus recorded {} logical actions.",
                    actions.len(),
                    recorded.steps.len()
                )?;
            }
            Ok(0)
        }
        SearchOutcome::Unsolvable { stats } => {
            writeln!(writer, "Result: proven no solution under the stated rules.")?;
            print_search_stats(stats, writer)?;
            Ok(3)
        }
        SearchOutcome::Stopped { reason, stats } => {
            writeln!(
                writer,
                "Result: {reason}; no proof of unsolvability or optimal route."
            )?;
            print_search_stats(stats, writer)?;
            Ok(4)
        }
    }
}


/// Execute a read-only CLI request, treating a downstream broken pipe as success.
fn run() -> ExitCode {
    let action = match parse_args(std::env::args_os()) {
        Ok(action) => action,
        Err(error) => {
            eprintln!("Error: {error}");
            let _ = print_usage(io::stderr().lock());
            return ExitCode::from(EXIT_USAGE_ERROR);
        }
    };
    let result: Result<u8, ConsoleError> = match action {
        CliAction::Help => print_usage(io::stdout().lock())
            .map(|()| 0)
            .map_err(ConsoleError::from),
        CliAction::Version => writeln!(
            io::stdout().lock(),
            "basic-route-planning {}",
            env!("CARGO_PKG_VERSION")
        )
        .map(|()| 0)
        .map_err(ConsoleError::from),
        CliAction::Solve {
            deal_path,
            solution_path,
            limits,
        } => {
            let data = match input::load_inputs(&deal_path, solution_path.as_deref()) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("Error: {error}");
                    return ExitCode::FAILURE;
                }
            };
            if data
                .solution
                .as_ref()
                .is_some_and(|solution| solution.reordered)
            {
                eprintln!(
                    "Warning: CSV rows reordered by SEQUENCE for display; source file unchanged."
                );
            }
            solve_console(&data, limits, io::stdout().lock())
        }
        CliAction::Inspect {
            deal_path,
            solution_path,
        } => {
            let data = match input::load_inputs(&deal_path, solution_path.as_deref()) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("Error: {error}");
                    return ExitCode::FAILURE;
                }
            };
            if data
                .solution
                .as_ref()
                .is_some_and(|solution| solution.reordered)
            {
                eprintln!(
                    "Warning: CSV rows reordered by SEQUENCE for display; source file unchanged."
                );
            }
            print_inputs(&data, io::stdout().lock())
                .map(|()| 0)
                .map_err(ConsoleError::from)
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(ConsoleError::Output(error)) if error.kind() == io::ErrorKind::BrokenPipe => {
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}


/// Return a process exit status after the complete offline console operation.
fn main() -> ExitCode {
    run()
}
