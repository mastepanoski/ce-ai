# Technical Design: Blocking Gate Check & Validation Receipt (Issue #334)

## 1. System Architecture & Flow

```
Agent Tool Invocation (Write|Edit under src/**)
                      │
                      ▼
             Claude Code PreToolUse
                      │
                      ▼
             ce-ai gate check
                      │
         ┌────────────┴────────────┐
         ▼                         ▼
   Kill-Switch?           Target Path under src/**?
   (env / --disabled)              │
   YES -> Exit 0 (Pass)     NO -> Exit 0 (Pass)
                                   │
                                   ▼
                       Evaluate Policy Decision
                      (Stage, Tier, Entry Point,
                        Artifacts, Edge Cases)
                                   │
         ┌─────────────────────────┴─────────────────────────┐
         ▼                                                   ▼
     DECISION: PASS                                  DECISION: BLOCKED
  (Valid OpenSpec, ce-debug,                      (Stage 4, Full Tier,
   Tier Minimal, or EdgeCase)                     missing proposal/spec/tasks)
         │                                                   │
         ▼                                                   ▼
Write GateReceipt (.validation.json)            Write GateReceipt (.validation.json)
Append gate-events.jsonl (Pass)                 Append gate-events.jsonl (Blocked)
Update state.gate_receipts                      Update state.gate_receipts
         │                                                   │
         ▼                                                   ▼
      Exit 0                                        Enforce Mode?
                                                  YES -> Print Stderr & EXIT CODE 2
                                                  NO  -> Print Advisory & Exit 0
```

## 2. Data Structures & Schemas

### 2.1 Gate Mode Enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GateMode {
    #[default]
    Enforce,
    Observe,
}

impl GateMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "enforce" | "blocking" => Some(GateMode::Enforce),
            "observe" | "advisory" => Some(GateMode::Observe),
            _ => None,
        }
    }
}
```

### 2.2 Gate Decision Enum Expansion
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    /// Stage 4 write blocked due to missing OpenSpec contract artifacts in enforce mode.
    Blocked,
    /// Stage 4 write that would be blocked in observe mode (Spike #333).
    WouldBlock,
    /// Authorized write (valid OpenSpec contract, ce-debug, tier minimal, or non-target path).
    Pass,
    /// Ambiguous or unadopted checkpoint state.
    Undetermined,
    /// Environmental or lifecycle anomaly isolated from the happy path.
    EdgeCase,
}
```

### 2.3 Gate Receipt Schema
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReceipt {
    pub timestamp: String,
    pub feature: String,
    pub target_path: String,
    pub decision: GateDecision,
    pub stage: Option<u32>,
    pub entry_point: Option<String>,
    pub tier: String,
    pub missing_artifacts: Vec<String>,
    pub reason: String,
}
```

### 2.4 State Extension (`src/state/state.rs`)
```rust
pub struct State {
    // ... existing fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_mode: Option<GateMode>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gate_receipts: BTreeMap<String, GateReceipt>,
}
```

### 2.5 CLI Arguments (`src/commands/gate.rs`)
```rust
#[derive(Args, Debug, Default, Clone)]
pub struct GateCheckArgs {
    /// Tool name being invoked (e.g. Write, Edit)
    #[arg(long)]
    pub tool: Option<String>,

    /// Target file path of the write/edit operation
    #[arg(long)]
    pub path: Option<String>,

    /// Gate evaluation mode: enforce (blocking) or observe (advisory)
    #[arg(long)]
    pub mode: Option<String>,

    /// Optional explicit entry point (e.g. ce-debug, ce-work)
    #[arg(long)]
    pub entry_point: Option<String>,

    /// Emergency kill-switch to immediately bypass gate check logic
    #[arg(long)]
    pub disabled: bool,
}
```

## 3. Core Logic & Pure Policy Evaluation

### 3.1 Policy Evaluation Function
```rust
pub fn evaluate_gate_policy(
    declared_stage: Option<WorkflowStage>,
    feature_name: Option<&str>,
    resolution: Option<FeatureResolution>,
    is_new_cycle_task: bool,
    has_uncommitted_spec: bool,
    tier: AdoptionTier,
    entry_point: Option<&str>,
    has_proposal: bool,
    has_spec: bool,
    has_tasks: bool,
    mode: GateMode,
) -> (GateDecision, Option<GateEdgeCase>, Vec<String>, String)
```

**Policy Rules:**
1. **Missing Stage:** If `declared_stage.is_none()`, return `(GateDecision::Undetermined, None, vec![], "no active workflow checkpoint found")`.
2. **Edge Cases:** If `resolution == MtimeFallback` / `is_new_cycle_task` / `has_uncommitted_spec`, return `(GateDecision::EdgeCase, Some(...), vec![], reason)`. Edge cases are NEVER blocked.
3. **Adoption Tier Exemption:** If `tier == AdoptionTier::Minimal`, return `(GateDecision::Pass, None, vec![], "project tier minimal permits writes without full OpenSpec")`.
4. **Direct Entry Point Exemption:** If `entry_point.map(|e| e.contains("ce-debug") || e.contains("debug")).unwrap_or(false)`, return `(GateDecision::Pass, None, vec![], "ce-debug direct entry point permits bug fix writes without upfront OpenSpec")`.
5. **Stage != 4:** If `stage != WorkflowStage::WorkTdd`, return `(GateDecision::Pass, None, vec![], "stage permits writes without Stage 4 OpenSpec contract")`.
6. **Stage 4 + Missing Artifacts:**
   Collect missing from `!has_proposal`, `!has_spec`, `!has_tasks`.
   If any missing:
   - If `mode == GateMode::Enforce`: return `(GateDecision::Blocked, None, missing, reason)`.
   - If `mode == GateMode::Observe`: return `(GateDecision::WouldBlock, None, missing, reason)`.
7. **Stage 4 + Complete Contract:**
   Return `(GateDecision::Pass, None, vec![], "Stage 4 (ce-work) active with complete OpenSpec contract")`.

## 4. Remediation Stderr Feedback Format

When a write is blocked, `ce-ai gate check` emits the following message on `stderr`:

```text
🛑 ce-ai gate check: Write blocked on '<target_path>'

Active Stage: Stage 4 (WorkTdd) for feature '<feature_name>'
Missing OpenSpec Contract Artifacts:
  ✖ proposal.md — Missing problem statement & boundaries
  ✖ spec.md     — Missing formal WHEN/THEN requirements
  ✖ tasks.md    — Missing implementation task checklist

Remediation Steps:
  1. Author the OpenSpec contract: Run `/ce-plan` or create the missing files in `openspec/changes/<feature_name>/`.
  2. If performing an emergency bugfix: Switch to direct entry via:
     ce-ai workflow checkpoint --stage 4 --task "ce-debug: <issue>"
  3. Emergency override: Set CE_AI_DISABLE_GATE_CHECK=1 in your environment.
```

And immediately exits with **Exit Code 2**.

## 5. Pre-Existing Bugfix in `probe_openspec_context_in`

In `src/commands/workflow.rs`, update the directory iteration:
```rust
if let Ok(read) = std::fs::read_dir(&openspec_dir) {
    for entry in read.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "archive" || name_str.starts_with('.') {
            continue;
        }
        if entry.path().is_dir() {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            entries.push((entry.path(), mtime));
        }
    }
}
```
This guarantees that `archive/` is never considered a candidate active feature.
