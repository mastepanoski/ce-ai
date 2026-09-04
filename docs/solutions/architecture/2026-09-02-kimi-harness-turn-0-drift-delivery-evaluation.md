---
module: harness::kimi
tags: [kimi, hooks, session-start, zero-step, negative-finding, global-only-config]
problem_type: architecture
---

# Kimi Code CLI Turn-0 Drift Delivery Evaluation

## Context & Objectives
Investigation of Kimi Code CLI (`kimi`) to determine whether a native lifecycle hook (`SessionStart` via `[[hooks]]` in `config.toml`) can deterministically deliver Turn-0 `RepoState` into the conversational context on project adoption.

## Findings & Architectural Evidence

### 1. Global-Only Hook Configuration
- Kimi Code CLI defines and reads lifecycle hooks exclusively from the user-level configuration file located at `~/.kimi-code/config.toml` (or `$KIMI_CODE_HOME/config.toml`).
- The CLI explicitly does **not** read project-local hook configurations (such as `<project>/.kimi-code/config.toml` or `<project>/.kimi/config.toml`).
- Project workspaces in Kimi Code CLI only recognize `.kimi-code/local.toml` (for directory remembering), `.mcp.json` (for MCP servers), and `AGENTS.md` (for instructions).

### 2. Project Isolation & Boundary Violation
- `ce-ai init-prj` is designed as a repository-level adoption command operating strictly within a target project directory (`target_dir`).
- Injecting a global hook into `~/.kimi-code/config.toml` during `ce-ai init-prj` would violate project isolation by mutating the user's host environment and triggering `ce-ai workflow resume` across all repositories, including those not adopted by `ce-ai`.

### 3. Hook Semantics & Output Ingestion
- Kimi Code CLI's `[[hooks]]` subsystem is built with fail-open semantics primarily evaluating exit codes (`0` for success, `2` for blocking operations).
- Standard output (`stdout`) from shell command hooks is consumed internally for flow control and permissions rather than being automatically displayed in the terminal UI or appended to conversational context without specific envelope structures.
- While plugins can declare `sessionStart.skill` loaders within `kimi.plugin.json`, standalone shell command hooks do not provide a reliable project-level context injection channel.

## Conclusion & Architectural Decision
Kimi Code CLI lacks a project-level lifecycle hook mechanism for synchronous Turn-0 context injection. Modifying user-global configuration files during project adoption would violate architectural boundaries and cause false triggers across unrelated projects.

Kimi Code CLI remains governed by the text-based Turn-0 Session Directives in `AGENTS.md` and `.kimi-code/AGENTS.md` injected during `ce-ai init-prj`.
