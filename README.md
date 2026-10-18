# basic-route-planning

Read and display a complete acquired Pyramid deal and its optional recorded
Solver solution. The console keeps the validated original deal and typed solution
in memory for subsequent offline work. It reads saved files; it does not capture
or control the guest.

Version **0.3.2** has passed the required host verification. Charlie accepted the
M03/M04 documentation and console presentation milestones on 18 October 2026.
The master implementation schedule records milestone status and Charlie's
separate acceptance. It is a local sibling-checkout reference at
`../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md`; that file is
maintained separately from this repository.

## Usage

```text
basic-route-planning <DEAL_JSON_PATH> [--solution <SOLUTION_CSV_PATH>]
basic-route-planning --help
basic-route-planning --version
```

From this checkout, the retained matched evidence can be displayed with:

```text
cargo run --locked -- Evidence/pyramid-recon-20261018-v6-01.json --solution Evidence/pyramid-recon-20261018-v6-01-solver.csv
```

The optional solution argument preserves JSON-only invocation. Input files are
read-only. Both requested files are loaded and validated before combined output;
SEQUENCE sorting occurs in memory and never rewrites the CSV.

The output includes deal identity and declared provenance/restoration metadata,
the original seven-row tableau with ranks, suits and numeric values, all 24 DRAW
cards in original encounter order, initial occupancy counts, the initially
accessible bottom-row locations, and an aligned recorded-action table. A warning
identifies solution rows whose file order differed from their SEQUENCE order.

## Positions and card values

Tableau locations use **one-based** `row|column`. Rows increase top to bottom,
columns left to right; row `r` contains columns `1..r`. The apex is `1|1`, and the
bottom row is `7|1` through `7|7`. These locations remain fixed after removals.
Bounds require `1 <= column <= row <= 7`.

The ordered DRAW listing numbers encounters **1..24**. Encounter **1 is the
initial DL card before any D key press**. Internal xSAR arrays remain zero-based.
`DL` means DRAW PILE LEFT and `DR` means DRAW PILE RIGHT. In a solution they refer
to the visible tops at that time, rather than a fixed stock encounter number.

Ranks have values Ace=1, 2..10, Jack=11, Queen=12, King=13. Suits are retained in
the original deal even though Pyramid matching uses rank values.

## Recorded solution CSV

The exact four-column header is:

```csv
SEQUENCE,ACTION,CARD1,CARD2
```

Normal CSV quoting is supported, including spreadsheet-exported quoted fields.
SEQUENCE contains each integer from 1 through the number of steps exactly once.
The reader rejects duplicates/gaps and displays steps in sequence order while
preserving CARD1/CARD2 order within each MATCH.

| ACTION | CARD1 | CARD2 | Recorded meaning |
| --- | --- | --- | --- |
| MATCH | `DL`, `DR` or `row\|column` | A distinct location in the same format | Match two cards whose values total 13. |
| KING | One location | Empty | Remove one King. |
| MOVE | Empty | Empty | Press D. |
| RECYCLE | Empty | Empty | Press D to recycle; recorded separately for clarity. |

For example:

```csv
SEQUENCE,ACTION,CARD1,CARD2
1,MATCH,7|6,7|7
2,MOVE,,
3,KING,DL,
```

This example illustrates the format and does not assert that these actions form
a valid route for an arbitrary deal. The console checks syntax, ordering, locations
and action fields. The later transition/replay milestone will check exposure,
rank sums, changing DL/DR contents, recycle legality and the final board state.
This intake milestone does not calculate an independent solution or prove optimality.

## Validation, storage and errors

- JSON uses `xsar::pyramid_json::read_from` and its **65,536-byte** document bound.
  The shared codec rejects unsupported schemas, malformed or contradictory card
  records, incomplete inventories and duplicate cards. The exported declaration
  `restoration=verified` is provenance, not a new live restoration observation.
- CSV has a separate **65,536-byte** input bound and **4,096-record** limit.
  Empty recordings, malformed headers, invalid fields/actions, duplicate/gapped
  sequences and out-of-bounds tableau locations are refused with context.
- `ConsoleData.deal` is the sole card-identity owner: fixed arrays for 28 tableau
  and 24 stock cards. `ConsoleData.position` adds exactly **52 bytes** of occupancy
  state, without copying cards. Typed solution steps occupy one ordered allocation.
  `parameters.rs` owns definitions/enums/constants; `utility.rs` owns checked
  mappings; `position.rs` owns mutable occupancy/accessibility. The public library
  exposes `input`, `parameters`, `position` and `utility` for consumer checks.
- `--help` and `--version` return exit code 0. Invalid command-line arguments return
  2; input or output failures return 1. A broken output pipe is handled gracefully.

The local xSAR dependency uses sibling `../xsar` with default features disabled
and `pyramid_json` enabled. Its revision/cleanliness is recorded during verification;
a path dependency is not a Git pin. Promotion dependency selection remains separate.

## Mutable position foundation

The owned `TriangularMatrix<T>` wraps `[T; 28]` and supports checked `get`,
`get_mut` and array inspection. `PyramidPosition::from_deal` initially marks all
52 original slots Occupied. Occupancy is a one-byte enum: Unknown, Occupied or
Empty. Card identities remain immutable in the complete deal.

An occupied bottom-row card is accessible. Above it, both children at
`(row + 1, column)` and `(row + 1, column + 1)` must be Empty. A parent that is
Unknown or Empty is not accessible, and an unknown child still blocks access.
Checked removal rejects invalid, covered, unknown or empty slots without mutation.
Unknown annotations are conservative and never imply removal.

Stock occupancy refers to original zero-based slots **0..23**. It does not track
the changing DL/DR tops or implement D, recycle, pair/King legality or a whole
solution replay. These transitions remain the later rules/replay milestone.

## Verification

Use the repository-selected toolchain. Required implementation checks are locked
Cargo check, full tests including ignored tests, strict private rustdoc, Clippy,
formatting and release build. Automated fixture paths are relative to the checkout.
The actual accepted evidence pair is also exercised through the resulting console
executable, followed by Charlie's visual comparison against the game.

On 18 October 2026, all required checks passed against unchanged, clean xSAR
`00986b8d621803fc98da85616444fc76c14638a9` (0.4.100): 10 unit tests, 27 CLI
tests and 9 public position-consumer tests, with no failures or ignored tests.
The release executable printed every card and all 48 recorded actions from the
retained evidence pair. See [the verification record](docs/Verification-0.3.2.md)
for commands, checksums, delivery paths and remaining acceptance boundaries.
