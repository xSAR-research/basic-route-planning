//! Portable CLI checks for saved Pyramid deals and recorded solution structure.
//!
//! Fixtures are synthetic. These checks do not claim legal or winning routes;
//! the retained live JSON/CSV pair has a separate data-comparison acceptance.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};


/// Process-local suffix avoiding collisions between concurrently running tests.
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);


/// Exact column names and order agreed for manual solution recordings.
const CSV_HEADER: &str = "SEQUENCE,ACTION,CARD1,CARD2\n";


/// Small formatting sample containing every action and both DRAW locations.
const SOLUTION: &str = "SEQUENCE,ACTION,CARD1,CARD2\n\
1,MATCH,7|7,7|3\n2,KING,7|5,\n3,MOVE,,\n4,RECYCLE,,\n5,MATCH,DL,DR\n";


/// Locate the executable built by Cargo for this integration-test target.
fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_basic-route-planning"))
}


/// Locate a synthetic fixture in this package without a host-specific path.
fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}


/// Run CLI argument cases while capturing both output streams.
fn run_cmd(arguments: &[&str]) -> Output {
    Command::new(bin_path())
        .args(arguments)
        .output()
        .expect("execute console")
}


/// Read a saved deal with an optional recording, accepting native filesystem paths.
fn run_deal(deal: &Path, solution: Option<&Path>) -> Output {
    let mut command = Command::new(bin_path());
    command.arg(deal);
    if let Some(solution) = solution {
        command.arg("--solution").arg(solution);
    }
    command.output().expect("execute console")
}


/// Own a uniquely created scratch file and remove only that file on drop.
struct TempFileGuard {
    /// Scratch file created using no-overwrite semantics.
    path: PathBuf,
}


impl TempFileGuard {
    /// Write test data outside the checkout without colliding with parallel tests.
    ///
    /// The process ID and atomic suffix identify each attempt. An existing path
    /// from another invocation is skipped, never overwritten or deleted.
    fn create(extension: &str, contents: &[u8]) -> Self {
        loop {
            let suffix = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "basic-route-planning-test-{}-{suffix}.{extension}",
                std::process::id()
            ));
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create scratch file '{}': {error}", path.display()),
            };
            let guard = Self { path };
            file.write_all(contents).expect("write scratch file");
            return guard;
        }
    }


    /// Borrow the owned scratch path without requiring UTF-8 filenames.
    fn path(&self) -> &Path {
        &self.path
    }
}


impl Drop for TempFileGuard {
    /// Remove only this guard's uniquely created scratch file.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}


/// Require failed input to print diagnostics without a partial successful report.
fn assert_input_failure(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "invalid input printed a partial report"
    );
    assert!(
        !output.stderr.is_empty(),
        "missing input failure diagnostic"
    );
    String::from_utf8(output.stderr.clone()).expect("UTF-8 diagnostic")
}


/// Reject one CSV sample while using an independently valid synthetic deal.
fn assert_csv_rejected(label: &str, contents: &[u8]) {
    let temporary = TempFileGuard::create("csv", contents);
    let output = run_deal(
        &fixture_path("pyramid-synthetic-v2.json"),
        Some(temporary.path()),
    );
    assert!(!output.status.success(), "accepted invalid CSV: {label}");
    let diagnostic = assert_input_failure(&output);
    assert!(
        diagnostic.to_ascii_lowercase().contains("solution") || diagnostic.contains("CSV"),
        "missing solution context for {label}: {diagnostic}"
    );
}


/// Extract action-table rows without depending on column-padding widths.
fn solution_rows(stdout: &str) -> Vec<Vec<&str>> {
    stdout
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .filter(|fields| {
            fields.len() >= 2
                && fields[0].parse::<usize>().is_ok()
                && matches!(fields[1], "MATCH" | "KING" | "MOVE" | "RECYCLE")
        })
        .collect()
}


