# Tasks: CodeGraph Native Init Support & gentle-ai Residual Cleanup

Total Estimated Changed Lines: ~120 LOC (Forecast: well within 400 LOC budget).

- [ ] **Task 1: Add `ce-ai tools init` subcommand in `src/commands/tools.rs`** (~45 LOC)
  - Add `Action::Init { tool: String, path: Option<PathBuf> }`.
  - Implement `init_tool(ctx, tool, path)`.
  - Check binary availability, existing `.codegraph/`, dry-run mode, and subprocess execution.
  - Return exit code `2` (`CeError::Usage`) if tool unsupported or binary missing.

- [ ] **Task 2: Add auto-init hook to `src/commands/init_prj.rs`** (~25 LOC)
  - Helper `init_codegraph_if_available(target_dir, quiet)`.
  - Invoked during project adoption when `codegraph` is on PATH and `.codegraph/` is missing.
  - Non-fatal error handling if initialization fails.

- [ ] **Task 3: Update `audit.rs` and `doctor.rs` messages** (~10 LOC)
  - In `src/commands/audit.rs:143`, change suggestion to `run 'codegraph init' or 'ce-ai tools init codegraph'`.
  - In `src/commands/doctor.rs:298`, add `(suggested: 'ce-ai tools init codegraph')`.

- [ ] **Task 4: Update Documentation and Spec References** (~15 LOC)
  - Fix `openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md` to use `# BEGIN CE-AI MANAGED BLOCK`.
  - Update `docs/user-guide/quick-start-workflow-guide.md` to reference `codegraph init` and `ce-ai tools init codegraph`.

- [ ] **Task 5: Unit and CLI Integration Tests** (~25 LOC)
  - Add integration tests in `tests/cli.rs` testing `ce-ai tools init` with invalid tool, missing binary/mock or existing index, dry-run mode.
  - Test `ce-ai audit` message verification.
