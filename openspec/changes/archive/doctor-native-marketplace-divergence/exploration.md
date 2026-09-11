# Exploration: Claude Code Native Marketplace Plugin Divergence

## Technical Investigation

### Claude Code Native Marketplace Layout
Claude Code manages native plugins under its configuration directory (`~/.claude` or `$CLAUDE_CONFIG_DIR`). The key files are:
1. `settings.json`: Contains user settings, including `enabledPlugins`:
   ```json
   {
     "enabledPlugins": {
       "compound-engineering@compound-engineering-plugin": true
     }
   }
   ```
2. `plugins/installed_plugins.json`: Registry of installed plugins, organized by plugin identifier with scope entries:
   ```json
   {
     "version": 2,
     "plugins": {
       "compound-engineering@compound-engineering-plugin": [
         {
           "scope": "local",
           "projectPath": "/path/to/project-a",
           "installPath": "/home/user/.claude/plugins/cache/.../3.8.4",
           "version": "3.8.4",
           "installedAt": "2026-05-25T13:39:00.182Z",
           "lastUpdated": "2026-05-25T13:39:00.182Z"
         },
         {
           "scope": "user",
           "installPath": "/home/user/.claude/plugins/cache/.../3.17.1",
           "version": "3.17.1",
           "installedAt": "2026-05-30T16:06:46.306Z",
           "lastUpdated": "2026-09-08T13:39:54.399Z"
         }
       ]
     }
   }
   ```
3. `plugins/cache/<marketplace>/<plugin>/<version>/`: Immutable version directories unpacked from the marketplace.

### ce-ai State & Content Management
`ce-ai` tracks its own installation tree under `~/.claude/compound-engineering/` (or `<workspace>/.claude/compound-engineering/`):
- `state.installed_harnesses`: Array of JSON objects. The entry for `claude` includes:
  ```json
  {
    "name": "claude",
    "scope": "global",
    "version": "compound-engineering-v3.24.0"
  }
  ```
- `state.release_provenance.tag`: e.g. `"compound-engineering-v3.24.0"`.

### Divergence Mechanism
When `enabledPlugins` has `compound-engineering@compound-engineering-plugin: true`, Claude Code looks for skills and reviewers in `~/.claude/plugins/cache/.../<version>/`.
Updating `ce-ai` updates `~/.claude/compound-engineering/skills/...`, but Claude Code continues using `~/.claude/plugins/cache/.../<version>/`.
Because the two storage trees do not intersect, version divergence is completely silent.

## Evaluated Options & Architectural Tradeoffs

| Option | Description | Pros | Cons | Verdict |
|---|---|---|---|---|
| **A: Full CLI `--json` mode in `doctor`** | Add `--json` flag to `doctor::Args` to serialize all findings and info | Provides CLI JSON format | Major scope creep; doctor currently has no JSON output format for other probes | Rejected |
| **B: Auto-update native plugin via CLI** | Execute `claude plugin update` during `ce-ai upgrade` or `doctor --fix` | Automatic resolution | Violates read-only invariant; introduces process failure and permission risks | Rejected |
| **C: Read-only detection with `doctor-info:` output & typed domain model** | Parse `installed_plugins.json` tolerantly, filter by applicability, emit `doctor-info:`, expose typed struct | Strict read-only; non-blocking; zero regressions; adheres to established doctor probe patterns | Requires user to manually run the remediation command | **Selected** |

## Scope Applicability & Resolution Logic
Claude Code supports three scopes:
- `user`: Global configuration. Always applies regardless of `cwd`.
- `project` / `local`: Scoped to a specific directory path recorded in `projectPath`. Applies when `cwd == projectPath` or `cwd.starts_with(projectPath)` (or if `repo_root` matches).

To prevent incorrect assumptions about Anthropic's internal precedence resolution, `ce-ai doctor` will report **every** applicable scope that is divergent. For instance, if `user` has `3.17.1` and `project` in the current repo has `3.8.4` while `ce-ai` has `3.24.0`, both will be reported clearly with their respective scopes.

## Version Normalization
Versions across systems carry varying prefixes:
- `ce-ai`: `"compound-engineering-v3.24.0"`, `"v3.24.0"`
- native marketplace: `"3.8.4"`, `"3.24.0"`, `"v3.24.0"`

Normalization algorithm:
1. Trim whitespace.
2. Strip prefix `"compound-engineering-"` or `"compound-engineering@"`.
3. Strip prefix `'v'`.
4. Resulting string (e.g. `"3.24.0"`) is compared for equality.
