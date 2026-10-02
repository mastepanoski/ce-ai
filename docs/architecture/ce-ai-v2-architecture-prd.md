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

The foundational operational rule is:
> **CE-AI must not require artifacts that Compound Engineering itself does not require.**

---

## 2. Foundational Architectural Principles (Inviolable Design Laws)

CE-AI v2 establishes two inviolable architectural principles that govern all design decisions:

### Principle 1: No Semantic Authority
> **CE-AI MUST NOT introduce mandatory workflow stages, artifacts, or transition requirements beyond those defined by the Compound Engineering contracts it integrates with.**
>
> **Optional integrations such as OpenSpec MAY introduce additional artifacts or constraints only when explicitly activated by the user or calling workflow.**
>
> **CE-AI MAY observe, validate, visualize, or coordinate these contracts, but MUST NOT silently redefine Compound Engineering workflow semantics.**

### Principle 2: Repository Reality Over Mirrored State
> **Durable repository artifacts and documented CE contracts are authoritative. CE-AI-derived workflow state is advisory and disposable: it MUST be reconstructable from authoritative sources and MUST NOT become an independent source of workflow truth.**

---

## 3. Core Domain vs. Optional Integrations

In CE-AI v1, OpenSpec was conflated with the core domain model and enforced as a mandatory Stage 2 write gate. CE-AI v2 strictly decouples OpenSpec from the core domain:

```text
                        Compound Engineering
                                  │
                      (owns workflow semantics)
                                  ▼
 ┌─────────────────────────────────────────────────────────────────┐
 │                           CE-AI Core                            │
 │                                                                 │
 │   ┌───────────────────────┐         ┌───────────────────────┐   │
 │   │   CE Compatibility    │         │     Host Adapters     │   │
 │   │ (docs_root, schemas)  │         │ (Claude, Codex, etc.) │   │
 │   └───────────────────────┘         └───────────────────────┘   │
 │                                                                 │
 │   ┌───────────────────────┐         ┌───────────────────────┐   │
 │   │ Workflow Observation  │         │  Fleet Coordination   │   │
 │   │ (advisory capabilities│         │  (version governance) │   │
 │   └───────────────────────┘         └───────────────────────┘   │
 └────────────────────────────────┬────────────────────────────────┘
                                  │
              ┌───────────────────┴───────────────────┐
              ▼                                       ▼
    ┌───────────────────┐                   ┌───────────────────┐
    │     OpenSpec      │                   │   Future Tools    │
    │    Integration    │                   │   & Sidecars      │
    │ (optional/opt-in) │                   │ (optional/opt-in) │
    └───────────────────┘                   └───────────────────┘
```

- **CE-AI Core:** Comprises CE compatibility (`docs_root`, `schema.yaml`, mode tokens), native host adapters, read-only workflow observation, and fleet version governance.
- **OpenSpec Integration:** Exists as an optional, opt-in capability for complex architectural initiatives (`ce-ai spec`). It imposes zero write gates on native Compound Engineering work.

---

## 4. Core Boundary Matrix

| Responsibility Area | Upstream Compound Engineering | Host IDE / Harness (Claude, Codex, etc.) | CE-AI v2 Operational Companion |
| :--- | :--- | :--- | :--- |
| **Workflow State** | **Authoritative.** Defined by git branches, commits, PRs, plan files, and run receipts. | Executes agent turns within session context. | **Advisory Observation.** Reads git artifacts directly to report observable capabilities. Zero authoritative stage cursors in `state.json`. |
| **Workflow Semantics** | **Authoritative.** Decides when planning, debugging, or compounding is needed. | Executes prompt instructions. | **No Semantic Authority.** Never requires artifacts (e.g. OpenSpec) that CE itself does not require. |
| **Installation & Distribution** | Authoritative source of skills, plugins, and release packages. | **Authoritative Execution.** Installs CE plugins via native marketplaces or plugin managers. | **Fleet Governance.** Pin release versions across hosts; drives host CLIs (`claude plugin`, `pi install`, etc.) without duplicating files. |
| **Knowledge Capture** | Authoritative skills (`ce-compound`, `ce-compound-refresh`) and validators (`compound audit`). | Displays documents in context. | **Audit & Tooling.** Discards parallel validators; verifies compliance with upstream `schema.yaml` (`schema_version`) and `docs_root`. |
| **Project Adoption** | Provides skills and instruction prompts. | Reads project configuration (`AGENTS.md`, `CLAUDE.md`). | **Adoption Injection.** Safely injects non-destructive marker-delimited instruction blocks into `AGENTS.md`. |
| **Harness Telemetry** | Unaware of host token spend. | Emits session event logs. | **Telemetry Ingestion.** Ingests usage across harnesses into a unified cost/token report. |

