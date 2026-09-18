# Design: Dynamic Skill Selection & Injection (Intelligent Skill Routing)

## Overview

The Dynamic Skill Selection & Injection layer enables `ce-ai` to dynamically identify semantic skill categories relevant to an incoming task description using the System 1 Pluggable Decision Engine, and then query the authoritative `SkillRegistry` to securely resolve, verify, and inject corresponding `SKILL.md` prompt references.

---

## Architectural Workflow & Security Boundary

```text
Task Description / User Prompt
             │
             ▼
  ┌────────────────────────────────────────────────────────┐
  │ 1. Intent Classification (System 1 Decision Engine)     │
  │    - Evaluate 7 semantic category questions:            │
  │      needs_architecture, needs_security, needs_testing, │
  │      needs_debugging, needs_documentation,              │
  │      needs_code_review, needs_research                  │
  │    - Filter by minimum_confidence_pct (default: 70%)    │
  │    - Yield candidate categories: ["security", "testing"]│
  └────────────────────────────────────────────────────────┘
             │ (Candidate Categories — Untrusted Input)
             ▼
  ┌────────────────────────────────────────────────────────┐
  │ 2. Authoritative Skill Registry Resolution              │
  │    - Query local SkillRegistry index                    │
  │    - Match categories against skill metadata & triggers │
  │    - Merge with explicit keyword matches                │
  │    - Filter by target harness (e.g. opencode, claude)   │
  └────────────────────────────────────────────────────────┘
             │
             ▼
  ┌────────────────────────────────────────────────────────┐
  │ 3. Cryptographic Verification & Boundary Check          │
  │    - Validate SHA256 matches indexed digest             │
  │    - Verify path is inside authorized roots             │
  │    - Deduplicate by skill name                          │
  └────────────────────────────────────────────────────────┘
             │
             ▼
  ┌────────────────────────────────────────────────────────┐
  │ 4. Prompt Injection Formatting                          │
  │    - Generate reproducible markdown prompt injection:   │
  │      <!-- ce-ai:skill_resolution status=... -->         │
  │      ## Skills to load before work:                     │
  │      - **security-review**: ...                         │
  └────────────────────────────────────────────────────────┘
```

---

## Data Structures & Configuration Schema

### 1. Skill Routing Configuration (`src/state/state.rs` & `src/decisions/skill_routing.rs`)

Nested inside `DecisionsConfig`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_skill_confidence")]
    pub minimum_confidence_pct: u32,
}

fn default_skill_confidence() -> u32 {
    70 // 70% confidence threshold
}

impl Default for SkillRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            minimum_confidence_pct: default_skill_confidence(),
        }
    }
}
```

### 2. Category Definitions & Evaluation Models (`src/decisions/skill_routing.rs`)

```rust
pub const RECOGNIZED_CATEGORIES: &[(&str, &str)] = &[
    ("architecture", "Does this task involve system design, architectural boundaries, or data modeling?"),
    ("security", "Does this task involve security reviews, authentication, authorization, or vulnerabilities?"),
    ("testing", "Does this task involve creating, updating, or analyzing unit, integration, or E2E tests?"),
    ("debugging", "Does this task involve diagnosing bugs, investigating errors, stack traces, or regressions?"),
    ("documentation", "Does this task involve writing or maintaining technical documentation, specifications, or guides?"),
    ("code_review", "Does this task involve reviewing code changes, checking style standards, or PR feedback?"),
    ("research", "Does this task require exploring scientific literature, codebase research, or deep algorithmic analysis?"),
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillClassification {
    pub category: String,
    pub confidence: f64,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillRoutingResult {
    pub task: String,
    pub candidate_categories: Vec<String>,
    pub classifications: Vec<SkillClassification>,
    pub resolved_skills: Vec<crate::source::registry::SkillEntry>,
    pub fallback_applied: bool,
    pub rationale: String,
    pub latency_ms: u64,
}
```

### 3. Skill Metadata Schema Extension (`src/source/registry.rs`)

`SkillEntry` and `SkillFrontmatter` are extended with `categories`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub scope: String,
    pub triggers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    pub sha256: String,
    pub harness_paths: BTreeMap<String, String>,
}
```

Frontmatter parsing extracts either `categories: [a, b]` or `decision.categories: [a, b]` from YAML frontmatter in `SKILL.md`.

---

## Matching & Resolution Algorithm

1. **Question Formulation**:
   - `SkillRouter::build_request(task)` constructs 7 `DecisionQuestion::Boolean` requests corresponding to the recognized categories.
2. **Confidence Filtering**:
   - For each category question, if answer `value == true` and `confidence >= (minimum_confidence_pct as f64 / 100.0)`, add category to `candidate_categories`.
3. **Registry Resolution**:
   - `SkillRegistry::resolve_with_routing(&self, harness, task, candidate_categories)`:
     - Collects skills whose `name`, `description`, `triggers`, or `categories` match any candidate category or the task query.
     - Enforces scope precedence: Workspace (`.ce-ai`) > Workspace (`.opencode`) > Global.
     - Verifies SHA256 hash against actual file on disk.
     - Validates canonical path boundaries.
4. **Degradation**:
   - If routing is disabled or the Decision Engine fails, standard keyword matching on `task` is used as fallback, returning `fallback_applied = true`.

---

## CLI & Diagnostic Output

### 1. `ce-ai skills resolve "<task description>" [--harness <harness>] [--json] [--verbose]`

Human-Readable Verbose Output:
```text
Skill Routing Diagnostics:
  security       0.97 [selected]
  testing        0.85 [selected]
  architecture   0.45 [ignored (< 70%)]
  debugging      0.20 [ignored (< 70%)]

Candidate Categories: security, testing

Resolved Skills (2):
  - security-review: Review authentication and sensitive endpoints
    Path: file:///Users/.../.config/opencode/skills/security-review/SKILL.md
  - backend-testing: Comprehensive backend test generation
    Path: file:///Users/.../.config/opencode/skills/backend-testing/SKILL.md
```

JSON Output:
```json
{
  "task": "Audit authentication cookies and generate unit tests",
  "candidate_categories": ["security", "testing"],
  "classifications": [
    { "category": "security", "confidence": 0.97, "selected": true },
    { "category": "testing", "confidence": 0.85, "selected": true }
  ],
  "resolved_skills": [...],
  "fallback_applied": false,
  "rationale": "Matched categories: security, testing",
  "latency_ms": 32
}
```

### 2. `ce-ai doctor` Integration

Checks `state.decisions.skills` and reports:
`doctor-info: decision-skills: active (threshold: 70%)` or `doctor-info: decision-skills: disabled`.
