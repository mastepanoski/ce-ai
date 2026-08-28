# Design: Pedagogical Guardrail Mode (`ce-ai guard`)

## Architecture & Data Flow

```
                      ┌─────────────────────────────────┐
                      │    CLI Entry / TUI Dispatch     │
                      │       `ce-ai guard ...`         │
                      └────────────────┬────────────────┘
                                       │
                                       ▼
                      ┌─────────────────────────────────┐
                      │    `src/commands/guard.rs`      │
                      │      (implements CeCommand)     │
                      └────────────────┬────────────────┘
                                       │
                ┌──────────────────────┴──────────────────────┐
                ▼                                             ▼
┌───────────────────────────────┐             ┌───────────────────────────────┐
│     StateStore Port / Fs      │             │     Doctor & Status Engine    │
│  `state.json` (write_atomic)  │             │  Integrity & Drift Detection  │
└───────────────────────────────┘             └───────────────────────────────┘
```

## Schema Definitions

In `src/state/state.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Intensity level of pedagogical oversight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GuardLevel {
    #[default]
    Junior,
    Strict,
}

impl std::fmt::Display for GuardLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "junior"),
            Self::Strict => write!(f, "strict"),
        }
    }
}

/// Pedagogical guardrail configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub enabled: bool,
    pub level: GuardLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// Extended State struct
pub struct State {
    // ... preexisting fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrail: Option<GuardrailConfig>,
}
```

## CLI Interface & Clap Arguments

In `src/commands/mod.rs`:

```rust
#[derive(Subcommand, Debug, Clone)]
pub enum GuardCommands {
    /// Enable pedagogical guardrail mode for junior developer oversight
    Enable {
        /// Oversight intensity: junior (default, batched) or strict (per-module)
        #[arg(long, default_value = "junior")]
        level: String,

        /// Target specific harness (defaults to global state)
        #[arg(long)]
        harness: Option<String>,
    },
    /// Disable pedagogical guardrail mode cleanly
    Disable {
        /// Target specific harness (defaults to global state)
        #[arg(long)]
        harness: Option<String>,
    },
    /// Report current guardrail status and integrity
    Status {
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Parser, Debug, Clone)]
pub struct GuardArgs {
    #[command(subcommand)]
    pub command: GuardCommands,
}
```

## Command Implementation (`src/commands/guard.rs`)

```rust
pub struct GuardCommand {
    pub args: GuardArgs,
}

impl CeCommand for GuardCommand {
    fn execute(&self, ctx: &Context) -> Result<(), CeError> {
        match &self.args.command {
            GuardCommands::Enable { level, harness } => run_guard_enable(ctx, level, harness.as_deref()),
            GuardCommands::Disable { harness } => run_guard_disable(ctx, harness.as_deref()),
            GuardCommands::Status { json } => run_guard_status(ctx, *json),
        }
    }
}
```

## Integration with `doctor` & `status`

1. **`src/commands/doctor.rs`**:
   - Evaluates `state.guardrail`:
     - If `Some(g)` with `g.enabled == true`: Reports `Guardrail: enabled (level: {level})` [OK].
     - If disabled or absent: Reports `Guardrail: disabled` [INFO].
2. **`src/commands/status.rs`**:
   - Displays guardrail mode and level in the status overview.
