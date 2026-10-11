//! Pyramid deal console intake and presentation.
//!
//! Loads an explicit Pyramid deal JSON document through the bounded `xsar::pyramid_json`
//! codec and prints its metadata, seven tableau rows, and 24-card DRAW stack.

use std::{
    ffi::OsString,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use xsar::games::pyramid::{CompleteDeal, ProvenanceKind, Restoration, STOCK_LEN, TABLEAU_LEN};


/// CLI exit code for usage or argument errors.
const EXIT_USAGE_ERROR: u8 = 2;


/// Bounded CLI actions parsed from OS arguments.
#[derive(Debug, Eq, PartialEq)]
enum CliAction {
    /// Show usage message to standard output and exit successfully.
    Help,
    /// Read and display a deal from an explicit filesystem path.
    InspectDeal(PathBuf),
}


/// Parse command-line arguments into a validated CLI action.
///
/// Accepts `--help` or `-h` to display usage, or exactly one filesystem path.
/// Extra or missing arguments return a descriptive error message.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<CliAction, String> {
    let mut args = args.into_iter();
    let _program_name = args.next();


    let first = match args.next() {
        Some(arg) => arg,
        None => {
            return Err("missing deal JSON file path".to_owned());
        }
    };


    if first == "--help" || first == "-h" {
        return Ok(CliAction::Help);
    }


    if let Some(extra) = args.next() {
        return Err(format!(
            "unexpected extra argument '{}'; expected exactly one file path",
            extra.to_string_lossy()
        ));
    }


    Ok(CliAction::InspectDeal(PathBuf::from(first)))
}


/// Return the canonical wire JSON string representation for a provenance kind.
const fn provenance_wire_str(kind: ProvenanceKind) -> &'static str {
    match kind {
        ProvenanceKind::SyntheticFixture => "synthetic_fixture",
        ProvenanceKind::ManualFixture => "manual_fixture",
        ProvenanceKind::ScreenRecognition => "screen_recognition",
    }
}


/// Return the canonical wire JSON string representation for a restoration status.
const fn restoration_wire_str(restoration: Restoration) -> &'static str {
    match restoration {
        Restoration::Verified => "verified",
        Restoration::NotApplicable => "not_applicable",
    }
}


/// Print usage information to the provided writer.
fn print_usage(mut writer: impl Write) -> io::Result<()> {
    writeln!(
        writer,
        "Usage: basic-route-planning <DEAL_JSON_PATH>\n\
         \n\
         Load and display a verified Pyramid deal JSON file.\n\
         \n\
         Options:\n\
           -h, --help    Print usage information and exit"
    )
}


/// Read and validate a complete Pyramid deal from the given filesystem path.
fn load_deal(path: &Path) -> Result<CompleteDeal, String> {
    let file = File::open(path)
        .map_err(|err| format!("failed to open '{}': {err}", path.to_string_lossy()))?;


    xsar::pyramid_json::read_from(file).map_err(|err| {
        format!(
            "failed to parse Pyramid deal from '{}': {err}",
            path.to_string_lossy()
        )
    })
}


/// Deterministically format and print a complete deal to the provided writer.
fn print_deal(deal: &CompleteDeal, mut writer: impl Write) -> io::Result<()> {
    let metadata = deal.metadata();


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
        "Restoration: {}",
        restoration_wire_str(metadata.restoration())
    )?;
    writeln!(writer)?;


    writeln!(writer, "Tableau (7 rows, {TABLEAU_LEN} cards):")?;
    for row in 0..7 {
        writeln!(writer, "  Row {row}:")?;
        for col in 0..=row {
            let card = deal
                .card_at(row, col)
                .expect("row and column within standard pyramid dimensions");
            writeln!(
                writer,
                "    col {col}: {card} (value: {}, suit: {})",
                card.rank().value(),
                card.suit().as_str()
            )?;
        }
    }
    writeln!(writer)?;


    writeln!(
        writer,
        "Stock DRAW stack ({STOCK_LEN} cards, original draw order):"
    )?;
    for (ordinal, card) in deal.stock().iter().enumerate() {
        writeln!(
            writer,
            "  [{ordinal:02}] {card} (value: {}, suit: {})",
            card.rank().value(),
            card.suit().as_str()
        )?;
    }


    Ok(())
}


/// Entry point for the Pyramid route planning console reader.
fn run() -> ExitCode {
    let action = match parse_args(std::env::args_os()) {
        Ok(action) => action,
        Err(err) => {
            eprintln!("Error: {err}");
            let _ = print_usage(io::stderr());
            return ExitCode::from(EXIT_USAGE_ERROR);
        }
    };


    match action {
        CliAction::Help => match print_usage(io::stdout().lock()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error: failed to write to stdout: {err}");
                ExitCode::FAILURE
            }
        },
        CliAction::InspectDeal(path) => {
            let deal = match load_deal(&path) {
                Ok(deal) => deal,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };


            match print_deal(&deal, io::stdout().lock()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("Error: failed to write to stdout: {err}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}


fn main() -> ExitCode {
    run()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;


    /// Test that --help and -h produce CliAction::Help.
    #[test]
    fn test_parse_args_help() {
        assert_eq!(
            parse_args([OsString::from("prog"), OsString::from("--help")]),
            Ok(CliAction::Help)
        );
        assert_eq!(
            parse_args([OsString::from("prog"), OsString::from("-h")]),
            Ok(CliAction::Help)
        );
    }


    /// Test that missing argument fails.
    #[test]
    fn test_parse_args_missing() {
        assert!(parse_args([OsString::from("prog")]).is_err());
    }


    /// Test that single file argument produces CliAction::InspectDeal.
    #[test]
    fn test_parse_args_single_path() {
        let action = parse_args([OsString::from("prog"), OsString::from("/path/to/deal.json")]);
        assert_eq!(
            action,
            Ok(CliAction::InspectDeal(PathBuf::from("/path/to/deal.json")))
        );
    }


    /// Test that extra arguments produce an error.
    #[test]
    fn test_parse_args_extra() {
        assert!(
            parse_args([
                OsString::from("prog"),
                OsString::from("file1.json"),
                OsString::from("file2.json"),
            ])
            .is_err()
        );
    }


    /// Test that provenance wire string mapping matches wire spec.
    #[test]
    fn test_provenance_wire_strings() {
        assert_eq!(
            provenance_wire_str(ProvenanceKind::SyntheticFixture),
            "synthetic_fixture"
        );
        assert_eq!(
            provenance_wire_str(ProvenanceKind::ManualFixture),
            "manual_fixture"
        );
        assert_eq!(
            provenance_wire_str(ProvenanceKind::ScreenRecognition),
            "screen_recognition"
        );
    }


    /// Test that restoration wire string mapping matches wire spec.
    #[test]
    fn test_restoration_wire_strings() {
        assert_eq!(restoration_wire_str(Restoration::Verified), "verified");
        assert_eq!(
            restoration_wire_str(Restoration::NotApplicable),
            "not_applicable"
        );
    }
}
