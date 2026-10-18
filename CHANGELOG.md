# Changelog

Versions follow Charlie's `major.minor.build` policy. Build counters never reset;
proposed versions remain provisional until the required verification passes.

## 0.3.2 — 18 October 2026, verified locally

- Add optional `--solution` input for the four-column recorded Solver CSV and a
  version option, preserving JSON-only invocation.
- Read and validate both requested inputs into owned Rust data before combined
  output. Retain the complete xSAR deal as sole card-identity owner and store
  compact typed solution steps in play order.
- Add an owned checked `TriangularMatrix<T>` and a separate 52-byte mutable
  tableau/stock occupancy model with Unknown, Occupied and Empty states. Validate
  both-child exposure and reject invalid/covered/unknown/empty removals without
  mutation. Original stock-slot states do not implement draw/recycle replay.
- Expose public input, parameter, position and utility modules for consumer checks.
  Print initial occupancy and accessible tableau locations in the console.
- Show fixed one-based tableau locations, all 24 original DRAW encounters and a
  readable recorded-action table. Recover SEQUENCE order in memory, report
  reordering and reject duplicate/gapped sequences without modifying the CSV.
- Add `parameters.rs` definitions/enums/constants and `utility.rs` checked forward
  and inverse row/column mappings. Bound CSV bytes/records and validate actions,
  field presence and triangular positions.
- Reconcile the xSAR lock entry with the selected sibling crate, including its
  required serde_json minimum 1.0.152 to 1.0.154, and replace host-specific
  automated fixture paths with portable paths. Preserve other existing registry
  resolutions.
- Refresh standing agent instructions and document input formats, storage and
  separate parser, route-validation and human-acceptance boundaries.

Version accounting: applies the agreed M02 and M04 minor increases to the console's
0.1.1 baseline in one qualifying changed revision, with one build increment.
M01/M03 belong to the acquisition application and retain their own pending version
application. The mutable position foundation starts part of M06; it does not
complete that milestone or consume its distinct minor increase.

Verification passed: formatting, locked all-target/all-feature check and tests
including ignored tests, strict private rustdoc, warning-free Clippy and release
build. All 46 tests passed (10 unit, 27 CLI, 9 public position-consumer), with
none ignored. The release executable also passed JSON-only and matched JSON/CSV
runs, checking all 28 tableau cards, 24 stock entries and 48 recorded actions.
See [the verification record](docs/Verification-0.3.2.md). Charlie accepted M03/M04
on 18 October 2026. Host checks and this acceptance do not establish a fresh
guest run or independent legal/optimal replay.
