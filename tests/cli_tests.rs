//! Integration tests for basic-route-planning CLI.
//!
//! Tests argument handling, help flags, live input, synthetic fixtures,
//! and refusal of adverse/malformed inputs.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};


/// Return the path to the compiled binary under test.
fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_basic-route-planning"))
}


/// Helper to run the binary with arguments and return output.
fn run_cmd(args: &[&str]) -> std::process::Output {
    Command::new(bin_path())
        .args(args)
        .output()
        .expect("failed to execute binary")
}


/// Temporary file guard that deletes the file upon dropping.
struct TempFileGuard {
    path: PathBuf,
}


impl TempFileGuard {
    /// Create a temporary file with the given content outside the repository.
    fn create(name: &str, content: &[u8]) -> Self {
        let dir = std::env::temp_dir().join("basic-route-planning-tests");
        fs::create_dir_all(&dir).expect("failed to create temp test directory");
        let path = dir.join(name);
        let mut file = File::create(&path).expect("failed to create temp file");
        file.write_all(content).expect("failed to write temp file");
        Self { path }
    }


    /// Path to the temporary file.
    fn path(&self) -> &Path {
        &self.path
    }
}


impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}


/// Verify that --help displays usage on stdout with exit code 0.
#[test]
fn test_cli_help_flag() {
    let output = run_cmd(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: basic-route-planning <DEAL_JSON_PATH>"));
    assert!(stdout.contains("-h, --help"));
    assert!(output.stderr.is_empty());
}


/// Verify that -h displays usage on stdout with exit code 0.
#[test]
fn test_cli_short_help_flag() {
    let output = run_cmd(&["-h"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: basic-route-planning <DEAL_JSON_PATH>"));
    assert!(output.stderr.is_empty());
}


/// Verify that invoking with no arguments exits with code 2 and usage on stderr.
#[test]
fn test_cli_missing_arguments() {
    let output = run_cmd(&[]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing deal JSON file path"));
    assert!(stderr.contains("Usage: basic-route-planning"));
}


/// Verify that invoking with extra arguments exits with code 2 and error on stderr.
#[test]
fn test_cli_extra_arguments() {
    let output = run_cmd(&["path1.json", "path2.json"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected extra argument 'path2.json'"));
    assert!(stderr.contains("Usage: basic-route-planning"));
}


/// Verify that opening a nonexistent file exits with code 1 and error on stderr.
#[test]
fn test_cli_nonexistent_file() {
    let output = run_cmd(&["/nonexistent/path/to/deal-99999.json"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to open '/nonexistent/path/to/deal-99999.json'"));
}


/// Verify loading the accepted live deal file with full tableau and stock output.
#[test]
fn test_cli_accepted_live_deal() {
    let live_path = "/home/charlie/tmp/pyramid-recon-20261011-v5-012json";
    let output = run_cmd(&[live_path]);
    assert!(
        output.status.success(),
        "live deal execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );


    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deal ID:     pyramid-recon-20261011-v5-02"));
    assert!(stdout.contains("Provenance:  screen_recognition"));
    assert!(stdout.contains("Producer:    qmp-qemu-socket/2.2.0"));
    assert!(stdout.contains("Restoration: verified"));


    // Tableau first card: (0, 0) is 2 of diamonds (value 2)
    assert!(stdout.contains("  Row 0:"));
    assert!(stdout.contains("    col 0: 2 of diamonds (value: 2, suit: diamonds)"));


    // Tableau last card: (6, 6) is 4 of spades (value 4)
    assert!(stdout.contains("  Row 6:"));
    assert!(stdout.contains("    col 6: 4 of spades (value: 4, suit: spades)"));


    // Stock first card: ordinal 0 is K of diamonds (value 13)
    assert!(stdout.contains("  [00] K of diamonds (value: 13, suit: diamonds)"));


    // Stock last card: ordinal 23 is Q of diamonds (value 12)
    assert!(stdout.contains("  [23] Q of diamonds (value: 12, suit: diamonds)"));
}


/// Verify loading the synthetic fixture v1.
#[test]
fn test_cli_synthetic_fixture_v1() {
    let path = "/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v1.json";
    let output = run_cmd(&[path]);
    assert!(
        output.status.success(),
        "synthetic v1 execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );


    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deal ID:     synthetic-ordered-deck-v1"));
    assert!(stdout.contains("Provenance:  synthetic_fixture"));
    assert!(stdout.contains("Restoration: not_applicable"));
    assert!(stdout.contains("Evidence ID: synthetic-deck-v1"));
    assert!(stdout.contains("  Row 0:"));
    assert!(stdout.contains("    col 0: A of clubs (value: 1, suit: clubs)"));
}


/// Verify loading the synthetic fixture v2.
#[test]
fn test_cli_synthetic_fixture_v2() {
    let path = "/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v2.json";
    let output = run_cmd(&[path]);
    assert!(
        output.status.success(),
        "synthetic v2 execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );


    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deal ID:     synthetic-ordered-deck-v1"));
    assert!(stdout.contains("Provenance:  synthetic_fixture"));
    assert!(stdout.contains("Restoration: not_applicable"));
    assert!(stdout.contains("Evidence ID: synthetic-deck-v1"));
    assert!(stdout.contains("  Row 0:"));
    assert!(stdout.contains("    col 0: A of clubs (value: 1, suit: clubs)"));
}


/// Verify refusal of malformed JSON input.
#[test]
fn test_cli_refuse_malformed_json() {
    let temp = TempFileGuard::create("malformed.json", b"{ not valid json }");
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
}


/// Verify refusal of oversized input (> 65,536 bytes).
#[test]
fn test_cli_refuse_oversized_json() {
    let oversized = vec![b' '; 65_537];
    let temp = TempFileGuard::create("oversized.json", &oversized);
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
    assert!(stderr.contains("exceeds 65536 bytes"));
}


/// Verify refusal of JSON with duplicate cards in deck.
#[test]
fn test_cli_refuse_duplicate_card() {
    let base =
        fs::read_to_string("/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v2.json")
            .expect("read synthetic fixture v2");
    // Replace "2 of clubs" with "A of clubs" to create a duplicate
    let corrupted = base.replacen("\"2 of clubs\"", "\"A of clubs\"", 1);
    let temp = TempFileGuard::create("duplicate.json", corrupted.as_bytes());
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
    assert!(stderr.contains("duplicate"));
}


/// Verify refusal of unsupported schema version.
#[test]
fn test_cli_refuse_unsupported_version() {
    let base =
        fs::read_to_string("/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v2.json")
            .expect("read synthetic fixture v2");
    let corrupted = base.replacen("\"schema_version\": 2", "\"schema_version\": 3", 1);
    let temp = TempFileGuard::create("unsupported_version.json", corrupted.as_bytes());
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
    assert!(stderr.contains("schema"));
}


/// Verify refusal of unknown fields in JSON.
#[test]
fn test_cli_refuse_unknown_fields() {
    let base =
        fs::read_to_string("/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v2.json")
            .expect("read synthetic fixture v2");
    let corrupted = base.replacen(
        "{\n  \"schema_version\"",
        "{\n  \"unexpected\": true,\n  \"schema_version\"",
        1,
    );
    let temp = TempFileGuard::create("unknown_field.json", corrupted.as_bytes());
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
    assert!(stderr.contains("invalid Pyramid JSON syntax or schema"));
}


/// Verify refusal of contradictory schema-2 card label and numeric value.
#[test]
fn test_cli_refuse_contradictory_card() {
    let base =
        fs::read_to_string("/home/charlie/repo/RUST/xsar/tests/fixtures/pyramid-synthetic-v2.json")
            .expect("read synthetic fixture v2");
    // Change value from 1 to 5 for "A of clubs"
    let corrupted = base.replacen("\"value\": 1", "\"value\": 5", 1);
    let temp = TempFileGuard::create("contradictory.json", corrupted.as_bytes());
    let output = run_cmd(&[temp.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse Pyramid deal"));
    assert!(stderr.contains("invalid card"));
}
