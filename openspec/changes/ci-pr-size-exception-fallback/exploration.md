# Exploration: CI PR Size Budget Fork Exception Fallback and OpenSpec Exemption

## 1. Problem Investigation & Empirical Observations

In PR #440, external contributor `oTTa` submitted a fix for OpenCode V2 plugin support from a fork.
The total lines in the PR were:
- `openspec/changes/opencode-v2-plugin-loader/`: ~356 lines of Markdown
- `docs/`: ~40 lines
- `CHANGELOG.md`: 9 lines
- Plugin implementation + tests: ~310 lines
- Total diff: 722 lines.

The `pr-size-budget` job evaluated total changed lines at 722 (> 400), failed the build, and instructed the author to obtain a `size:exception` label.
The contributor attempted to run:
```bash
gh pr edit 440 --repo mastepanoski/ce-ai --add-label "size:exception"
```
which failed with:
```text
GraphQL: oTTa does not have the correct permissions to execute `AddLabelsToLabelable` (addLabelsToLabelable)
```

## 2. Alternatives Considered

### Option 1: GitHub Action with `pull_request_target` to Auto-Label Forks
- **Mechanism**: A dedicated workflow listening to `pull_request_target` with `issues: write` permission reads PR comments (e.g. `/size-exception <reason>`) or body and attaches the label.
- **Evaluation**: Rejected. `pull_request_target` introduces elevated privilege risks if not strictly isolated; introduces unnecessary workflow orchestration complexity and latency just to flip a label bit.

### Option 2: Solely Exclude `openspec/changes/**` and `docs/**` from Diff
- **Mechanism**: In `pr-size-budget`, filter out markdown/spec paths from `git diff --numstat`.
- **Evaluation**: Incomplete on its own. While it fixes PRs where OpenSpec docs pushed a small code change over the limit, it does not solve cases where the code itself genuinely exceeds 400 lines (e.g. large refactoring, unavoidable platform migrations) created by fork contributors who still cannot apply labels.

### Option 3: Dual Enhancement (Exempt Specs from Code Count + PR Description Directive Fallback) — Selected
- **Mechanism**:
  1. Compute Code lines separately from OpenSpec and docs. Code review cognitive burden is about code logic, control flow, and tests.
  2. If code lines > 400, evaluate both:
     - `labels` containing `size:exception` (maintainer-applied).
     - `body` containing a recognized directive/section (`### Size Exception`, `Size-Exception:`, `<!-- size-exception -->`, etc.).
  3. Emit a breakdown in `$GITHUB_STEP_SUMMARY` showing Code lines vs Docs lines.
- **Evaluation**: Selected. Meets all requirements, gives fork contributors self-service unblocking capability without granting repo write permissions, respects the 400 LOC code review boundary, and maintains human review gate via branch protection.
