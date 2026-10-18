# Console 0.3.2 verification and review

Pre-promotion verification checkpoint, 18 October 2026. Required host
verification passed; Charlie accepted M03/M04 on 18 October 2026. The master
schedule is maintained separately at the local sibling-checkout reference
`../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md`.

## Delivered behaviour

The console reads a complete saved deal through xSAR and optionally reads the
recorded solution CSV before printing either. It prints fixed one-based tableau
locations, all 24 original DRAW encounters, initial occupancy/accessibility and
the recorded actions in SEQUENCE order. Sorting preserves source bytes and each
MATCH's selection order. JSON-only invocation remains supported.

`CompleteDeal` is the sole owner of card identities. A separate 52-byte
`PyramidPosition` holds 28 tableau and 24 original stock-slot occupancies.
The checked owned `TriangularMatrix<T>` supports `get`, `get_mut` and array
inspection. Occupied bottom-row cards are accessible; other occupied cards
require both directly covering children Empty. Unknown blocks access. Invalid,
covered, empty and unknown removals return errors without changing state.

The stock portion records original zero-based slots 0..23. Dynamic DL/DR tops,
D/recycle transitions, rank legality and independent replay remain the full M06
work after agreement of M05 rules. Recorded action structure is validated here;
the saved route has not been independently replayed or proved optimal.

## Baselines and version outcome

- At this checkpoint, console HEAD was
  `2f10adc12de56ca064baa063999064ab9524f834` and the verified implementation was
  uncommitted. Manifest and lockfile agree on **0.3.2**. Subsequent promotion is
  recorded in Git history.
- xSAR remains **0.4.100**, clean at
  `00986b8d621803fc98da85616444fc76c14638a9`. Its compiler-facing files were
  hashed before and after final consumer verification and remained unchanged.
- qmp-qemu-socket remains **2.2.100**, HEAD
  `842646e24b179fb4c4ee2cd04f1dc28b8cfed30b`. This slice changes its documents
  and local standing instructions.
- Version 0.3.2 applies the two agreed M02/M04 minor credits from 0.1.1 with
  one qualifying build increment. Partial M06 consumes no additional minor credit.
  M01/M03 application-version credits remain pending qualifying application code.
- Dependencies add `csv 1.4.0` and `thiserror 2.0.21`. The only existing registry
  resolution changed is required `serde_json 1.0.152 → 1.0.154`, matching current
  xSAR's minimum. Other pre-existing registry versions/checksums were preserved.
- The verification work performed no commit, staging, push, installation, guest
  input or symlink/configuration change. The sibling dependency remains a path,
  not a Git pin.

## Newly run checks

Toolchain: `rustc 1.101.0-nightly (8006091eb 2026-10-17)`, Cargo
`1.101.0-nightly (3960fa69b 2026-10-12)`, host `x86_64-unknown-linux-gnu`.

All commands below passed in this checkout:

```text
cargo +nightly metadata --locked --format-version 1
cargo +nightly fmt --package basic-route-planning -- --check
cargo +nightly check --locked --all-targets --all-features
cargo +nightly test --locked --all-targets --all-features -- --include-ignored
RUSTDOCFLAGS='-D warnings -D missing_docs' cargo +nightly doc --locked --no-deps --document-private-items
cargo +nightly clippy --locked --all-targets --all-features -- -D warnings
cargo +nightly build --locked --release
```

Tests: **46 passed, 0 failed, 0 ignored** — 10 unit, 27 CLI integration and
9 public position-consumer tests. These cover every unique matrix index and
distinct stored value, genuine mutable access, invalid/extreme bounds, neither/
either/both child removal, unknown blocking, refusal without mutation and
unchanged original identities/source bytes. CLI cases include quoted CSV/CRLF,
sequence reordering, field/size/count refusals, broken pipes and output failures.

The resulting release executable passed `--version`, JSON-only input and the
matched JSON/CSV input. Independent comparison of its output checked all 28
tableau card labels/values, all 24 ordered stock entries, initial occupancy and
all 48 action/location rows: 21 MATCH, 22 MOVE, 4 KING and 1 RECYCLE. Both evidence
hashes and the consumer source snapshot remained unchanged through verification.

