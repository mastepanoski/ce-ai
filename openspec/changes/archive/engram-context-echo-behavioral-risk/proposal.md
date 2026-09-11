# Proposal: Documentation of Engram Context Echo Behavioral Feedback Risk

## Problem Statement

During the investigation of Issue #313, a recurring symptom was analyzed where the `ce-ai` workflow FSM progress banner (`tasks progress: 0/N completed`) appeared to stubbornly persist across distinct agent sessions even after work had progressed.

Two potential causes were initially hypothesized:
1. Purely visual artifact between disconnected systems.
2. A cross-cutting functional feedback loop where persistent memory (Engram) directly interacted with `ce-ai`'s stage inference engine.

A deep technical audit of both `ce-ai` and the third-party Engram plugin codebase (`github.com/Gentleman-Programming/engram`) established that neither codebase has a functional code defect. Instead, the loop is cognitive: when agents re-hydrate context via `mem_context`, Engram's `FormatContext` displays the last 10 user prompts verbatim. If a user previously pasted a `ce-ai` banner into a prompt, the LLM reads that historical text and can conflate it with the live repository state, echoing the warning or stalling workflow progression.

## Scope Boundaries

### In Scope
- Author formal documentation capturing the root cause, technical evidence (with file and line citations in `ce-ai` and Engram), cognitive mechanics, practiced mitigation, and upstream recommendations in `docs/solutions/architecture/engram-context-echo-behavioral-risk.md`.
- Add an architectural vocabulary definition in `CONCEPTS.md`.
- Add a cross-reference advisory note in `docs/user-guide/fsm-and-checkpoints-explained.md` under the tasks desync section.

### Out of Scope
- Any code modifications in `src/`.
- Opening PRs or issues against the upstream third-party `github.com/Gentleman-Programming/engram` repository.
- Modifying `ce-ai`'s hermetic stage inference engine.
- SemVer bump (documentation-only change).

## Success Criteria
1. Architecture documentation, concepts, and user guide accurately record the investigation findings.
2. File and line citations for `ce-ai` (`src/commands/workflow.rs`) and Engram (`mcp.go`, `store.go`) are exact.
3. Upstream recommendations are clearly designated as advisory for a third-party project.
4. All quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test --all-features`) remain 100% passing.
