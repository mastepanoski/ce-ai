# Design: Multi-Harness Usage Ingestion Adapters

## System Architecture

```text
src/commands/usage.rs (sync)
        │
        ▼
src/harness/usage/mod.rs (UsageAdapter registry)
 ┌──────────────┼──────────────┬──────────────┐
 ▼              ▼              ▼              ▼
claude.rs   opencode.rs     codex.rs       pi.rs
(~/.claude) (~/opencode)   (~/.codex)     (~/.pi)
 └──────────────┴──────────────┴──────────────┘
        │
        ▼ Normalization
  UsageRecord (src/capture/ledger.rs)
        │
        ▼ Atomic Append & Dedup
.ce-ai/usage/<author>.jsonl
```

## Trait Definition (`src/harness/usage/mod.rs`)

```rust
pub trait UsageAdapter: Send + Sync {
    /// Canonical harness identifier matching HarnessKind (e.g. "claude", "opencode", "codex", "pi").
    fn harness_name(&self) -> &'static str;

    /// Checks whether local transcripts for this harness exist on disk.
    fn is_available(&self, home: &Path) -> bool;

    /// Discovers and parses all session transcripts, extracting normalized UsageRecords.
    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError>;
}
```

## Adapter Implementations

### 1. `src/harness/usage/claude.rs`
- Resolves: `<home>/.claude/projects/`
- Format: JSON Lines.
- Mappings:
  - `input_tokens`: `usage.input_tokens`
  - `output_tokens`: `usage.output_tokens`
  - `cache_read`: `usage.cache_read_input_tokens`
  - `cache_write`: `usage.cache_creation_input_tokens`
  - `reasoning_tokens`: `0`

### 2. `src/harness/usage/opencode.rs`
- Resolves:
  - Linux/macOS: `<home>/.local/share/opencode/sessions/` or `<home>/.config/opencode/sessions/`
  - Windows: `<APPDATA>/opencode/sessions/`
- Format: JSON session state files containing event arrays.
- Mappings:
  - `session_id`: `session.id` or filename UUID
  - `cwd_basename`: `session.workspace` or directory basename
  - `model`: `message.model`
  - `input_tokens`: `message.tokens.input`
  - `output_tokens`: `message.tokens.output`
  - `cache_read`: `message.tokens.cache_read`
  - `cache_write`: `message.tokens.cache_write`
  - `reasoning_tokens`: `message.tokens.reasoning`
- Availability guard: If a turn has no `tokens` object, skip it without manufacturing fake zero-token records.

### 3. `src/harness/usage/codex.rs`
- Resolves: `<home>/.codex/sessions/*.jsonl`
- Format: Turn events with OpenAI-format API responses.
- Mappings:
  - `session_id`: `session_id` from event header
  - `model`: `response.model`
  - `input_tokens`: `response.usage.prompt_tokens`
  - `output_tokens`: `response.usage.completion_tokens`
  - `cache_read`: `response.usage.prompt_tokens_details.cached_tokens`
  - `reasoning_tokens`: `response.usage.completion_tokens_details.reasoning_tokens`
  - `cache_write`: `0`

### 4. `src/harness/usage/pi.rs`
- Resolves: `<home>/.pi/agent/sessions/*.json`
- Format: JSON session turn objects.
- Mappings:
  - `session_id`: Session ID field or file stem
  - `model`: LLM model ID
  - `input_tokens`, `output_tokens`, `cache_read`

## Deduplication & Storage

- All records pass into `crate::capture::ledger::append_records(&ctx.config_dir, &author, &typed)`.
- Existing deduplication via `dedup_key` (`format!("{}|{}|{}", r.harness, r.session_id, r.timestamp)`) prevents duplicate entries on repeated runs of `ce-ai usage sync`.
- File writes use atomic append semantics.
