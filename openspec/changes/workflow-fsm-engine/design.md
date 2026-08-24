# Design: Real 7-Stage Workflow FSM Engine & Context Recovery

## Data Schemas

### 1. `WorkflowStage` Enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStage {
    Ideation = 1,
    OpenSpec = 2,
    ExecutionPlan = 3,
    WorkTdd = 4,
    Verification = 5,
    KnowledgeCapture = 6,
    GitShipping = 7,
}

impl WorkflowStage {
    pub fn number(&self) -> u32 {
        *self as u32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowStage::Ideation => "ideation",
            WorkflowStage::OpenSpec => "openspec",
            WorkflowStage::ExecutionPlan => "plan",
            WorkflowStage::WorkTdd => "work",
            WorkflowStage::Verification => "verify",
            WorkflowStage::KnowledgeCapture => "compound",
            WorkflowStage::GitShipping => "ship",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "1" | "ideation" | "brainstorm" => Ok(WorkflowStage::Ideation),
            "2" | "openspec" | "spec" => Ok(WorkflowStage::OpenSpec),
            "3" | "plan" | "executionplan" => Ok(WorkflowStage::ExecutionPlan),
            "4" | "work" | "tdd" | "worktdd" => Ok(WorkflowStage::WorkTdd),
            "5" | "verify" | "verification" => Ok(WorkflowStage::Verification),
            "6" | "compound" | "knowledgecapture" => Ok(WorkflowStage::KnowledgeCapture),
            "7" | "ship" | "gitshipping" => Ok(WorkflowStage::GitShipping),
            _ => Err(CeError::Usage(format!(
                "invalid workflow stage '{s}'. Valid stages: 1 (ideation), 2 (openspec), 3 (plan), 4 (work), 5 (verify), 6 (compound), 7 (ship)"
            ))),
        }
    }

    pub fn can_transition_to(&self, target: WorkflowStage) -> bool {
        let current_num = self.number();
        let target_num = target.number();
        // Allowed: Reset to Stage 1, same stage, advance by 1 stage, or rewind by 1 stage
        target_num == 1 || target_num == current_num || target_num == current_num + 1 || (current_num > 1 && target_num == current_num - 1)
    }
}
```

### 2. `WorkflowState` Struct & `state.json` Backward Compatibility
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    pub stage: WorkflowStage,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
    pub updated_at: String,
}
```
Backward compatibility: If `state.workflow` is `None` but `state.last_update_check` contains a legacy string `"phase | task | timestamp"`, `State::get_workflow()` parses the legacy string into `WorkflowState` as fallback!

### 3. CLI Command Signature for `checkpoint`
```rust
#[derive(clap::Subcommand)]
pub enum Action {
    Status {
        #[arg(long)]
        json: bool,
    },
    Checkpoint {
        /// Stage identifier (1..7 or ideation, openspec, plan, work, verify, compound, ship).
        #[arg(long, short = 's', alias = "phase")]
        stage: String,
        /// Active subtask (e.g. "Authoring proposal.md").
        #[arg(long, short = 't')]
        task: String,
        /// Feature or change name (e.g. "dry-run-purity").
        #[arg(long, short = 'f')]
        feature: Option<String>,
    },
    Resume {
        #[arg(long)]
        json: bool,
    },
}
```

### 4. Context Recovery Probing Fallback Logic
In `workflow resume`:
1. Inspect `state.workflow.feature_name`.
2. If `feature_name` is set, probe `openspec/changes/<feature_name>/`.
3. Fallback: If `feature_name` is `None` or directory does not exist:
   - Check the most recently modified directory under `openspec/changes/`.
   - Probe active git branch name (`git rev-parse --abbrev-ref HEAD`).
4. Read existing files (`proposal.md`, `spec.md`, `tasks.md`), count completed `[x]` vs pending `[ ]` tasks, and output hand-off context.
