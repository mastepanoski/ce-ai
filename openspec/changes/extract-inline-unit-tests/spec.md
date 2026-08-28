# Formal Specification & Requirements Matrix (#265)

## 1. Requirements

### REQ-EXTRACT-1: Pure Structural Test Relocation
- **WHEN** any source file `src/**/*.rs` is compiled in test mode (`cargo test`),
- **THEN** all unit tests MUST execute from their respective extracted test files located under `src/**/tests/*.rs` or `src/tests/*.rs`.

### REQ-EXTRACT-2: Zero Behavior or Logic Regressions
- **WHEN** running `cargo test` and `make e2e` across all platforms (Linux, macOS, Windows),
- **THEN** exactly the same test suite (119 unit/CLI tests, 5 security tests) MUST pass 100% green without any skipped, modified, or silenced assertions.

### REQ-EXTRACT-3: Preservation of Encapsulation
- **WHEN** extracting unit tests that inspect `pub(crate)` or private helper functions,
- **THEN** the parent file MUST declare `mod tests;` as a child module using `#[path = "..."]`, and NO internal production items shall be widened to public visibility solely to accommodate test extraction.

### REQ-EXTRACT-4: Clean Production Source Files
- **WHEN** inspecting any functional source file in `src/`,
- **THEN** the file MUST NOT contain multi-line inline test bodies (`mod tests { ... }`), keeping functional files clean and focused solely on business logic.

### REQ-EXTRACT-5: PR Size Gate Compliance
- **WHEN** submitting pull requests for each domain extraction slice,
- **THEN** each PR MUST contain fewer than 400 lines of changes and pass 100% of CI matrix checks.

## 2. Acceptance Criteria Matrix

| Requirement | Test Method | Pass Criteria |
| :--- | :--- | :--- |
| `REQ-EXTRACT-1` | `grep -rn '#\[cfg(test)\]' src/` | Only module forward declarations (`mod tests;`), zero multi-line inline test bodies. |
| `REQ-EXTRACT-2` | `cargo test --all-targets` | 119 unit/CLI + 5 security tests passed; 0 failed; 0 ignored. |
| `REQ-EXTRACT-3` | `cargo clippy --all-targets -- -D warnings` | Zero visibility warnings, 0 clippy warnings. |
| `REQ-EXTRACT-4` | Line count audit | Substantial reduction in functional source file lengths across all 36 files. |
| `REQ-EXTRACT-5` | `gh pr checks` | 100% green matrix on Linux, macOS, Windows. |
