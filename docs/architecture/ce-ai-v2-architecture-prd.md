# Product Requirements Document & Technical Architecture: CE-AI v2

**Document Version:** 2.0.0-draft  
**Date:** 2026-10-01  
**Status:** Approved for Implementation Planning  
**Target Milestone:** CE-AI v2.0.0  

---

## 1. Executive Summary & Context

`ce-ai` was originally conceived as a CLI harness manager and workflow governance layer around [Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin). Over successive releases, `ce-ai` expanded into state tracking, file synchronization, manifest hashing, documentation debt analysis, and workflow enforcement.

Following a detailed architectural code-review by the upstream maintainer of Compound Engineering, several structural boundary issues were surfaced:
1. **Divergent State Models:** `ce-ai` maintains a global stage cursor in `state.json` and attempts complex heuristic reconciliation against the repository. Upstream CE considers repository artifacts (plans, branches, commits, PRs, run receipts) as the sole authoritative state. Storing state separately causes persistent drift and requires brittle reconciliation logic.
2. **False Linearity Assumption:** CE workflows are non-linear (trivial work skips planning, `ce-debug` has its own loop, `ce-compound` writes documentation only when justified). CE-AI's rigid 7-stage FSM fails to reflect this reality.
3. **Cross-Host Sync Antipattern:** Scraping release tarballs and copying raw `skills/` directly onto disk bypasses host-specific packaging rewrites, causes conflicts with native marketplace installations (`external-duplicate`), and creates dual sources of truth.
4. **Documentation Debt Amplification:** Enforcing rigid write gates that mandate formal multi-file OpenSpec documentation before any write increases maintenance overhead, contradicting CE's goal of lightweight, trustworthy knowledge capture.
5. **Coupling to Volatile Internals:** `ce-ai` had coupled to internal layout details, prose, and file hashes rather than stable integration contracts (skill names, mode tokens like `mode:return-to-caller`, versioned schemas, and native host installers).

### The Strategic Pivot: CE-AI v2
CE-AI v2 reframes the product's identity:
> **CE-AI is not an authoritative workflow orchestrator or package duplicator.**  
> **CE-AI is a Cross-Host Operational Companion for Compound Engineering.**

- **Compound Engineering owns engineering semantics and workflow artifacts.**
- **AI Hosts (Claude Code, OpenCode, Codex, Pi, Cursor, Copilot) own native execution and packaging.**
- **CE-AI coordinates the environment around them:** fleet version governance, native installation orchestration, environment readiness, multi-harness auditing, and advisory workflow observation.

---

## 2. Core Boundary Matrix

| Responsibility Area | Upstream Compound Engineering | Host IDE / Harness (Claude, Codex, etc.) | CE-AI v2 Operational Companion |
| :--- | :--- | :--- | :--- |
| **Workflow State** | **Authoritative.** Defined by git branches, commits, PRs, plan files, and run receipts. | Executes agent turns within session context. | **Advisory Observation.** Reads git artifacts directly to provide read-only diagnostic summaries. No authoritative stage cursor in `state.json`. |
| **Installation & Distribution** | Authoritative source of skills, plugins, and release packages. | **Authoritative Execution.** Installs CE plugins via native marketplaces or plugin managers. | **Fleet Governance.** Pin release versions across hosts; drives host CLIs (`claude plugin`, `pi install`, etc.) without duplicating files. |
| **Knowledge Capture** | Authoritative skills (`ce-compound`, `ce-compound-refresh`) and validators (`compound audit`). | Displays documents in context. | **Audit & Tooling.** Discards parallel validators; verifies compliance with upstream `schema.yaml` (`schema_version`) and `docs_root`. |
| **Project Adoption** | Provides skills and instruction prompts. | Reads project configuration (`AGENTS.md`, `CLAUDE.md`). | **Adoption Injection.** Safely injects non-destructive marker-delimited instruction blocks into `AGENTS.md`. |
| **Harness Telemetry** | Unaware of host token spend. | Emits session event logs. | **Telemetry Ingestion.** Ingests usage across harnesses into a unified cost/token report. |

---

## 3. The 5 Strategic Shifts (Addressing Upstream Critique)

