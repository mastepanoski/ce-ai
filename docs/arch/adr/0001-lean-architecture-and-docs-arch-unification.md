# ADR-0001: Lean Living Architecture and docs/arch/ Unification

- **Status**: Accepted
- **Date**: 2026-10-23
- **Deciders**: Compound Engineering Core Team & Architecture Guild
- **Context**: OpenSpec SDD Migration & Multi-Repo Constellation Governance

---

## Context & Problem Statement

Prior versions of `ce-ai` relied on an adapted Spec-Driven Development (SDD) convention centered around a 5-file package (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) stored in `openspec/changes/<feature>/`.

While effective for formal waterfall specifications, this model introduced several friction points in agile development:
1. **Duplicate Progress Tracking**: Both `tasks.md` in OpenSpec and implementation plans in `docs/plans/` tracked checkboxes (`- [x]`), resulting in artificial task desync warnings when agents executed native Compound Engineering workflows.
2. **Cognitive Overhead**: Requiring a 5-file artifact set even for straightforward modular features imposed excessive ceremony and slowed down iterative Scrum cycles.
3. **Multi-Repository Constellations**: Enterprise development across multiple decoupled repositories lacked a unified, local-first declarative manifest to define inter-component dependencies and contract interfaces.
4. **Living Architecture Fragmentation**: Domain knowledge was dispersed across completed changes rather than continuously aggregated in living architecture specifications.

---

## Decision Drivers

- **Zero Semantic Authority**: Compound Engineering must not enforce rigid artifact rituals beyond what is strictly necessary.
- **Single Source of Truth (SSOT)**: Implementation plans in `docs/plans/` must serve as the sole source of truth for task progress and FSM stage deduction.
- **Living Architecture Accretion**: Core architectural domain models, ADRs, contracts, and deltas belong in a unified directory (`docs/arch/`).
- **Constellation Awareness**: Support multi-repo constellations via declarative `docs/arch/system.yaml` manifests with OWASP LLM-02 anti-credential validation.
- **100% Backward Compatibility**: Dual-path resolution must seamlessly support unmigrated repositories using `openspec/` without requiring forced upgrades.

---

## Decision

We have unified the architectural structure and governance model under `docs/arch/`:

1. **Living System Specs (`docs/arch/system/`)**: Canonical, evergreen domain specifications (e.g. `core.md`, `security.md`) that aggregate capabilities across features.
2. **Architecture Decision Records (`docs/arch/adr/`)**: Monotonically numbered ADRs documenting architectural decisions and tradeoffs.
3. **Formal Contracts (`docs/arch/contracts/`)**: Interface schemas and API specifications referenced across constellation dependencies.
4. **Temporary Deltas (`docs/arch/deltas/`)**: Optional delta packages during active architectural feature cycles, automatically promoted to system specs upon archive.
5. **Historical Archive (`docs/arch/archive/`)**: Permanent archive for completed deltas, supporting milestone compaction.
6. **Declarative Constellation Manifest (`docs/arch/system.yaml`)**: Declares system identity, component boundaries, and inter-repo dependencies with local-first and remote-graceful resolution.
7. **SSOT in `docs/plans/`**: Task progress and checkpoint stage inference read directly from `docs/plans/*.md` checkboxes (`- [x]`).
8. **Assisted Migration Command**: `ce-ai init-prj --migrate-arch` migrates legacy `openspec/` directories to `docs/arch/` atomically with dirty-tree safety checks.

---

## Consequences

### Positive
- Eradicated false-positive task desync warnings by establishing `docs/plans/` as the single source of truth.
- Streamlined developer velocity by removing mandatory 5-file SDD ceremony in favor of vertical Scrum slices with TDD.
- Enabled multi-repo fleet filtering (`ce-ai fleet status --system <id>`) with automated credential leak protection.
- Living architecture specs now accrete continuously as features are archived.

### Neutral / Trade-offs
- Repositories migrating from `openspec/` must run `ce-ai init-prj --migrate-arch` or `ce-ai doctor --fix --migrate-arch`.
- Unmigrated repositories continue operating via the dual-path compatibility layer without disruption.
