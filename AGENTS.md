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

## ✅ Definition of Done (DoD) & Technical Justifications

Before declaring any task or issue completed, an AI agent MUST satisfy all criteria of the **Definition of Done**:

### 1. Code Quality & Architectural Integrity
- [ ] **Zero Clippy Warnings (`cargo clippy --all-targets --all-features -- -D warnings`)**:
  - *Justification*: Prevents unhandled error cases, memory leaks, unsound unsafe blocks, and performance anti-patterns before runtime compilation.
- [ ] **Strict Formatting (`cargo fmt --check`)**:
  - *Justification*: Ensures consistent codebase style, eliminates whitespace noise in Git diffs, and prevents merge conflicts across different operating systems.
- [ ] **Atomic Writes (`crate::state::write_atomic`)**:
  - *Justification*: Guarantees configuration files (`state.json`, `opencode.json`) are never left corrupted by unexpected process crashes or power loss (NIST SP 800-53 CP-9/CP-10 & ISO 27002 Control 8.9 compliance).
- [ ] **Cross-Platform Path Joining (`PathBuf::join`)**:
  - *Justification*: Operating systems use different path separators (Windows `\` vs Unix `/`). Hardcoded slashes cause test failures, path comparison bugs, and file lookup panics on Windows runners.
- [ ] **Preservation of Unmanaged User Configurations**:
  - *Justification*: Users rely on custom plugins and custom skills in `opencode.json`. Clobbering user keys destroys user configuration; targeted JSON merging protects user customization.

### 2. Testing & Empirical Verification
- [ ] **100% Passing Unit & Integration Tests (`cargo test`)**:
  - *Justification*: Prevents functional regressions in archive extraction, manifest indexing, state diff calculation, and exit code mappings.
- [ ] **Containerized Docker E2E Gate (`make e2e`)**:
  - *Justification*: Validates real-world installation, sync, model setting, and uninstallation in a clean, isolated Linux container environment independent of host machine state.
- [ ] **100% Green Cross-Platform CI Matrix**:
  - *Justification*: Native binaries behave differently across operating systems. Verifying Linux (`ubuntu-latest`), macOS (`macos-latest`), and Windows (`windows-latest`) guarantees multi-platform reliability.
- [ ] **Automated PR Rejection on CI Failure**:
  - *Justification*: Enforces zero-tolerance for broken code on `main`. Automatically blocks PR merges and requests changes when any CI or security check fails.

### 3. Compliance, Governance & Security
- [ ] **ISO/IEC 27001/27002 Compliance (SHA256 Manifests & `cargo-audit`)**:
  - *Justification*: Cryptographic SHA256 indexing detects asset tampering or file drift, while `cargo-audit` guarantees zero known CVE supply-chain vulnerabilities in third-party crates.
- [ ] **ISO/IEC 42001 & NIST AI RMF 1.0 Compliance**:
  - *Justification*: Scoping model assignments per agent role (`ce-brainstorm`, `ce-plan`, `ce-work`) ensures capability/cost matching while maintaining transparent state logging.
- [ ] **Zero Secrets, Tokens, or Transient File Exposure**:
  - *Justification*: Prevents catastrophic credential leaks (API keys, SSH keys, bearer tokens) and prevents committing local transient metadata (`.atl/`, `.pi/`, `.codegraph/`).

### 4. Documentation & Git Delivery
- [ ] **Updated User Documentation**:
  - *Justification*: Outdated docs create user confusion, improper CLI usage, and support overhead. Any schema, command, or flag change requires updating `README.md`, `SECURITY.md`, or CLI `--help` strings.
- [ ] **Conventional Commits & Clean Git History**:
  - *Justification*: Clear commit messages provide auditability, enable automated changelog generation, and allow easy regression bisecting.

---

## 📜 Verification Checklist for Agents

- [ ] Code compiles without warnings (`-D warnings`).
- [ ] Formatting complies with `cargo fmt`.
- [ ] All unit and CLI integration tests pass (`cargo test`).
- [ ] Containerized Docker E2E gate passes (`make e2e`).
- [ ] All GitHub Actions CI jobs pass green across Linux, macOS, and Windows.
- [ ] Definition of Done (DoD) criteria fully satisfied.
