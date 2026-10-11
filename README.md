# basic-route-planning

Console route-planning workspace for card deal exploration.

Outcome 2 implements intake and presentation for complete Pyramid deals exported
by the reconnaissance producer via the shared `xsar::pyramid_json` codec.

## Usage

```bash
# Display help and usage information
basic-route-planning --help

# Load and print a complete Pyramid deal JSON document
basic-route-planning <DEAL_JSON_PATH>
```

## Features

- **Strict bounded ingestion:** Opens the file read-only and consumes up to
  65,536 bytes using `xsar::pyramid_json::read_from`. Refuses oversized files,
  syntax errors, duplicate cards, incomplete decks, unsupported schema versions,
  unknown fields, and contradictory schema-2 records.
- **Deterministic display:** Prints deal metadata (deal ID, declared provenance,
  producer, source, evidence ID, restoration status), all seven tableau rows
  (0..6, columns 0..row), and all 24 DRAW cards in original order (ordinals 0..23)
  with canonical card labels, numeric values 1..13, and suits.
- **Robust error handling:** Gracefully handles standard output broken pipes;
  returns exit code 0 for `--help`, exit code 2 for invalid CLI arguments, and
  exit code 1 for unreadable or invalid deal files.
