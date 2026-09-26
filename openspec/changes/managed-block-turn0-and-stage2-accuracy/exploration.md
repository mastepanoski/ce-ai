# Exploration: Managed Adoption Block Turn-0 Directives & Progressive OpenSpec Accuracy

## Architectural Context
When a project adopts `ce-ai` via `ce-ai init-prj --tier full`, `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs` generates the markdown section injected into `AGENTS.md` (or `CLAUDE.md`). This text serves as the operating directive for AI coding agents operating in the adopted repository.

## Findings & Tradeoffs

### 1. Turn-0 Directives Analysis
- **Current Text**: Unconditionally instructs agents: "At the start of EVERY new session or after context compaction, before running any task or reading historical chat assumptions, the AI agent MUST run: `ce-ai workflow resume`".
- **Problem**: Supported harnesses (Claude, Cursor, Codex, Copilot, Pi, Antigravity) execute `ce-ai workflow resume --json` during `SessionStart` hooks automatically. Injecting this output into Turn 0 context already informs the agent of the Git branch, SHA256 integrity, and OpenSpec progress. If the agent reads an unconditional "MUST run", it executes an extraneous tool call, consuming tokens and turn latency.
- **Solution**: Explicitly state that supported harnesses have hooks auto-installed, and agents should treat pre-injected context as current. Only run `ce-ai workflow resume` manually if hook context is absent or suspected stale.

### 2. Progressive OpenSpec Requirements Analysis
- **Current Text**: Requires all 5 files (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) to exist before writing feature code.
- **Problem**: This creates a circular dilemma during Stage 2: `tasks.md` is an executable breakdown derived in Stage 3 (`ce-plan`) from the frozen Stage 2 contract. Demanding `tasks.md` before Stage 2 is complete causes confusion.
- **Solution**: Clarify that Stage 2 defines and freezes the contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`). Stage 3 generates `tasks.md`. The requirement before opening a PR remains all 5 files, but prior to Stage 3 planning, only the 4-file contract is required.

### 3. Version Bumping & Test Invariants
- `BLOCK_VERSION` must be bumped from 6 to 7.
- Past solution note `docs/solutions/test-failures/adoption-block-version-bump-test-coordination-2026-08-25.md` establishes that:
  - `CUR_BLOCK_VERSION` in `tests/cli.rs` must be kept in lockstep.
  - Fixtures simulating older versions (v1, v2, v5) correctly trigger `StaleVersion` (`v < BLOCK_VERSION`).
  - Hardcoded version numbers in test assertions (e.g. line 2717 in `tests/cli.rs`) must use `CUR_BLOCK_VERSION`.
  - Minimal tier remains byte-identical and retains its SHA256 match short-circuit.
