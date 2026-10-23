# Proposal: CE-AI v2 Architectural Pivot & PRD

## Problem Statement

Following code-level architectural review from the upstream maintainer of Compound Engineering (`EveryInc/compound-engineering-plugin`), `ce-ai` has been identified as duplicating upstream responsibilities, maintaining divergent state models, and building on brittle internal layout assumptions rather than documented integration contracts:

1. **Dual State of Truth & Inappropriate FSM Authority:** `ce-ai` maintains a global stage cursor in `state.json` and attempts complex heuristic reconciliation against the repository. Upstream Compound Engineering explicitly considers repository artifacts (plans, branches, commits, PRs, run receipts) as the sole authoritative state. Storing state separately causes persistent drift and forces a brittle reconciliation layer.
2. **False Linearity Assumption:** Upstream CE is not a rigid 7-stage linear pipeline. Trivial tasks skip planning, `ce-debug` follows a distinct diagnostic loop, and `ce-compound` writes documentation only when novel learnings warrant it.
3. **Cross-Host Sync Antipattern:** Copying raw `skills/` from release tarballs bypasses per-target rewrites performed by native packaging, clashes with native marketplace installations (`external-duplicate`), and creates dual sources of truth.
4. **Documentation Debt Amplification:** Imposing rigid gate checks that mandate formal multi-file OpenSpec documentation before any write increases maintenance overhead, contradicting CE's goal of lightweight, trustworthy knowledge capture.
5. **Concrete Coupling Incompatibilities:** Current code has 7 active coupling errors with upstream CE (solution frontmatter schema, component fields, `docs_root` configuration, brainstorm directory paths, dead-path language scoping, Pi extension collision, and fragile gate exemptions).

## In-Scope

- Formalize the **CE-AI v2 Architecture PRD** (`docs/architecture/ce-ai-v2-architecture-prd.md`) addressing all 5 maintainer critique points.
- Pivot `ce-ai`'s core mission from "authoritative workflow orchestrator" to **"A Cross-Host Operational Companion for Compound Engineering"**.
- Specify the transition of the Workflow FSM from an authoritative gatekeeper to a **read-only advisory observation model**.
- Specify the replacement of file-level skill copying/hashing with **Fleet Version Governance & Native Installer Orchestration**.
- Specify the adoption of upstream doc hygiene tooling (`compound audit`, `schema.yaml` with `schema_version`) and configurable `docs_root`.
- Define the Rust CE Compatibility Layer (`CeRelease`, `CeCapabilities`, `CeSkillContract`, `CeArtifactSchema`, `CeNativeInstaller`, `CeDocsRoot`).
- Accrete `CONCEPTS.md` with the new domain vocabulary while strictly preserving existing terms.

## Out-of-Scope

- Full code implementation and deletion of legacy Rust structs in this PR (this PR defines the architectural specification, PRD, and migration blueprint; code implementation will follow phased OpenSpec increments).
- Breaking existing v1.x CLI commands before the v2.0 deprecation lifecycle is activated.

## Success Criteria

- Complete, rigorous Architectural PRD published in `docs/architecture/ce-ai-v2-architecture-prd.md`.
- Full OpenSpec package (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) complete and verified.
- `CONCEPTS.md` monotonically accreted and passing strict linting (`scripts/validate-concepts.py` / `cargo run -- doc lint --strict`).
- Clear migration roadmap establishing phased transition from v1.x to v2.0.
