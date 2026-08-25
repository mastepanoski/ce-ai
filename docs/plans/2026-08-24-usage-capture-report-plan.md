---
type: feat
origin: docs/brainstorms/2026-08-24-usage-analytics-requirements.md
openspec: openspec/changes/usage-capture-report/
---

# feat: Usage Capture, Ledger and Reporting

## Summary

Deterministic token capture via pre-commit hook reading local harness persistence files (Claude JSONL first), normalized into a shard-per-author ledger committed to the repo, with aggregated reporting by date/session/model and coverage markers distinguishing captured from uncaptured periods. Human hours recorded via interactive commit-time prompt and manual `hours log`.

## Problem Frame

ce-ai orchestrates AI agents across multiple harnesses but has no visibility into what that orchestration costs. Each harness persists usage data locally in its own format (Claude: 363MB JSONL verified; OpenCode: SQLite with native token columns; Codex: rollout JSONL; Pi: session JSONL), but nothing aggregates across harnesses or correlates with projects and commits. Meanwhile, human review time — the other half of client billing — is entirely untracked. Without these metrics, model assignments cannot be evaluated for ROI and client work cannot be billed in tokens+hours.

---

## Scope

**In scope:** Claude Code JSONL adapter (Tier-A, highest real volume); shard-per-author ledger; pre-commit hook capture; `usage report` with coverage markers; `hours log/list`; consent gate.
**Out of scope:** OpenCode/Codex/Pi/Kimi/Antigravity adapters (Fase 2+, same trait pattern); cost amounts (external calculation); TUI panel; WakaTime/zeittracker integrations (enum ready).

---

## Implementation Units

### U1. Ledger schema + shard I/O
**Goal:** Append-only JSONL shard-per-author with dedup by deterministic key.
**Files:** `src/usage/ledger.rs` (new)
**Approach:** `UsageRecord` struct (serde). Shard path = `.ce-ai/usage/<git-user-slug>.jsonl`. Append via `write_atomic` per line. Read = parse all lines, dedup by `harness+session_id+window_start`.
**Patterns to follow:** Mirror `state::write_atomic` pattern for durability.
**Test scenarios:**
- Append record → file grows by one line
- Dedup: same harness+session+window appended twice → second is no-op
- Empty shard read → empty vec
- Corrupt line → skipped with warning

### U2. Marker manager
**Goal:** Per-project local-only marker tracking last-captured timestamp per source.
**Files:** `src/usage/marker.rs` (new)
**Approach:** `<config_dir>/markers/<hash(cwd)>.json` with `{last_captured: {claude: ts, opencode: ts, ...}}`. Bootstrap = current timestamp (no historical backfill).
**Patterns to follow:** Mirror state/mod.rs read/write pattern.
**Test scenarios:**
- First call returns None (bootstrap)
- After set, subsequent read returns stored timestamp
- Different cwd hash → independent markers

### U3. Claude JSONL adapter (incremental)
**Goal:** Extract UsageRecords from Claude transcripts newer than marker timestamp.
**Files:** `src/harness/usage/claude.rs` (new)
**Approach:** Stream-parse JSONL lines; filter entries where timestamp > marker AND usage fields present. Map `input_tokens/cache_read_input_tokens/cache_creation_input_tokens/output_tokens` + `model` + `sessionId` + timestamp to UsageRecord.
**Test scenarios:**
- Fixture JSONL with known usage → correct UsageRecord extraction
- Lines without usage fields → skipped
- Malformed JSONL line → warning + skip

### U4. `usage sync` command + hook wiring
**Goal:** Orchestrate adapter→ledger pipeline; wire into pre-commit hook.
**Files:** `src/commands/usage.rs` (new), `src/main.rs` (subcommand), `.githooks/pre-commit` (extend)
**Approach:** `usage sync` iterates Tier-A adapters with active local data, captures since marker, appends to shard, updates marker. Hook calls `ce-ai usage sync --quiet` before commit.
**Test scenarios:**
- Sync after Claude session → records in shard, marker updated
- Re-sync without new activity → no-op (marker unchanged)
- Sync with unreadable source → warning + remaining sources processed

### U5. `usage report` aggregation + coverage markers
**Goal:** Aggregate ledger by filters with SHA256-verified coverage markers per source/period.
**Files:** `src/commands/usage_report.rs` (new or part of usage.rs)
**Approach:** Read all shards, filter by date/model/user, aggregate tokens. Coverage matrix reports `captured`/`uncaptured` per source/period based on marker presence vs expected sources. Output table (human) or `--json`.
**Test scenarios:**
- Report after capture → totals match ledger sum
- Filter by date range → subset only
- `--json` → valid JSON with same data
- Missing source period → `uncaptured` marker

### U6. Hours log + interactive prompt
**Goal:** Record human time via commit-time prompt (TTY-gated) and manual backfill.
**Files:** `src/commands/hours.rs` (new or part of usage.rs)
**Approach:** Hook prompts minutes+activity when TTY available; HourRecord appended to same shard. `hours log` manual entry. `hours list` query.
**Test scenarios:**
- Interactive commit → prompt shown, HourRecord written
- Non-interactive (no TTY) → skipped silently, tokens still captured
- `hours log --minutes 90 --activity review` → record created

---

## Dependencies

- U1 ← U2, U4, U5, U6 (all write through ledger)
- U2 ← U3 (adapter needs marker to know incremental window)
- U4 ← U1, U2, U3 (orchestrates all)

## System-Wide Impact

- New `src/usage/` module (ledger, marker, adapters)
- New CLI subcommand group (`ce-ai usage`, `ce-ai hours`)
- Extended `.githooks/pre-commit`
- No changes to existing install/sync/uninstall paths
