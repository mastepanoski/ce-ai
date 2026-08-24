# Exploration: Technical Investigation & Architecture Options

## Investigated Options

### Option A: Pure Prompting / Memory-Only Enforcement (Status Quo)
- **Mechanism**: Include detailed instructions in `AGENTS.md` and rely on LLM context retention.
- **Trade-offs**: Low implementation cost, but high failure rate in long sessions due to context compaction and token dilution. Unacceptable for ISO 27001 / ISO 42001 compliance.

### Option B: Local Git Hooks Only
- **Mechanism**: Place pre-commit / pre-push bash scripts in `.git/hooks/`.
- **Trade-offs**: Works locally, but can be easily bypassed with `git push --no-verify` or skipped on fresh clones if `core.hooksPath` is not set.

### Option C: Deterministic Dual-Layer Enforcement (Selected)
- **Mechanism**:
  1. **Layer 1 (Platform Boundary)**: GitHub API branch protection on `main` enforcing PR requirements and green status checks.
  2. **Layer 2 (Compact Index + Doctor Probes)**: High-density ~25-line Invariant Index at the top of `AGENTS.md` combined with `ce-ai doctor` health probes.
- **Trade-offs**: Requires `gh` CLI for setup and doctor checks, but guarantees zero-bypass protection against accidental main branch corruption.

## Architectural Trade-Off Analysis

| Criteria | Option A (Prompting) | Option B (Local Hooks) | Option C (Dual-Layer) |
| :--- | :---: | :---: | :---: |
| **Resilience to Compaction** | ❌ Poor | 🟢 High | ✅ 100% Deterministic |
| **Bypass Resistance** | ❌ None | ⚠️ Moderate | ✅ Fail-Closed |
| **Multi-OS Support** | ✅ Universal | ⚠️ Requires Bash | ✅ Cross-Platform Rust |
| **ISO 42001 Compliance** | ❌ Non-compliant | ⚠️ Partial | ✅ Fully Compliant |
