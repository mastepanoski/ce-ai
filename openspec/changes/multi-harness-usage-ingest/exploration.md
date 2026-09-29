# Exploration: Multi-Harness Usage Ingestion Adapters

## Technical Investigation

### Current State
In `src/commands/usage.rs`, `sync(ctx)` executes:
```rust
let claude_projects = std::path::PathBuf::from(&home).join(".claude/projects");
let records = crate::harness::usage::claude::read_usage(&claude_projects, None, &author, None)?;
```
`src/harness/usage/` only contains `claude.rs` and `mod.rs`.

### Provider Storage & Schema Analysis

1. **Claude Code (`claude`)**:
   - Location: `~/.claude/projects/<project_hash>/*.jsonl`
   - Format: JSON Lines. Each line is an event.
   - Usage Payload: `message.usage { input_tokens, output_tokens, cache_read_input_tokens, cache_creation_input_tokens }`.

2. **OpenCode (`opencode`)**:
   - Location: OpenCode stores session state and events in JSON files under `~/.local/share/opencode/` (Linux/macOS) or `%APPDATA%/opencode/` (Windows).
   - Format: Session JSON objects containing an array of message turns or event logs.
   - Usage Payload: Assistant response events contain token counts: `{ tokens: { input, output, reasoning, cache_read, cache_write } }` or provider-native payloads.
   - Quirk: Not all local providers report tokens; missing blocks must be skipped without creating zero-value records when no execution occurred.

3. **OpenAI Codex CLI (`codex`)**:
   - Location: `~/.codex/sessions/` or `~/.codex/history/` JSONL files.
   - Format: Turn events with OpenAI API completion responses.
   - Usage Payload: Standard OpenAI usage object:
     ```json
     "usage": {
       "prompt_tokens": 1200,
       "completion_tokens": 350,
       "total_tokens": 1550,
       "prompt_tokens_details": { "cached_tokens": 800 },
       "completion_tokens_details": { "reasoning_tokens": 50 }
     }
     ```
   - Mapping:
     - `input_tokens`: `prompt_tokens`
     - `output_tokens`: `completion_tokens`
     - `cache_read`: `prompt_tokens_details.cached_tokens`
     - `reasoning_tokens`: `completion_tokens_details.reasoning_tokens`

4. **Pi (`pi`)**:
   - Location: `~/.pi/agent/sessions/` JSON files.
   - Format: Array of agent turns with tool calls and completion statistics.
   - Usage Payload: Step execution metrics recording model, input tokens, output tokens, and timestamp.

## Evaluated Architectural Options

### Option 1: Ad-hoc Sequential Function Calls in `src/commands/usage.rs`
- Sequentially call `claude::read_usage()`, `opencode::read_usage()`, `codex::read_usage()`, `pi::read_usage()` in a single monolithic function.
- *Pros*: Quick to write initially.
- *Cons*: High coupling, fragile error propagation (one broken transcript aborts the entire sync), violates single responsibility principle, hard to test with mock directories.

### Option 2: Unified `UsageAdapter` Trait in `src/harness/usage/`
- Define a trait `UsageAdapter` with:
  ```rust
  pub trait UsageAdapter: Send + Sync {
      fn harness_name(&self) -> &'static str;
      fn is_available(&self, home: &Path) -> bool;
      fn read_records(&self, home: &Path, author: &str, since: Option<&str>) -> Result<Vec<UsageRecord>, CeError>;
  }
  ```
- Each harness implements this trait in its own isolated module (`claude.rs`, `opencode.rs`, `codex.rs`, `pi.rs`).
- The `sync` command loops over all registered adapters, queries each independently, logs per-harness summaries, and aggregates records into the ledger.
- *Pros*:
  - Complete isolation: a corrupted OpenCode file does not break Claude or Codex ingestion.
  - Testability: every adapter can be unit-tested in isolation against static JSON/JSONL fixtures.
  - Extensibility: adding new harnesses in the future requires only implementing the trait.

## Decision & Tradeoffs

We select **Option 2**. The `UsageAdapter` abstraction fits `ce-ai`'s existing adapter architecture (matching `HarnessAdapter`), encapsulates provider-specific path discovery, and ensures robust error containment across platforms.
