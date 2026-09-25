# Specification: Self-Explaining PR Directives & Review Readiness

## Requirements & Acceptance Criteria

### Requirement 1: Managed Block Template Enhancement
**WHEN** `ce-ai init-prj` is executed with `--tier full`,  
**THEN** the injected managed block in `AGENTS.md` MUST contain the `### 🚀 Self-Explaining Pull Requests & Review Readiness` section prescribing:
1. Documenting what was ruled out (from `exploration.md`).
2. Documenting which rule decided it (from `design.md` / `AGENTS.md`).
3. Attaching empirical verification evidence upfront in collapsible `<details>` blocks.
4. Gating review requests behind 100% green automated/browser checks.
5. Routing review requests to domain owners / `CODEOWNERS`.

**WHEN** `ce-ai init-prj` is executed with `--tier minimal` or `--tier orchestrator`,  
**THEN** the injected managed block MUST contain proportional directives for self-explaining PRs, upfront evidence, and pre-review automated checks.

### Requirement 2: Adoption Block Version Bump & Staleness Detection
**WHEN** `BLOCK_VERSION` is bumped to `6`,  
**THEN** projects adopted with `v <= 5` MUST be classified by `ce-ai doctor` and `ce-ai status` as `StaleVersion`, outputting a recommendation to re-run `ce-ai init-prj`.

**WHEN** `ce-ai init-prj` is re-run on an adopted project,  
**THEN** the existing managed block MUST be cleanly replaced with the v6 block, and `state.json` updated with `block_version: 6`.

### Requirement 3: Repository Governance Alignment
**WHEN** an AI agent inspects `ce-ai`'s root `AGENTS.md` or `CONTRIBUTING.md`,  
**THEN** it MUST find explicit guidance affirming that:
1. The 400 LOC review boundary is strictly preserved (atomic changes are required for clear self-explanation).
2. Upfront verification evidence in PR bodies should use collapsible `<details>` blocks to respect the ~100–150 line description budget without counting against code diff LOC.
