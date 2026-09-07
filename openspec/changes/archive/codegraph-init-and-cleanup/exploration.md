# Exploration: CodeGraph Native Init Support & gentle-ai Residual Cleanup

## 1. Technical Investigation

### A. Current CodeGraph Execution Surface
- Upstream CLI: `codegraph` provides a standalone binary with `codegraph init [options] [path]`.
- Running `codegraph init [path]` scans the project root and builds `.codegraph/` (SQLite WAL database and symbol graph).
- If `.codegraph/` already exists, `codegraph status [path]` shows index health and statistics.
- When `ce-ai tools install codegraph` runs, it only writes the MCP server definition `codegraph mcp` to `opencode.json` (and other harnesses). It never builds the index.

### B. Current Erroneous Guidance in ce-ai
- `src/commands/audit.rs:143`:
  ```rust
  detail: ".codegraph/ index not initialized (run 'gentle-ai codegraph init')"
  ```
  `audit` issues a warning and prompts users to run `gentle-ai codegraph init`. This is an outdated artifact from when `gentle-ai` was used as a wrapper.
- `docs/user-guide/quick-start-workflow-guide.md:205`:
  Instructs users: `Run gentle-ai codegraph init --cwd <worktree-root> inside the new worktree.`
- `openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md:34-37`:
  ```gitignore
  <!-- gentle-ai:ce-ai-ignore:start -->
  .ce-ai/skills-registry.json
  compound-engineering/
  <!-- gentle-ai:ce-ai-ignore:end -->
  ```
  This is confusing because `ce-ai` configuration sentinel blocks are `# BEGIN CE-AI MANAGED BLOCK`.

## 2. Options for ce-ai Orchestration

### Option 1: CLI Subcommand `ce-ai tools init <tool> [path]`
- Introduce `Action::Init { tool: String, path: Option<PathBuf> }` to `ce-ai tools`.
- For `codegraph`:
  - Verify `codegraph` binary is in `PATH`. If missing: `CeError::Usage("codegraph binary not found on PATH. Install it first (e.g. 'npm install -g @colbymchenry/codegraph')")`.
  - Resolve target directory (defaults to current dir or `path`).
  - If `.codegraph/` already exists in target directory: inform user that index already exists.
  - Run `codegraph init <path>` and forward/display output.

### Option 2: Auto-Init in `ce-ai init-prj`
- When `ce-ai init-prj` adopts a repository:
  - If `codegraph` is present on `PATH` and `target_dir.join(".codegraph").exists()` is false:
    - In dry-run: log that `codegraph init` would be executed.
    - In normal mode: execute `codegraph init` on `target_dir`.
    - If execution fails (e.g., non-zero exit or timeout), log as a non-fatal warning so project adoption is never blocked.

### Option 3: Auto-Init in `ce-ai tools install codegraph`
- When `ce-ai tools install codegraph` runs:
  - After registering the MCP server, if current working directory is a git repo and has `codegraph` on `PATH` and `.codegraph/` does not exist:
    - Trigger `codegraph init` automatically.

## 3. Decision
Combine Options 1, 2, and 3:
- Expose `ce-ai tools init codegraph [path]`.
- Provide automated index creation in `ce-ai init-prj` and `ce-ai tools install codegraph` when `codegraph` is on `PATH` and `.codegraph/` is missing.
- Update `audit.rs`, `doctor.rs`, `quick-start-workflow-guide.md`, and `exploration.md`.