---

## 5. The 5 Strategic Shifts (Addressing Upstream Critique)

### Shift 1: Probabilistic Execution vs. Deterministic State & Observable Capabilities
- **Upstream Finding:** CE draws the determinism line strictly inside individual skills (e.g. `ce-commit-push-pr` git checks, `ce-sweep` state file, `ce-work` `mode:return-to-caller` receipts). The maintainer emphasized: *"The stages also aren't linear in CE. Trivial work skips planning, ce-debug has its own path, and ce-compound only writes a doc when there's something worth capturing."*
- **v2 Architecture:**
  - Deprecate `current_stage` in `state.json`.
  - Shift FSM from an authoritative gating mechanism to a derived, read-only **Observable Capabilities Matrix**.
  - Rather than asking *"What stage are we in?"*, CE-AI asks:
    > *"What do we objectively know about the current state of the workflow?"*
  - Reconstruct workflow state dynamically from repository reality (`docs_root/plans/`, git branch, PR status, run receipts).

### Shift 2: Fleet Version Governance over File-Level Hashing
- **Upstream Finding:** Copying raw `skills/` out of GitHub release tarballs bypasses per-host transforms (e.g. Bun converter, native manifest formatting) and creates dual sources of truth alongside native installs (`external-duplicate`).
- **v2 Architecture:**
  - Retire `src/source/archive.rs` and SHA256 file-level diffing against target directories.
  - Implement **Fleet Version Governance**:
    - Users specify target release versions (e.g. `ce-ai fleet pin v1.32.0`).
    - CE-AI drives each host's native plugin manager to install/update the pinned release.
    - CE-AI audits version parity across hosts (`ce-ai fleet status`).

### Shift 3: Decoupling Semantic Authority & OpenSpec
- **Upstream Finding:** Mandating formal OpenSpec packages (`proposal.md`, `spec.md`, `tasks.md`) for every change before allowing code edits inflates documentation debt: *"Requiring proposal/spec/tasks docs before any write grows the pile that then has to be kept trustworthy."*
- **v2 Architecture:**
  - **Eliminate `OpenSpec Required` as a mandatory invariant.**
  - **Eliminate Stage 2 / OpenSpec as an obligatory workflow stage.**
  - **Eliminate the write gate based on existence of proposal/spec/tasks.**
  - Reposition OpenSpec as an optional, opt-in integration for complex tasks where formal spec-driven development is explicitly desired by the user or agent.
  - Align documentation hygiene with upstream `compound audit` and schema validators instead of maintaining divergent blocking checks.

### Shift 4: Immediate Coupling Bug Remediation (Remediated in v1.75.1)
The 7 specific coupling bugs identified by the maintainer were remediated in v1.75.1:
1. **Frontmatter Schema:** Adopted upstream `schema.yaml` standard (optional `tags` and `title`; `applies_when` knowledge-track only).
2. **Component Fields:** Updated parser to read `component`, `related_components`, and `components`.
3. **Dynamic Docs Root:** Resolved `<docs_root>` dynamically from `.compound-engineering/config.yaml` (default `docs/`).
4. **Brainstorm Directory:** Recognized `docs/plans/*-requirements.md` as modern brainstorms alongside legacy dirs.
5. **Dead-Path Checks:** Expanded verification beyond `.rs` to multi-language source code files.
6. **Pi Extension Collision:** Relocated companion extension to `.pi/extensions/ce-ai-companion.ts`, migrating legacy files.
7. **Gate Exemption Precision:** Endured exact string or structured prefix matching for `ce-debug`.

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

