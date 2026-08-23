# OpenSpec Proposal: Token Efficiency & Context Quality Audit

- **Change:** `token-efficiency-and-context-quality-audit`
- **Issue:** #117
- **Author:** Antigravity AI
- **Date:** 2026-08-23
- **Status:** Proposed

---

## 🎯 1. Problem & Context

While `ce-ai doctor` validates environment invariants and fails on errors, `ce-ai` lacks an advisory, scored subcommand to measure **token efficiency** and **context quality** across all installed agent harnesses (OpenCode, Claude Code, Cursor, Pi, etc.).

Developers cannot easily check whether CLI compressors (RTK) are intercepting output, whether MCP server sprawl (>5 global servers) is wasting context tokens, whether duplicated prompt blocks are burning context, or whether code-intelligence and memory sidecars (CodeGraph, Engram, Context7) are present.

---

## 🚀 2. Proposed Solution

Introduce `ce-ai audit`, a multi-harness, capability-based audit engine.

1. **Capability Matrix**:
   - `cli-output-compression` (Tokens): Checks CLI output compressor presence and interceptor hook wiring per harness.
   - `mcp-sprawl` (Tokens): Checks count of configured MCP servers per harness.
   - `prompt-duplication` (Tokens): Detects duplicate prompt text blocks ($\ge$ 200 chars across $\ge$ 3 agent configs).
   - `persistent-memory` (Grounding): Checks persistent memory store (`engram`).
   - `docs-grounding` (Grounding): Checks library docs provider (`context7`).
   - `code-intelligence` (Grounding): Verifies repo code index (`.codegraph/`).
   - `learnings-library` (Grounding): Verifies `<repo>/docs/solutions/`.
2. **Scored & Advisory Output**:
   - Computes overall score percentage (e.g. `score: 78% (7 pass / 2 warn / 0 fail)`).
   - Always returns `Ok(())` by default (Exit 0).
3. **CLI Flags**:
   - `--json`: Renders machine-readable JSON report.
   - `--fail-under <pct>`: Fails with non-zero exit code if score falls below specified percentage in CI.

---

## 🔒 3. Scope Boundaries

### In Scope
- `ce-ai audit` subcommand and detector traits in `src/commands/audit.rs`.
- Multi-harness config parsing via `HarnessKind`.
- `--json` and `--fail-under <pct>` CLI flags.
- Comprehensive unit and CLI integration tests.

### Out of Scope / Non-Goals
- Read-only execution: no file mutations during audit.
- No network blocking calls.
