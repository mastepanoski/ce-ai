# Exploration: Dynamic Skill Selection & Injection

## Technical Investigation & Options Evaluated

### 1. How Should Semantic Skill Advice Be Modeled?

- **Option A: Provider Returns Full Skill Names / File Paths Directly**
  - *Evaluation:* The external LLM/classifier receives a task description and directly returns a list of skill names (e.g. `["security-review", "test-strategy"]`) or file paths.
  - *Risk:* External provider becomes an untrusted vector for path traversal; provider might hallucinate non-existent skills; tightly couples external providers to internal repository skill names.
- **Option B: Two-Tier Separation of Concerns (Category Intent ➔ Authoritative Registry) — SELECTED**
  - *Evaluation:* The Decision Provider classifies abstract semantic categories (`security`, `testing`, `architecture`, `debugging`, `review`, etc.). `ce-ai`'s local `SkillRegistry` then matches those categories against verified, SHA256-indexed local skill definitions.
  - *Benefits:* Provider output is treated as untrusted input. Local file boundaries, scope precedence, and cryptographic verification cannot be bypassed.

### 2. Category Question Schema

Tasks often span multiple domains (e.g., "Refactor database migration with unit tests" spans both `architecture` and `testing`).
Two inquiry patterns evaluated:
- **Pattern 1: Multi-select Choice Question**:
  A single question `relevant_categories` returning a list of choices.
  *Limitation:* Does not provide per-category confidence scores, making thresholding difficult.
- **Pattern 2: Dedicated Boolean Inquiries with Confidence Scores — SELECTED**:
  Structured boolean questions:
  - `needs_architecture`: boolean
  - `needs_security`: boolean
  - `needs_testing`: boolean
  - `needs_debugging`: boolean
  - `needs_documentation`: boolean
  - `needs_code_review`: boolean
  - `needs_research`: boolean
  *Benefits:* Clean, independent probability distribution. Allows filtering by `minimum_confidence_pct` (e.g., only categories with $\ge 70\%$ confidence trigger candidate inclusion).

### 3. State Schema & Compatibility

In `src/state/state.rs`:
Extend `DecisionsConfig` with an optional `skills: SkillRoutingConfig`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_minimum_confidence")]
    pub minimum_confidence_pct: u32, // default: 70
}

fn default_minimum_confidence() -> u32 {
    70
}

impl Default for SkillRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            minimum_confidence_pct: default_minimum_confidence(),
        }
    }
}
```
Integer percentage (`u32`) preserves `State`'s strict `Eq` trait compliance.

### 4. Matching Policy: Category-to-Skill Resolution

A skill entry matches candidate categories if:
1. The skill's frontmatter explicitly lists the category in `categories` (e.g. `categories: [security, authentication]`).
2. OR (for backward compatibility), the skill's name, triggers, or description contain the category keyword.

Resolved skills from category matching are merged with any exact keyword matches from the user's prompt query, sorted deterministically by registry precedence (Workspace `.ce-ai` > Workspace `.opencode` > Global), deduplicated by name, and verified by SHA256 before injection.

### 5. Failure Modes & Graceful Degradation Matrix

| Scenario | Decision Engine Behavior | Skill Registry Behavior | Exit Code |
| :--- | :--- | :--- | :--- |
| Skill Routing Disabled (`enabled = false`) | No network call made | Resolves via keyword matching | 0 |
| Provider Timeout / Network Error | Circuit breaker records failure | Falls back to keyword matching | 0 |
| Budget Ceilings Exceeded | Skips provider invocation | Falls back to keyword matching | 0 |
| Zero Categories Meet Confidence Threshold | Empty category list | Falls back to keyword matching | 0 |
| Skill SHA256 Tampered | N/A | Degrades status to `fallback-fuzzy`, rejects unverified skill | 0 |

---

## Architectural Decision Records (ADRs)

- **ADR-1: Provider as Intent Classifier, Registry as Security Authority**: Decision providers never return file paths or skill identifiers. They suggest high-level intent categories; the local registry retains exclusive authority over skill discovery, scoping, and integrity.
- **ADR-2: Integer Confidence Thresholding**: Storing confidence thresholds as integer percentages (`0..=100`) avoids floating-point precision issues and maintains `Eq` trait derivation on `State`.
- **ADR-3: Additive Category Hints**: Semantic categories augment deterministic search; they never suppress exact matches or override local project scopes.
