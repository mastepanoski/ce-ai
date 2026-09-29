# Specification: Multi-Harness Usage Ingestion Adapters

## Requirements

### Requirement 1: Multi-Harness Discovery & Ingestion
- **WHEN** `ce-ai usage sync` is executed without a `--harness` filter,
- **THEN** it MUST inspect all registered usage adapters (`claude`, `opencode`, `codex`, `pi`), query their availability on disk, and ingest all newly discovered records into the local author's ledger.

### Requirement 2: OpenCode Usage Normalization
- **WHEN** an OpenCode session contains assistant responses with token telemetry,
- **THEN** the adapter MUST extract records setting `harness = "opencode"` and mapping `input`, `output`, `cache_read`, `cache_write`, and `reasoning` tokens.
- **WHEN** an OpenCode event contains no token metrics,
- **THEN** the adapter MUST skip that event without creating fictitious zero-token records and without failing the sync.

### Requirement 3: Codex Usage Normalization
- **WHEN** Codex CLI session logs contain OpenAI API usage fields (`prompt_tokens`, `completion_tokens`, `cached_tokens`, `reasoning_tokens`),
- **THEN** the adapter MUST extract records setting `harness = "codex"`, mapping cached tokens to `cache_read` and reasoning tokens to `reasoning_tokens`.

### Requirement 4: Pi Usage Normalization
- **WHEN** Pi agent session files contain turn execution statistics,
- **THEN** the adapter MUST extract records setting `harness = "pi"`, mapping input and output tokens accurately.

### Requirement 5: Scoped Harness Ingestion
- **WHEN** `ce-ai usage sync --harness <name>` is executed,
- **THEN** it MUST only invoke the specified harness adapter.
- **WHEN** an unsupported harness name is supplied to `--harness`,
- **THEN** the command MUST exit with a `CeError::Usage` (exit code 2).

### Requirement 6: Idempotent Ledger Appending
- **WHEN** `ce-ai usage sync` runs repeatedly against unchanged session logs,
- **THEN** it MUST deduplicate entries via `dedup_key` (`harness|session_id|timestamp`) and append 0 duplicate records to the author's `.ce-ai/usage/<author>.jsonl` file.

### Requirement 7: Graceful Non-Blocking Missing Transcripts
- **WHEN** an installed harness has no sessions on disk or its directory does not exist,
- **THEN** the sync command MUST report that the harness has no records and continue processing the remaining harnesses without error (exit code 0).