### Shift 1: Probabilistic Execution vs. Deterministic State
- **Upstream Finding:** CE draws the determinism line strictly inside individual skills (e.g. `ce-commit-push-pr` git checks, `ce-sweep` state file, `ce-work` `mode:return-to-caller` receipts). CE deliberately avoids a global stage cursor because the repo artifacts *are* the state.
- **v2 Architecture:**
  - Deprecate `current_stage` in `state.json`.
  - Shift FSM from an authoritative gating mechanism to a derived, read-only advisory observation model (`Workflow Observation / Advisory FSM`).
  - Model workflow as a directed graph rather than a rigid 7-stage linear pipeline:
    - *Planning Loop:* Requirements -> Plan -> Work -> Review -> Ship
    - *Tactical/Bugfix Loop:* Problem -> Debug/Work -> Verify -> Ship
    - *Hygiene Loop:* Refresh / Compound -> PR
  - State recovery (`ce-ai workflow resume`) directly inspects repo artifacts (`docs_root/plans/`, git branch, PR status, run receipts), matching `ce-handoff` and `ce-work` conventions.

### Shift 2: Fleet Version Governance over File-Level Hashing
- **Upstream Finding:** Copying raw `skills/` out of GitHub release tarballs bypasses per-host transforms (e.g. Bun converter, native manifest formatting) and creates dual sources of truth alongside native installs (`external-duplicate`).
- **v2 Architecture:**
  - Retire `src/source/archive.rs` and SHA256 file-level diffing against target directories.
  - Implement **Fleet Version Governance**:
    - Users specify target release versions (e.g. `ce-ai fleet pin v1.32.0`).
    - CE-AI drives each host's native plugin manager to install/update the pinned release.
    - CE-AI audits version parity across hosts:
      ```text
      $ ce-ai fleet status
      Target Release: v1.32.0
      ├── Claude Code:  v1.32.0 (✓ Aligned via marketplace)
      ├── OpenCode:     v1.32.0 (✓ Aligned via extension)
      ├── Codex:        v1.31.0 (△ Outdated — run 'ce-ai fleet sync')
      └── Pi:           v1.32.0 (✓ Aligned via npm)
      ```

### Shift 3: Integrated Documentation Hygiene over Intrusive Write Gates
- **Upstream Finding:** Mandating formal OpenSpec packages (`proposal.md`, `spec.md`, `tasks.md`) for every change before allowing code edits inflates documentation debt. CE addresses documentation quality natively via `ce-compound`, `ce-compound-refresh`, and `compound audit`.
- **v2 Architecture:**
  - Remove intrusive write-blocking gates for lightweight or bug-focused workflows.
  - Align with upstream `compound audit` by embedding or invoking upstream validation scripts rather than maintaining divergent rust-based frontmatter checks.
  - Treat documentation checks as advisory diagnostics in `ce-ai doctor` rather than blocking gates.

### Shift 4: Immediate Coupling Bug Remediation
The review noted 7 specific coupling bugs in `ce-ai` that must be resolved:
1. **Frontmatter Schema:** Adopt upstream `schema.yaml` standard (`module`, `date`, `problem_type`, `component`, `severity`; optional `schema_version`, `related_components`, `tags`, and knowledge-track-only `applies_when`).
2. **Component Fields:** Update parser to read `component` and `related_components` (deprecate `components`).
3. **Dynamic Docs Root:** Resolve `<docs_root>` dynamically from `.compound-engineering/config.yaml` (default `docs/`).
4. **Brainstorm Directory:** Recognize `docs/plans/` requirements-only plans as modern brainstorms (`docs/brainstorms/` is legacy).
5. **Dead-Path Checks:** Expand dead-path verification beyond `.rs` to all repository source code files (`.ts`, `.js`, `.py`, `.go`, `.rb`, etc.).
6. **Pi Extension Collision:** Ensure CE-AI companion files never write to `.pi/extensions/compound-engineering.ts` (reserved for upstream CE).
7. **Gate Exemption Precision:** Replace substring matching (`"ce-debug"`) with structured command/task token parsing.

### Shift 5: Stable Integration Contracts
Build exclusively on documented upstream surfaces:
- Skill names and documented mode tokens (`mode:return-to-caller`, etc.).
- The `docs_root` configuration rule (`plans/`, `solutions/`).
- The learning schema in `schema.yaml` with `schema_version`.
- Native host installer commands, audited by released plugin version.
- **Never build on:** internal `skills/` file layouts, markdown prose, bundled scripts, or file hashes.

