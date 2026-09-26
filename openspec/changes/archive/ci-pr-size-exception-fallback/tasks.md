# Tasks: CI PR Size Budget Fork Exception Fallback and OpenSpec Exemption

## Work Units

### Work Unit 1: Update `.github/workflows/ci.yml` PR Size Budget Job (~40 LOC)
- [x] Split line diff computation into `CODE_TOTAL` (excluding lockfiles, `openspec/changes/**`, and `docs/**`) and `DOCS_TOTAL`.
- [x] Update `$GITHUB_STEP_SUMMARY` table to show Code, Docs & OpenSpec, and Combined Total.
- [x] Add PR body retrieval (`gh pr view --json body`) and regex pattern matching for size exception directives.
- [x] Update boundary check condition: pass if `HAS_LABEL == "true"` OR `HAS_BODY_EXEMPTION == "true"`.
- [x] Improve error message to explain PR description fallback.

### Work Unit 2: Update Governance Documentation & PR Template (~30 LOC)
- [x] Update `CONTRIBUTING.md` §2 (Counting Contract & Enforcement Status) reflecting the OpenSpec/docs separation and PR body exception option.
- [x] Update `.github/PULL_REQUEST_TEMPLATE.md` to reference the `### Size Exception` section in PR descriptions.

### Work Unit 3: Versioning & Changelog (~15 LOC)
- [x] Bump PATCH version in `Cargo.toml` (`1.72.2` -> `1.72.3`).
- [x] Add entry in `CHANGELOG.md` under `[1.72.3]`.

### Work Unit 4: Verification Gate (~0 LOC)
- [x] Verify shell regex against test strings in local environment.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test`.
- [x] Run `cargo run -- doc lint --strict`.
