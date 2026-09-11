# Technical Design: Spike Observe-Only — Medir (Sin Bloquear) Escritura en ce-work Sin OpenSpec Aprobado

## 1. System Architecture

The observe-only gate check introduces a non-blocking diagnostic pipeline:

```
[Claude Code Tool Call: Write/Edit]
           │
           ▼ (PreToolUse Hook)
┌────────────────────────────────────────────────────────┐
│               `ce-ai gate check` CLI                   │
│                                                        │
│  1. Check Kill-Switch (Env Var / Flag)                 │
│     └─► If active ──► Immediate Exit 0 (No-Op)         │
│                                                        │
│  2. Extract Tool & Path (Flags or stdin JSON)          │
│     └─► If NOT Write/Edit or NOT under `src/**`        │
│          ──► Exit 0 (Non-target)                       │
│                                                        │
│  3. Read Active Checkpoint (`state.rs`)                │
│     └─► Missing / Corrupt ──► Undetermined             │
│                                                        │
│  4. Evaluate Edge Cases (Issue #337)                   │
│     ├─► Mtime Fallback ──► EdgeCase(MtimeFallback)     │
│     ├─► Worktree Uncommitted ──► EdgeCase(Uncommitted) │
│     └─► Stale Cycle Reset ──► EdgeCase(StaleCycle)     │
│                                                        │
│  5. Evaluate Pure Decision (`evaluate_gate_decision`)  │
│     ├─► Stage == 4 && Missing OpenSpec ──► WouldBlock  │
│     └─► Artifacts Present / Stage != 4 ──► Pass        │
│                                                        │
│  6. Append Record to `~/.ce-ai/gate-events.jsonl`      │
│  7. Always Exit 0 (Write proceeds without hindrance)   │
└────────────────────────────────────────────────────────┘
           │
           ▼
[Telemetry Read by `ce-ai doctor` & `ce-ai status`]
```

## 2. Data Schemas & Models

### 2.1 Decision Enums (`src/commands/gate.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    WouldBlock,
    Pass,
    Undetermined,
    EdgeCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateEdgeCase {
    MtimeFallback,
    WorktreeUncommitted,
    StaleCycleGuard,
}
```

### 2.2 Telemetry Record (`gate-events.jsonl`)
Each event appended to `~/.ce-ai/gate-events.jsonl` follows this schema:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEventRecord {
    pub timestamp: String,
    pub harness: String,
    pub tool: String,
    pub path: String,
    pub workspace: String,
    pub branch: Option<String>,
    pub stage: Option<u32>,
    pub feature: Option<String>,
    pub decision: GateDecision,
    pub edge_case: Option<GateEdgeCase>,
    pub reason: String,
}
```

### 2.3 Aggregated Metrics (`GateStats`)
```rust
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GateStats {
    pub total_observed: usize,
    pub would_block: usize,
    pub pass: usize,
    pub undetermined: usize,
    pub mtime_fallback: usize,
    pub worktree_uncommitted: usize,
    pub stale_cycle_guard: usize,
}
```

## 3. Pure Decision Engine Specification

```rust
pub fn evaluate_gate_decision(
    declared_stage: Option<WorkflowStage>,
    feature_name: Option<&str>,
    resolution: Option<FeatureResolution>,
    is_new_cycle_task: bool,
    has_uncommitted_spec: bool,
    has_proposal: bool,
    has_spec: bool,
    has_tasks: bool,
) -> (GateDecision, Option<GateEdgeCase>, String) {
    // 1. Checkpoint integrity check
    let stage = match declared_stage {
        Some(s) => s,
        None => {
            return (
                GateDecision::Undetermined,
                None,
                "no active workflow checkpoint found".to_string(),
            );
        }
    };

    // 2. Edge case bucket routing (never mixed with happy path)
    if resolution == Some(FeatureResolution::MtimeFallback) {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::MtimeFallback),
            "workflow feature resolved via mtime fallback without git branch".to_string(),
        );
    }
    if is_new_cycle_task {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::StaleCycleGuard),
            "same-branch multi-cycle reset detected in active task".to_string(),
        );
    }
    if has_uncommitted_spec {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::WorktreeUncommitted),
            "openspec directory has uncommitted files in current worktree".to_string(),
        );
    }

    // 3. Happy path evaluation
    if stage == WorkflowStage::WorkTdd {
        let feat = feature_name.unwrap_or("unknown");
        if !has_proposal || !has_spec || !has_tasks {
            let mut missing = Vec::new();
            if !has_proposal { missing.push("proposal.md"); }
            if !has_spec { missing.push("spec.md"); }
            if !has_tasks { missing.push("tasks.md"); }
            return (
                GateDecision::WouldBlock,
                None,
                format!("Stage 4 (ce-work) active for '{feat}' without approved OpenSpec contract (missing: {})", missing.join(", ")),
            );
        }
        return (
            GateDecision::Pass,
            None,
            format!("Stage 4 (ce-work) active with complete OpenSpec contract for '{feat}'"),
        );
    }

    // Non-Stage 4 writes pass
    (
        GateDecision::Pass,
        None,
        format!("Stage {} ({}) permits writes without Stage 4 OpenSpec contract", stage.number(), stage.as_str()),
    )
}
```

## 4. CLI Contract

### 4.1 Subcommand Definition (`src/main.rs`)
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ...
    #[command(subcommand)]
    Gate(GateCommands),
}

#[derive(Subcommand, Debug)]
pub enum GateCommands {
    /// Observe and record gate telemetry for agent tool writes (Spike #333)
    Check(GateCheckArgs),
}

#[derive(Args, Debug, Default)]
pub struct GateCheckArgs {
    /// Tool name being invoked (e.g., Write, Edit)
    #[arg(long)]
    pub tool: Option<String>,

    /// File path targeted by the write operation
    #[arg(long)]
    pub path: Option<String>,

    /// Emergency kill-switch to immediately bypass gate check logic
    #[arg(long)]
    pub disabled: bool,
}
```

### 4.2 Stdin Parsing (Claude Code `PreToolUse`)
When `--tool` and `--path` are omitted, `run_gate_check` attempts to parse a JSON payload from `stdin`:
- Checks `tool_name` or `tool` for string value matching `Write` or `Edit` (case-insensitive).
- Checks `tool_input.path`, `tool_input.file_path`, `input.path`, `input.file_path` for target path string.
- If payload is unparseable or does not match a target tool, exits cleanly with code `0`.

### 4.3 Claude Code Hook Configuration (`src/harness/claude.rs`)
Added helpers:
- `ensure_claude_gate_hook(settings_path: &Path) -> Result<bool, CeError>`
- `remove_claude_gate_hook(settings_path: &Path) -> Result<bool, CeError>`
- `has_claude_gate_hook(settings_path: &Path) -> bool`
Adds a `PreToolUse` matcher `Write|Edit` executing `ce-ai gate check`.

## 5. Observability Reporting

In `ce-ai status` and `ce-ai doctor`:
```rust
let stats = load_gate_stats(&ctx.config_dir).unwrap_or_default();
if stats.total_observed > 0 {
    println!(
        "gate-check: {} observed ({} would-block, {} pass, {} undetermined, {} edge-case: {} mtime_fallback, {} worktree_uncommitted, {} stale_cycle_guard)",
        stats.total_observed,
        stats.would_block,
        stats.pass,
        stats.undetermined,
        stats.mtime_fallback + stats.worktree_uncommitted + stats.stale_cycle_guard,
        stats.mtime_fallback,
        stats.worktree_uncommitted,
        stats.stale_cycle_guard,
    );
}
```
If `stats.total_observed == 0`, prints:
`gate-check: 0 observed`
Zero sensitive write contents or modified code lines are ever displayed or persisted.
