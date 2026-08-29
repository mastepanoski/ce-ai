---
module: test-architecture
tags: [tests, refactor, separation-of-concerns, file-layout, path-attribute]
problem_type: architecture
---

# Separation of Test Code from Functional Code Across Source Files

## Problem
Prior to [Issue #265](https://github.com/mastepanoski/ce-ai/issues/265), unit tests throughout `ce-ai` were embedded directly at the bottom of production source files within inline `#[cfg(test)] mod tests { ... }` blocks across 36 source files. While common in Rust for small modules, as domain implementations expanded (e.g., harness adapters, sync matrix, TUI, commands), source files grew bloated (several exceeding 500–1,000 LOC), mixing production business logic with hundreds of lines of test fixtures and assertions.

## Solution
Extracted all inline `mod tests` blocks into dedicated sibling test files under domain-specific `tests/` directories using Rust's explicit `#[path = "..."]` module declaration pattern:

```rust
#[cfg(test)]
#[path = "tests/<module>.rs"]
mod tests;
```

For directory entrypoints (`mod.rs`), test files were mapped to `tests/mod_tests.rs`:

```rust
#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
```

### Execution Strategy & Phasing
The refactor was divided into 7 bounded, domain-scoped units to respect PR size budgets and preserve incremental verification:
1. **Unit T1 (PR #266)**: Formal OpenSpec definition (`openspec/changes/extract-inline-unit-tests/`).
2. **Unit T2 (PR #267)**: State and OpenCode domains (`src/state/`, `src/opencode/`).
3. **Unit T3 (PR #268)**: Source retrieval and error domains (`src/source/`, `src/error.rs`).
4. **Unit T4 (PR #269)**: Harness adapters part 1 (`agents.rs`, `pi.rs`, `claude.rs`, `copilot.rs`, `grok.rs`, `codex.rs`).
5. **Unit T5 (PR #270)**: Harness adapters part 2 and TUI (`custom.rs`, `agy.rs`, `cursor.rs`, `fx.rs`, `kimi.rs`, `harness/mod.rs`, `tui/mod.rs`).
6. **Unit T6 (PR #271)**: Command handlers part 1 (`upgrade.rs`, `tools.rs`, `audit.rs`, `models.rs`, `guard.rs`).
7. **Unit T7 (PR #272)**: Command handlers part 2 (`sync.rs`, `doctor.rs`, `init_prj.rs`, `workflow.rs`, `install.rs`).

## Key Learnings
1. **Zero-Logic Drift Guarantee:** Using `#[path = "..."]` retains exact `super::*` access to module-private items and visibility without altering any production structs, visibility modifiers, or test assertions.
2. **Deterministic Mutex Sharing:** Test synchronization utilities (such as `HARNESS_ENV_LOCK` in `src/harness/tests/mod_tests.rs`) remained accessible to child adapter test modules without needing global test harness modifications.
3. **Clean Codebase Structure:** Production files now focus purely on functional logic and interfaces, improving readability, compile-time inspection, and cognitive focus.