---

## 4. CE Compatibility Layer Specification (Rust Architecture)

CE-AI v2 introduces a dedicated, strongly-typed domain module `src/compat/`:

```rust
// src/compat/release.rs
use semver::Version;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CeRelease {
    pub version: Version,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub capabilities: CeCapabilities,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CeCapabilities {
    pub supports_return_to_caller: bool,
    pub supports_schema_version: bool,
    pub supports_docs_root: bool,
}

// src/compat/docs.rs
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CeDocsConfig {
    pub docs_root: PathBuf,
}

impl CeDocsConfig {
    pub fn discover(repo_root: &Path) -> Self {
        let config_file = repo_root.join(".compound-engineering/config.yaml");
        let docs_root = if config_file.exists() {
            Self::parse_docs_root(&config_file).unwrap_or_else(|| PathBuf::from("docs"))
        } else {
            PathBuf::from("docs")
        };
        Self { docs_root }
    }

    pub fn plans_dir(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.docs_root).join("plans")
    }

    pub fn solutions_dir(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.docs_root).join("solutions")
    }
}

// src/compat/schema.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CeSolutionFrontmatter {
    pub schema_version: Option<String>,
    pub module: String,
    pub date: chrono::NaiveDate,
    pub problem_type: String, // bugfix, architecture, pattern, discovery, config, preference
    pub component: String,
    #[serde(default)]
    pub related_components: Vec<String>,
    pub severity: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub applies_when: Option<String>,
}
```

---

## 5. CLI Command Evolution

| v1 Command | v2 Target State | Rationale |
| :--- | :--- | :--- |
| `ce-ai sync` | `ce-ai fleet sync` | Shifts from copying files to driving native host package managers. |
| `ce-ai status` | `ce-ai fleet status` | Focuses on multi-host version alignment and health. |
| `ce-ai workflow status` | `ce-ai workflow observe` (or non-blocking `status`) | Purely advisory visualization of repo artifacts; no stage cursor mutations. |
| `ce-ai install` | Deprecated in favor of `ce-ai init-prj` + `ce-ai fleet pin` | Eliminates raw tarball extraction into host directories. |
| `ce-ai models` | Maintained (`ce-ai models`) | Continues to provide multi-harness model profile management. |
| `ce-ai usage` | Maintained (`ce-ai usage`) | Continues to provide cross-harness token spend reporting. |
| `ce-ai doctor` | Maintained (`ce-ai doctor`) | Shift checks to native plugin presence, version alignment, and upstream `compound audit`. |

---

## 6. Phased Migration Roadmap

```text
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|     Phase 1       |      |     Phase 2       |      |     Phase 3       |      |     Phase 4       |
| Immediate Fixes   | ---> | Compatibility &   | ---> | Advisory FSM &    | ---> | CE-AI v2.0 GA     |
| (v1.75.x)         |      | Fleet Engine      |      | Read-Only State   |      | Full Migration    |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
  • Frontmatter schema       • src/compat/ module       • Deprecate cursor         • Remove file-sync
  • Component fields         • ce-ai fleet commands     • Artifact-only resume     • Clean CLI surfaces
  • docs_root discovery      • Native host drivers      • Non-blocking advice      • Final docs & guides
```

### Phase 1: Immediate Bugfixes & Upstream Alignment (v1.75.x)
- Resolve the 7 coupling bugs identified by the maintainer.
- Implement `.compound-engineering/config.yaml` `docs_root` parser.
- Update `docs/solutions/` schema validation to match upstream `schema.yaml`.

### Phase 2: Introduction of CE Compatibility Layer & Fleet Subsystem (v1.76.0)
- Add `src/compat/` module in Rust.
- Introduce `ce-ai fleet pin`, `status`, and `sync` commands targeting native host package managers.
- Mark raw tarball copying as deprecated.

### Phase 3: Transition FSM to Read-Only Advisory Observer (v1.77.0)
- Deprecate `state.json` `current_stage` cursor.
- Implement `WorkflowObserver` reading git branch, commit status, PR status, and `plans/`.
- Convert `ce-ai workflow status` into an advisory diagnostic report.

### Phase 4: Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0)
- Remove `src/source/archive.rs` and file diff/restore watcher engine.
- Update documentation and `README.md` to reflect the new "Cross-Host Operational Companion" identity.
