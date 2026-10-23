# Offline solver 0.8.4 candidate verification

Verified candidate checkpoint, 18 October 2026. **All required host verification
passed for 0.8.4: 80 tests, zero failed or ignored.** M01–M04 remain accepted.
At this checkpoint, M05–M09 awaited M09 console/game acceptance.

**19 October acceptance update:** Charlie successfully followed the derived
49-action route on the fresh v7 deal and cleared the tableau and both piles.
M01–M09 are now Complete and accepted. The separate
[acceptance record](Acceptance-0.8.4.md) preserves his trace and deal checksum.
M10/M11 remain Planned; M10's execution contract requires separate approval.
The host results below remain the original checkpoint and were not rerun for
this documentation/live-acceptance update. No version increment is earned.

The single master schedule is a **local sibling-checkout reference**, maintained
separately at `../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md` relative
to the console repository root. No commit, push, deployment or guest input is
part of this candidate handoff.

## Contract and scope

- Read a complete saved deal and optional recorded CSV. Preserve existing display.
  With `--solve`, independently validate the supplied recording before search and
  replay the derived route before reporting its legality/proof status.
- Use the pinned `mchung94/pyramid-solver` reference profile at
  `c6dde73fd27ec817e2f1329f2a761b384cd8c663`: both pile tops playable, stock/waste
  pairs allowed, preserved encounter order on turnover, no more than two recycles
  and a 28-card tableau-clear goal. MATCH/KING/MOVE/RECYCLE each cost one.
- Preserve immutable xSAR card identities, separate occupancy and compact search
  state. A* uses cached ancestor/descendant rank masks and a lower-bound heuristic;
  Dijkstra supplies a manageable zero-heuristic reference. Independent replay uses
  explicit piles, without calling packed-state transitions/successors/pruning.
- Output derived card identities and targets, after-action pile tops, remaining
  tableau, logical cost, statistics and solved/no-solution/interrupted status.
- Defaults: 2,000,000 states, 2,000,000 expansions, 512 MiB collection allowance,
  120 seconds. A conservative 1,024 bytes per history limits the default memory
  allowance to 524,288 histories; this is not a process RSS guarantee.
- Library cancellation and the elapsed deadline are cooperative. Bounded per-deal
  cache preparation occurs before the search timer starts; reported search time
  excludes preparation, input and output. CLI Ctrl+C remains native OS termination,
  without a signal-to-library cancellation handler.

This profile is an explicit offline contract, not newly observed proof of Windows
rules. Accepted-recording replay, derived-route verification, live effect and
full-game completion are separate claims.

## Versions and verification state

Console **0.8.4 is verified**. The preceding 0.8.3 revision passed full host
verification and applied the five distinct M05–M09 minor credits from accepted
0.3.2 with one build increment. A subsequent checked-access correction changes
Rust again, so this candidate required the next build increment only. The five
minor credits are not applied twice. Final required tests passed and established
build 4 for this changed revision. xSAR stays **0.4.100**, unchanged/clean at
`00986b8d621803fc98da85616444fc76c14638a9`. Application Rust stays **2.2.100**.
No new sibling source, protocol or JSON schema change is part of this candidate.

| Required evidence | Actual 0.8.4 result |
| --- | --- |
| Package-scoped formatting | Passed. |
| Locked all-target/all-feature check | Passed. |
| Full tests, including ignored tests | 80 passed, 0 failed, 0 ignored: 29 unit, 27 CLI, 9 position-consumer, 4 solver-CLI and 11 solver tests. |
| Strict private rustdoc | Passed with warnings and missing documentation denied. |
| Clippy with warnings denied | Passed across all targets/features. |
| Additional production safety Clippy | Library/binaries passed with indexing/slicing, unwrap, expect and panic denied. |
| Release build and executable version | Passed; executable reports 0.8.4. |
| Accepted 48-step independent replay | Passed; recording legally clears the tableau under the stated profile. |
| Upstream 15/44 costs and derived-route replay | Passed; actual 44-cost executable run also passed. |
| Manageable reference-search comparison | Passed. |
| Bound/cancellation/refusal checks | Passed in the full suite. |
| Actual executable derived route | Optimal under the stated profile; independent replay passed; 48 actions, final tableau 0, equal to recorded 48. |
| Source/evidence/dependency preservation | Source and matched evidence snapshots unchanged through the run; all tracked xSAR hashes unchanged and xSAR clean. |
| M09 human acceptance | Awaited at the 18 October host checkpoint; accepted on 19 October in the separate acceptance record. |
| Live guest execution/compatibility | No live run by the original host work. Charlie subsequently reported successful manual 49-action play on the fresh v7 deal; automated execution remains unimplemented. |

Actual-deal search: **159,012 expanded, 318,985 generated, 185,472 retained,
26,460 peak frontier, 0.145 seconds** in the retained release output.
Executable SHA-256:
`ebafa907ffe1936a71be36823d1db7ddedeeba1cc8787d5c65cc26d4fd6431cf`.

