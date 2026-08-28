# Technical Exploration & Architectural Tradeoffs (#265)

## 1. Context & File Inventory

A survey of `src/` shows 36 files with embedded `#[cfg(test)]` modules distributed across 7 domains:

```
src/
├── error.rs                  (1 test module)
├── state/                    (7 files with tests: mod.rs, ports.rs, backups.rs, journal.rs, profiles.rs, state.rs, diff.rs)
├── opencode/                 (3 files with tests: config.rs, manifest.rs, plugins.rs)
├── source/                   (5 files with tests: cache.rs, tools_registry.rs, registry.rs, release.rs, archive.rs)
├── harness/                  (11 files with tests: agents.rs, pi.rs, claude.rs, copilot.rs, grok.rs, codex.rs, custom.rs, agy.rs, mod.rs, cursor.rs, fx.rs, kimi.rs)
├── commands/                 (9 files with tests: upgrade.rs, tools.rs, audit.rs, models.rs, sync.rs, doctor.rs, init_prj.rs, workflow.rs, install.rs, guard.rs)
└── tui/                      (1 file with tests: mod.rs)
```

Total lines to extract: **~4,367 lines** across 36 files.

## 2. Structural Options Evaluated

### Option A: Move all unit tests to the root `tests/` crate
- **Mechanism:** Relocate tests into `tests/unit_*.rs`.
- **Pros:** Completely separates test files into the root `tests/` tree.
- **Cons:** **Rejected.** Rust's integration test crate (`tests/*.rs`) only links against the library's public (`pub`) interface. Many unit tests in `ce-ai` (e.g. `diff.rs`, `journal.rs`, `ports.rs`, `tools_registry.rs`, `agents.rs`) exercise private or internal helper functions (`pub(crate)` or private). Making all internal structures `pub` to appease tests violates encapsulation.

### Option B: Sibling test files (e.g. `src/state/state_tests.rs`)
- **Mechanism:** Sibling file for every source file.
- **Pros:** Simple file naming.
- **Cons:** Clutters domain directories with double the file count (e.g. `src/harness/` would grow from 13 to 26 files in the same flat folder).

### Option C: Dedicated `tests/` subfolder per domain with `#[cfg(test)] #[path = "tests/<name>.rs"] mod tests;` (RECOMMENDED)
- **Mechanism:** Inside each domain (e.g. `src/state/`), create `src/state/tests/` containing `state.rs`, `diff.rs`, `ports.rs`, etc. The functional source file declares:
  ```rust
  #[cfg(test)]
  #[path = "tests/state.rs"]
  mod tests;
  ```
- **Pros:**
  - Clean separation: all unit tests for a domain live inside `src/<domain>/tests/`.
  - Scoping is preserved: `mod tests;` remains a child module of the source file, maintaining complete `super::*` access to private and crate-level items.
  - Zero pollution of production root directories.
  - Standard Rust 2021 module path resolution works identically on Linux, macOS, and Windows.
- **Cons:** Requires explicit `#[path = "..."]` attribute on the module declaration, which is standard Rust practice for custom test file layouts.

## 3. PR Chunking & Boundary Forecast

To stay strictly under the 400-changed-line PR size threshold, the refactoring work is partitioned into 6 distinct, sequential PR slices:

1. **PR 1 (OpenSpec & Spec Contract):** Five OpenSpec markdown files.
2. **PR 2 (Domain Slice 1: `state/` & `opencode/`):** 10 files (~380 LOC).
3. **PR 3 (Domain Slice 2: `source/` & `error.rs`):** 6 files (~360 LOC).
4. **PR 4 (Domain Slice 3: `harness/` Part 1 — Adapters):** 6 files (~370 LOC).
5. **PR 5 (Domain Slice 4: `harness/` Part 2 & `tui/`):** 6 files (~390 LOC).
6. **PR 6 (Domain Slice 5: `commands/` Part 1):** 5 files (~380 LOC).
7. **PR 7 (Domain Slice 6: `commands/` Part 2 & DoD / Release):** 5 files (~380 LOC).

Every slice is verifiable with 100% green CI matrix.