/// Check original-deal metadata, all fixed tableau positions and DRAW ordinals.
fn assert_synthetic_display(stdout: &str) {
    for expected in [
        "Saved original deal",
        "synthetic-ordered-deck-v1",
        "synthetic_fixture",
        "not_applicable",
        "synthetic-deck-v1",
        "Tableau (7 rows, 28 cards)",
    ] {
        assert!(
            stdout.contains(expected),
            "missing report detail {expected}"
        );
    }
    let tableau = [
        "1|1=AC(1)",
        "2|1=2C(2)",
        "2|2=3C(3)",
        "3|1=4C(4)",
        "3|2=5C(5)",
        "3|3=6C(6)",
        "4|1=7C(7)",
        "4|2=8C(8)",
        "4|3=9C(9)",
        "4|4=10C(10)",
        "5|1=JC(11)",
        "5|2=QC(12)",
        "5|3=KC(13)",
        "5|4=AD(1)",
        "5|5=2D(2)",
        "6|1=3D(3)",
        "6|2=4D(4)",
        "6|3=5D(5)",
        "6|4=6D(6)",
        "6|5=7D(7)",
        "6|6=8D(8)",
        "7|1=9D(9)",
        "7|2=10D(10)",
        "7|3=JD(11)",
        "7|4=QD(12)",
        "7|5=KD(13)",
        "7|6=AH(1)",
        "7|7=2H(2)",
    ];
    for expected in tableau {
        assert!(stdout.contains(expected), "missing tableau card {expected}");
    }
    let mut stock_lines = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('['));
    for ordinal in 1..=24 {
        let line = stock_lines.next().expect("all 24 DRAW entries are printed");
        assert!(line.trim_start().starts_with(&format!("[{ordinal:02}]")));
    }
    assert!(stock_lines.next().is_none());
    assert!(stdout.contains("3 of hearts"));
    assert!(stdout.contains("K of spades") || stdout.contains("King of spades"));
    assert!(stdout.contains("DL"));
    assert!(stdout.to_ascii_lowercase().contains("before"));
    assert!(!stdout.contains("0|0="));
}


/// Help documents the optional solution argument and writes no diagnostic.
#[test]
fn test_cli_help_flags() {
    for flag in ["--help", "-h"] {
        let output = run_cmd(&[flag]);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage: basic-route-planning <DEAL_JSON_PATH>"));
        assert!(stdout.contains("--solution <SOLUTION_CSV_PATH>"));
        assert!(stdout.contains("--help"));
        assert!(output.stderr.is_empty());
    }
}


/// Both version flags identify the manifest version without opening any file.
#[test]
fn test_cli_version_flags() {
    for flag in ["--version", "-V"] {
        let output = run_cmd(&[flag]);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("basic-route-planning"));
        assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
        assert!(output.stderr.is_empty());
    }
}


