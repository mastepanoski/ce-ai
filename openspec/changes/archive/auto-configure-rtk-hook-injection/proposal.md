# Proposal: Auto-configure rtk hook injection for natively-supported harnesses

## 1. Problem Statement
`rtk` (CLI output token reduction engine) currently has **no auto-registration mechanism** in `ce-ai`. Unlike companion MCP servers (`codegraph`, `engram`) which are wired into harness JSON/TOML configuration files during `ce-ai install` and `ce-ai sync`, `rtk` is a stdout pre-processor wired via agent execution hooks (`PreToolUse`, command rewrites). It is currently surfaced only as:
- An `Info`-level line in `ce-ai audit` (`CliCompressionDetector` in `src/commands/audit.rs`).
- A manual suggestion in `ce-ai tools status` (`ce-ai tools install rtk`).

As a consequence, users must manually discover, install, and wire `rtk` into their agents using `rtk init --global`. Furthermore:
1. **Harness Disparity**: RTK officially supports **Claude Code, Cursor, Copilot, and Codex** (along with Gemini CLI, Aider, Windsurf, Cline). It does **not** officially cover `Opencode`, `Custom`, `Deepseek`, `Pi`, `Grok`, `Kimi`, `Agy`, or `Fx`.
2. **Blast Radius & Silent Drop Risk**: Because RTK hooks intercept shell command execution and summarize output, a misconfigured or overly-aggressive filter can silently swallow stdout with exit code 0 (reproduced empirically with `rtk gh issue view <n> --comments`). In automated, scripted, or CI contexts, silent output loss is more catastrophic than a loud failure.
3. **No Opt-Out Mechanism**: Without explicit opt-out flags (`--skip-rtk` / `--skip-companions`) or environment variables (`CE_AI_SKIP_RTK` / `CE_AI_SKIP_COMPANIONS`), users cannot disable hook injection in environments where output buffering or filtering is undesirable.

## 2. In-Scope / Out-of-Scope Boundaries

### In-Scope:
- Implement `src/harness/rtk.rs` defining the RTK support matrix (`Claude`, `Cursor`, `Copilot`, `Codex`), hook configuration execution, hook detection, and opt-out evaluation.
- Auto-run the equivalent of `rtk init --global --agent <name>` during `ce-ai install` and `ce-ai init-prj` for natively-supported harnesses.
- Provide explicit, documented no-ops for unsupported harnesses (`Opencode`, `Custom`, `Deepseek`, `Pi`, `Grok`, `Kimi`, `Agy`, `Fx`) with zero failure risk.
- Ensure non-fatal behavior when the `rtk` binary is absent from `PATH` (diagnostic warning, continue execution).
- Implement explicit opt-out via CLI flags (`--skip-rtk`, `--skip-companions`) and environment variables (`CE_AI_SKIP_RTK=1`, `CE_AI_SKIP_COMPANIONS=1`) across `install` and `init-prj`.
- Support symmetric hook removal during `ce-ai uninstall` for supported harnesses.
- Escalate `CliCompressionDetector` in `src/commands/audit.rs` from `Info` to `Warn` for supported harnesses that lack the RTK hook or binary, while maintaining `Info` for unsupported harnesses.
- Add RTK hook health diagnostics to `src/commands/doctor.rs`, including documentation of the silent output filter failure mode.
- Bump SemVer to `1.42.0` (MINOR) and update `CHANGELOG.md`.

### Out-of-Scope:
- Companion MCP servers (`codegraph`, `engram`), which were completed in Issue #307.
- `sequential-thinking` integration (tracked in Issue #309).
- Modifying upstream `rtk` binary source code or altering RTK filter rules directly.

## 3. Risk Evaluation
- **Hook Side Effects in CI/Automated Environments**: Intercepting agent tool use can modify command outputs. Mitigated by explicit opt-out CLI flags and environment variables (`CE_AI_SKIP_RTK=1`, `CE_AI_SKIP_COMPANIONS=1`).
- **Missing Dependency Resilience**: If `rtk` is not installed on the system, neither `ce-ai install` nor `ce-ai init-prj` will fail; they emit a helpful diagnostic message and continue.
- **Cross-Platform & Directory Isolation**: In unit and integration tests, `$HOME` redirection ensures test harnesses never pollute the user's real `~/.claude`, `~/.cursor`, etc.
- **Pre-existing Config Preservation**: `rtk init -g` merges cleanly into existing `settings.json` or `hooks.json` without overwriting unrelated user configurations.

## 4. Success Criteria
- Unit tests verify RTK support matrix (`is_rtk_supported`), opt-out parsing (`is_rtk_opted_out`), and command construction.
- `ce-ai install --harness <supported>` configures RTK hook when `rtk` is present on `PATH`.
- `ce-ai install --harness <unsupported>` executes as a clean no-op.
- Flags `--skip-rtk` / `--skip-companions` and env vars `CE_AI_SKIP_RTK` / `CE_AI_SKIP_COMPANIONS` cleanly bypass RTK configuration.
- `ce-ai audit` produces `Warn` for supported harnesses missing RTK hooks, and `Info` for unsupported ones.
- `ce-ai doctor` verifies RTK status and documents the silent output filter risk.
- All DoD gates pass: `cargo fmt`, `cargo clippy`, `cargo test`, and `make e2e`.