| Artefact | SHA-256 |
| --- | --- |
| `target/release/basic-route-planning` | `76dd88aba08e4f83ffa8689392636a3f5360858eed605f0a2959be8698759e09` |
| `Evidence/pyramid-recon-20261018-v6-01.json` | `a5187947b08836869caaa0b39407ed14d4b0af8bb71e225b5ba8495843903c94` |
| `Evidence/pyramid-recon-20261018-v6-01-solver.csv` | `eebb96e05b260616448f1e292615273ee0e3415970eb46edbaef4ffa0b50edf5` |

Final privately retained run identifier: `pyramid-final-validation-ajvpema5`.
Its `foreground.log`, `verification.json`, `accepted-pair.txt` and
`accepted-json-only.txt` preserve commands, checksums, source/dependency snapshots
and the actual console output. Reading saved files establishes no fresh live
restoration or game completion. Charlie's prior M01/M02 acceptance is separate.

## First failures and correction

The first lockfile guard stopped before code checks because current xSAR needs
serde_json 1.0.154. The next attempt found a test helper's invalid
`String::as_ptr` associated-function use; it was corrected to a method call.
Those failure logs remain privately retained under run identifiers
`pyramid-console-validation-lhejgd42` and `pyramid-position-validation-6fvwc7a1`.

Final review found an earlier `cargo fmt --all` had also formatted xSAR. Each of
its 22 changed files exactly matched formatting of its clean HEAD bytes. The
formatting edits were backed up and reversed with content guards; the Git index
was preserved. Correction evidence is in
the private run identifier `xsar-format-recovery-4wghifw1`. All required consumer checks
were then rerun against clean xSAR. Both standing instructions now restrict
formatting to the owning package. The formatter's `--all` option includes local
path dependencies, as documented in the
[official cargo-fmt source](https://github.com/rust-lang/rustfmt/blob/master/src/cargo-fmt/main.rs).

## Complete review paths and proposed promotion text

Console delivery paths, relative to this repository's root:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/Verification-0.3.2.md
src/lib.rs
src/main.rs
src/input.rs
src/parameters.rs
src/position.rs
src/utility.rs
tests/cli_tests.rs
tests/position_tests.rs
tests/fixtures/README.md
tests/fixtures/pyramid-synthetic-v1.json
tests/fixtures/pyramid-synthetic-v2.json
Evidence/pyramid-recon-20261018-v6-01.json
Evidence/pyramid-recon-20261018-v6-01-solver.csv
```

Proposed console commit title: `Add recorded solution display and checked Pyramid position model`

Proposed body: Read complete deals and bounded recorded CSV actions before
presentation. Add fixed one-based layout/DRAW output and a separate occupancy
model with checked mutation and conservative exposure. Reconcile the xSAR lock
entry and portable fixtures; apply M02/M04 version credits as 0.3.2. Required
host checks and 46 tests pass; human acceptance and full route replay remain separate.

Application documentation is a separate delivery. Local sibling-checkout paths
are listed here for coordination and are outside the console promotion:

```text
../qmp-qemu-socket/docs/Codex-Current-Status.md
../qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
```

Proposed application documentation commit title: `Record Pyramid evidence, milestones and console verification`

Proposed body: Index the accepted matched acquisition/recording, maintain all
eleven minor milestones and record the verified console/position foundation.
Preserve host/live acceptance boundaries, remaining M05/M06 work and version credits.

Local ignored instruction files were also refreshed and backed up; keep them
outside promotion:

```text
AGENTS.md
../qmp-qemu-socket/AGENTS.md
```

Original and intermediate instruction backups remain in the private directory
identified by `pyramid-agents-backup-av1cll4s`. The matched Evidence files are
included unchanged. Other pre-existing Evidence files, including the two ODS
workbooks, and `Game-Win-Handoff.md` are outside these delivery paths.