/// Missing, extra and unknown arguments are usage errors rather than input failures.
#[test]
fn test_cli_invalid_arguments() {
    for arguments in [
        vec![],
        vec!["one.json", "two.json"],
        vec!["one.json", "--solution"],
        vec!["one.json", "--unknown"],
        vec!["one.json", "--solution", "one.csv", "extra.csv"],
    ] {
        let output = run_cmd(&arguments);
        assert_eq!(output.status.code(), Some(2), "arguments: {arguments:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
}


/// Opening a missing deal reports its filename and prints no partial report.
#[test]
fn test_cli_nonexistent_file() {
    let missing = fixture_path("nonexistent-deal-file.json");
    assert!(!missing.exists());
    let diagnostic = assert_input_failure(&run_deal(&missing, None));
    assert!(diagnostic.contains("failed to open"));
    assert!(diagnostic.contains("nonexistent-deal-file.json"));
}


/// Schema 1 and 2 decode into the same correctly labelled original card positions.
#[test]
fn test_cli_synthetic_fixture_versions() {
    for name in ["pyramid-synthetic-v1.json", "pyramid-synthetic-v2.json"] {
        let output = run_deal(&fixture_path(name), None);
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_synthetic_display(&String::from_utf8_lossy(&output.stdout));
    }
}


/// Malformed JSON is rejected by the shared decoder before any deal is printed.
#[test]
fn test_cli_refuse_malformed_json() {
    let temporary = TempFileGuard::create("json", b"{ not valid json }");
    let diagnostic = assert_input_failure(&run_deal(temporary.path(), None));
    assert!(diagnostic.contains("failed to parse Pyramid deal"));
}


/// The shared JSON decoder's 65,536-byte bound remains in force.
#[test]
fn test_cli_refuse_oversized_json() {
    let temporary = TempFileGuard::create("json", &vec![b' '; 65_537]);
    let diagnostic = assert_input_failure(&run_deal(temporary.path(), None));
    assert!(diagnostic.contains("failed to parse Pyramid deal"));
    assert!(diagnostic.contains("exceeds 65536 bytes"));
}


/// Duplicate cards fail even with internally consistent schema-2 card fields.
#[test]
fn test_cli_refuse_duplicate_card() {
    let base = fs::read_to_string(fixture_path("pyramid-synthetic-v2.json")).expect("read fixture");
    let corrupted = base.replacen(
        "\"card\": \"2 of clubs\",\n      \"value\": 2",
        "\"card\": \"A of clubs\",\n      \"value\": 1",
        1,
    );
    assert_ne!(base, corrupted);
    let temporary = TempFileGuard::create("json", corrupted.as_bytes());
    let diagnostic = assert_input_failure(&run_deal(temporary.path(), None));
    assert!(diagnostic.contains("duplicate"));
}


/// Unknown occupants cannot enter an intake that requires a complete known deal.
#[test]
fn test_cli_refuse_unknown_card() {
    let base = fs::read_to_string(fixture_path("pyramid-synthetic-v2.json")).expect("read fixture");
    let corrupted = base.replacen("\"card\": \"A of clubs\"", "\"card\": \"unknown\"", 1);
    assert_ne!(base, corrupted);
    let temporary = TempFileGuard::create("json", corrupted.as_bytes());
    assert!(assert_input_failure(&run_deal(temporary.path(), None)).contains("invalid card"));
}


/// Unsupported schema versions retain the shared codec's rejection behaviour.
#[test]
fn test_cli_refuse_unsupported_version() {
    let base = fs::read_to_string(fixture_path("pyramid-synthetic-v2.json")).expect("read fixture");
    let corrupted = base.replacen("\"schema_version\": 2", "\"schema_version\": 3", 1);
    assert_ne!(base, corrupted);
    let temporary = TempFileGuard::create("json", corrupted.as_bytes());
    assert!(assert_input_failure(&run_deal(temporary.path(), None)).contains("schema"));
}


/// Unrecognised JSON fields are refused rather than silently discarded.
#[test]
fn test_cli_refuse_unknown_fields() {
    let base = fs::read_to_string(fixture_path("pyramid-synthetic-v2.json")).expect("read fixture");
    let corrupted = base.replacen(
        "{\n  \"schema_version\"",
        "{\n  \"unexpected\": true,\n  \"schema_version\"",
        1,
    );
    assert_ne!(base, corrupted);
    let temporary = TempFileGuard::create("json", corrupted.as_bytes());
    assert!(
        assert_input_failure(&run_deal(temporary.path(), None))
            .contains("invalid Pyramid JSON syntax or schema")
    );
}


/// A card label and contradictory numeric value remain invalid schema-2 data.
#[test]
fn test_cli_refuse_contradictory_card() {
    let base = fs::read_to_string(fixture_path("pyramid-synthetic-v2.json")).expect("read fixture");
    let corrupted = base.replacen("\"value\": 1", "\"value\": 5", 1);
    assert_ne!(base, corrupted);
    let temporary = TempFileGuard::create("json", corrupted.as_bytes());
    assert!(assert_input_failure(&run_deal(temporary.path(), None)).contains("invalid card"));
}


/// Solution output preserves sequence, action and first/second card ordering.
#[test]
fn test_cli_print_solution() {
    let temporary = TempFileGuard::create("csv", SOLUTION.as_bytes());
    let output = run_deal(
        &fixture_path("pyramid-synthetic-v2.json"),
        Some(temporary.path()),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
    assert_synthetic_display(&stdout);
    assert!(stdout.contains("(5 recorded actions)"));
    let rows = solution_rows(&stdout);
    assert_eq!(rows.len(), 5);
    for (row, expected) in rows.iter().zip([
        ["1", "MATCH", "7|7", "7|3"],
        ["2", "KING", "7|5", "-"],
        ["3", "MOVE", "-", "-"],
        ["4", "RECYCLE", "-", "-"],
        ["5", "MATCH", "DL", "DR"],
    ]) {
        assert_eq!(&row[..4], &expected);
    }
}


/// Quoted spreadsheet fields with CRLF line endings are accepted without alteration.
#[test]
fn test_cli_quoted_crlf_solution() {
    let contents = b"\"SEQUENCE\",\"ACTION\",\"CARD1\",\"CARD2\"\r\n\
\"1\",\"MATCH\",\"7|7\",\"7|3\"\r\n\"2\",\"KING\",\"DL\",\"\"\r\n\
\"3\",\"MOVE\",\"\",\"\"\r\n\"4\",\"RECYCLE\",\"\",\"\"\r\n";
    let temporary = TempFileGuard::create("csv", contents);
    let output = run_deal(
        &fixture_path("pyramid-synthetic-v2.json"),
        Some(temporary.path()),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
    let rows = solution_rows(&stdout);
    assert_eq!(rows.len(), 4);
    assert_eq!(&rows[0][..4], &["1", "MATCH", "7|7", "7|3"]);
    assert_eq!(&rows[1][..4], &["2", "KING", "DL", "-"]);
}


/// SEQUENCE determines play order; file permutations only change the warning stream.
#[test]
fn test_cli_solution_sequence_order() {
    let ordered = TempFileGuard::create("csv", SOLUTION.as_bytes());
    let reordered = TempFileGuard::create(
        "csv",
        b"SEQUENCE,ACTION,CARD1,CARD2\n\
5,MATCH,DL,DR\n2,KING,7|5,\n4,RECYCLE,,\n1,MATCH,7|7,7|3\n3,MOVE,,\n",
    );
    let deal = fixture_path("pyramid-synthetic-v2.json");
    let expected = run_deal(&deal, Some(ordered.path()));
    let actual = run_deal(&deal, Some(reordered.path()));
    assert!(expected.status.success());
    assert!(
        actual.status.success(),
        "{}",
        String::from_utf8_lossy(&actual.stderr)
    );
    assert_eq!(actual.stdout, expected.stdout);
    let warning = String::from_utf8_lossy(&actual.stderr).to_ascii_lowercase();
    assert!(warning.contains("sequence"));
    assert!(warning.contains("order"));
}


/// A missing recording fails before printing the already valid deal.
#[test]
fn test_cli_nonexistent_solution() {
    let missing = fixture_path("nonexistent-solution-file.csv");
    assert!(!missing.exists());
    let output = run_deal(&fixture_path("pyramid-synthetic-v2.json"), Some(&missing));
    assert!(assert_input_failure(&output).contains("nonexistent-solution-file.csv"));
}


/// Header names, their order and field count must match the agreed contract.
#[test]
fn test_cli_refuse_wrong_csv_headers() {
    for (label, contents) in [
        ("empty file", ""),
        ("header only", CSV_HEADER),
        ("old columns", "ACTION,CARD1,CARD2\nMOVE,,\n"),
        ("wrong case", "sequence,ACTION,CARD1,CARD2\n1,MOVE,,\n"),
        ("wrong order", "ACTION,SEQUENCE,CARD1,CARD2\nMOVE,1,,\n"),
        ("duplicate name", "SEQUENCE,ACTION,CARD1,CARD1\n1,MOVE,,\n"),
        (
            "extra column",
            "SEQUENCE,ACTION,CARD1,CARD2,EXTRA\n1,MOVE,,,\n",
        ),
    ] {
        assert_csv_rejected(label, contents.as_bytes());
    }
}


/// Short and long data records are refused rather than shifting card positions.
#[test]
fn test_cli_refuse_wrong_csv_record_width() {
    for record in ["1,MOVE,\n", "1,MOVE,,,\n"] {
        assert_csv_rejected(record, format!("{CSV_HEADER}{record}").as_bytes());
    }
}


/// Sequences must be nonzero, unique and contiguous from one without overflow.
#[test]
fn test_cli_refuse_invalid_sequences() {
    for (label, records) in [
        ("zero", "0,MOVE,,\n"),
        ("duplicate", "1,MOVE,,\n1,MOVE,,\n"),
        ("gap", "1,MOVE,,\n3,MOVE,,\n"),
        ("missing one", "2,MOVE,,\n"),
        ("negative", "-1,MOVE,,\n"),
        ("nonnumeric", "first,MOVE,,\n"),
        ("overflow", "999999999999999999999999999999999,MOVE,,\n"),
    ] {
        assert_csv_rejected(label, format!("{CSV_HEADER}{records}").as_bytes());
    }
}


/// Action names and required/forbidden card fields are validated before display.
#[test]
fn test_cli_refuse_invalid_action_fields() {
    for (label, record) in [
        ("unknown", "1,DRAW,,\n"),
        ("lowercase", "1,move,,\n"),
        ("MATCH missing first", "1,MATCH,,DR\n"),
        ("MATCH missing second", "1,MATCH,DL,\n"),
        ("MATCH same pile", "1,MATCH,DL,DL\n"),
        ("MATCH same slot", "1,MATCH,7|1,7|1\n"),
        ("KING missing", "1,KING,,\n"),
        ("KING second", "1,KING,DL,DR\n"),
        ("MOVE first", "1,MOVE,DL,\n"),
        ("MOVE second", "1,MOVE,,DR\n"),
        ("RECYCLE first", "1,RECYCLE,DL,\n"),
        ("RECYCLE second", "1,RECYCLE,,DR\n"),
    ] {
        assert_csv_rejected(label, format!("{CSV_HEADER}{record}").as_bytes());
    }
}


/// Coordinates enforce one-based row/column limits and the triangular row shape.
#[test]
fn test_cli_refuse_invalid_card_locations() {
    for location in [
        "0|1",
        "8|1",
        "1|0",
        "1|2",
        "3|4",
        "7|8",
        "7|7|1",
        "1:1",
        "dl",
        "DRAW",
        "-1|1",
        "999999999999999999999999|1",
    ] {
        let contents = format!("{CSV_HEADER}1,KING,{location},\n");
        assert_csv_rejected(location, contents.as_bytes());
    }
}


/// The apex and both bottom corners are accepted without gameplay replay checks.
#[test]
fn test_cli_accept_triangular_boundaries() {
    let temporary = TempFileGuard::create(
        "csv",
        b"SEQUENCE,ACTION,CARD1,CARD2\n1,KING,1|1,\n2,MATCH,7|1,7|7\n",
    );
    let output = run_deal(
        &fixture_path("pyramid-synthetic-v2.json"),
        Some(temporary.path()),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
    let rows = solution_rows(&stdout);
    assert_eq!(&rows[0][..4], &["1", "KING", "1|1", "-"]);
    assert_eq!(&rows[1][..4], &["2", "MATCH", "7|1", "7|7"]);
}


/// Invalid UTF-8 is rejected instead of silently replacing damaged card fields.
#[test]
fn test_cli_refuse_invalid_utf8_csv() {
    let mut contents = CSV_HEADER.as_bytes().to_vec();
    contents.extend_from_slice(b"1,KING,");
    contents.push(0xff);
    contents.extend_from_slice(b",\n");
    assert_csv_rejected("invalid UTF-8", &contents);
}


/// CSV byte bounds are enforced independently of the JSON decoder.
#[test]
fn test_cli_refuse_oversized_csv() {
    assert_csv_rejected("65,537-byte CSV", &vec![b' '; 65_537]);
}


/// More than 4,096 records fails even when total bytes remain below the file cap.
#[test]
fn test_cli_refuse_too_many_solution_records() {
    let mut contents = CSV_HEADER.to_owned();
    for sequence in 1..=4097 {
        use std::fmt::Write as _;
        writeln!(contents, "{sequence},MOVE,,").expect("append CSV record");
    }
    assert!(contents.len() <= 65_536);
    assert_csv_rejected("4,097 records", contents.as_bytes());
}


/// Failed report writes produce a contextual failure on Linux's full-device sink.
#[cfg(target_os = "linux")]
#[test]
fn test_cli_output_write_failure() {
    let sink = OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open Linux full sink");
    let output = Command::new(bin_path())
        .arg(fixture_path("pyramid-synthetic-v2.json"))
        .stdout(sink)
        .output()
        .expect("execute console with failed stdout sink");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("stdout"));
}


/// An early-closing output consumer is a successful pipe termination without errors.
#[test]
fn test_cli_broken_pipe_is_success() {
    let mut contents = CSV_HEADER.to_owned();
    for sequence in 1..=4096 {
        use std::fmt::Write as _;
        writeln!(contents, "{sequence},MOVE,,").expect("append CSV record");
    }
    assert!(contents.len() <= 65_536);
    let temporary = TempFileGuard::create("csv", contents.as_bytes());
    let mut child = Command::new(bin_path())
        .arg(fixture_path("pyramid-synthetic-v2.json"))
        .arg("--solution")
        .arg(temporary.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn console with early-closing output consumer");
    drop(child.stdout.take());
    let output = child.wait_with_output().expect("wait for broken-pipe exit");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}
