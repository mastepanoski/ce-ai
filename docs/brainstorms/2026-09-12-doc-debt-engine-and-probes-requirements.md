# Requirements: Documentation Technical Debt Diagnostic Engine & Probes (Increments 1 & 2)

- **Date:** 2026-09-12
- **Topic:** `doc-debt-engine-and-probes`
- **Scope:** Standard (Bounded Cross-Cutting CLI Feature)
- **Handoff Target:** `/ce-plan` (OpenSpec Stage 2 & 3)
- **Provenance:** Seeded from `docs/ideation/2026-09-12-docs-debt-consolidation-and-doctor-ideation.html`

---

## 1. Summary

Implement a unified, non-blocking documentation technical debt diagnostic engine across `RepoState`, `workflow resume`, `workflow status`, and `ce-ai doctor`. The feature delivers an extensible umbrella block (`DocDebtReport`) with configurable thresholds in `.ce-ai.json`, first-class No-Git resilience (tri-state reporting), and three deterministic diagnostic probes: (1) Git-aware OpenSpec subtask desync detection, (2) stale pending OpenSpec inactivity watchdog, and (3) `docs/solutions/` drift auditing (dead file paths and YAML frontmatter validation). Additionally, it includes a pedagogical, professor-style guide for beginners (`docs/user-guide/doc-hygiene-and-debt-explained.md`) and a newbie-accessible update to `README.md` explaining the problem and value while preserving the ≤ 100 lines limit. Active compaction (`ce-ai archive compact`) is explicitly decoupled into a subsequent increment.

---

## 2. Problem & Grounding Context

In Compound Engineering, documentation acts as the project's living contract and memory layer. Over prolonged iteration cycles, documentation inevitably accumulates technical debt:

1. **Subtask Desync Stranding Completed Features:** `probe_unarchived_completed_changes` in `src/commands/workflow.rs` strictly checks `completed == total`. Features with 100% top-level tasks checked (`- [x] Task 1`) but open subtasks (`- [ ] 1.1`), or with code already merged to `main` while a post-merge release checkbox remains open (e.g. `doctor-kimi-marketplace-divergence`, `harness-manifest-sha256-coverage`, and `linux-musl-static-release`), are ignored by `doctor` and linger indefinitely in `openspec/changes/`.
2. **Silent OpenSpec Inactivity Rot:** Changes abandoned or stalled with 0% or partial task completion receive zero diagnostics from `doctor`, cluttering active change directories.
3. **Solution Library Drift in `docs/solutions/`:** Refactorings and file moves render past solution file paths obsolete. AI coding agents searching `docs/solutions/` ingest dead file pointers and hallucinate deleted APIs or superseded patterns.
4. **Substrate Concealment Antipattern:** Collapsing missing substrate (such as environments without `.git`) to `clean` or `0` creates false confidence by equating absence of diagnostic signal with absence of debt.
5. **Newbie Comprehension Gap:** Documentation technical debt is often perceived as an abstract aesthetic concern. Without a pedagogical, grounded explanation of *why* doc rot misleads AI coding agents and *how* automated probes protect development velocity, beginners struggle to grasp the value of doc hygiene.

---

## 3. Architecture & Phased Strategy

To maintain strict compliance with the ~200 LOC per work unit policy and prevent oversized pull requests (>400 LOC), this feature is phased as follows:

```
[Increment 1: The Umbrella] ──► [Increment 2: Core Diagnostic Probes] ──► [Increment 3: Active Compaction (Separate PR)]
- DocDebtReport in RepoState    - Probe 1: Git-aware desync & landing     - ce-ai archive compact
- .ce-ai.json thresholds         - Probe 2: Stale pending inactivity      - Generational archive rollup
- Tri-state & No-Git support    - Probe 5: Solution path & YAML lint      - Milestone summaries
- Turn-0 delivery banner        - Actionable CLI guidance in doctor
                                - Pedagogical Newbie Guide & README Update
```

---

## 4. Requirements (R-IDs)

### R1. First-Class Substrate Tri-State (`ProbeStatus<T>`)
Every probe in the documentation debt engine MUST report its evaluation state as an explicit tri-state enum:
- `Clean`: The substrate was inspected and no technical debt was detected.
- `Debt(T)`: The substrate was inspected and technical debt findings were identified.
- `Unknown`: The underlying substrate is missing, inaccessible, or indeterminate (e.g. No `.git` directory, missing `tasks.md`, unreadable directory).

Under no circumstances shall a probe return `Clean` or `0` when the substrate is missing.