Toolchain: `rustc 1.101.0-nightly (8006091eb 2026-10-17)`, Cargo
`1.101.0-nightly (3960fa69b 2026-10-12)`, host `x86_64-unknown-linux-gnu`.
The following commands passed in this repository:

```text
cargo +nightly fmt --package basic-route-planning -- --check
cargo +nightly check --locked --all-targets --all-features
cargo +nightly test --locked --all-targets --all-features -- --include-ignored
RUSTDOCFLAGS='-D warnings -D missing_docs' cargo +nightly doc --locked --no-deps --document-private-items
cargo +nightly clippy --locked --all-targets --all-features -- -D warnings
cargo +nightly clippy --locked --lib --bins -- -D warnings -D clippy::indexing_slicing -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
cargo +nightly build --locked --release
target/release/basic-route-planning --version
target/release/basic-route-planning Evidence/pyramid-recon-20261018-v6-01.json --solution Evidence/pyramid-recon-20261018-v6-01-solver.csv
target/release/basic-route-planning Evidence/pyramid-recon-20261018-v6-01.json --solution Evidence/pyramid-recon-20261018-v6-01-solver.csv --solve
target/release/basic-route-planning tests/fixtures/upstream-44.json --solve
```

Private final run identifier: `pyramid-solver-final-meo3ifw4`. Its
`verification.json`, named command logs, `accepted-display.log`,
`accepted-derived-route.log` and `upstream44-derived-route.log` preserve the
actual results. These current results are separate from preceding 0.8.3 evidence.

## Inputs and licence

| Input | Retained SHA-256 |
| --- | --- |
| `Evidence/pyramid-recon-20261018-v6-01.json` | `a5187947b08836869caaa0b39407ed14d4b0af8bb71e225b5ba8495843903c94` |
| `Evidence/pyramid-recon-20261018-v6-01-solver.csv` | `eebb96e05b260616448f1e292615273ee0e3415970eb46edbaef4ffa0b50edf5` |

These accepted M01/M02 identities were rechecked unchanged through final candidate
verification. The recorded route has 48 actions; its independent legal replay
passed separately from the newly derived 48-action optimal route. Synthetic upstream decks
in `tests/fixtures/upstream-15.json` and `tests/fixtures/upstream-44.json` preserve
reference expectations of 15 and 44 logical actions. See the
[attribution](../third_party/README.md) and
[full MIT notice](../third_party/pyramid-solver-LICENSE).

## First failure and correction

The first 44-cost fixture attempt reached the existing 512 MiB collection allowance
at 524,288 histories. It established a resource stop, not an impossible deal.
Ancestor/descendant rank-mask pruning was strengthened and cached once per deal;
the same budget is retained. The corrected 0.8.3 revision then passed full host verification. Its actual-deal
route was optimal under the reference profile and passed independent replay:
48 derived versus 48 recorded actions; 159,012 expanded states, 318,985 generated
transitions, 185,472 retained histories, peak frontier 26,460, approximately
0.133 seconds. Its executable SHA-256 was
`68ca73955d2a236edf2b7ca8d5d471c0a843eaf747eb906d62d21c9586bd0b3a`.

Final audit then identified direct indexing in the independent validator against
the checked-access policy. Correcting that implementation produced provisional
0.8.4 and required the complete verification again. All final checks, including
the additional production safety Clippy restrictions, then passed. The earlier
0.8.3 result remains separate from current 0.8.4 evidence. Private preceding run identifier:
`pyramid-solver-candidate-b8vih99e`; operational logs are not published here.

## Verified source snapshot

The candidate was uncommitted during verification. These SHA-256 values identify
its exact manifests, compiler-facing inputs and fixtures independently of a future
commit. They were unchanged across the final run. Documentation was completed
afterward from the retained results and is outside this compiler-input snapshot.

