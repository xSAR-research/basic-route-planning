# Changelog

Versions follow Charlie's `major.minor.build` policy. Build counters never reset;
proposed versions remain provisional until the required verification passes.

## 0.8.4 — publication summary, 23 October 2026

- Publish the accepted offline Pyramid solver work: bounded A*/Dijkstra, explicit
  stock/waste and recycle rules, independent route replay and `--solve` output
  with card identities, fixed one-based locations, pile updates and proof/search
  statistics. Preserve saved-deal and recorded-CSV display.
- Retain the strict four-column CSV contract, checked compact position storage,
  the accepted recordings and the pinned upstream fixtures with their MIT notice.
  The console performs no guest capture or input.
- Preserve console 0.8.4 source and its bounded defaults. Original qualification
  passed 80 tests; the later xSAR 0.8.104 private-consumer check also passed 80
  tests with only the private lock entry adjusted. These are retained results,
  not a new test run or a new console release.
- Synchronise the local xSAR lock entry to 0.8.104 for paired publication. The
  resulting lockfile is byte-identical to that tested private-consumer lockfile;
  all other lock entries, the manifest and Rust source remain unchanged.
- Preserve Charlie's successful manual 49-action v7 trial and M05–M09 acceptance.
  In the current master schedule M01–M10 are accepted; M11/full target-aware
  automation remain separate application work. Earlier dated entries below
  retain the milestone status and dependency versions at their original checkpoint.
- Preparing this Git checkpoint does not claim a new guest trial, automated
  execution by the console, package publication or an additional build increment.

## 0.8.4 — human acceptance, 19 October 2026

- Charlie followed the Rust-generated 49-action route on the newly acquired v7
  deal successfully: 24 MATCH, 20 MOVE, 4 KING and 1 RECYCLE, ending with tableau
  zero and both piles empty. Retain the supplied trace and exact deal identity in
  [Acceptance-0.8.4.md](docs/Acceptance-0.8.4.md).
- Mark M05–M09 Complete and accepted, retaining M01–M04 acceptance. M10/M11 remain
  Planned; guarded automated execution requires its separate M10 approval.
- Record human/live evidence separately from the earlier host verification,
  automated execution and a three-board game. Versions, minor-credit accounting
  and build counters are unchanged by this documentation/acceptance update.

## 0.8.4 — verified candidate, 18 October 2026

- Replace direct indexing in independent replay with checked access following
  final audit of the successfully verified 0.8.3 revision.
- Retain the same solver rules, limits and five M05–M09 minor credits. This
  changed revision earns only the next build increment after complete revalidation.
  At this host-verification checkpoint, M09 was the next human acceptance point.
- Pass all required checks and additional production safety Clippy restrictions:
  80 tests, zero failed/ignored. Actual-deal solve returns an independently replayed
  optimal 48-action route under the stated profile, equal to the recorded cost.
  M05–M09 awaited human acceptance at this checkpoint; no guest run or promotion
  was claimed by that host work. The later acceptance is recorded above.

## 0.8.3 — preceding solver revision, host verification passed

- Add bounded A* and zero-heuristic Dijkstra under the explicitly pinned upstream
  Pyramid rules: playable stock/waste tops, preserved recycle order, at most two
  recycles, tableau-clear goal and one logical unit per removal/Draw/Recycle.
- Add independent explicit-pile route replay for supplied recordings and calculated
  routes, without reusing search transitions or pruning.
- Extend the existing console with `--solve`, state/expansion/memory/time limits,
  card-and-target route output, resulting pile tops, remaining tableau and proof/
  interruption statistics. Preserve existing saved-input display.
- Retain compact immutable deal ownership and separate position/search state.
  Cache ancestor/descendant rank masks once per deal for sound pruning after an
  initial 44-cost fixture attempt reached the unchanged 512 MiB collection budget.
- Adapt the pinned upstream 15/44-cost fixtures with their full MIT notice;
  add independent accepted-recording replay and bounded solver/refusal checks.

Version accounting: verified 0.8.3 applies the five distinct M05–M09 minor credits
from 0.3.2 in one revision with one build increment. Its actual-deal route passed
independent replay with 48 derived versus 48 recorded logical actions. The later
checked-access correction is verified as 0.8.4; it does not reapply those minor
credits. At this checkpoint xSAR remained 0.4.100 and application Rust was 2.2.100;
the separate later v7 calibration release is 2.4.101. Current verification is in
[Verification-0.8.4.md](docs/Verification-0.8.4.md). No commit, push, deployment or
guest execution is authorised by the candidate.

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
