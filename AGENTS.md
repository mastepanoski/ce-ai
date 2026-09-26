# AGENTS.md — AI Agent Operating Directives

This document defines mandatory instructions, architectural boundaries, and operational constraints for AI agents (including Gemini, Antigravity, OpenCode, Claude, Cursor, Copilot) working within or interacting with the `ce-ai` codebase.

---

## ⚡ Hard-Gate Invariant Index (Non-Negotiable Delivery Rules)

All AI agents MUST enforce these hard invariants deterministically at every session start:
1. **Never Direct Commit to `main`**: All changes MUST be committed on feature branches (`feat/*` or `fix/*`) and opened as a PR (`gh pr create`).
2. **100% Green CI Matrix Gate**: NEVER merge a PR until ALL GitHub Actions CI matrix jobs pass (`gh pr checks --watch`).
3. **Atomic File Writes**: Mutations to `state.json` or `opencode.json` MUST use `crate::state::write_atomic`.
4. **Preserve User Configs**: NEVER overwrite unmanaged custom plugins or custom skills in `opencode.json`.
5. **No Dummy Fallbacks**: NEVER comment out failing assertions, mask errors with empty catches, or ignore CLI errors.
6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.
7. **Strict Exit Codes**: Map all errors to `CeError` enum exit codes (`0` Success, `1` Runtime, `2` Usage, `3` State, `4` IO, `5` Network, `6` Verification).
8. **Preserve Active Worktrees**: NEVER run `git worktree remove` or delete sibling worktrees in `<repo>-worktrees/` without explicit USER permission or verifying creation within the current turn.
9. **Mandatory Versioning & CHANGELOG**: Every merged feature/fix MUST bump SemVer in `Cargo.toml`, update `CHANGELOG.md`, tag release (`vX.Y.Z`), and create a GitHub Release. Homebrew distribution is owned exclusively by the `mastepanoski/homebrew-ce-ai` tap (self-updating); no formula is maintained in this repository.
10. **Post-Merge Lifecycle & Clean State**: Immediately after merging a PR, switch to `main`, run `git pull`, delete merged local branches (`git branch -d`), prune remotes (`git fetch --prune`), remove turn-created temporary worktrees, and run `ce-ai workflow status`. If any completed OpenSpec changes exist, the AI agent MUST run `ce-ai archive <feature>` (and submit the corresponding archive PR) so the repository FSM is left at `✓ Ready (100%)` with zero unarchived change warnings before concluding.
11. **Zero AI Attribution & No Co-Author Trailers**: AI agents MUST NEVER add "Co-Authored-By", AI attribution trailers, or "Generated with [Agent]" footers/badges to git commit messages or PR descriptions. All commits MUST use clean Conventional Commits only.
12. **Monotonic Concept Accretion**: `CONCEPTS.md` MUST NEVER suffer destructive deletion or blind overwrite. Agents MUST use surgical edits (`Edit`/append), execute a mandatory pre-read, and verify changes with `ce-ai doc lint --strict`.

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
    ├── init_prj.rs    # ce-ai init-prj project adoption implementation
    ├── deinit_prj.rs  # ce-ai deinit-prj project de-adoption implementation
    ├── sync.rs        # ce-ai sync implementation
    ├── upgrade.rs     # ce-ai upgrade implementation
    ├── models.rs      # ce-ai models set/list/profile implementation
    ├── status.rs      # ce-ai status implementation
    ├── uninstall.rs    # ce-ai uninstall implementation
    └── doctor.rs      # ce-ai doctor health check implementation
```

`docs/solutions/` documents solutions to past problems (bugs, best practices, workflow patterns), organized by category with YAML frontmatter (`module`, `tags`, `problem_type`). Relevant when implementing or debugging in documented areas.

`CONCEPTS.md` defines shared domain vocabulary (entities, named processes, status concepts) for this project. Relevant when orienting to the codebase or discussing domain concepts.

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

5. **Mandatory OpenSpec Before Code Changes**:
   - NO code changes or feature implementations shall be made without creating or updating a formal spec under `openspec/changes/<feature_name>/` containing `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.

6. **Standardized Exit Code Compliance**:
   - Errors MUST map to standard `CeError` exit codes: `0` (Success), `1` (Runtime), `2` (Usage), `3` (State), `4` (IO), `5` (Network), `6` (Verification).

