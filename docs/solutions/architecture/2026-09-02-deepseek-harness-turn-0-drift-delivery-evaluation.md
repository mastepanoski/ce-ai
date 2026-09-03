---
module: harness::deepseek
tags: [deepseek, dsh, hooks, cordis, session-start, zero-step, negative-finding]
problem_type: architecture
---

# DeepSeek Harness (dsh) Turn-0 Drift Delivery Evaluation

## Context & Objectives
Investigation of `deepseek-ai/deepseek-harness` (`dsh`) to determine whether a native lifecycle hook or plugin can deterministically deliver Turn-0 `RepoState` into the conversational context.

## Findings & Architectural Evidence

### 1. "Everything is a Plugin" (Cordis Architecture)
- DeepSeek Harness is built on the **Cordis** plugin framework. It does not use standalone declarative JSON/TOML configuration files for project hooks (unlike Claude Code's `.claude/settings.json`, Codex's `.codex/config.toml`, or Copilot's `.github/hooks/hooks.json`).
- Extensions are configured through Cordis profile bundles, YAML patch layers (`cordis.patch.yml`), or internal TypeScript plugins mounted into the Cordis tree.

### 2. External Hook Bridges (`dsh-hook-protocol`)
- `dsh` provides compatibility bridges (`dsh-hooks-claude-code` and `dsh-hooks-codex`) powered by the shared library `@deepseek-ai/dsh-hook-protocol`.
- These bridges emulate Claude Code or Codex hooks by consuming their respective configurations.

### 3. Detached Execution & Race Condition in `SessionStart`
- As documented in `packages/hooks/hooks-codex/README.md` and `packages/hooks/hooks-claude-code/README.md`:
  > *"SessionStart: This event runs before the first turn. Unlike other events, it does not generate a hook/invoked or hook/result record in the persistence log."*
  > *"Known Limitations: Detached Execution: Because the hook runs detached, there is a risk that the SessionStart context may miss the first request. The system includes a TODO(session-start-gating) to address this timing issue."*
- Because execution is detached from the active agent context and lacks gating, `SessionStart` cannot reliably guarantee that injected context arrives before the first LLM request is dispatched.

### 4. `ce-ai` Project Status
- In `ce-ai`, `HarnessKind::Deepseek` is currently de-scoped and returns an explicit usage exit code (`install.rs:89`, `uninstall.rs:82`):
  > *"deepseek harness is unsupported during developer preview (DeepSeek Harness 'dsh' uses YAML patch layers under ~/.dsh). Please use a supported native harness (opencode, claude, codex, copilot, cursor, grok, kimi, agy, pi, fx)."*

## Conclusion
DeepSeek Harness currently lacks a deterministic, native project-level mechanism for synchronous Turn-0 context injection. Forcing an implementation through Cordis YAML patch overlays or detached bridge hooks would introduce race conditions and violate `ce-ai`'s zero-symptom-patch invariant. DeepSeek remains governed by the textual Turn-0 directives in `AGENTS.md`.
