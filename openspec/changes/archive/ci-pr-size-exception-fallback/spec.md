# Specification: CI PR Size Budget Fork Exception Fallback and OpenSpec Exemption

## 1. Requirements

### REQ-1: Separation of Code and Documentation/OpenSpec Counting
- **WHEN** `pr-size-budget` executes on a pull request,
- **THEN** it MUST calculate `CODE_TOTAL` excluding `Cargo.lock`, `*.lock`, `openspec/changes/**`, and `docs/**`.
- **THEN** it MUST calculate `DOCS_TOTAL` covering `openspec/changes/**` and `docs/**`.
- **THEN** both metrics MUST be displayed in the workflow step summary.

### REQ-2: Evaluation Threshold
- **WHEN** `CODE_TOTAL` is less than or equal to 400 lines,
- **THEN** the boundary check MUST succeed without requiring any label or body directive, regardless of `DOCS_TOTAL`.

### REQ-3: Dual Exemption Evaluation
- **WHEN** `CODE_TOTAL` exceeds 400 lines,
- **THEN** the check MUST succeed if either:
  1. The PR carries the GitHub label `size:exception`, OR
  2. The PR description body contains a recognized size exception section or directive (`### Size Exception`, `Size-Exception:`, `<!-- size-exception -->`, etc.).

### REQ-4: Fork-Friendly Rejection Guidance
- **WHEN** `CODE_TOTAL` exceeds 400 lines and neither condition in REQ-3 is met,
- **THEN** the workflow MUST exit with a non-zero status code and emit an error message explicitly instructing the author that they can add a `### Size Exception` section to the PR description (or ask a maintainer for the label).

### REQ-5: Documentation Alignment
- **WHEN** a contributor reads `CONTRIBUTING.md` or `.github/PULL_REQUEST_TEMPLATE.md`,
- **THEN** the instructions MUST accurately describe the exclusion of OpenSpec changes and docs from the code budget and explain the PR description exception mechanism.