7. **Mandatory Pull Request Workflow (No Direct Push to `main`)**:
   - Direct commits and pushes to `main` are strictly forbidden for feature development, refactoring, and bug fixes.
   - Agents MUST create feature branches (`feature/<name>` or `fix/<name>`), push to origin, and open a Pull Request (`gh pr create`).
   - PRs must wait for 100% green GitHub Actions CI status checks before merging.
   - Every PR description MUST be self-explaining: explicitly declare what was ruled out, which rule decided it, attach empirical verification evidence in collapsible blocks upfront, and verify all automated checks pass before requesting review.

8. **Documentation Style Compliance (Diátaxis + Cognitive Load)**:
   - ALL documentation changes (`README.md`, `docs/`, guides) MUST follow the project style guide at [`docs/references/docs-styling.md`](./docs/references/docs-styling.md).
   - `README.md` MUST stay ≤ 100 lines: title + what/why, Quick Path (install → first command → verification), then an audience-labeled documentation map. Deep internals belong in `docs/`, never inline.
   - Every document MUST have exactly one Diátaxis intent (tutorial / how-to / reference / explanation). Do not blend quadrants within a section.
   - Documentation must self-route two audiences: newbies via the Quick Start tutorial path, seniors directly to reference/architecture.

9. **Zero AI Attribution & No Co-Author Trailers**:
   - AI agents MUST NEVER add "Co-Authored-By", agent attribution trailers, or "Generated with [Agent]" footers/badges to git commit messages or PR descriptions.
   - All commits MUST use clean Conventional Commits only.

10. **Monotonic Concept Accretion & Anti-Clobber Protection (`CONCEPTS.md`)**:
   - During Stage 6 (`ce-compound` / Vocabulary Capture), agents MUST read `CONCEPTS.md` immediately before editing.
   - Agents MUST use surgical in-place editing (`Edit` / `replace_file_content` / append) and NEVER perform full-file `Write` overwrites that truncate existing terms.
   - All modifications must be verified via `scripts/validate-concepts.py` or `ce-ai doc lint --strict`, ensuring existing entries grow monotonically.
   - Reports must state empirical before/after entry counts (e.g. `N entries (was M, +X added, 0 removed)`).

---

## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement

Regardless of whether Compound Engineering, Spec-Driven Development, or pair programming workflows are used, all AI agents MUST strictly follow the 7-stage development cycle:

```
[Stage 1: Ideation (ce-brainstorm)] ➔ [Stage 2: OpenSpec Definition (MANDATORY)] ➔ [Stage 3: Execution Plan (ce-plan)]
   ➔ [Stage 4: TDD & Implementation (ce-work)] ➔ [Stage 5: Verification (project quality gates)]
   ➔ [Stage 6: Knowledge Capture (ce-compound)] ➔ [Stage 7: Git Shipping (ce-commit-push-pr)]
```

### Stage 2 OpenSpec Enforcement Requirements
OpenSpec is authored **progressively**: Stage 2 writes `proposal.md`, `exploration.md`, `design.md`, and `spec.md`; Stage 3 (`/ce-plan`) then generates the executable checklist `tasks.md` from the frozen contract. Before creating any PR or writing feature code, agents MUST verify that `openspec/changes/<feature_name>/` contains all five files:
- `proposal.md`: Problem statement, in-scope/out-of-scope boundaries, risk evaluation, and success criteria.
- `exploration.md`: Technical investigation, evaluated options, and architectural tradeoffs.
- `design.md`: Technical design, system architecture, structs, data schemas, and API/CLI contracts.
- `spec.md`: Formal requirements using `WHEN ... THEN ...` format and explicit acceptance criteria.
- `tasks.md`: Atomic, executable task checklist with TDD verification steps (generated by `/ce-plan`).
- When authoring `tasks.md`, work units MUST carry per-work-unit changed-line estimates (~200 LOC target; rescopes may only narrow them) so the PR-level forecast is derivable — see [CONTRIBUTING.md](CONTRIBUTING.md) §4. (Adopted policy, v1.22.1.)

### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into the OpenSpec files above (`proposal.md`, `exploration.md`) and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with OpenSpec — that duplicates work and burns tokens. Ideation artifacts are retained by default as the permanent raw-history record that OpenSpec deliberately does not duplicate — "disposable" never means deleting them; removal is an ordinary reversible git decision, never a workflow step. Skip ideation skills entirely when requirements and approach are already clear.

