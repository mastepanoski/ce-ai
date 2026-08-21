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

6. **Mandatory OpenSpec Before Code Changes**:
   - NO code changes or feature implementations shall be made without creating or updating a formal spec under `openspec/changes/<feature_name>/` containing `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.

---

## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement

Regardless of whether Compound Engineering, Spec-Driven Development, or pair programming workflows are used, all AI agents MUST strictly follow the 7-stage development cycle:

```
[Stage 1: Ideation (ce-brainstorm)] ➔ [Stage 2: OpenSpec Definition (MANDATORY)] ➔ [Stage 3: Execution Plan (ce-plan)]
   ➔ [Stage 4: TDD & Implementation (ce-work)] ➔ [Stage 5: Verification (cargo test / make e2e)]
   ➔ [Stage 6: Knowledge Capture (ce-compound)] ➔ [Stage 7: Git Shipping (ce-commit-push-pr)]
```

### Stage 2 OpenSpec Enforcement Requirements
Before creating any PR or writing feature code, agents MUST verify that `openspec/changes/<feature_name>/` contains:
- `proposal.md`: Problem statement, in-scope/out-of-scope boundaries, risk evaluation, and success criteria.
- `exploration.md`: Technical investigation, evaluated options, and architectural tradeoffs.
- `design.md`: Technical design, system architecture, structs, data schemas, and API/CLI contracts.
- `spec.md`: Formal requirements using `WHEN ... THEN ...` format and explicit acceptance criteria.
- `tasks.md`: Atomic, executable task checklist with TDD (Red-Green-Refactor) verification steps.

## ✅ Definition of Done (DoD) for AI Agents

Before declaring any task or issue completed, an AI agent MUST satisfy all criteria of the **Definition of Done**:

### 1. Code Quality & Architectural Integrity
- [ ] Code compiles without warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- [ ] Code formatting adheres strictly to `cargo fmt --check`.
- [ ] All state/config mutations use `crate::state::write_atomic` (zero unbuffered overwrites).
- [ ] Operating system paths use cross-platform `PathBuf::join` methods (no hardcoded `/` or `\` in join calls).
- [ ] User configuration keys in `opencode.json` (or other harness configs) are preserved without clobbering.

### 2. Testing & Empirical Verification
- [ ] All unit and CLI integration tests pass (`cargo test`).
- [ ] Containerized Docker E2E gate passes (`make e2e` or `cargo test --test e2e`).
- [ ] Cross-platform CI pipeline passes 100% green on GitHub Actions across Linux (`ubuntu-latest`), macOS (`macos-latest`), and Windows (`windows-latest`).
- [ ] Pull Requests MUST pass 100% of CI status checks; any failing PR is automatically rejected with `REQUEST_CHANGES` and commented with failure guidance.

### 3. Compliance, Governance & Security
- [ ] Aligns with **ISO/IEC 27001/27002** (SHA256 manifests, atomic writes, `cargo-audit` clean).
- [ ] Aligns with **ISO/IEC 42001** and **NIST AI RMF 1.0** (model role scoping, transparent state).
- [ ] Zero secrets, tokens, credentials, or transient files (`.atl/`, `.pi/`, `.codegraph/`) committed to version control.

### 4. Documentation & Git Delivery
- [ ] Updated user documentation (`README.md`, `SECURITY.md`, `AI_POLICY.md`, or CLI `--help` strings) if flags, subcommands, or schemas were altered.
- [ ] Clear, value-communicating Git commit message added and pushed to remote branch.

---

## 📜 Verification Checklist for Agents

- [ ] Code compiles without warnings (`-D warnings`).
- [ ] Formatting complies with `cargo fmt`.
- [ ] All unit and CLI integration tests pass (`cargo test`).
- [ ] Containerized Docker E2E gate passes (`make e2e`).
- [ ] All GitHub Actions CI jobs pass green across Linux, macOS, and Windows.
- [ ] Definition of Done (DoD) criteria fully satisfied.
