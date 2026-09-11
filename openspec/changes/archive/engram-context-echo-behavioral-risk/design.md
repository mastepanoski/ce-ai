# Design: Documentation Structure for Engram Context Echo Risk

## Document Artifacts & Placements

### 1. Architectural Solution Document
- **Location**: `docs/solutions/architecture/engram-context-echo-behavioral-risk.md`
- **Diátaxis Type**: Explanation
- **Sections**:
  1. Front-matter: YAML metadata (`category: architecture`, `tags`, `components`, `applies_when`).
  2. Context & Symptom: The persistent FSM banner across sessions and initial hypotheses.
  3. Technical Investigation:
     - Hermetic isolation in `ce-ai` (`src/commands/workflow.rs`).
     - Mechanism in third-party Engram plugin (`mcp.go`, `store.go`).
  4. Behavioral Feedback Loop Analysis: How LLMs confuse re-hydrated past prompts with current repository truth.
  5. Mitigations:
     - Practiced mitigation: manual memory editing.
     - Advisory upstream recommendation: historical framing in `FormatContext`.
     - `ce-ai` architectural invariant: continued isolation from external memory sources.

### 2. Domain Vocabulary Definition
- **Location**: `CONCEPTS.md`
- **Term**: `Engram Context Echo (Behavioral Loop)`
- **Summary**: Definitional description linking to the architectural solution document.

### 3. User Guide Cross-Reference
- **Location**: `docs/user-guide/fsm-and-checkpoints-explained.md`
- **Section**: `#### ⚠️ What Happens When Tasks Desync?`
- **Format**: GitHub-style `> [!NOTE]` alert explaining that perceived persistence across sessions is often an echo from memory plugins, with a link to the architecture doc.
