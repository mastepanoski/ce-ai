# Proposal: CI PR Size Budget Fork Exception Fallback and OpenSpec Exemption

## 1. Problem Statement

External contributors submitting pull requests from GitHub repository forks do not have write/triage permissions on the upstream `mastepanoski/ce-ai` repository. When a PR exceeds the 400-line review boundary enforced by the `pr-size-budget` CI workflow (`.github/workflows/ci.yml`), the CI job fails and advises the author to add or request a `size:exception` label.

When a fork contributor attempts to add the label via `gh pr edit <N> --add-label "size:exception"`, GitHub rejects the mutation with:
```text
GraphQL: <user> does not have the correct permissions to execute `AddLabelsToLabelable` (addLabelsToLabelable)
```
Consequently, external contributors cannot unblock CI independently. Furthermore, the existing line-counting contract (`git diff --numstat`) counts all non-lockfile text files without excluding mandatory OpenSpec specification files (`openspec/changes/<feature>/*.md`) or documentation (`docs/**/*.md`), even though `CONTRIBUTING.md` §2 explicitly declares that "Pure-documentation changes are exempt" and "(git diff --numstat tracks code files only)". Because a complete 5-file OpenSpec package easily spans 250–350 lines, legitimate feature/fix code changes are artificially penalized and pushed past the 400 LOC boundary by their own mandatory governance documents.

## 2. In-Scope / Out-of-Scope Boundaries

### In-Scope
- Update `.github/workflows/ci.yml` (`pr-size-budget` job) to:
  1. Exclude `openspec/changes/**` and `docs/**` from the core code-line budget, reporting Code lines vs Docs & OpenSpec lines separately in `$GITHUB_STEP_SUMMARY`.
  2. Support a fallback PR body directive/section (e.g. `### Size Exception` or `<!-- size-exception -->` or `size:exception:`) allowing authors from forks or local checkouts to provide an exemption rationale directly in their PR description.
  3. Emit actionable, empathetic error and notice messages explaining both mechanisms.
- Update `CONTRIBUTING.md` to formally document the counting exclusions and the PR description exception mechanism for external fork contributors.
- Update `.github/PULL_REQUEST_TEMPLATE.md` to guide authors on using the `### Size Exception` section when code changes exceed 400 lines.
- Bump SemVer in `Cargo.toml` (PATCH) and record changes in `CHANGELOG.md`.

### Out-of-Scope
- Changing the 400 LOC boundary threshold for actual code review.
- Modifying repository branch protection or merge requirements.
- Changing `ce-ai` CLI runtime code (the change is CI workflow, templates, and documentation).

## 3. Risk Evaluation & Mitigation
- **Risk**: PR authors bypass the 400-line boundary indiscriminately by adding a dummy header.
  - **Mitigation**: GitHub branch protection still requires repository maintainer approval before any PR can be merged; the PR description rationale is publicly visible for reviewers to scrutinize during code review.
- **Risk**: CI workflow breaks due to syntax errors in shell scripting.
  - **Mitigation**: Robust portable regex tested across POSIX grep and bash, with graceful fallbacks.

## 4. Success Criteria
1. When code changes ≤ 400 LOC, `pr-size-budget` passes regardless of OpenSpec package size.
2. When code changes > 400 LOC, `pr-size-budget` passes if `size:exception` label exists OR if PR description includes an explicit `### Size Exception` section / directive.
3. If neither is present, `pr-size-budget` fails with a clear message explaining how fork contributors can add the rationale to their PR body.
4. All local tests, formatters, and clippy passes cleanly.
