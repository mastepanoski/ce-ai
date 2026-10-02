# Exploration: CE-AI v2 Architectural Pivot & PRD

## Context & Upstream Feedback Analysis

The upstream maintainer of Compound Engineering (`EveryInc/compound-engineering-plugin`) reviewed `ce-ai`'s codebase and architecture, providing authoritative feedback across five critical architectural axes:

### Axis 1: Probabilistic Execution vs. Deterministic State
- **Upstream Reality:** CE enforces determinism only within individual skills managing transitions (e.g. `ce-commit-push-pr` git checks, `ce-sweep` state file, `ce-work` `mode:return-to-caller` run receipts).
- **ce-ai Antipattern:** Maintaining an external `state.json` with a global stage cursor (`current_stage`). Because repository artifacts (plan, branch, commit, PR) are the true state, `state.json` continually drifts, necessitating huge heuristic reconciliation code.
- **Evaluation:** Keeping `state.json` as an authoritative cursor is fundamentally flawed. Resuming after compaction or session boundaries should inspect artifacts (`ce-handoff`, `ce-work` recovery) directly.

### Axis 2: Cross-Host Sync and Drift
- **Upstream Reality:** Each host (Claude Code, OpenCode, Codex, Pi, Cursor, Copilot) manages plugins through its own native package/marketplace system. Upstream builds specific rewrites per target.
- **ce-ai Antipattern:** Scraping release tarballs and copying raw `skills/` directly onto disk bypasses target rewrites. Running a watch-loop to restore user edits alongside native installs creates dual sources of truth and `external-duplicate` warnings.
- **Evaluation:** CE-AI should not attempt to be a package installer that duplicates files. Instead, CE-AI should govern versions across hosts by driving native installers (`claude plugin install`, `codex ...`) and auditing version alignment.

### Axis 3: Documentation and Knowledge Debt
- **Upstream Reality:** CE addresses knowledge hygiene internally: `ce-compound` has a high bar (only capturing delta reasoning not evident in code/tests), and `ce-compound-refresh` + `compound audit` validate frontmatter, broken links, and dead paths deterministically.
- **ce-ai Antipattern:** Enforcing mandatory OpenSpec files (`proposal.md`, `spec.md`, `tasks.md`) for every change before allowing code edits inflates documentation volume and creates maintenance drag.
- **Evaluation:** CE-AI should align with CE's tiered documentation model (e.g. lightweight ODD fast-paths for bugs/trivial edits) and integrate directly with `compound audit` rather than building conflicting parallel validators.

### Axis 4: Coupling Incompatibilities
The review identified seven concrete bugs/divergences:
1. `docs/solutions/` schema mismatch: CE requires `module`, `date`, `problem_type`, `component`, `severity` (with optional `tags` and `applies_when` limited to knowledge-track). CE-AI required `title`, `tags`, `applies_when`.
2. Component attribute parsing: CE uses `component` + `related_components`; CE-AI reads `components`.
3. Plan directory: Upstream writes new brainstorms as requirements-only plans in `docs/plans/`; `docs/brainstorms/` is legacy.
4. Docs root: All CE paths respect `docs_root` in `.compound-engineering/config.yaml`; CE-AI hardcodes `docs/`.
5. Dead path detection: CE-AI checks only `.rs` under `src/` and `tests/`; CE supports multi-language codebases.
6. Pi extension collision: CE-AI writes `.pi/extensions/compound-engineering.ts`, colliding with CE's own Pi entrypoint.
7. Gate exemption: CE-AI uses fragile substring matching for `ce-debug`.

### Axis 5: Integration Points vs. Anti-Patterns
- **What to build on (Stable Contracts):**
  - Documented skill names and mode tokens (`mode:return-to-caller`, returned JSON fields).
  - `docs_root` configuration in `.compound-engineering/config.yaml`.
  - Upstream `schema.yaml` with explicit `schema_version`.
  - Native host install CLI commands, audited by released plugin version.
- **What NOT to build on (Volatile Internals):**
  - `skills/` directory layout.
  - Skill markdown prose and instructions.
  - Bundled scripts, references, and internal caches.
  - File SHA256 hashes.

## Ruled-Out Alternatives

1. **Retaining Authoritative FSM with Enhanced Heuristics:**
   - *Ruled Out:* Continuing to sync `state.json` against repo artifacts using more complex regexes or heuristics.
   - *Reason:* Fundamentally duplicates state. Two sources of truth will always diverge. Upstream explicitly advised against storing state separately from git artifacts.
2. **Maintaining File-Level Tarball Sync alongside Native Installers:**
   - *Ruled Out:* Downloading GitHub release `.tar.gz` and copying files to disk.
   - *Reason:* Skips platform-specific compilation/rewrites and produces `external-duplicate` conflicts.
3. **Maintaining Bespoke Solution Schema Validation:**
   - *Ruled Out:* Retaining custom frontmatter keys (`applies_when` for bug-track docs).
   - *Reason:* Breaks upstream compatibility with `compound audit` and flags valid CE documents as invalid.
