# Proposal: Multi-Harness Usage Ingestion Adapters

## Problem Statement

Currently, `ce-ai usage sync` only supports Claude Code. It hardcodes transcript discovery to `~/.claude/projects/` and delegates exclusively to `src/harness/usage/claude.rs`. Although `ce-ai usage report` can display historical records stored in the ledger for any harness, no ingestion adapters exist for other supported AI coding harnesses such as OpenCode, Codex, or Pi.

Each AI coding harness stores session transcripts and token metrics in distinct locations and formats (or does not record them locally). Without dedicated per-harness adapters, developers using OpenCode, Codex, or Pi cannot synchronize or track their local token consumption via `ce-ai usage sync`.

## In Scope

- Implement standalone ingestion adapters in `src/harness/usage/`:
  - `src/harness/usage/opencode.rs`: Parse OpenCode session events/messages.
  - `src/harness/usage/codex.rs`: Parse Codex CLI session transcripts.
  - `src/harness/usage/pi.rs`: Parse Pi session logs.
- Normalize metrics from all providers into canonical `UsageRecord` (`input_tokens`, `output_tokens`, `cache_read`, `cache_write`, `reasoning_tokens`).
- Update `src/harness/usage/mod.rs` to expose all provider adapters through a unified interface.
- Extend `src/commands/usage.rs` `sync` to support multi-harness discovery (probing installed harnesses or accepting optional `--harness <name|all>`).
- Explicit handling of missing or unrecorded usage: detect when logs lack token fields without inventing metrics or masking errors.
- Test fixtures for each provider under `tests/fixtures/usage/` and comprehensive unit/integration test coverage.

## Out of Scope

- Remote cloud billing API scraping (e.g. querying Anthropic Console, OpenAI Platform, or cloud dashboards over HTTP). Ingestion is strictly local to on-disk harness transcripts.
- Real-time stream interception (proxying LLM network requests).
- Altering the on-disk format of existing shard ledgers (`.ce-ai/usage/<author>.jsonl`).
- Deprecating or modifying `ce-ai usage report` output schema.

## Risk Evaluation

1. **Transcript Format Drift**: Upstream harnesses (OpenCode, Codex, Pi) may evolve their JSON or log formats between releases.
   *Mitigation*: Write permissive serde parsers that ignore unneeded fields and fail gracefully without crashing when encounters unexpected line structures.
2. **Missing Token Fields**: Some local runs or models do not report prompt cache or reasoning tokens.
   *Mitigation*: Default missing optional token fields to 0, but explicitly skip or log warnings when entire usage payload is absent, never inventing artificial values.
3. **Cross-Platform Path Differences**: Windows, Linux, and macOS store application data in different standard paths (`~/.local/share`, `%APPDATA%`, `~/Library/Application Support`).
   *Mitigation*: Use cross-platform path resolution helpers (`dirs` or platform-aware home expansion).

## Success Criteria

1. `ce-ai usage sync` successfully ingests valid usage records from OpenCode, Codex, and Pi transcripts alongside Claude.
2. All extracted records validate against `UsageRecord` and deduplicate deterministically via `dedup_key`.
3. If an installed harness has no sessions or lacks usage metadata, `sync` reports that status without failure and without inventing metrics.
4. 100% test coverage with realistic fixtures for each supported harness.
