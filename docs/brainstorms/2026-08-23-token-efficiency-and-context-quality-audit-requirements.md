# Token Efficiency & Context Quality Audit Requirements

- **Date:** 2026-08-23
- **Issue:** #117 (`ce-ai audit` subcommand for token-efficiency and context-quality auditing)
- **Status:** Approved (Brainstorm Completed)
- **Scope Tier:** Standard / Feature

---

## 🎯 1. Overview & Problem Statement

`ce-ai` currently lacks a first-class subcommand to measure and audit the **token efficiency** and **context quality** of an AI coding agent workspace across multiple harnesses (OpenCode, Claude Code, Cursor, Pi, etc.).

While `ce-ai doctor` enforces mandatory configuration invariants and fails on errors, there is no **advisory, scored audit** that evaluates whether token-saving compressors are wired, whether MCP server sprawl is under control, whether duplicate prompt blocks waste context, or whether code intelligence and persistent memory sidecars are available.

This feature introduces `ce-ai audit` as a **multi-harness, capability-based audit engine**.

---

## 🚀 2. Goals & Success Criteria

1. **Multi-Harness & Capability-Based**: Audit evaluates abstract capabilities (e.g. `cli-output-compression`, `persistent-memory`, `code-intelligence`) rather than hard-coded vendor names.
2. **Capability Matrix**:
   - `cli-output-compression` (Tokens): Checks CLI output compressor presence and interceptor hook wiring per harness (e.g. `rtk`).
   - `mcp-sprawl` (Tokens): Audits count of configured MCP servers per harness (WARN if >5 global servers).
   - `prompt-duplication` (Tokens): Detects duplicate prompt text blocks ($\ge$ 200 chars across $\ge$ 3 agent configs) and estimates wasted KB.
   - `persistent-memory` (Grounding): Audits persistent memory availability (`engram`).
   - `docs-grounding` (Grounding): Audits library docs provider (`context7`).
   - `code-intelligence` (Grounding): Verifies codebase index presence (`.codegraph/`).
   - `learnings-library` (Grounding): Verifies `<repo>/docs/solutions/` presence and doc count.
3. **Advisory & Scored Output**:
   - `run()` returns `Ok(())` by default (Exit 0).
   - Renders a categorized console output (`[repo]`, `[tokens]`, `[grounding]`, `[hygiene]`) with an overall percentage score (e.g. `score: 78% (7 pass / 2 warn / 0 fail)`).
4. **CLI Ergonomics & CI Support**:
   - Supports `--json` flag for machine-readable output.
   - Supports `--fail-under <pct>` flag (e.g. `--fail-under 80`) to exit with non-zero exit code if score falls below threshold in CI pipelines.
5. **Zero Mutation / Read-Only**: `ce-ai audit` never mutates configuration files or source code.

---

## 🔒 3. Scope Boundaries & Non-Goals

### In Scope
- `ce-ai audit` CLI subcommand.
- Capability registry (`Detector` trait pattern) in `src/commands/audit.rs`.
- Multi-harness config parsing (`HarnessKind::detect_installed_harnesses`).
- `--json` and `--fail-under <pct>` flags.
- Unit & CLI integration test coverage.

### Out of Scope / Non-Goals
- No automatic fixing or mutation during audit (fixing belongs to explicit commands like `ce-ai tools install` or `ce-ai sync`).
- No long blocking network calls during audit execution.

---

## 🛡️ 4. Risk Matrix & Management

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| **R1** | Hard-coding vendors breaks when user replaces RTK/Engram | High | Capability-based detectors (`satisfied-by: <tool>`); unrecognized tools emit INFO, not FAIL. |
| **R2** | Secret/API key leakage in audit output | High | Audit context parsing strips credentials; only tool presence/counts are output. |
| **R3** | Breaking CI pipelines unexpectedly | Medium | Advisory by default (Exit 0); `--fail-under` must be explicitly requested. |

---

## 📋 5. User-Facing CLI Ergonomics

### A. `ce-ai audit` Default Output
```text
== [ce-ai Agent Environment Audit] ==
harnesses detected: opencode, claude

[repo]      PASS code-intelligence        .codegraph/ index present
[repo]      PASS learnings-library        docs/solutions/ (12 docs)
[tokens]    WARN mcp-sprawl/opencode      10 servers configured globally (>5)
[tokens]    PASS cli-compression/claude   satisfied-by: rtk 0.45.0 (hook wired)
[tokens]    INFO cli-compression/stats    saved ~91% bash output bytes (rtk gain)
[grounding] PASS persistent-memory        satisfied-by: engram
[grounding] PASS docs-grounding           satisfied-by: context7
[hygiene]   WARN prompt-duplication/opencode  ~28KB duplicated across 12 agents

score: 78% (7 pass / 2 warn / 0 fail)
```

---

## 🔄 6. OpenSpec Handoff & Next Steps

This requirements document is frozen in `docs/brainstorms/2026-08-23-token-efficiency-and-context-quality-audit-requirements.md`.

Next phase: **Stage 2 (OpenSpec Definition)** in `openspec/changes/token-efficiency-and-context-quality-audit/`:
- `proposal.md`
- `exploration.md`
- `design.md`
- `spec.md`
