# AGENTS.md — AI Agent Operating Directives

This document defines mandatory instructions, architectural boundaries, and operational constraints for AI agents (including Gemini, Antigravity, OpenCode, Claude, Cursor, Copilot) working within or interacting with the `ce-ai` codebase.

---

## 🛡️ Governance & Compliance Standards

All AI agent operations on this repository MUST strictly follow:
1. **ISO/IEC 27001 & 27002**: Information Security & Cryptographic Integrity Controls.
2. **ISO/IEC 42001**: Artificial Intelligence Management System (AIMS).
3. **NIST AI Risk Management Framework (AI RMF 1.0)**.

---

## 📐 Codebase Architecture & Key Boundaries

`ce-ai` is built in modular Rust (2021 edition) with clean domain separation:

```
src/
├── main.rs            # CLI entry point, Clap subcommand parser, TUI dispatch
├── tui.rs             # Full-screen Ratatui & Crossterm interactive dashboard
├── error.rs           # CeError enum, exit code mapping (Usage=2, State=3, IO=4, Network=5, Verification=6)
├── state/             # State management
│   ├── state.rs       # state.json schema & ModelAssignment structs
│   ├── diff.rs        # Manifest & filesystem drift calculation (Copy, Restore, Remove)
│   ├── profiles.rs    # Model profile snapshots & save/load logic
│   └── mod.rs         # Atomic file writer (write_atomic with tempfile + rename)
├── opencode/          # OpenCode harness integration
│   ├── config.rs      # opencode.json reader/writer & plugin array preservation
│   ├── manifest.rs    # manifest.json SHA256 file index tracking
│   └── plugins.rs     # Managed directory paths (~/.config/opencode/compound-engineering)
├── source/            # Plugin package resolution & retrieval
│   ├── archive.rs     # Shared tar.gz extraction & source root finder
│   └── github.rs      # GitHub release API fetcher & fallback tag resolution
└── commands/          # Subcommand implementations
    ├── mod.rs         # Shared Context struct
    ├── install.rs     # ce-ai install implementation
    ├── sync.rs        # ce-ai sync implementation
    ├── upgrade.rs     # ce-ai upgrade implementation
    ├── models.rs      # ce-ai models set/list/profile implementation
    ├── status.rs      # ce-ai status implementation
    ├── uninstall.rs   # ce-ai uninstall implementation
    └── doctor.rs      # ce-ai doctor health check implementation
```

---

## 🚫 Mandatory Agent Constraints (DO NOT VIOLATE)

1. **No Superficial Symptom Patches**:
   - Never suppress errors with dummy fallbacks, empty try-catches, or commenting out failing tests.
   - Trace root causes to upstream logic.

2. **Atomic Writes Only**:
   - Filesystem mutations targeting `state.json` or `opencode.json` MUST use `crate::state::write_atomic`. Direct unbuffered file overwrites are forbidden.

3. **Preserve User Configurations**:
   - When modifying `opencode.json`, NEVER delete or replace unmanaged user plugins or custom skills. Always parse the JSON structure, update targeted keys, and re-serialize cleanly.

4. **100% Verification Before Hand-off**:
   - Agents MUST verify changes by running:
     ```bash
     cargo fmt --check
     cargo clippy --all-targets --all-features -- -D warnings
     cargo test
     make e2e
     ```

5. **Strict Exit Codes**:
   - Exit codes MUST conform to `CeError` specification:
     - `0`: Success
     - `2`: Usage / CLI argument error
     - `3`: State / manifest error
     - `4`: I/O or filesystem error
     - `5`: Network / HTTP error
     - `6`: Integrity / Verification error

---

## 📜 Verification Checklist for Agents

- [ ] Code compiles without warnings (`-D warnings`).
- [ ] Formatting complies with `cargo fmt`.
- [ ] All 18+ unit and CLI integration tests pass (`cargo test`).
- [ ] Containerized Docker E2E gate passes (`make e2e`).
- [ ] Updated documentation if CLI arguments or schemas modified.
