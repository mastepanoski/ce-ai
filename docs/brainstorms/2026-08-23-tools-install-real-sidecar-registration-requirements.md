# Tools Install Real Sidecar Registration Requirements

- **Date:** 2026-08-23
- **Issue:** #158 (P0 — `ce-ai tools install` is a no-op that prints success without installing or registering anything)
- **Status:** Approved (Brainstorm Completed)
- **Scope Tier:** P0 Bug Fix & Sidecar Registration Engine

---

## 🎯 1. Overview & Problem Statement

Currently, `ce-ai tools install <tool>` prints two mock success lines (`println!("tools: '{tool}' MCP server registration completed successfully.")`) without performing any filesystem mutation, config merge, tool binary execution, or health verification.

This P0 defect causes `ce-ai status`, `ce-ai doctor`, and `ce-ai audit` to infer false readiness or missing dependencies while users believe the sidecar was successfully provisioned.

This feature fixes Issue #158 by turning `ce-ai tools install <tool>` into a **real, atomic, capability-verified provisioning engine**.

---

## 🚀 2. Goals & Acceptance Criteria

1. **Real Registration & Provisioning**:
   - `context7`: Merges `context7` MCP server definition into `opencode.json` (`mcpServers.context7`).
   - `codegraph`: Initializes CodeGraph index via `gentle-ai codegraph init --cwd <repo_root>` if inside a git project.
   - `engram`: Merges `engram` MCP server definition into `opencode.json`.
   - `rtk`: Merges `rtk` compressor configuration or hook definition into `opencode.json`.
2. **Preserve User Configurations (`write_atomic`)**:
   - Mutations target specific JSON keys (`mcpServers.<tool>`) using `crate::state::write_atomic`. Unmanaged user MCP servers and custom skills are preserved byte-for-byte.
3. **Loud Post-Probe Verification**:
   - After performing registration, `ce-ai tools install` executes a capability/health probe (`extract_tool_version`).
   - **HARD RULE**: If the probe fails or binary is absent, `ce-ai tools install` MUST fail with non-zero exit code (`CeError::Verification` or `CeError::Runtime`) and SHALL NEVER output a success message.
4. **Idempotence & Dry-Run Invariants**:
   - Re-running `tools install` twice on a clean workspace produces the same deterministic state.
   - `--dry-run` previews proposed JSON changes without modifying disk.

---

## 🔒 3. Scope Boundaries & Non-Goals

### In Scope
- `src/commands/tools.rs` refactoring.
- Per-tool registration logic (`context7`, `codegraph`, `engram`, `rtk`).
- Post-install capability verification.
- Unit and CLI integration test coverage in `tests/cli.rs`.

### Out of Scope / Non-Goals
- Attempting root system package manager installs (`brew install`, `apt install`) without user interaction. If a system binary is missing, `tools install` configures the harness entry, probes the binary, and fails loudly with explicit installation instructions if the binary is missing.

---

## 🔄 4. OpenSpec Handoff & Next Steps

This requirements document is frozen in `docs/brainstorms/2026-08-23-tools-install-real-sidecar-registration-requirements.md`.

Next phase: **Stage 2 (OpenSpec Definition)** in `openspec/changes/tools-install-real-sidecar-registration/`:
- `proposal.md`
- `exploration.md`
- `design.md`
- `spec.md`
- `tasks.md`
