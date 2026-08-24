# OpenSpec Requirements: Release v0.7.0 Specifications

## Feature 1: Workspace-Level Overrides (`.ce-ai.json`)

### Requirement 1.1: Local Precedence Resolution
- **WHEN** a `.ce-ai.json` file exists in the current repository root,
- **THEN** `ce-ai` MUST load `.ce-ai.json` overrides and merge them on top of global `~/.config/ce-ai/state.json`.

### Requirement 1.2: Fallback to Global State
- **WHEN** a configuration key (e.g. `installed_harnesses`) is omitted from `.ce-ai.json`,
- **THEN** `ce-ai` MUST fallback to the corresponding value defined in global `state.json`.

---

## Feature 2: Complete Multi-Harness Uninstall (`ce-ai uninstall --all`)

### Requirement 2.1: Multi-Harness Targeting Parity
- **WHEN** `ce-ai uninstall --harness <name|all>` is executed,
- **THEN** `ce-ai` MUST resolve the targeted harness adapter(s) matching `install --harness all` capabilities.

### Requirement 2.2: Full Asset Removal with `--all`
- **WHEN** `ce-ai uninstall --all` is executed,
- **THEN** `ce-ai` MUST remove managed loader scripts (`compound-engineering.js`), skills directories (`.opencode/skills/compound-engineering`), and SHA256 manifests across target harnesses.

### Requirement 2.3: Interactive Prompt and `--yes` Flag
- **WHEN** `ce-ai uninstall --all` is run without `--yes` / `-y`,
- **THEN** `ce-ai` MUST prompt for confirmation before deleting files.
- **WHEN** `--yes` is specified,
- **THEN** `ce-ai` MUST bypass the interactive confirmation prompt.
