# Exploration: Workspace-Scoped OpenCode Manifest Resolution

## Context & Architecture
`ce-ai install` supports `--scope <global|workspace>` (default: `global`).
When `--scope workspace` is used inside a Git repository:
- `target_base_dir` resolves to `git rev-parse --show-toplevel`.
- `config_dir` in `install.rs` becomes `target_base_dir`.
- OpenCode manifest is written to `<repo_root>/compound-engineering/install-manifest.json`.
- OpenCode settings are written to `<repo_root>/opencode.json`.
- Plugin loader is installed into `<repo_root>/compound-engineering/plugins/compound-engineering.js`.
- An entry is appended to `state.installed_harnesses` in `~/.ce-ai/state.json`.

However, `state.installed_harnesses` only stored:
```json
{
  "name": "opencode",
  "version": "local",
  "source": { "kind": "local", "path": "..." },
  "installed_at": "...",
  "last_synced_at": "..."
}
```
It did not record the installation scope or target directory.

Later, commands like `doctor`, `status`, and `sync` look exclusively at `ctx.opencode_config_dir` (`~/.config/opencode`).

## Evaluated Options

### Option 1: Store scope and target_dir in state.installed_harnesses and resolve contextually
- At install time, record `"scope": scope_arg` and `"target_dir": config_dir.display().to_string()` in the `state.installed_harnesses` entry.
- When loading state in commands (`doctor`, `status`, `sync`), resolve the active OpenCode directory:
  1. If `ctx.workspace_root` matches an `opencode` entry with `"scope": "workspace"`, or if `<workspace_root>/compound-engineering/install-manifest.json` exists, use `workspace_root`.
  2. Otherwise use `ctx.opencode_config_dir`.
- **Pros**: Explicit, deterministic, preserves legacy installs via directory fallback, allows multi-repo worktrees to coexist cleanly.
- **Cons**: Minor change to JSON schema in `state.json` (additive).

### Option 2: Hardcode heuristic directory probe only (no state change)
- Only check if `<workspace_root>/compound-engineering/install-manifest.json` exists at runtime.
- **Pros**: Zero state schema changes.
- **Cons**: State and disk are decoupled; `state.installed_harnesses` still doesn't know where the harness was installed, leading to ambiguity if both global and workspace installations exist.

### Decision
Choose **Option 1**: Store `"scope"` and `"target_dir"` in `state.installed_harnesses`, while also including the fallback check for `<workspace_root>/compound-engineering/install-manifest.json` to handle existing installations.
