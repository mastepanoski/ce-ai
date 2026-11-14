# Plan: Contextual QA Skills Heuristics in ce-ai doctor & init-prj

## Overview
Add non-blocking heuristic detection for Playwright and Cypress test suites in projects during `ce-ai doctor` and `ce-ai init-prj`. When web/E2E test suites are detected and tactical QA skills are not yet installed, emit an advisory hint suggesting relevant Agent Skills (e.g., `npx skills add petrkindlmann/qa-skills playwright-pom`).

## Tasks

### Phase 1: Detection Engine (`src/source/tools_registry.rs` or `src/commands/doctor.rs`)
- [x] Implement `detect_qa_framework_opportunity(project_dir: &Path) -> Option<QaFrameworkSuggestion>` checking for:
  - Playwright configs (`playwright.config.{ts,js,mjs,cjs}`) or `@playwright/test` in `package.json`.
  - Cypress configs (`cypress.config.{ts,js,mjs,cjs}`, `cypress.json`) or `cypress` in `package.json`.
  - Verification that skill is not already installed in `.skills/`, `skills/`, `.agent/skills/`, or `.ce-ai/skills-registry.json`.
- [x] Unit tests for `detect_qa_framework_opportunity` covering Playwright, Cypress, `package.json`, existing skill suppression, and non-web projects.

### Phase 2: Integration with `ce-ai doctor` (`src/commands/doctor.rs`)
- [x] Wire detection into project adoption health check or doctor project evaluation.
- [x] Ensure output is formatted as `doctor-info:` and never counted as a failure finding or strict failure.
- [x] Add unit test in `src/commands/tests/doctor.rs`.

### Phase 3: Integration with `ce-ai init-prj` (`src/commands/init_prj.rs`)
- [x] Wire advisory hint into `adopt_project` when `!ctx.quiet` and not in dry-run mode (or as dry-run notice).
- [x] Add unit test in `src/commands/tests/init_prj.rs`.

### Phase 4: Verification & Definition of Done
- [x] Run `cargo fmt --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test`.
- [x] Add CLI integration test in `tests/cli.rs`.
- [x] Update `CHANGELOG.md` with new contextual QA skills recommendation feature.
- [x] Bump version in `Cargo.toml` (`3.11.0` - minor bump for new feature) and update `Cargo.lock`.