### R2. Umbrella `DocDebtReport` in `RepoState`
Extend `RepoState` in `src/commands/workflow.rs` with an optional or defaulted field:
```rust
pub doc_debt: Option<DocDebtReport>
```
`DocDebtReport` aggregates the tri-state results of all active documentation probes, their respective finding counts, and overall substrate availability. Serialization MUST support JSON output (`--json`) for automation consumers.

### R3. Workspace Override Thresholds in `.ce-ai.json`
`State::load_with_workspace_overrides` (in `src/state/state.rs` and `src/state/ports.rs`) MUST deserialize an optional `doc_hygiene` configuration block from workspace-local `.ce-ai.json`:
```json
{
  "doc_hygiene": {
    "stale_spec_days": 21,
    "check_solution_paths": true,
    "require_solution_frontmatter": true
  }
}
```
Default values apply when `.ce-ai.json` is omitted or does not define `doc_hygiene`.

### R4. Probe 1 — Git-Aware OpenSpec Desync & Landing Probe
A dedicated probe scanning all non-archive directories in `openspec/changes/`:
1. **Parent-Task Desync:** Identifies change folders where 100% of top-level checklist items (`- [x] **Task N**`) are checked, but one or more indented subtasks (`- [ ] N.M`) remain unchecked.
2. **Git-Merged Landing:** When Git substrate is available, verifies if commits referencing `openspec/changes/<feature>/` or matching the feature slug have already landed on `HEAD` (or the default branch `main`).
3. **Superseded Release:** Verifies if `tasks.md` cites a release version tag that has already been surpassed in `Cargo.toml` or git tags.
When any condition matches, the probe reports the feature as `CandidateCompleted` rather than dormant pending.

### R5. Probe 2 — Stale Pending OpenSpec Decay Watchdog
A probe scanning `openspec/changes/` for incomplete changes:
1. Calculates elapsed days since the last Git commit touching `openspec/changes/<feature>/`.
2. In No-Git environments, gracefully falls back to filesystem metadata (`mtime`).
3. If elapsed time exceeds `doc_hygiene.stale_spec_days` (default: 21 days) and tasks remain incomplete without matching Probe 1, reports the feature as `StalePending` with elapsed days and completion ratio (`completed/total`).

### R6. Probe 5 — Solution Library Drift Probe
A probe scanning all markdown files in `docs/solutions/**/*.md`:
1. **File Path Existence Check:** Extracts backticked project file paths matching repository patterns (e.g. `src/**/*.rs`, `tests/**/*.rs`). Validates that the referenced files exist in the working directory. Reports dead file pointers. External URLs (`http://`, `https://`) and general symbols are ignored to eliminate false positives.
2. **YAML Frontmatter Linter:** Validates that each solution file contains valid YAML frontmatter containing non-empty `title`, `category` (or `module`), `problem_type`, `tags`, and `applies_when`. Reports missing or malformed frontmatter.

### R7. Actionable Diagnostic Delivery in `doctor`
`ce-ai doctor` (in `src/commands/doctor.rs`) MUST report documentation debt findings as non-blocking diagnostic warnings:
- Desynced / Landed OpenSpecs: `doctor-warn: openspec change '<feature>' is merged or tasks completed with open subtasks (progress: X/Y) — run 'ce-ai archive <feature> --auto-mark' or 'ce-ai archive <feature> --status "..."' to archive`
- Stale Pending OpenSpecs: `doctor-warn: openspec change '<feature>' has been pending for X days with no activity (progress: X/Y) — resume, shelve, or archive with '--status "abandoned/superseded"'`
- Solution Drift: `doctor-warn: solution '<rel_path>' references non-existent path '<dead_path>'` or `doctor-warn: solution '<rel_path>' missing required YAML frontmatter '<field>'`
Findings remain informational/warning by default (`Ok(())`) to prevent breaking builds.

### R8. Compact Turn-0 Banner in `workflow resume` & `status`
`ce-ai workflow resume` and `workflow status` MUST format a single-line summary of `DocDebtReport`:
- When Git is present and debt exists: `doc debt: 2 unarchived (code merged), 1 stale spec (34d), 2 dead solution links`
- When Git substrate is missing: `doc debt: 1 stale spec [git: n/a], 2 dead solution links`
- When no debt exists: Omit the line or emit compact zero-indicator depending on verbosity.
Execution time of Turn-0 probes MUST remain strictly under 15ms.

### R9. Pedagogical "Professor-Style" Guide for Newbies
Author a dedicated educational guide in `docs/user-guide/doc-hygiene-and-debt-explained.md`:
1. **Diátaxis Intent:** Pure Explanation quadrant.
2. **Audience & Tone:** Tailored for a newcomer/beginner. Written with an empathetic, pedagogical professor persona ("Professor's Walkthrough") using intuitive analogies:
   - Comparing documentation in AI systems to the AI's long-term memory.
   - Explaining why documentation rots (code moves fast, docs stand still).
   - Demonstrating the catastrophic consequence: AI assistants hallucinating deleted functions and re-introducing fixed bugs because they read stale solutions.
   - Showing why OpenSpec changes get stranded (human developers forget to tick subtasks or post-merge tags).
   - Explaining how `ce-ai doctor` acts as an automated health checkup and how to remediate findings in seconds.
