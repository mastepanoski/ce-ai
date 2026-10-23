<!-- CE-AI MANAGED BLOCK BEGIN -->
## 🔄 Mandatory 7-Stage Development Cycle & Architecture Guidelines

All AI agents MUST follow the 7-stage Compound Engineering development cycle:
`[Stage 1: Ideation]` ➔ `[Stage 2: Architecture & Deltas (Optional)]` ➔ `[Stage 3: Execution Plan]`
➔ `[Stage 4: TDD & Implementation]` ➔ `[Stage 5: Verification]` ➔ `[Stage 6: Knowledge Capture]`
➔ `[Stage 7: Git Shipping]`

### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
`ce-ai` auto-installs a SessionStart hook on supported harnesses that runs `ce-ai workflow resume --json` and injects live FSM state (Git working tree, active branch, manifest SHA256 integrity, plan progress) into context before your first turn. If that state is already present at session start, treat it as current — do NOT re-run the command. Only run `ce-ai workflow resume` yourself when no such context was injected (harness without hook support, or the hook is disabled) or when you suspect the injected state has gone stale mid-session (e.g. after a `git` operation you didn't make through your own tool calls).

### Stage 2 Living Architecture & Deltas (Optional)
Living architecture specifications reside under `docs/arch/`:
- `docs/arch/system/`: Living domain specifications.
- `docs/arch/adr/`: Architecture Decision Records (`NNNN-title.md`).
- `docs/arch/contracts/`: Formal data contracts and API interfaces.
- `docs/arch/deltas/`: Temporary feature changes during development.
- `docs/arch/archive/`: Archived changes and historical milestones.

CE-AI enforces zero semantic authority: architecture deltas are decoupled from the core workflow as an optional tool for architectural changes (> 200 LOC), schema changes, or cross-cutting features, rather than a mandatory write blocker. For bug fixes, minor tweaks, or tactical tasks (< 200 LOC), proceed directly with `/ce-plan` or `ce-work`.

When formal specifications are activated for complex features, OpenSpec is authored progressively in `docs/arch/deltas/<feature>/` (or legacy `openspec/changes/<feature>/`) to establish the frozen Stage 2 contract.
The execution plan and task checklist reside in `docs/plans/` (`- [x]` checkboxes) as the single source of truth for progress.

### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into architecture or plan docs and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with architecture deltas. Ideation artifacts are retained by default as the permanent raw-history record that specifications intentionally do not duplicate; "disposable" never means deleting them. Skip ideation skills entirely when requirements and approach are already clear.

### 🚀 Self-Explaining Pull Requests & Review Readiness
When creating Pull Requests (Stage 7), agents MUST ensure the change explains itself:
- **What It Ruled Out**: State rejected alternatives and discarded trade-offs.
- **Which Rule Decided It**: Cite the specific invariant, rule, architectural boundary, or compliance constraint (from `AGENTS.md`) that determined the solution.
- **Upfront Evidence**: Attach empirical validation proof (test run summaries, CLI logs, reproduction traces, or browser check results) directly to the PR body using collapsible `<details><summary>` blocks (`not asked for later`).
- **Pre-Review Automated Checks**: Run and verify 100% green status across all local linters, unit tests, and browser/E2E checks BEFORE requesting human review or marking the PR ready.
- **Reviewer Routing**: Identify domain owners or `CODEOWNERS` and assign appropriate reviewers.

### 🧹 Post-Merge Lifecycle & Clean State
Immediately after merging a PR on `main`, agents MUST run `ce-ai workflow status` and archive completed changes via `ce-ai archive <feature>` to seal change packages and maintain zero workflow drift.
<!-- CE-AI MANAGED BLOCK END -->