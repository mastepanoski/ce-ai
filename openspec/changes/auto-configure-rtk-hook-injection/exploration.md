# Exploration: RTK Hook Injection & Activation Mechanisms

## 1. Technical Investigation of RTK CLI Activation

`rtk` (Token Reduction Engine) operates by intercepting agent command execution via hooks rather than registering as an MCP server. Investigation of `rtk init` behavior across native agents revealed the exact command patterns and file side-effects:

### Harness Command Mapping & File Mutations
1. **Claude Code**:
   - Command: `rtk init -g --auto-patch --agent claude`
   - Config file: `$HOME/.claude/settings.json`
   - Injected Hook: `PreToolUse` matcher `"Bash"` -> `{"type": "command", "command": "rtk hook claude"}`
   - Artifacts: `$HOME/.claude/RTK.md` and `@RTK.md` reference in `CLAUDE.md`.
   - Removal: `rtk init -g --uninstall --agent claude`
2. **Cursor Agent**:
   - Command: `rtk init -g --auto-patch --agent cursor`
   - Config file: `$HOME/.cursor/hooks.json`
   - Injected Hook: `preToolUse` matcher `"Shell"` -> `{"command": "rtk hook cursor", "matcher": "Shell"}`
   - Removal: `rtk init -g --uninstall --agent cursor`
3. **GitHub Copilot**:
   - Command: `rtk init -g --copilot`
   - Config file: `$HOME/.copilot/hooks/rtk-rewrite.json`
   - Injected Hook: `PreToolUse` -> `{"type": "command", "command": "rtk hook copilot", "cwd": ".", "timeout": 5}`
   - Artifacts: `$HOME/.copilot/copilot-instructions.md`
   - Removal: `rtk init -g --uninstall --copilot`
4. **Codex CLI**:
   - Command: `rtk init -g --codex`
   - Config file: `$HOME/.codex/RTK.md`
   - Artifacts: `@.../RTK.md` reference in `$HOME/.codex/AGENTS.md`.
   - Removal: `rtk init -g --uninstall --codex`

### Upstream Directory Precondition Quirk
Empirical testing revealed that `rtk init -g` unconditionally attempts to create a temp file in `$HOME/.claude/` (even when invoked for `--agent cursor`). If `$HOME/.claude` or the target harness directory does not exist, `rtk` fails with:
```
rtk: Failed to create temp file in .../.claude: No such file or directory (os error 2)
```
**Architecture Rule**: The `ce-ai` RTK module must ensure `$HOME/.claude` and the target harness directory exist before spawning `rtk init`.

---

## 2. Support Matrix Analysis & Architectural Tradeoffs

### Comparison Across Supported vs Unsupported Harnesses
| Harness | Supported by `ce-ai` | Supported by `rtk` | Action during `install`/`init-prj` |
| :--- | :--- | :--- | :--- |
| **Claude** | Yes | Yes | `rtk init -g --auto-patch --agent claude` |
| **Cursor** | Yes | Yes | `rtk init -g --auto-patch --agent cursor` |
| **Copilot** | Yes | Yes | `rtk init -g --copilot` |
| **Codex** | Yes | Yes | `rtk init -g --codex` |
| **Opencode** | Yes | No | Explicit documented no-op |
| **Pi** | Yes | No | Explicit documented no-op (No-MCP/skills-only) |
| **Grok** | Yes | No | Explicit documented no-op |
| **Kimi** | Yes | No | Explicit documented no-op |
| **Agy** | Yes | No | Explicit documented no-op |
| **Fx** | Yes | No | Explicit documented no-op |
| **Deepseek** | Yes (de-scoped) | No | Explicit documented no-op |
| **Custom** | Yes | No | Explicit documented no-op |

### Evaluated Options for Unsupported Harnesses
- **Option A: Attempt generic wrapper script or shell aliasing**.
  - *Tradeoff*: High risk of interfering with harness execution, non-standard hook APIs, and fragile rollback.
  - *Verdict*: **Rejected**.
- **Option B: Explicit, documented no-op with clear diagnostic logging**.
  - *Tradeoff*: Zero risk of breakage, completely predictable, aligns with the contract that `ce-ai` only configures natively-supported integrations.
  - *Verdict*: **Adopted**.

---

## 3. Blast Radius & Silent Drop Failure Mode Investigation

### The Observed Failure Mode
During empirical testing of `rtk`, we observed that:
```bash
rtk gh issue view 308 --comments
```
exits with returncode `0` but writes **zero bytes to stdout and zero bytes to stderr**.
In contrast, raw `gh issue view 308 --comments` produces the full issue metadata.

### Root Cause
`rtk` includes specialized parsers for CLI commands (e.g. `gh`, `git`, `docker`, `cargo`). When a specific sub-command flag combination (such as `gh ... --comments`) fails to match RTK's expected JSON format or regex filters, the processor's output filter drops the output without raising a non-zero exit code.

### Architectural Mitigations
1. **Granular & Blanket Opt-Out**:
   - `--skip-rtk`: Skips only RTK hook injection.
   - `--skip-companions`: Skips all companion injection (both RTK and companion MCPs).
   - `CE_AI_SKIP_RTK=1` & `CE_AI_SKIP_COMPANIONS=1`: Environment variables providing immediate escape hatches in CI/CD, scripting, or automated headless runs.
2. **Missing Binary Robustness**:
   - If `rtk` is missing from `PATH`, `install` and `init-prj` must emit a notice/warning and continue successfully (`Ok(())`). Under no circumstances should a missing RTK binary abort the installation.
3. **Doctor Visibility**:
   - `doctor` reports RTK hook presence on supported harnesses and prints a clear advisory noting the potential for stdout alterations on wrapped subcommands like `gh issue view --comments`.
