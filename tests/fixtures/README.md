# Synthetic console fixtures

`pyramid-synthetic-v1.json` and `pyramid-synthetic-v2.json` are verbatim copies
of xSAR's synthetic foundation-contract fixtures. Both describe the same ordered
standard deck using the two supported JSON schemas. Their provenance explicitly
records that no screen recognition or live restoration was performed.

The integration tests locate these fixtures through `CARGO_MANIFEST_DIR`; test
data needs no particular host path or retained live export. The package still
needs its configured xSAR source dependency to build.

CSV samples are small synthetic recordings constructed by the tests. They check
format, bounded input, coordinates, ordering and console presentation, without
asserting a legal or winning gameplay route. The acquired JSON and Charlie's
corresponding Solver recording remain separately in `Evidence/` for explicit
data-comparison acceptance.
