# Specification: Workspace-Scoped OpenCode Manifest Resolution

## Requirements

### R1: State Recording of Installation Scope
- **WHEN** `ce-ai install --harness opencode --scope workspace` is executed,
  **THEN** the entry added to `state.installed_harnesses` MUST include `"scope": "workspace"` and `"target_dir": "<workspace_root>"`.
- **WHEN** `ce-ai install --harness opencode` (global scope default) is executed,
  **THEN** the entry added to `state.installed_harnesses` MUST include `"scope": "global"`.

### R2: Contextual OpenCode Configuration Directory Resolution
- **WHEN** commands query the active OpenCode directory via `ctx.resolve_opencode_dir(&state)`,
  **THEN** if the current execution is within a workspace that contains `<workspace>/compound-engineering/install-manifest.json` or has a matching workspace-scoped entry in `state.installed_harnesses`, it MUST return `<workspace>`.
  **ELSE** it MUST return `ctx.opencode_config_dir`.

### R3: Doctor Health Check Consistency
- **WHEN** `ce-ai doctor` is executed inside a workspace that was installed with `--scope workspace`,
  **THEN** it MUST verify `install-manifest.json`, `managed_dir`, and the `SessionStart` plugin against the workspace root.
  **AND** it MUST NOT report `state-inconsistent: opencode state entry and install manifest disagree` or `opencode: SessionStart plugin missing or outdated` if the workspace installation is intact.
- **WHEN** `ce-ai doctor` is executed where `state.installed_harnesses` contains an entry for `opencode`, but the corresponding manifest (`install-manifest.json`) does not exist in the resolved directory,
  **THEN** it MUST report `state-inconsistent: opencode state entry and install manifest disagree`.

### R4: Status and Sync Subcommand Consistency
- **WHEN** `ce-ai status` is run in a workspace-scoped install,
  **THEN** it MUST evaluate drift against the workspace manifest rather than reporting `drift: unknown (no install manifest)`.
- **WHEN** `ce-ai sync` is run in a workspace-scoped install,
  **THEN** it MUST read and write the manifest and assets in the workspace root.
