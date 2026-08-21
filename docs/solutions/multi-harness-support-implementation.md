# Solution: Multi-Harness Support Implementation (`ce-ai` v0.3.0)

## Problem Statement
`ce-ai` originally managed only OpenCode (`opencode.json`). As teams use multiple AI coding tools (Claude Code, Pi, Cursor, Copilot, Codex, Grok, Kimi, Antigravity, DeepSeek, FX), `ce-ai` needed a unified domain model and adapter interface to support multi-harness installation, sync, model assignment translation, and host harness auto-probing.

## Solution Architecture
1. **`HarnessKind` Enum (`src/harness/mod.rs`)**:
   - Represents all 12 supported harness targets.
   - Implements `FromStr` mapping lowercase names (e.g. `opencode`, `claude`, `pi`, `cursor`, `copilot`, `fx.sh`) to enum variants.
   - Provides `detect_installed_harnesses(home)` to probe host filesystem presence.

2. **`HarnessAdapter` Trait & Native Adapters (`src/harness/`)**:
   - **OpenCode**: `opencode.rs` (`~/.config/opencode/opencode.json`)
   - **Claude Code**: `claude.rs` (`~/.claude.json`)
   - **Pi**: `pi.rs` (`~/.pi/config.json`)
   - **Cursor**: `cursor.rs` (Injects/strips `<!-- CE-AI MANAGED BLOCK -->` in `.cursorrules`)
   - **Copilot**: `copilot.rs` (Injects/strips `<!-- CE-AI MANAGED BLOCK -->` in `.github/copilot-instructions.md`)
   - **Generic JSON**: `generic_json.rs` (Codex, Grok, Kimi, AGY, DeepSeek, FX)
   - **Custom**: `custom.rs` (Fallback for `--harness custom`)

3. **Multi-Harness Model Role Translation**:
   - `ce-ai models set <slot> <provider/model>` syncs model assignments across all active harness configuration files simultaneously.

4. **Containerized E2E Gate (`e2e_runner.sh` & `make e2e`)**:
   - Validates multi-harness installation, probing, model setting, and uninstallation in isolated Docker environments.

## DoD Verification
- Version bumped to `0.3.0` in `Cargo.toml` and documented in `CHANGELOG.md` adhering to SemVer.
- 54/54 unit tests passing cleanly.
- Zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- 100% green CI matrix across Linux, macOS, and Windows.
