//! Offline presentation of an immutable Pyramid deal and manual solution recording.
//!
//! Inputs are bounded and fully validated before presentation. The console retains
//! the original xSAR fixed arrays and compact typed actions, without live capture,
//! dynamic pile reconstruction, route search or guest input.

use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use xsar::games::{
    cards::Suit,
    pyramid::{CompleteDeal, ProvenanceKind, Restoration},
};

use basic_route_planning::{
    input,
    parameters::{
        ArgumentError, CliAction, ConsoleData, EXIT_USAGE_ERROR, Occupancy, PyramidPosition,
        RecordedSolution, STOCK_LEN, TABLEAU_CELL_WIDTH, TABLEAU_LEN, TABLEAU_ROWS,
    },
    utility::tableau_index,
};


/// Parse JSON-only or combined JSON/CSV usage while preserving OS path spelling.
///
/// # Errors
/// Refuses missing paths, unknown options and extra arguments with usage context.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<CliAction, ArgumentError> {
    let mut args = args.into_iter();
    let _program = args.next();
    let first = args
        .next()
        .ok_or_else(|| ArgumentError("missing deal JSON file path".to_owned()))?;
    let action = if first == "--help" || first == "-h" {
        CliAction::Help
    } else if first == "--version" || first == "-V" {
        CliAction::Version
    } else {
        if first.to_string_lossy().starts_with('-') {
            return Err(ArgumentError(format!(
                "unexpected option '{}'; expected deal JSON file path",
                first.to_string_lossy()
            )));
        }
        let deal_path = PathBuf::from(first);
        let solution_path = match args.next() {
            None => None,
            Some(option) if option == "--solution" => {
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    ArgumentError("missing solution CSV file path after --solution".to_owned())
                })?))
            }
            Some(extra) => {
                return Err(ArgumentError(format!(
                    "unexpected extra argument '{}'; expected --solution <SOLUTION_CSV_PATH>",
                    extra.to_string_lossy()
                )));
            }
        };
        CliAction::Inspect {
            deal_path,
            solution_path,
        }
    };
    if let Some(extra) = args.next() {
        return Err(ArgumentError(format!(
            "unexpected extra argument '{}'",
            extra.to_string_lossy()
        )));
    }
    Ok(action)
}


/// Write the supported offline invocation and options to the supplied writer.
///
/// # Errors
/// Propagates writer failures, including a closed downstream output pipe.
fn print_usage(mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "Usage: basic-route-planning <DEAL_JSON_PATH> [--solution <SOLUTION_CSV_PATH>]\n\
         \n\
         Display the saved original Pyramid deal and optional recorded solution.\n\
         \n\
         Options:\n\
           --solution <SOLUTION_CSV_PATH>  Read the manual solution CSV\n\
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
    let result = match action {
        CliAction::Help => print_usage(io::stdout().lock()),
        CliAction::Version => writeln!(
            io::stdout().lock(),
            "basic-route-planning {}",
            env!("CARGO_PKG_VERSION")
        ),
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
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: failed to write to stdout: {error}");
            ExitCode::FAILURE
        }
    }
}


/// Return a process exit status after the complete offline console operation.
fn main() -> ExitCode {
    run()
}