3. **Style Guide Compliance:** Follows `docs/references/docs-styling.md` (no mixed quadrants, repo-relative paths, clear signposting).

### R10. Newbie-Friendly `README.md` & Documentation Map Update
Update `README.md` to communicate the problem and solution to any newcomer:
1. Explain in the introductory overview and quick path *why* `ce-ai doctor` inspects documentation health (preventing AI hallucination and stalled specifications).
2. Add an audience-labeled row to the Documentation Map table:
   `| 🎓 [Documentation Debt & Hygiene Explained](docs/user-guide/doc-hygiene-and-debt-explained.md) | **Beginner** | Explanation — why doc rot misleads AI agents, OpenSpec lifecycle & doctor health checks |`
3. **Hard-Gate Invariant:** Strictly preserve the `README.md` length constraint: **MUST stay ≤ 100 lines**.

---

## 5. Scope Boundaries

### Included in Increments 1 & 2
- Pure diagnostic observation, reporting, and configuration.
- Extending `RepoState` and serializing tri-state `DocDebtReport`.
- Probes 1, 2, and 5 with No-Git fallbacks.
- Formatted output in `workflow resume`, `workflow status`, and `doctor`.
- Pedagogical newbie guide (`docs/user-guide/doc-hygiene-and-debt-explained.md`).
- Concise, newbie-friendly `README.md` update (≤ 100 lines).
- Unit and CLI integration test coverage validating both Git-present and No-Git environments.

### Excluded / Deferred
- **Increment 3 (Compaction):** `ce-ai archive compact`, generational archive rollups, and tarball packaging are deferred to a dedicated, independent PR.
- **Living Spec Promotion (Idea 4):** Formal promotion of delta specs to `openspec/specs/` is deferred as an architectural methodology standard.
- **Automated Solution Clustering (Idea 6):** Defer clustering / vector similarity until solution volume warrants it.
- **Automated Silent Disk Mutation:** No silent modification of files during `doctor`.

---

## 6. Edge Cases & Substrate Degradation Scenarios

| Scenario | Substrate State | Probe Behavior | Output Contract |
| :--- | :--- | :--- | :--- |
| **Clean Repo with Git** | `.git` present, all clean | All probes evaluate to `Clean` | `doc debt: clean` (or omitted) |
| **No `.git` Directory** | `.git` absent (tarball / export) | Probe 1 & Probe 2 git landing checks return `Unknown`. Probe 2 falls back to `mtime`. Probe 5 checks files normally. | Banner displays `[git: n/a]`. Never reports `0 debt` for git-dependent checks. |
| **Empty `docs/solutions/`** | Directory missing or empty | Probe 5 returns `Clean` (nothing to drift) | No warnings emitted. |
| **Empty `openspec/changes/`** | Directory missing or empty | Probes 1 & 2 return `Clean` | No warnings emitted. |
| **Corrupted `tasks.md`** | Unparseable / non-UTF8 | Probe returns `Debt(MalformedTasks)` | Explicit warning pointing to file. |
| **Dirty Working Tree** | Uncommitted edits in change folder | Archival hints remind user to commit before archiving | Respects existing safe mover guards. |

---

## 7. Success & Verification Criteria

1. **Reproduction Test:** The 3 real-world stranded changes in `ce-ai` (`doctor-kimi-marketplace-divergence`, `harness-manifest-sha256-coverage`, and `linux-musl-static-release`) are immediately detected by Probe 1 and surfaced in `doctor` with actionable archival commands.
2. **Substrate Hermeticity:** Tests executing in a non-git temporary directory (`tempfile::tempdir()` without `git init`) verify that probes return `Unknown` / `[git: n/a]` and do not panic or emit false `Clean` reports.
3. **Turn-0 Performance:** In-memory benchmark confirms `probe_repo_state` with all three probes executes in under 15ms.
4. **Pedagogical Doc Quality:** The guide in `docs/user-guide/doc-hygiene-and-debt-explained.md` explains documentation technical debt using intuitive analogies accessible to someone on Day 1 of Compound Engineering.
5. **README Invariant:** `README.md` stays ≤ 100 lines while clearly explaining the problem and linking to the new guide.
6. **Code Quality Gates:**
   - Zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
   - Strict formatting (`cargo fmt --check`).
   - 100% passing tests (`cargo test`).
