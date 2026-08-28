# Exploration: Pedagogical Guardrail Mode (`ce-ai guard`)

## Technical Investigation & Context

### 1. Architectural Placement: Binary Logic vs Managed Asset

| Approach | Mechanics | Tradeoffs | Decision |
|:---|:---|:---|:---|
| **Option A: Blocking Binary Interceptor** | Intercept shell/IDE calls with an inline daemon/proxy blocking execution until approval | High platform complexity, brittle IPC across 11 AI harnesses, high friction | **Rejected** |
| **Option B: Managed Asset + Lifecycle Governance** | CLI governs lifecycle, persists state, indexes skills, and tracks SHA256 integrity; specialized pedagogical skills execute the didactic flow | Clean domain boundaries, cross-harness portability, non-destructive, audited by `doctor` | **Selected** |

### 2. Command Surface & Lifecycle Integration

- **Option 1: Subcommand on `init-prj` (`ce-ai init-prj --guardrail junior`)**
  - *Evaluation:* Couples pedagogical assistance to project adoption. Makes it impossible to toggle guardrails on existing global harness configurations without modifying repository rules.
  - *Decision:* **Rejected**.

- **Option 2: Dedicated Lifecycle Subcommand (`ce-ai guard enable|disable|status`)**
  - *Evaluation:* Orthogonal, explicit, and self-contained. Allows developers or team leads to toggle oversight per harness or globally.
  - *Decision:* **Selected**.

### 3. Anti-Bottleneck Human-In-The-Loop (HITL) Design

To avoid developer fatigue and mode abandonment (Risk R2), gating must be risk-tiered rather than step-by-step:

1. **Auto-Approved Tier:** Routine tests, formatting, refactoring within existing specs run without interruptions.
2. **Hard Checkpoint 1 (Spec Contract Approval):** Stage 2 OpenSpec freeze requires junior developer confirmation before code generation begins.
3. **Hard Checkpoint 2 (Architectural Decisions & Tradeoffs):** Any non-trivial structural decision must present at least two viable alternatives with pros/cons before proceeding.
4. **Lightweight Explain-Back:** Before commit (`Stage 7`), the assistant prompts the junior developer to explain in 1–2 sentences what was changed and why.

### 4. State Management & Schema Evolution

In `src/state/state.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuardLevel {
    Junior,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailState {
    pub enabled: bool,
    pub level: GuardLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    pub updated_at: String,
}
```
Using `#[serde(default, skip_serializing_if = "Option::is_none")]` on `State::guardrail` ensures that legacy `state.json` files continue to load seamlessly.