### Stage 7 PR & Review Readiness Directives ("A Change That Explains Itself")
When creating Pull Requests (`ce-commit-push-pr`), agents MUST ensure the change explains itself:
- **What It Ruled Out**: State rejected alternatives and discarded trade-offs (distilled from OpenSpec `exploration.md`).
- **Which Rule Decided It**: Cite the specific invariant, rule, architectural boundary, or compliance constraint (from `AGENTS.md` or `design.md`) that determined the solution.
- **Upfront Evidence**: Attach empirical validation proof (test run summaries, CLI logs, reproduction traces, or browser check results) directly to the PR body using collapsible `<details><summary>` blocks (`not asked for later`).
- **Pre-Review Automated Checks**: Run and verify 100% green status across all local linters, unit tests, and browser/E2E checks BEFORE requesting human review or marking the PR ready.
- **Reviewer Routing**: Identify domain owners or `CODEOWNERS` and assign appropriate reviewers.

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
- [ ] **Windows PowerShell Installer Gate (`install.ps1`)**:
  - *Justification*: Automatically verifies `scripts/install.ps1` downloads, extracts, and validates `ce-ai.exe` execution on native `windows-latest` GitHub Actions runners.
- [ ] **Automated PR Rejection on CI Failure**:
  - *Justification*: Enforces zero-tolerance for broken code on `main`. Automatically blocks PR merges and requests changes when any CI or security check fails.

### 3. Compliance, Governance & Security
- [ ] **ISO/IEC 27001/27002 Compliance (SHA256 Manifests & `cargo-audit`)**:
  - *Justification*: Cryptographic SHA256 indexing detects asset tampering or file drift, while `cargo-audit` guarantees zero known CVE supply-chain vulnerabilities in third-party crates.
- [ ] **ISO/IEC 42001 & NIST AI RMF 1.0 Compliance**:
  - *Justification*: Scoping model assignments per agent role (`ce-brainstorm`, `ce-plan`, `ce-work`) ensures capability/cost matching while maintaining transparent state logging.
- [ ] **Local Security Pre-Commit Hook (`make hooks` / `.githooks/pre-commit`)**:
  - *Justification*: Scans staged files for secret key leaks, blocks transient metadata (`.atl/`, `.pi/`), and runs formatting/linter/tests before git commit.
- [ ] **Zero Secrets, Tokens, or Transient File Exposure**:
  - *Justification*: Prevents catastrophic credential leaks (API keys, SSH keys, bearer tokens) and prevents committing local transient metadata (`.atl/`, `.pi/`, `.codegraph/`).

### 4. Documentation & Git Delivery
- [ ] **SemVer Versioning & `CHANGELOG.md` Maintenance**:
  - *Justification*: Follows Semantic Versioning (`MAJOR.MINOR.PATCH` in `Cargo.toml`). Any breaking API/CLI change bump `MAJOR`, new feature bump `MINOR`, bug fix bump `PATCH`. All notable changes MUST be documented in `CHANGELOG.md` following the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) standard.
- [ ] **Updated User Documentation**:
  - *Justification*: Outdated docs create user confusion, improper CLI usage, and support overhead. Any schema, command, or flag change requires updating `README.md`, `SECURITY.md`, `CHANGELOG.md`, or CLI `--help` strings.
- [ ] **Conventional Commits & Clean Git History**:
  - *Justification*: Clear commit messages provide auditability, enable automated changelog generation, and allow easy regression bisecting.
- [ ] **Self-Explaining Pull Request & Upfront Evidence**:
  - *Justification*: Human review is the primary cognitive bottleneck. Stating what was ruled out, which rule decided it, and attaching empirical evidence upfront in collapsible blocks minimizes back-and-forth and prevents unverified claims.

---

## 📜 Verification Checklist for Agents

- [ ] Code compiles without warnings (`-D warnings`).
- [ ] Formatting complies with `cargo fmt`.
- [ ] All unit and CLI integration tests pass (`cargo test`).
- [ ] Containerized Docker E2E gate passes (`make e2e`).
- [ ] All GitHub Actions CI jobs pass green across Linux, macOS, and Windows.
- [ ] Definition of Done (DoD) criteria fully satisfied.

<!-- ce-ai:block begin v=6 tier=minimal sha256=0b96d6153224fe2d8db634928e94010079841528114282b9273966896e9041f2 -->
## 🔄 Compound Engineering Workflow Guidelines

AI agents operating on this codebase should follow structured planning and verification:
- Validate scope boundaries before making changes.
- Ensure all unit, integration, and linter tests pass before committing.
- Ensure PRs explain what was ruled out, which rule decided it, and attach verification evidence upfront in collapsible blocks.
- Document key technical learnings and post-mortem fixes.
<!-- ce-ai:block end -->