| Repository-relative path | SHA-256 |
| --- | --- |
| `Cargo.lock` | `018e804e4223958f7e58cf7d753c3f03bc80828e36b2252ef2c03d33d4500d48` |
| `Cargo.toml` | `5efd35585b1975cdadd72c69c7f6b18d5083cc84fc624ae468c25006c0e5e36e` |
| `rust-toolchain.toml` | `e28ffeaac8c80ee45b3c426e5a85a5869fb62a45e1daf75025f4f8be97713d3a` |
| `rustfmt.toml` | `f0d3774f75c7bebb65b7d4173cddf5c1ea57c5c272a5e12e873fcfdc012ac3b4` |
| `src/input.rs` | `064df149951560a6cfa02aeee3fe8dcdaaedd4d50f56fe3cf58f353d6755ec4f` |
| `src/lib.rs` | `51f5715c160cbcab2d2eb8cbe7445cefbfe65cfb43b7e2f119b64f9a1746915a` |
| `src/main.rs` | `8cd6d32c8c7cdf38fe19ee3886126413ec2ef42309eaa7463745732e8cd56fe2` |
| `src/parameters.rs` | `7e9262e70d4b01de77d707da39abdfb813f4b3390b3e06f2e837548e7992cfd9` |
| `src/position.rs` | `8c017b8281749109ef01796f0345e269f31082fffdbf3221e88b102f0cf56b88` |
| `src/replay.rs` | `6d3f44d515959626afc217b15c9c88b093a457d186fbf39ca2ab4657b2737c5d` |
| `src/rules.rs` | `caed3a03ca5f69a6aa97e28315628d222d6c42bd10182ee04ac265e574936750` |
| `src/search.rs` | `c8ed0848ffd7674acbcb4b250704822bd493e63a74f861fdf0b81ef120ee782e` |
| `src/utility.rs` | `58df4732a0cfb4ce596cc1e68e58d8cfe631c4a1200c7bedf14f7ddc1fa5b90e` |
| `tests/cli_tests.rs` | `e69a3ec393d977f6476870c4bab85941a8d44cb7163b3b860d12de2ca377de05` |
| `tests/fixtures/README.md` | `9330a5ef15f71275bcf1556d554d998b00ca50bf78848181ee44e5d7e9bac623` |
| `tests/fixtures/pyramid-synthetic-v1.json` | `2ae7f44d0aa41e7a381ac7dd9b88f4895c8e0770bf63b55713bd17ca28d653b3` |
| `tests/fixtures/pyramid-synthetic-v2.json` | `c5046abcf5f4e2353898de5c0e439ebab7299c302230cabe0baa3a2efb194863` |
| `tests/fixtures/upstream-15.json` | `7d4f35746dd6e7fc5ba18e6ad13f6638472e191a46e4de602939201f6f35a49a` |
| `tests/fixtures/upstream-44.json` | `43315bc03d5ca9f82de11a41399ab2166d2f1906c443372de2890d3ea073e56c` |
| `tests/position_tests.rs` | `3711477613c28b6689096a788dfc59409ed8896f2098b35c5f18370122c9561b` |
| `tests/solver_cli_tests.rs` | `53a8825c4bf51a894e64be7be32e1a8bce6c1c7c56af31ef7b64729391ac92aa` |
| `tests/solver_tests.rs` | `4f5a5070a3f30581bb6b17bb511112221f97cd43de985a8f6ab9213dc4f603e9` |

## Candidate review paths

The changed console paths below are relative to this repository root. This is
review scope only and authorises no staging, commit, push or deployment:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/Verification-0.8.4.md
docs/Acceptance-0.8.4.md
Evidence/pyramid-recon-20261019-v7-01.json
src/lib.rs
src/main.rs
src/parameters.rs
src/rules.rs
src/replay.rs
src/search.rs
tests/solver_cli_tests.rs
tests/solver_tests.rs
tests/fixtures/upstream-15.json
tests/fixtures/upstream-44.json
third_party/README.md
third_party/pyramid-solver-LICENSE
```

Unchanged review dependencies also included input/position/utility modules,
existing CLI/position tests, synthetic fixtures and the toolchain/formatter
configuration listed in the source snapshot above. They are not additional
changed-file promotion paths. The matched original JSON/CSV are unchanged inputs.
Local AGENTS, both ODS workbooks, private logs and operational notes are outside
candidate promotion. The existing Game-Win-Handoff.md packet is unchanged and
excluded.

Application tracking documents are a **separate local documentation delivery**,
relative to this console checkout:

```text
../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
../qmp-qemu-socket/docs/Codex-Current-Status.md
../qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
```

Proposed future console commit title: `feat(pyramid): add bounded offline route search and replay`

Proposed body: Implement the explicit pinned reference rules, bounded A*/Dijkstra,
independent replay and derived-route console output. Retain complete deal ownership,
resource/cancellation outcomes and licensed 15/44 fixtures. Record 0.8.4 verification:
80 tests pass and the actual deal yields an independently replayed optimal 48-action
route under the stated profile on the historical v6 deal. Record subsequent
manual acceptance on the fresh v7 deal with its 49-action trace.

Proposed future application-documentation commit title:
`docs(pyramid): record verified solver and manual acceptance`

Proposed body: Keep all eleven minor milestones current, retain M01–M04 acceptance
and record M05–M09 host verification, manual acceptance, immutable inputs and
version accounting. Keep M10/M11 Planned, with M10 execution approval separate.
These proposed texts authorise no Git action.

## Acceptance and next implementation approval

M09 was accepted on 19 October after Charlie's successful manual trial on the
fresh v7 deal. The [separate record](Acceptance-0.8.4.md) distinguishes that
operator report from host checks, automated input and three-board completion.
Stop before **M10 implementation**: propose and obtain approval for its guarded
execution contract before changing input authority or sending guest input.
M10/M11 remain Planned. Commit, push and deployment remain Charlie's decisions.
