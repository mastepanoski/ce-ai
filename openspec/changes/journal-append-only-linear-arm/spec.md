# Spec: Append-Only JSONL Operation Journal

## Requirements & Acceptance Criteria

### R1: Append-Only JSONL Disk Format
- **WHEN** `Journal::begin` initializes a fresh journal at `install-journal.json`,
- **THEN** it MUST write an initial single-line JSON header containing `command` and `started_at` terminated by a newline (`\n`).
- **AND** each subsequent mutation recorded via `Journal::arm` MUST be appended as an individual single-line JSON record terminated by a newline (`\n`).

### R2: Linear-Time Mutation Arming ($O(1)$ per arm)
- **WHEN** `Journal::arm` is called $N$ times,
- **THEN** each call MUST append ONLY the newly armed `RecordedOp` record to disk without reading, re-serializing, or re-writing previously recorded operations.
- **AND** the cumulative disk bytes written across $N$ mutations MUST grow linearly $O(N)$ rather than quadratically $O(N^2)$.

### R3: Durability Guarantee Before Target Mutation
- **WHEN** `Journal::arm` writes a `RecordedOp` to the journal file,
- **THEN** it MUST call `File::sync_all()` on the open journal file handle before returning `Ok(())`.
- **AND** the prior file content and applied state MUST be durably flushed to disk before the calling command performs any target filesystem mutation.

### R4: Resilience to Incomplete Trailing Records
- **WHEN** `Journal::begin` encounters an existing journal file whose final line is incomplete (e.g. truncated due to process termination mid-write),
- **THEN** it MUST ignore the incomplete trailing line.
- **AND** it MUST successfully recover and roll back all preceding complete `RecordedOp` entries in reverse order.
- **AND** if the header line itself is malformed/corrupt, it MUST emit a warning and treat the journal as absent without crashing.

### R5: Preserved Fault-Injection Semantics
- **WHEN** `CE_AI_FAIL_AFTER_WRITES=N` is configured in the environment,
- **THEN** `Journal::arm` MUST successfully append and sync mutation #$N+1$ to disk before returning `Err(CeError::Runtime(...))`.
- **AND** subsequent invocation of `Journal::begin` MUST roll back exactly the operations recorded up to that failure point.

### R6: Backward Compatibility with Legacy Single-Blob Format
- **WHEN** `Journal::begin` or `recorded_command` encounters a stale journal written in the legacy single-blob JSON format (`{"command": "...", "started_at": "...", "ops": [...]}`),
- **THEN** it MUST parse the legacy format, roll back the recorded operations, and clear the file.

### R7: Windows File Handle Closure Before Unlink
- **WHEN** `Journal::complete` is called,
- **THEN** it MUST close/drop the open `File` handle before attempting to delete the journal file at `path`.
