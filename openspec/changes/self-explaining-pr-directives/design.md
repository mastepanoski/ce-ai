# Design: Self-Explaining PR Directives & Upfront Review Readiness

## Architecture & System Design

This design updates the governance contracts injected into adopted projects by `ce-ai init-prj` and enforced in `ce-ai`'s own repository.

### 1. Managed Block Content Evolution (`src/commands/init_prj.rs`)

The managed block content returned by `render_block_content(tier: AdoptionTier)` is extended with explicit Stage 7 directives.

#### `AdoptionTier::Full`
Adds a dedicated subsection under Stage 7:
```markdown
### 🚀 Self-Explaining Pull Requests & Review Readiness
When creating Pull Requests (Stage 7), agents MUST ensure the change explains itself:
- **What It Ruled Out**: State rejected alternatives and discarded trade-offs (distilled from `exploration.md`).
- **Which Rule Decided It**: Cite the specific invariant, rule, architectural boundary, or compliance constraint (from `AGENTS.md` or `design.md`) that determined the solution.
- **Upfront Evidence**: Attach empirical validation proof (test run summaries, CLI logs, reproduction traces, or browser check results) directly to the PR body using collapsible `<details><summary>` blocks (`not asked for later`).
- **Pre-Review Automated Checks**: Run and verify 100% green status across all local linters, unit tests, and browser/E2E checks BEFORE requesting human review or marking the PR ready.
- **Reviewer Routing**: Identify domain owners or `CODEOWNERS` and assign appropriate reviewers.
```

#### `AdoptionTier::Minimal`
Extends the minimal checklist:
```markdown
- Ensure PRs explain what was ruled out, which rule decided it, and attach verification evidence upfront.
- Verify all automated and browser checks pass 100% green before requesting human review.
```

#### `AdoptionTier::Orchestrator`
Adds orchestration delegation rules:
```markdown
- Enforce self-explaining PRs: mandate that subagents document rejected alternatives, governing rules, and attach empirical verification evidence in collapsible blocks before requesting review.
- Gate reviews on 100% green pre-review automated and browser checks.
```

### 2. Version Bump (`BLOCK_VERSION = 6`)
In `src/commands/init_prj.rs`:
```rust
pub const BLOCK_VERSION: u32 = 6;
```
This automatically updates:
- The block header generated during adoption: `<!-- ce-ai:block begin v=6 tier=... sha256=... -->`.
- The `block_version` stored in `state.json` (`ProjectAdoptionEntry`).
- The staleness detector `check_adoption_block_status`, which classifies existing v5 blocks as `StaleVersion { version: 5 }` and prompts the user/agent to run `ce-ai init-prj` to refresh the block.

### 3. Pinned Test Synchronization (`tests/cli.rs`)
Update:
```rust
const CUR_BLOCK_VERSION: u32 = 6;
```
Ensure all test fixtures asserting block header versions, drift detection, and stale-version upgrades are updated in lockstep.

### 4. Documentation & Policy Alignment
- In `AGENTS.md`: Add the Stage 7 Self-Explaining PR directives to the 7-stage workflow breakdown and update the DoD checklist.
- In `CONTRIBUTING.md`: Add a clarifying note in § "PR Size Boundaries & Bounded Corrections" confirming that collapsible evidence blocks in PR descriptions do not count against code LOC budgets, and affirming that the 400 LOC review boundary remains strictly in effect.
