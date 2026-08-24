# OpenSpec Requirements: Per-Harness Native Directories

- **Change:** `harness-containment-safety-gate`
- **Issue:** #157 (P0)

---

## 📜 Acceptance Criteria

### Requirement 1: Per-Harness Directory Isolation
- **WHEN** the user runs `install`, `uninstall`, `sync`, or `models set` for a non-OpenCode harness (e.g. `cursor`, `claude`, `pi`, `copilot`)
- **THEN** `ce-ai` SHALL provision files strictly inside `harness.harness_dir(home_dir)` (e.g. `~/.cursor/`, `~/.config/claude/`)
- **AND** SHALL NOT write synthetic files inside `~/.config/opencode/`.

### Requirement 2: Clean Uninstall Isolation
- **WHEN** the user uninstalls a non-OpenCode harness (e.g. `ce-ai uninstall --harness cursor`)
- **THEN** `ce-ai` SHALL clean configuration and remove managed files strictly inside `harness.harness_dir(home_dir)`
- **AND** SHALL leave zero residual artifacts in `~/.config/opencode/`.

### Requirement 3: `--harness all` Multi-Harness Isolation
- **WHEN** the user runs `install` or `uninstall` with `--harness all`
- **THEN** `ce-ai` SHALL provision each active harness in its respective native directory (`~/.config/opencode/`, `~/.cursor/`, `~/.config/claude/`, etc.)
- **AND** SHALL NOT contaminate unrelated harness directories.
