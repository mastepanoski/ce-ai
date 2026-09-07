# Spec: Engram Context Echo Behavioral Risk Documentation

## Requirements & Acceptance Criteria

### R1: Architectural Solution Document Format & Accuracy
- **WHEN** `docs/solutions/architecture/engram-context-echo-behavioral-risk.md` is authored,
- **THEN** it MUST contain complete YAML front-matter with `category: architecture`, `tags`, `components`, and `applies_when`.
- **AND** it MUST cite `src/commands/workflow.rs` (lines 48, 235, 725, 862) demonstrating zero dependency on external memory.
- **AND** it MUST cite Engram's `mcp.go` (lines 1564, 1613, 1934) and `store.go` (lines 3298, 3341-3343) documenting `mem_context` invocation, 10-prompt limit, and verbatim formatting.
- **AND** it MUST characterize the feedback loop as an LLM behavioral/attention artifact, not a software bug.
- **AND** it MUST explicitly state that upstream recommendations are advisory for a third-party project.

### R2: Vocabulary Entry in `CONCEPTS.md`
- **WHEN** `CONCEPTS.md` is updated,
- **THEN** it MUST include an entry for `Engram Context Echo (Behavioral Loop)` summarizing the phenomenon and linking to `docs/solutions/architecture/engram-context-echo-behavioral-risk.md`.

### R3: Contextual Warning in User Guide
- **WHEN** `docs/user-guide/fsm-and-checkpoints-explained.md` is updated,
- **THEN** it MUST include a concise note under the tasks desync section informing operators that persistent banners across sessions may stem from memory plugin re-hydration rather than `ce-ai` state.

### R4: Zero Code Impact & Zero SemVer Bump
- **WHEN** the documentation changes are applied,
- **THEN** NO files under `src/` or `tests/` shall be modified.
- **AND** `Cargo.toml` and `Cargo.lock` version MUST remain `1.44.2`.