// src/observation/state.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservableWorkflowState {
    pub active_work: bool,
    pub active_branch: Option<String>,
    pub plan: Option<PlanObservation>,
    pub verification: VerificationObservation,
    pub handoff: Option<HandoffObservation>,
    pub knowledge_capture: KnowledgeObservation,
    pub openspec: Option<OpenSpecObservation>, // None if OpenSpec is inactive
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanObservation {
    pub path: std::path::PathBuf,
    pub completed_items: usize,
    pub total_items: usize,
    pub is_requirements_only: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationObservation {
    pub has_evidence: bool,
    pub uncommitted_changes: usize,
    pub review_receipt_stamped: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeObservation {
    pub required: bool,
    pub doc_detected: bool,
    pub doc_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenSpecObservation {
    pub feature: String,
    pub path: std::path::PathBuf,
    pub is_sealed: bool,
}
```

---

## 7. CLI Command Evolution

| v1 Command | v2 Target State | Rationale |
| :--- | :--- | :--- |
| `ce-ai sync` | `ce-ai fleet sync` | Shifts from copying files to driving native host package managers. |
| `ce-ai status` | `ce-ai fleet status` | Focuses on multi-host version alignment and health. |
| `ce-ai workflow status` | `ce-ai workflow observe` (or non-blocking `status`) | Purely advisory visualization of repo capabilities; no stage cursor mutations. |
| `ce-ai gate check` | Purely advisory / opt-in | Eliminates blocking write gates; validates contracts only when requested. |
| `ce-ai install` | Deprecated in favor of `ce-ai init-prj` + `ce-ai fleet pin` | Eliminates raw tarball extraction into host directories. |
| `ce-ai models` | Maintained (`ce-ai models`) | Continues to provide multi-harness model profile management. |
| `ce-ai usage` | Maintained (`ce-ai usage`) | Continues to provide cross-harness token spend reporting. |
| `ce-ai doctor` | Maintained (`ce-ai doctor`) | Shift checks to native plugin presence, version alignment, and upstream `compound audit`. |

---

## 8. Phased Migration Roadmap

```text
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|     Phase 1       |      |     Phase 2       |      |     Phase 3       |      |     Phase 4       |
| Immediate Fixes   | ---> | Compatibility &   | ---> | Advisory FSM &    | ---> | CE-AI v2.0 GA     |
| (v1.75.x)         |      | Fleet Engine      |      | OpenSpec Decouple |      | Full Migration    |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
  • Frontmatter schema       • src/compat/ module       • Deprecate cursor         • Remove file-sync
  • Component fields         • ce-ai fleet commands     • Observable state matrix  • Clean CLI surfaces
  • docs_root discovery      • Native host drivers      • Decouple OpenSpec gate   • Final docs & guides
```

### Phase 1: Immediate Bugfixes & Upstream Alignment (v1.75.x — Completed in v1.75.1)
- Resolved the 7 coupling bugs identified by the maintainer.
- Implemented `.compound-engineering/config.yaml` `docs_root` parser.
- Updated `docs/solutions/` schema validation to match upstream `schema.yaml`.

### Phase 2: Introduction of CE Compatibility Layer & Fleet Subsystem (v1.76.0)
- Add `src/compat/` module in Rust.
- Introduce `ce-ai fleet pin`, `status`, and `sync` commands targeting native host package managers.
- Mark raw tarball copying as deprecated.

### Phase 3: Transition FSM to Read-Only Advisory Observer & Decouple OpenSpec (v1.77.0)
- Deprecate `state.json` `current_stage` cursor.
- Implement `ObservableWorkflowState` matrix reading git branch, commit status, PR status, and `plans/`.
- Remove `OpenSpec Required` write gate, establishing OpenSpec strictly as an optional tool.
- Convert `ce-ai workflow status` into an advisory diagnostic report.

### Phase 4: Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0)
- Remove `src/source/archive.rs` and file diff/restore watcher engine.
- Update documentation and `README.md` to reflect the new "Cross-Host Operational Companion" identity.
