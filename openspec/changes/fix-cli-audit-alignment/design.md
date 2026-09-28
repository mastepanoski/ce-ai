# Design

## Usage filtering

`usage report` will parse `--from` and `--to` as RFC 3339 timestamps and apply
an inclusive timestamp filter to records read from all ledger shards. Invalid
timestamps or an inverted range return `CeError::Usage` (exit 2). `--by` is
not yet an aggregation feature; the command will accept only `record` (the
current row-per-record output) and reject other values with guidance rather
than silently accept a no-op flag.

## Truthful command surface

- `usage` help says capture is Claude Code transcript capture and report is
  filtered ledger output.
- `gate` describes enforcement by default and observe mode as advisory.
- `guard` describes persisted configuration/status, and a mismatching disable
  scope returns a usage error without changing state.
- `tools install` says it registers an MCP definition and does not install the
  companion binary.
- `audit` describes configuration hygiene heuristics rather than token
  measurement.

## Documentation

Add one Diátaxis reference page enumerating each top-level command and its
truthful boundary. Link it from README's documentation map without exceeding
the 100-line README limit. Correct invalid install, archive, and route
examples in user guides; explain the required `--harness` and positional task.

## Verification

Add hermetic command tests for date filtering, invalid filter input, and guard
scope mismatch. Run the full Rust quality matrix plus strict document lint.
