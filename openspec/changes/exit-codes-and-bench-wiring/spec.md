# Spec Delta
- **WHEN** state.json is corrupt/unreadable/unpersistable, **THEN** ce-ai
  exits with code 3.
- **WHEN** a remote fetch fails while propagating (tarball download),
  **THEN** ce-ai exits with code 5.
- **WHEN** filesystem I/O fails outside state.json, **THEN** exit code is 4.
- **WHEN** `make bench` runs, **THEN** both perf-bound tests execute and pass.
