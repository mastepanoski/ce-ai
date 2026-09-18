# Specification: Dynamic Skill Selection & Injection (Intelligent Skill Routing)

## Requirements

### R1: Semantic Intent Category Classification
- **WHEN** a task description is evaluated for skill resolution and skill routing is enabled (`enabled = true`)
- **THEN** the system MUST construct a `DecisionRequest` querying semantic category dimensions:
  `needs_architecture`, `needs_security`, `needs_testing`, `needs_debugging`, `needs_documentation`, `needs_code_review`, `needs_research`.
- **WHEN** the Decision Engine returns responses for these category questions
- **THEN** only categories evaluated with `value == true` and `confidence >= (minimum_confidence_pct as f64 / 100.0)` MUST become candidate routing categories.

### R2: Skill Classification Metadata & Schema Extension
- **WHEN** `SkillRegistry::build` or scan processes a `SKILL.md` file
- **THEN** it MUST parse optional `categories` from YAML frontmatter (e.g. `categories: [security, review]`).
- **WHEN** serializing or indexing skill entries
- **THEN** the `categories` field MUST be persisted in `skills-registry.json` without breaking backward compatibility.

### R3: Authoritative Precedence & Cryptographic Invariants
- **WHEN** resolving skills from candidate categories
- **THEN** the decision provider output MUST be treated strictly as untrusted category hints.
- **WHEN** matching skills
- **THEN** the `SkillRegistry` MUST deterministically enforce scope precedence:
  Workspace (`.ce-ai/skills`) > Workspace (`.opencode/skills`) > Global (`~/.config/...`).
- **WHEN** candidate skills are identified
- **THEN** the system MUST verify the target file's cryptographic SHA256 matches its registered index before inclusion.
- **WHEN** a file has been tampered with, deleted, or escapes authorized root boundaries
- **THEN** the system MUST reject the unverified file and report `fallback-fuzzy` degradation status.

### R4: Configurable Confidence Thresholds
- **WHEN** configuring skill routing in `state.json` (`[decisions.skills]`)
- **THEN** `minimum_confidence_pct` MUST be stored as an integer percentage (`u32`, default: 70).
- **WHEN** evaluating decisions
- **THEN** any category with confidence below `minimum_confidence_pct` MUST NOT be included in candidate categories.

### R5: Graceful Degradation & Fallback Behavior
- **WHEN** skill routing is disabled (`enabled = false`), or the decision provider times out, encounters an HTTP error, or trips the circuit breaker
- **THEN** `ce-ai` MUST immediately fall back to existing deterministic keyword matching without erroring (exit code 0).
- **WHEN** zero categories satisfy the confidence threshold
- **THEN** `ce-ai` MUST fall back to existing deterministic keyword matching.

### R6: CLI Advisory Ergonomics & Diagnostics
- **WHEN** executing `ce-ai skills resolve "<task>"`
- **THEN** `ce-ai` MUST output the standard prompt injection markdown block with resolved skills.
- **WHEN** the `--json` flag is provided
- **THEN** `ce-ai` MUST output structured JSON containing `task`, `candidate_categories`, `classifications`, `resolved_skills`, `fallback_applied`, and `latency_ms`.
- **WHEN** the `--verbose` flag is provided
- **THEN** `ce-ai` MUST print detailed per-category confidence scores and selection indicators.
- **WHEN** `ce-ai doctor` is executed
- **THEN** it MUST inspect `state.decisions.skills` and report the active threshold or disabled status.

---

## Acceptance Criteria

1. **Deterministic Offline Tests**: Unit and integration tests verify multi-category matching, threshold filtering, and fallback behavior using `MockDecisionProvider` without network calls.
2. **Security & Cryptographic Invariant**: Tampered skills or simulated path escape attempts are rejected, preserving 100% pass on `tests/security.rs`.
3. **Graceful Fallback Invariant**: Simulated provider timeouts, network errors, or circuit breaker trips gracefully return keyword matches with exit code 0.
4. **CLI Ergonomics**: `ce-ai skills resolve "audit auth tokens and generate unit tests" --json` outputs both `security` and `testing` candidate categories and resolves corresponding skills.
5. **Quality Gates**: Zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`), formatted code (`cargo fmt --check`), 100% test pass (`cargo test`).
