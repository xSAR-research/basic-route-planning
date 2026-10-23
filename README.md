# basic-route-planning

Version **0.8.5** passed **80 host tests** against xSAR **0.8.105**. The timing refactor names the existing 120-second default in [`src/parameters.rs`](src/parameters.rs), preserving CLI overrides and solver behaviour. See the [timing catalogue](https://github.com/xSAR-research/qmp-qemu-socket/blob/main/docs/Timing-Constants.md). The 0.8.4 manual acceptance below remains historical evidence.

Read a complete saved Pyramid deal and its optional recorded Solver CSV, display
both, and calculate a minimum-cost offline route with `--solve`. The console keeps
immutable card identities separate from mutable position and search state. It
reads files and sends no guest input.

The **0.8.4 solver passed all required host verification** after a checked-access
correction to the previously verified 0.8.3 solver revision: 80 tests passed with
none failed or ignored. On 19 October, Charlie followed its 49-action route on
the fresh native-v7 deal successfully, clearing the tableau and both piles.
**M01–M09 are Complete and accepted**; see the
[manual acceptance record](docs/Acceptance-0.8.4.md). The application records M10 guarded execution as accepted;
M11 gameplay verification remains separate.
The single master schedule is maintained separately at
the **local sibling-checkout reference**
`../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md`.

## Usage

```text
basic-route-planning <DEAL_JSON_PATH> [--solution <SOLUTION_CSV_PATH>] [--solve]
basic-route-planning --help
basic-route-planning --version
```

1. Display the saved original board, ordered DRAW inventory and recorded solution:

   ```text
   cargo run --locked -- Evidence/pyramid-recon-20261018-v6-01.json --solution Evidence/pyramid-recon-20261018-v6-01-solver.csv
   ```

2. Independently validate the recording, calculate a route and print the derived
   actions with card identities, targets, resulting pile tops and remaining tableau:

   ```text
   cargo run --release --locked -- Evidence/pyramid-recon-20261018-v6-01.json --solution Evidence/pyramid-recon-20261018-v6-01-solver.csv --solve
   ```

JSON-only invocation remains supported, including `--solve` without a recording.
Requested input files are loaded and validated before display. CSV sorting happens
in memory and preserves source bytes and each MATCH's CARD1/CARD2 selection order.
Plain inspection checks input structure; solving additionally requires any supplied
recording to pass independent legal replay and end with a cleared tableau.

## Positions and recording

Tableau coordinates use one-based fixed `row|column`: rows increase top to bottom,
columns left to right, and row `r` has columns `1..r`. Apex `1|1`; bottom row
`7|1` through `7|7`. Bounds are `1 <= column <= row <= 7`. Removal never renumbers
remaining cards. DRAW encounters are printed 1..24; encounter 1 is the initial
DL card before the first D press. Internal original card indices remain zero-based.

`DL` is DRAW PILE LEFT and `DR` is DRAW PILE RIGHT. In an action they identify
current visible pile tops, not fixed original stock slots. Values are Ace=1,
2..10, Jack=11, Queen=12 and King=13. The original deal retains suits even though
Pyramid matching uses ranks.

The exact recording header is:

```csv
SEQUENCE,ACTION,CARD1,CARD2
```

| ACTION | CARD1 | CARD2 | Meaning and logical cost |
| --- | --- | --- | --- |
| MATCH | `DL`, `DR` or `row\|column` | A distinct location | Remove two accessible cards totalling 13; cost 1. |
| KING | One location | Empty | Remove an accessible King; cost 1. |
| MOVE | Empty | Empty | Move DL to DR with D; cost 1. |
| RECYCLE | Empty | Empty | Turn the exhausted DRAW pile over with D; cost 1. |

Normal CSV quoting is supported. SEQUENCE must contain each integer 1..N once;
the reader rejects gaps/duplicates, restores sequence order in memory and reports
reordering. CSV input is limited to **65,536 bytes and 4,096 records**; the accepted
recording's 48 steps are not a universal solution-length restriction.

## Explicit offline rules

The candidate uses the reference profile from
[mchung94/pyramid-solver at c6dde73](https://github.com/mchung94/pyramid-solver/tree/c6dde73fd27ec817e2f1329f2a761b384cd8c663).
Both visible DL and DR cards may be played, including a DL/DR pair. Removing a top
reveals the next card in that pile. MOVE transfers the current DL top to DR.
RECYCLE requires empty DL and nonempty DR, preserves original encounter order
among remaining cards and may occur at most twice. The goal is to clear the
**28-card tableau**, with one unit per MATCH, KING, MOVE or RECYCLE.

A pair's logical cost is one, although independent guest play uses two ordered
clicks. Acquisition, restoration, elapsed time and QMP packets are outside this
objective. A minimum-cost route under this profile is not new proof that every
rule matches the current Windows guest. A one-tableau clear is separate from
Pyramid's complete three-board game.

## Search limits and outcomes

Only `--solve` accepts search-limit options:

| Option | Default | Bound |
| --- | --- | --- |
| `--max-states <N>` | 2,000,000 | Retained history/state records. |
| `--max-expansions <N>` | 2,000,000 | Expanded search states. |
| `--max-memory-mib <N>` | 512 | Conservative collection allowance in MiB. |
| `--max-seconds <N>` | 120 | Cooperative elapsed-time deadline in seconds. |

The collection estimate charges **1,024 bytes per retained history**. At 512 MiB,
this allowance permits at most **524,288 histories**, even with a larger state
limit. It is an estimate for search collections, not a bound on process RSS.
A* uses the rank-count lower bound and cached ancestor/descendant rank masks for
sound pruning. Dijkstra provides a zero-heuristic reference for manageable cases.

Solved, proved no solution, and interrupted outcomes are distinct. CLI exit codes
are 0 for success/help/version, 1 for input/replay/output errors, 2 for invalid
arguments, 3 for proved no solution, and 4 for a search limit/cancellation outcome.
A broken output pipe is handled as success. A resource stop proves neither
unsolvability nor a minimum route.

The elapsed-time budget and library cancellation are cooperative rather than
instantaneous. Bounded per-deal cache preparation occurs before the search timer
starts; reported search time excludes that preparation, file intake and output.

The library accepts an `AtomicBool` cancellation flag, checked cooperatively.
The CLI uses native operating-system Ctrl+C termination; it does not install a
signal-to-library-cancellation handler.

## Storage and independent validation

JSON uses `xsar::pyramid_json::read_from` and its **65,536-byte** document bound.
The shared decoder rejects unsupported schemas, malformed/contradictory card
records, duplicates and incomplete inventories. Exported restoration metadata
is provenance, not a new live observation.

`ConsoleData.deal` is the sole card-identity owner. The separate 52-byte occupancy
model retains Unknown/Occupied/Empty states. The owned `TriangularMatrix<T>` uses
`[T; 28]` with checked immutable/mutable access. Unknown children block exposure;
an occupied higher card needs both children Empty. Search uses a compact presence
mask/cursor/recycle state and bounded predecessor records rather than copying decks.

`parameters.rs` owns shared definitions/enums/constants, `utility.rs` checked
mappings, `position.rs` conservative occupancy, `rules.rs` pure transitions,
`search.rs` bounded search and `replay.rs` independent validation. Replay uses
explicit tableau presence and stock/waste vectors and does not call search
transitions, successors or heuristics. It checks both MATCH selections before
mutation and validates rank, exposure, pile order, recycle count and final clear.

The local xSAR dependency remains relative `../xsar`, defaults off, with
`pyramid_json`. It is unchanged at 0.4.100 for this candidate. A path dependency
is not a Git revision pin.

## Verification and acceptance

Version 0.8.4 passed package-scoped formatting, locked all-target/all-feature check
and full tests including ignored tests, strict private rustdoc, warnings-denied
Clippy, extra production Clippy restrictions and release build. **80 tests passed,
0 failed, 0 ignored:** 29 unit, 27 CLI, 9 position-consumer, 4 solver-CLI and 11
solver tests. The accepted 48-action recording passed independent replay; licensed
15/44-cost fixtures, reference-search comparison and bound/refusal cases passed.

The actual executable calculated an optimal 48-action route under the stated
profile, equal to the recorded 48, and independently replayed it to final tableau
0. Search reported 159,012 expanded, 318,985 generated, 185,472 retained and
26,460 peak frontier in 0.145 seconds. Source/evidence snapshots stayed unchanged
and the sibling xSAR remained clean. See
[Verification-0.8.4.md](docs/Verification-0.8.4.md) for commands, executable hash,
first-failure history and the actual output boundary.

On 19 October, Charlie accepted M05–M09 after successfully playing all 49 printed
actions on `pyramid-recon-20261019-v7-01`: 24 MATCH, 20 MOVE, 4 KING and 1 RECYCLE,
ending with tableau zero and both piles empty. The
[manual acceptance record](docs/Acceptance-0.8.4.md) preserves his supplied trace
and its original-deal checksum. This is a different deal from the historical
48-action host regression above.

The next implementation approval point is **M10: guarded independent guest
execution**. M10/M11 remain Planned. One manually cleared tableau does not
establish automated execution or a three-board GAME WIN. Versions stay unchanged;
this acceptance authorises no commit, push or deployment.

The prior accepted 0.3.2 milestone remains documented in
[its historical verification record](docs/Verification-0.3.2.md).
Licensed upstream fixtures and notices are listed in
[third_party/README.md](third_party/README.md).
