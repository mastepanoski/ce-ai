# OpenSpec Requirements & Acceptance Criteria

**Change Identifier:** `multi_harness_support`  

---

## Requirements

### Requirement 1: Multi-Harness Flag Parsing & Resolution
- **WHEN** the user invokes `ce-ai install --harness <name>`, **THEN** the CLI MUST validate `<name>` against `HarnessKind`. If invalid or de-scoped (such as `deepseek`), return exit code `2` (Usage Error) with actionable error guidance.
- **WHEN** the user invokes `ce-ai install --all`, **THEN** the CLI MUST probe for installed host harness configuration directories across the 10 supported native harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) and execute installation for each detected harness.

### Requirement 2: Native Per-Harness Adapters & Configuration Merging
- **WHEN** installing or updating a native harness, **THEN** `ce-ai` MUST delegate to the corresponding `HarnessAdapter` (`OpenCode`, `Claude`, `Pi`, `Cursor`, `Copilot`, `Codex`, `Grok`, `Kimi`, `Agy`, `Fx`), writing to its native host directory in its native configuration format (JSON for OpenCode/Claude/Kimi/Agy/Fx/Copilot/Cursor, TOML for Codex/Grok, Skills directory for Pi, MDC/Markdown for Cursor/Copilot project rules), preserving unmanaged user keys and writing back via `write_atomic`.

### Requirement 3: Project Rules Adoption & Management
- **WHEN** adopting a project via `ce-ai init-prj`, **THEN** `ce-ai` MUST write non-destructive managed blocks to `AGENTS.md` (root) and derived stub rule files (`CLAUDE.md`, `.cursor/rules/compound-engineering.mdc`, `.github/copilot-instructions.md`, `.codex/AGENTS.md`, `.grok/AGENTS.md`, `.kimi-code/AGENTS.md`, `.agents/rules/compound-engineering.md`, `.pi/AGENTS.md`, `.fx/AGENTS.md`).

### Requirement 4: Custom Harness Fallback Mode
- **WHEN** the user runs `ce-ai install --harness custom`, **THEN** the CLI MUST support custom plugin directory configuration.

### Requirement 5: Model Role Syncing Across Harnesses
- **WHEN** `ce-ai models set <slot> = <model>` is run, **THEN** `ce-ai` MUST update central `state.json` and sync model assignments across configured harnesses.

### Requirement 6: Manifest Tracking & Backup Restoration
- **WHEN** any harness is installed or modified, **THEN** SHA256 file hashes MUST be saved in `install-manifest.json`.
- **WHEN** `ce-ai uninstall --harness <name>` is executed, **THEN** `ce-ai` MUST restore the timestamped pre-install backup and remove managed files cleanly.
