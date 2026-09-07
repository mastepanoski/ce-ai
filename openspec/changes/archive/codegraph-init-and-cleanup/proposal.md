# Proposal: CodeGraph Native Init Support & Removal of Legacy gentle-ai Residuals

## 1. Problem Statement
In `ce-ai`, users and companion tools rely on CodeGraph (`.codegraph/` index) for structural code intelligence and symbol navigation. However:
1. **Misdirected Guidance**: `ce-ai audit` hardcodes `detail: ".codegraph/ index not initialized (run 'gentle-ai codegraph init')"` ([`src/commands/audit.rs:143`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/audit.rs#L143)) and `docs/user-guide/quick-start-workflow-guide.md` instructs users to run `gentle-ai codegraph init`. `ce-ai` should never instruct users to run commands from external tools when `codegraph` provides a native CLI (`codegraph init [path]`) and `ce-ai` can orchestrate it directly.
2. **Missing Native Init Capability**: `ce-ai tools` only supports `status` and `install` (which registers the MCP server). Neither `ce-ai tools` nor `ce-ai init-prj` provides automated initialization of the `.codegraph/` index for a project.
3. **Erroneous Residual References in Docs & Specs**: Historical OpenSpec drafts ([`openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md`](file:///Users/mastepanoski/projects/web/ai/ce-ai/openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md)) used placeholder comments such as `<!-- gentle-ai:ce-ai-ignore:start -->` instead of the canonical `ce-ai` block markers (`# BEGIN CE-AI MANAGED BLOCK`). Citing `gentle-ai` in `ce-ai` configuration documentation creates confusion.

## 2. In-Scope / Out-of-Scope
- **In-Scope**:
  - Add `ce-ai tools init <tool> [path]` subcommand (supporting `codegraph`).
  - Auto-initialize `.codegraph/` during `ce-ai init-prj` if `codegraph` is present on `PATH` and `.codegraph/` is missing.
  - Auto-initialize `.codegraph/` in `ce-ai tools install codegraph` if `codegraph` is present on `PATH` and `.codegraph/` is missing in the current repo.
  - Update `src/commands/audit.rs` to suggest `codegraph init` or `ce-ai tools init codegraph`.
  - Update `src/commands/doctor.rs` to suggest `ce-ai tools init codegraph`.
  - Update documentation ([`quick-start-workflow-guide.md`](file:///Users/mastepanoski/projects/web/ai/ce-ai/docs/user-guide/quick-start-workflow-guide.md)) to reference `codegraph init` / `ce-ai tools init codegraph`.
  - Clean up erroneous `gentle-ai` references in `exploration.md`.
- **Out-of-Scope**:
  - Re-implementing CodeGraph's internal indexing engine (ce-ai invokes upstream `codegraph init` subprocess).
  - Removing historical credits/acknowledgements in `DISCLAIMER.md` or `README.md` (which legitimately credit upstream inspirations).

## 3. Risk Evaluation
- **Subprocess Failure Risk**: `codegraph init` may fail or take time if run on a non-code or enormous directory. Mitigation: non-blocking execution, graceful non-fatal warning in `init-prj`, dry-run support, checking git root before running.
- **Missing Binary Risk**: If `codegraph` is not on `PATH`, `ce-ai tools init codegraph` will return a clear, actionable `CeError::Usage` explaining how to install it.

## 4. Success Criteria
- `ce-ai tools init codegraph` initializes `.codegraph/` if `codegraph` is installed, or provides an actionable error if not found.
- `ce-ai init-prj` initializes `.codegraph/` if `codegraph` is installed on `PATH` and `.codegraph/` does not exist.
- `ce-ai audit` reports `run 'codegraph init'` or `ce-ai tools init codegraph` without mentioning `gentle-ai`.
- Zero residual `gentle-ai:ce*` comments in `openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md`.
- 100% test coverage with unit and CLI integration tests.
