# Exploration: Kimi Code CLI Turn-0 Drift Delivery Evaluation

## Technical Investigation

### 1. Configuration Discovery Scope
- Kimi Code CLI defines and reads hooks exclusively from the user-level configuration file located at `~/.kimi-code/config.toml` (or `$KIMI_CODE_HOME/config.toml`).
- The CLI explicitly does not load hooks from `<project>/.kimi-code/config.toml` or `<project>/.kimi/config.toml`.
- Project workspaces only support `.kimi-code/local.toml` (directory tracking), `.mcp.json` (MCP servers), and `AGENTS.md` (instructions).

### 2. Project Boundary & Isolation Evaluation
- In `ce-ai`, `init-prj` operates strictly on a target project workspace.
- Mutating `~/.kimi-code/config.toml` during `init-prj` would create a global side effect, executing `ce-ai workflow resume` across all repositories where Kimi runs.

### 3. Alternative Approaches Evaluated
- **Plugin Manifests (`kimi.plugin.json`):** Plugins can load skills via `sessionStart.skill`, but cannot run arbitrary project-specific shell commands without installing a full Kimi plugin into the user's plugin cache.
- **Text Directives:** `ce-ai init-prj` already injects mandatory Turn-0 execution directives into `AGENTS.md` and `.kimi-code/AGENTS.md`. This remains the intended, reliable mechanism for Kimi.
