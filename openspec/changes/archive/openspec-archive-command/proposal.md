# Proposal: OpenSpec Change Archival CLI Command & Ledger Synchronization

## Problem Statement
In `ce-ai v1.44.2` (Issue #323), `probe_unarchived_completed_changes` was introduced to detect change packages under `openspec/changes/` where all tasks in `tasks.md` are marked complete (`- [x]`). However, automatic directory moving was deliberately excluded from that probe to avoid unrequested background git tree mutations. Archival remained purely a manual convention:
1. Identifying completed change folders across `openspec/changes/`.
2. Executing shell commands (`mkdir -p openspec/changes/archive/ && git mv openspec/changes/<feature> openspec/changes/archive/<feature>`).
3. Manually editing markdown tables and sweep logs in `openspec/changes/archive/README.md`.
4. Manually handling changes that shipped under Criterion 2 (STATUS-verified shipped with live code/CHANGELOG evidence) despite residual open checkboxes.

Because no command-line tool existed, neither human operators nor AI agents consistently performed this tedious manual chore. As a result, **33 completed change folders** accumulated in `openspec/changes/`, causing continuous diagnostic noise (33 warning lines on every Turn-0 `ce-ai workflow resume` and `ce-ai doctor` execution).

An execution command is required to turn diagnosis into frictionless self-healing, providing a git-safe, atomic, and dual-criteria archival engine with batch remediation.

## In-Scope
1. **CLI Commands (`ce-ai workflow archive` & `ce-ai archive`)**:
   - `ce-ai workflow archive [feature]`: primary command within workflow subsystem.
   - `ce-ai archive [feature]`: top-level alias registered in CLI command dispatch for maximum developer ergonomics.
   - Targeting: archives named feature or defaults to active feature recorded in `state.json`.
2. **Batch Remediation & Preview (`--all`, `--dry-run`)**:
   - `--all`: Sweeps and archives all fully-completed change folders detected by `probe_unarchived_completed_changes`.
   - `--dry-run`: Reports planned moves and ledger updates without filesystem or git index mutations.
3. **Dual Completion Criteria Support**:
   - **Criterion 1 (Mechanical)**: Requires 100% tasks marked `[x]` (`completed == total > 0`).
   - **Criterion 2 (STATUS-Attested)**: Allows archiving changes with residual unchecked tasks via `--status "<release evidence>"`, injecting `> STATUS: <evidence>` into `tasks.md`.
4. **Git Safety & Zero-Data-Loss Mechanics**:
   - Collision detection: asserts `archive/<feature>` does not exist before moving.
   - Dirty tree check: verifies target folder has no uncommitted git conflicts or dirty modifications outside `tasks.md`.
   - Safe mover: uses `git mv` in git worktrees, falling back to atomic directory rename + git staging.
   - Journal integration: armed in install journal for rollback recovery on unexpected interruption.
5. **Ledger & State Synchronization**:
   - Programmatically appends newly archived features to the historical ledger in `openspec/changes/archive/README.md`.
   - Clears active feature context in `state.json` if the archived feature was currently active.
6. **Empirical Quality Verification**:
   - Unit tests covering safe mover, criterion validation, status injection, and ledger updating.
   - Integration CLI tests in `tests/cli.rs` covering `archive`, `archive --all`, `archive --dry-run`, `archive --status`, collision errors, and incomplete task rejections.

## Out-of-Scope
- **Unconditional Background Daemons**: No background daemon or file watcher that moves directories automatically without operator invocation.
- **Deleting Changes**: OpenSpec change folders are permanently preserved for auditability; deleting folders is strictly prohibited.
- **Automated Remote Push / PR Creation**: Git operations remain local (staging and index moves); publishing to remote repositories is handled by standard shipping workflows.

## Risk Evaluation & Mitigation
- **Risk (Accidental Data Loss during Move):** A moving operation might fail halfway or collide with existing archive directories.
  - *Mitigation:* Explicit collision checks (`dest.exists()`), validation of git working tree cleanliness, and journaling for rollback.
- **Risk (Fabricated STATUS Attestation):** Agents or developers might bypass task auditing with meaningless status strings.
  - *Mitigation:* Require `--status` to be non-empty and formatted with minimum length checks, recording the attestation prominently at line 1 of `tasks.md` and in the audit ledger.
- **Risk (Merge Conflicts across Branches):** Moving directories in git can cause tree conflicts if other branches are modifying files inside `openspec/changes/<feature>/`.
  - *Mitigation:* Archival occurs only on fully completed features, and `git mv` preserves git commit history cleanly so renames are tracked across merges.

## Success Criteria
1. Single command `ce-ai archive <feature>` successfully moves completed change to `archive/` and updates `archive/README.md`.
2. Running `ce-ai archive --all` archives all 33 backlog changes, resulting in `ce-ai doctor` reporting `openspec ledger: clean (0 pending archival)`.
3. Strict exit code mapping: Exit 0 on success, Exit 2 on bad usage/missing feature, Exit 3 on collision, Exit 6 on incomplete tasks without `--status`.
4. 100% green tests across `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings`.
