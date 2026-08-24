# Contributing to `ce-ai`

First off, thank you for considering contributing to `ce-ai`! It is through contributions like yours that `ce-ai` remains a secure, high-quality, and robust plugin manager for AI coding harnesses.

---

## 🛡️ Governance & Security Compliance Standards

All contributions to `ce-ai` must comply with our foundational security and AI management standards:

- **ISO/IEC 27001 & 27002**: Strict Information Security Management, access controls, supply chain security, and cryptographic integrity.
- **NIST Cybersecurity Framework (CSF) & SP 800-53**: Secure software development lifecycle, continuous vulnerability monitoring, and atomic system restoration.
- **ISO/IEC 42001**: Artificial Intelligence Management System (AIMS) governance for AI tool delivery and agent model management.
- **NIST AI Risk Management Framework (AI RMF 1.0)**: Systematic Risk Mapping, Measurement, Governance, and Management for AI integration plugins.

Refer to [`SECURITY.md`](./SECURITY.md) and [`AI_POLICY.md`](./AI_POLICY.md) for full compliance directives.

---

## 🛠️ Development Setup & Workflow

### Prerequisites
- **Rust toolchain** (latest stable release): `rustup update stable`
- **Docker Engine** (required for containerized E2E gate testing)
- **Node.js 20+** & `npm`

### Local Setup
```bash
# Clone the repository
git clone git@github.com:mastepanoski/ce-ai.git
cd ce-ai

# Build the debug binary
cargo build

# Run unit and integration test suite
cargo test

# Run formatting and clippy linter checks
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Docker E2E Gate Execution
Before submitting a pull request, you MUST verify that the containerized end-to-end gate passes cleanly:
```bash
make e2e
```
This builds an isolated Linux container (`Dockerfile.e2e`) and validates `ce-ai install`, `sync`, `models`, `status`, and `uninstall` flows against OpenCode.

---

## 📜 Pull Request Guidelines

1. **PR Template**: Open all pull requests using the repository [PR template](./.github/PULL_REQUEST_TEMPLATE.md); every verification gate and checklist item must be satisfied before review.
2. **Branch Naming**: Use descriptive branch names: `feature/description`, `fix/issue-name`, or `docs/update`.
3. **Conventional Commits**: Commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
   - `feat(models): add support for custom profile export`
   - `fix(sync): resolve atomic rename lock race`
   - `docs(security): update ISO 27001 risk matrix`
4. **100% Test Coverage**: New features or bug fixes must include unit or CLI integration tests.
5. **No Breaking Changes**: Preserve existing CLI contracts and JSON schema formats unless explicitly documented and reviewed.
6. **Security Audit**: Code must pass `cargo audit` without open vulnerability alerts.
7. **Changed-Lines Forecast**: Every PR description includes a changed-lines forecast; the 400-line review boundary and bounded correction policy below apply.

---

---

## 📏 PR Size Boundaries & Bounded Corrections

These boundaries apply equally to human and AI-agent contributors. Documentation tone follows the [Documentation Style Guide](docs/references/docs-styling.md).

### Review Boundary

**400 changed lines** per pull request (added + deleted per the counting contract). In Gentle AI this boundary is enforced by the **Review Workload Guard** as a delivery-policy decision — single PR vs chained PRs (`stacked-to-main` or `feature-branch-chain`) with strategies `ask-on-risk`, `auto-chain`, `single-pr`, `exception-ok`. It is a review-workload split signal, never a cap on what an agent may write.

- Below the boundary: zero extra ceremony.
- Above it: chain into independently shippable slices (each with its own tests and rollback boundary) or attach an explicitly approved **size exception** documented in the PR.
- Every PR description includes a **Changed-Lines Forecast** (`git diff --numstat origin/main...HEAD`) so reviewers see the budget before reading code.

### Counting Contract

- Sum added + deleted lines across non-binary files (`numstat` binary rows report `-` and are skipped).
- Excluded from the budget: lockfiles (`Cargo.lock`) and any path declared as generated/vendored in `.gitattributes`. Whitespace-only churn does not count.

### Bounded Correction Policy

Fixes responding to CI/review findings on an open PR cap at:

```text
min(200, ceil(original_pr_changed_lines / 2))
```

changed lines per review cycle.

- At most **one bounded correction per cycle**; defects exceeding the budget become scoped follow-up issues/PRs reviewed independently — re-planning is the escape hatch, never another patch round.
- Maintainer-approved exceptions are documented in the PR.
- Pure-documentation changes are exempt.

### Work-Unit Budgets (OpenSpec)

`tasks.md` work units target ~200 changed lines each; a rescope may only narrow a budget — widening requires a new spec revision.

### Size Is Not Risk

Volume triggers splitting and review-burden handling only. Risk classification stays evidence-based — authentication, payments, data-loss surfaces, shell/process execution — regardless of line count.

### Source Constants

Guard trigger conditions live in the SDD orchestrator asset (`internal/assets/hermes/sdd-orchestrator.md`: `400-line budget risk: High`, `estimated changed lines exceed 400`).

Adopted from Gentle AI v2.4.0 (`Gentleman-Programming/gentle-ai`): `LargeChangeLines=400`, `MaxCorrectionChangedLines=200`, `CorrectionBudget` (floor-two variant), `MaxCompactCorrectionAttempts=1` (`internal/reviewtransaction/{risk,compact}.go`); `DefaultRuntimeChangedLines=200`, `DefaultRuntimeAttemptLimit=2` (`internal/sddstatus/runtime_ledger.go`). This document is authoritative for this repository regardless of upstream drift.

### Enforcement Status

Documentation-first. A fast-follow CI job will compute numstat totals per PR and gate above-boundary changes behind a `size:exception` label; until then the PR-template forecast is mandatory.

---

## 💬 Community Code of Conduct

Please note that this project is released with a Contributor Code of Conduct. By participating in this project you agree to abide by its terms. See [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
