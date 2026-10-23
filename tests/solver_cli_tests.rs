//! Public executable checks for derived routes, bounds and refusal behaviour.

use std::{
    path::PathBuf,
    process::{Command, Output},
};


/// Resolve the licensed smallest upstream fixture portably.
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/upstream-15.json")
}


/// Run the actual console executable against the fixture and supplied options.
fn run(options: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_basic-route-planning"))
        .arg(fixture())
        .args(options)
        .output()
        .expect("run offline solver console")
}


/// An actual derived report contains a proven, independently replayed 15-step route.
#[test]
fn solver_cli_prints_derived_card_locations_and_proof() {
    let output = run(&["--solve"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("UTF8 report");
    assert!(text.contains("Derived solution (15 logical actions)"));
    assert!(text.contains("independent route replay PASS"));
    assert!(text.contains("two recycles"));
    assert!(text.contains("DL after"));
    assert!(text.contains("Final tableau: 0 cards"));
    let table = text.split("Sequence Action").nth(1).expect("derived table");
    let rows: Vec<_> = table
        .lines()
        .filter(|line| {
            line.split_whitespace()
                .next()
                .is_some_and(|word| word.parse::<usize>().is_ok())
        })
        .collect();
    assert_eq!(rows.len(), 15);
    assert!(rows.iter().all(|line| line.contains('=')));
    assert!(output.stderr.is_empty());
}


/// Each zero bound is a distinct interruption and never a false no-solution proof.
#[test]
fn solver_cli_reports_budget_exhaustion_with_distinct_exit_code() {
    for (option, label) in [
        ("--max-states", "state limit"),
        ("--max-expansions", "expansion limit"),
        ("--max-memory-mib", "memory limit"),
        ("--max-seconds", "time limit"),
    ] {
        let output = run(&["--solve", option, "0"]);
        assert_eq!(output.status.code(), Some(4), "{option}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains(label), "{text}");
        assert!(!text.contains("proven no solution"));
        assert!(!text.contains("Derived solution ("));
    }
}


/// Duplicate, malformed or unscoped limits are refused before opening any input.
#[test]
fn solver_cli_refuses_invalid_and_duplicate_options() {
    for options in [
        vec!["--max-states", "1"],
        vec!["--solve", "--solve"],
        vec!["--solve", "--max-seconds"],
        vec!["--solve", "--max-memory-mib", "18446744073709551615"],
        vec!["--solve", "--max-states", "+2"],
        vec!["--solve", "--max-states", "2", "--max-states", "3"],
    ] {
        let output = run(&options);
        assert_eq!(output.status.code(), Some(2), "{options:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
}


/// Solving refuses a structurally valid but illegal recording before output.
#[test]
fn solver_cli_rejects_unrelated_recording_before_presentation() {
    let csv = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Evidence/pyramid-recon-20261018-v6-01-solver.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_basic-route-planning"))
        .arg(fixture())
        .arg("--solve")
        .arg("--solution")
        .arg(csv)
        .output()
        .expect("run mismatched route comparison");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("independent replay failed"));
}
