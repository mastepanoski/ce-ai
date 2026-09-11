# Proposal: Extract Inline Unit Tests into Dedicated Test Files (#265)

## 1. Problem Statement
Currently, unit tests in `ce-ai` are embedded inline inside the same source files as production code using `#[cfg(test)] mod tests { ... }` blocks across 36 files in `src/`. This represents approximately **4,367 lines (~32%)** of test code interleaved directly into functional sources (e.g. `src/tui/mod.rs` contains 96% test code, and `src/commands/sync.rs` carries 226 test lines within 1,115 lines of total file length).

This inline mixing causes:
- **Poor Navigation & Cognitive Load:** Developers inspecting functional implementations must scroll past massive test fixtures and assertions.
- **Inflated PR Diff Sizes:** Minor test adjustments alter functional files, making code reviews harder and conflicting with the 400-changed-line review boundary documented in `CONTRIBUTING.md`.
- **Inconsistent Layout:** The repository has dedicated integration tests in `tests/` (`tests/cli.rs`, `tests/e2e.rs`, `tests/security.rs`), but unit tests remain mixed with source code.

## 2. In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **Mechanical Extraction:** Relocate all inline `#[cfg(test)] mod tests` blocks from `src/**/*.rs` into dedicated test files using the explicit `#[cfg(test)] #[path = "tests/<module>.rs"] mod tests;` or `#[path = "<module>_tests.rs"]` pattern.
- **Zero Behavioral / Assertion Changes:** Pure structural refactoring without adding, removing, renaming, or weakening any assertions or test logic.
- **Private Item Accessibility:** Retaining child module scoping (`mod tests`) so tests continue to access `pub(crate)` and private functions without making private items `pub`.
- **Domain-Partitioned Execution:** Chaining the refactor into self-contained, reviewable PRs under the 400-line limit across domain slices (`state`, `opencode`, `source`, `harness`, `commands`, `tui`, `error`).

### Out-of-Scope:
- Converting unit tests into integration tests (moving unit tests to the root `tests/` crate would break access to private internal functions).
- Refactoring test assertions, mocks, or business logic.
- Adding new test cases or altering production interfaces.

## 3. ISO/IEC 42001 & NIST AI RMF Risk Register

| Risk ID | Description | Severity | Mitigation |
| :--- | :--- | :--- | :--- |
| **R1** | Test drop or regression during mechanical relocation | High | Continuous verification gate (`cargo test` passes 100% of 119 tests before/after each unit). |
| **R2** | Inadvertent promotion of private APIs to `pub` | Medium | Maintain `mod tests;` as a child module of the parent so private visibility is preserved. |
| **R3** | Non-standard module resolution across platforms | Low | Use explicit `#[path = "..."]` attributes validated across Linux, macOS, and Windows CI runners. |
| **R4** | PR review fatigue / oversized diffs | Medium | Partition refactor into 6 isolated domain slices strictly `<400` LOC each. |

## 4. Success Criteria
1. **100% Separation:** Zero inline `#[cfg(test)] mod tests { ... }` multiline bodies remaining in functional source files under `src/`.
2. **100% Test Parity:** Exactly 119 unit/CLI tests + 5 security tests pass with zero skipped or modified assertions.
3. **Zero Lint / Compiler Warnings:** `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` report 0 findings.
4. **Clean Domain Layout:** Each domain under `src/` has a clean `tests/` subdirectory hosting the extracted test modules.
