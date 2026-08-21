# OpenSpec Design: Release v0.5.0 Architecture

## Data Schemas (`src/state/state.rs`)
- `Scope`: `Global`, `Workspace(PathBuf)`.
- `ToolStatus`: `Engram`, `CodeGraph`, `Context7`, `RTK` status (`installed: bool`, `path: Option<PathBuf>`, `mcp_registered: bool`).
- `WorkflowCheckpoint`: `stage: String`, `task: String`, `updated_at: String`.

## Command Implementations
- `install.rs`: Scope resolution via `git rev-parse --show-toplevel`.
- `tools.rs`: Subcommands `status` and `install <name>`.
- `workflow.rs`: Subcommands `status`, `checkpoint`, and `resume`.
