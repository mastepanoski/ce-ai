# Technical Design: Context-Exhaustion Resilience

## Architecture Overview

```
[User / AI Agent]
       │
       ├── 1. Read AGENTS.md (Top ~25-Line Compact Invariant Index)
       ├── 2. Local Git Pre-Commit / Pre-Push Check (.githooks/ + core.hooksPath)
       ├── 3. ce-ai doctor Diagnostic Probes (Branch Protection + Hook Config)
       └── 4. GitHub API Branch Protection Gate (PR + Status Checks Required)
```

## System Interfaces & Schemas

### 1. `ce-ai doctor` Health Probes (`src/commands/doctor.rs`)
- Add `check_branch_protection_health`:
  - Executes `gh api repos/{owner}/{repo}/branches/main/protection`.
  - If HTTP status != 200 or `required_pull_request_reviews` / `required_status_checks` is missing, emits `Finding::BranchProtectionMissing`.
- Add `check_git_hooks_health`:
  - Executes `git config --get core.hooksPath`.
  - If output != `.githooks`, emits `Finding::GitHooksNotConfigured`.

### 2. Branch Protection Script (`scripts/protect-branch.sh`)
- Automated script using `gh api` to configure branch protection on `main`:
  - Required pull requests (`required_approving_review_count: 0`, `dismiss_stale_reviews: false`).
  - Required status checks (enforces CI matrix status checks before merging).
  - Enforces linear history and blocks force pushes.

### 3. Compact Invariant Index (`AGENTS.md`)
- A high-density ~25-line section at the top of `AGENTS.md` containing mandatory, non-negotiable imperative rules:
  1. NEVER commit or push directly to `main`. Always create a feature branch and open a PR (`gh pr create`).
  2. ALWAYS wait for 100% green CI matrix status checks (`gh pr checks --watch`) before merging.
  3. ALWAYS run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` before hand-off.
  4. ALWAYS use atomic writes (`crate::state::write_atomic`) for `state.json` and `opencode.json`.
  5. NEVER output superficial symptom patches or swallow failing assertions.
