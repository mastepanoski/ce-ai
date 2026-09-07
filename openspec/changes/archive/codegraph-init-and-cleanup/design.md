# Design: CodeGraph Native Init Support & gentle-ai Residual Cleanup

## 1. CLI Changes

### `src/commands/tools.rs`
Add `Init` to `Action`:
```rust
#[derive(clap::Subcommand)]
pub enum Action {
    /// Check installation, version freshness, and health status of companion tools.
    Status,
    /// Install or provision a specific companion tool (engram, codegraph, context7, rtk).
    Install {
        /// Name of tool (engram, codegraph, context7, rtk).
        tool: String,
    },
    /// Initialize workspace index or configuration for a companion tool (e.g. codegraph).
    Init {
        /// Name of tool (codegraph).
        tool: String,
        /// Target project path (defaults to current directory).
        path: Option<PathBuf>,
    },
}
```

Implement `init_tool(ctx: &Context, tool: &str, path: Option<&Path>) -> Result<(), CeError>`:
- Validate `tool == "codegraph"`. If other: `CeError::Usage("tool '{tool}' does not support init. Supported: codegraph")`.
- Determine target directory: `path.unwrap_or(current_dir)`.
- If `.codegraph/` exists in target directory:
  - Print `tools: codegraph index already initialized at '<path>'` and return `Ok(())`.
- Check if `codegraph` executable exists on `PATH` using `which` or `Command::new("codegraph").arg("--version")`.
  - If missing: return `CeError::Usage("codegraph binary not found on PATH. Install it first (e.g. npm install -g @colbymchenry/codegraph)")`.
- If `ctx.dry_run`:
  - Print `tools: [dry-run] would run 'codegraph init' in '<path>'`.
  - Return `Ok(())`.
- Execute `Command::new("codegraph").arg("init").arg(target_dir).status()`.
  - If status is success: print `✓ Initialized CodeGraph index at '<path>'`.
  - If failure: return `CeError::Runtime(format!("codegraph init failed with exit code {code}"))`.

### `src/commands/init_prj.rs`
In `run()` after adopting the project and writing rule files:
```rust
if !ctx.dry_run {
    init_codegraph_if_available(&target_dir, ctx.quiet);
}
```
Helper function `init_codegraph_if_available(target_dir: &Path, quiet: bool)`:
- Check if `target_dir.join(".codegraph").exists()`. If exists, do nothing.
- Probe `codegraph --version`. If it fails (not on PATH), return silently.
- Execute `codegraph init` in `target_dir`.
- If successful and not quiet: print `✓ Initialized CodeGraph index (.codegraph/)`.
- If fails: log non-fatal warning to `stderr` without failing adoption.

### `src/commands/audit.rs`
Update `CodeIntelDetector`:
```rust
detail: ".codegraph/ index not initialized (run 'codegraph init' or 'ce-ai tools init codegraph')".into()
```

### `src/commands/doctor.rs`
Update `codegraph` probe:
```rust
println!("doctor-info: codegraph index (.codegraph/) not initialized (suggested: 'ce-ai tools init codegraph')");
```

## 2. Documentation & Spec Cleanup
1. `docs/user-guide/quick-start-workflow-guide.md`:
   Update line 205:
   `Run codegraph init <worktree-root> (or ce-ai tools init codegraph) inside the new worktree.`
2. `openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md`:
   Replace `<!-- gentle-ai:ce-ai-ignore:start -->` and `<!-- gentle-ai:ce-ai-ignore:end -->` with canonical `# BEGIN CE-AI MANAGED BLOCK` and `# END CE-AI MANAGED BLOCK`.
