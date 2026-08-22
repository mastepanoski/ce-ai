# Implementation Plan: Project Adoption Engine (`ce-ai init-prj` / `ce-ai deinit-prj`)

## Overview
Implement the **Project Adoption Engine** allowing developers and AI agents to adopt project repositories non-destructively by injecting and managing marker-delimited Compound Engineering workflow blocks in `AGENTS.md` and derived harness stubs (`CLAUDE.md`).

---

## 🏗️ Architecture & Component Boundaries

```
src/
├── main.rs                 # CLI Subcommand Parser (InitPrj, DeinitPrj)
├── harness/
│   ├── mod.rs              # HarnessAdapter trait extension (instruction_file, derived_stubs)
│   └── *.rs                # Harness implementations (OpenCode, Claude, Cursor, Codex, etc.)
├── state/
│   └── state.rs            # ProjectAdoptionEntry & AdoptionTier schemas in state.json
├── commands/
│   ├── mod.rs              # Shared Context struct
│   ├── init_prj.rs         # ce-ai init-prj implementation & template rendering
│   ├── deinit_prj.rs       # ce-ai deinit-prj implementation & clean restoration
│   ├── status.rs           # Project adoption status reporting & drift detection
│   └── doctor.rs           # Project adoption health probes & diagnostic warnings
└── tui.rs                  # TUI Action Shortcuts ([I] Init Prj, [D] Deinit Prj)
```

---

## 🧪 Implementation Stages & Task Checklist

### Stage 1: Core Schemas & Harness Trait Extensions
- [ ] Add `AdoptionTier` enum and `ProjectAdoptionEntry` struct to `src/state/state.rs`.
- [ ] Add `projects: Vec<ProjectAdoptionEntry>` array to `State` struct in `src/state/state.rs`.
- [ ] Extend `HarnessAdapter` trait in `src/harness/mod.rs` with `canonical_instruction_file()` and `derived_stub_files()`.

### Stage 2: Subcommands `ce-ai init-prj` & `ce-ai deinit-prj`
- [ ] Implement CLI flag parsing for `InitPrj` (`--tier`, `--force`) and `DeinitPrj` in `src/main.rs`.
- [ ] Implement `src/commands/init_prj.rs`:
  - Resolve project root via `git rev-parse`.
  - Enclose template block in `<!-- ce-ai:block begin v=1 tier=... -->` and `<!-- ce-ai:block end -->`.
  - Write files and update `state.json` via `write_atomic`.
- [ ] Implement `src/commands/deinit_prj.rs`:
  - Extract managed block.
  - Delete `AGENTS.md` if `created_file: true` and empty; otherwise restore original pre-init bytes.
  - Remove registry entry from `state.json`.

### Stage 3: Diagnostics & Observability (`status` & `doctor`)
- [ ] Extend `ce-ai status` to display adopted projects and SHA integrity status.
- [ ] Extend `ce-ai doctor` to probe for missing instruction files and manual marker drift.

### Stage 4: TUI & Skill Integration
- [ ] Add `[I] Init Prj` and `[D] Deinit Prj` shortcuts to TUI in `src/tui.rs`.
- [ ] Package slash command `/ce-ai-init-prj` and skill `ce-ai-init-prj` for OpenCode harness.

### Stage 5: Verification & Governance
- [ ] Add CLI integration tests in `tests/cli.rs` verifying byte-for-byte roundtrip `init-prj` ➔ `deinit-prj`.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e`.
- [ ] Update `README.md`, `ROADMAP.md`, `CONCEPTS.md`, and `CHANGELOG.md`.
