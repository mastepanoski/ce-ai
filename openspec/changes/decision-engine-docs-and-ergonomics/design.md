# Design: Decision Engine Documentation & CLI Ergonomics

## System Architecture & Data Schema

### 1. CLI Actions Architecture (`src/commands/decisions.rs`)

```rust
#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Display Decision Engine status, provider health, API key status, and budget consumption.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Configure or test Decision Provider authentication credentials.
    Auth {
        #[arg(long, num_args = 0..=1)]
        key: Option<Option<String>>,
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        check: bool,
    },
    /// Quick-setup wizard or preset configuration for the Decision Engine.
    Setup {
        /// Preset configuration: recommended, shadow, local, or off/disabled.
        #[arg(long, default_value = "recommended")]
        preset: String,
    },
    /// Get or set the operational execution mode (active, shadow, off).
    Mode {
        /// Target mode to set: active, shadow, or off. If omitted, displays current mode.
        mode: Option<String>,
    },
    /// Test decision evaluation with a sample structured query.
    Test {
        #[arg(long)]
        provider: Option<String>,
    },
    /// Evaluate execution risk policy for a command or tool invocation.
    CheckRisk { ... },
    /// Evaluate work readiness and verification advisory for an ODD task or CE stage.
    CheckReadiness { ... },
    /// Evaluate model routing recommendation for a given task description.
    Route(crate::commands::models::RouteArgs),
}
```

### 2. Operational Mode Controller (`handle_mode`)

The `handle_mode` handler implements idempotent inspection and atomic mode switching:
1. **Inspection (`mode` is `None`)**:
   - Inspects `state.decisions`.
   - If absent or `!enabled` or `mode == DecisionMode::Off`, outputs:
     `Decision Engine mode: off (disabled)`.
   - Else outputs:
     `Decision Engine mode: <mode> (provider: <provider>)`.
2. **Mutation (`mode` is `Some(target)`)**:
   - Parses `target` via `DecisionMode::parse(target)`.
   - Rejects invalid values with `CeError::Usage`.
   - If target is `DecisionMode::Off`:
     - Updates `config.enabled = false` and `config.mode = DecisionMode::Off`.
     - Serializes state and persists atomically using `crate::state::write_atomic`.
     - Prints confirmation: `Decision Engine mode set to: off (disabled)`.
   - If target is `DecisionMode::Active` or `DecisionMode::Shadow`:
     - Updates `config.enabled = true` and `config.mode = target`.
     - If `state.decisions` was uninitialized, provisions recommended preset defaults with the requested mode.
     - Serializes state and persists atomically using `crate::state::write_atomic`.
     - Prints confirmation: `Decision Engine mode set to: <mode> (provider: <provider>)`.

### 3. Preset Expansion (`handle_setup`)

Supports `"off" | "disabled"` alongside existing presets:
```rust
"off" | "disabled" => {
    let mut cfg = state.decisions.unwrap_or_default();
    cfg.enabled = false;
    cfg.mode = DecisionMode::Off;
    cfg
}
```
Validation error string is updated to:
`"invalid preset '{preset_name}'. Valid presets: recommended, shadow, local, off"`

### 4. Presets vs. Modes Conceptual Matrix

| Concept | Scope | Values | Description |
| :--- | :--- | :--- | :--- |
| **Preset** | Configuration template | `recommended` | Provider: `jev`, Mode: `active`, Budget: $5, all modules on |
| | | `shadow` | Provider: `jev`, Mode: `shadow`, Budget: $5, all modules on |
| | | `local` | Provider: `mock`, Mode: `active`, Offline mock models |
| | | `off` | Mode: `off`, `enabled: false`, zero evaluations |
| **Mode** | Runtime execution state | `active` | Evaluations actively inform execution & risk policies |
| | | `shadow` | Evaluations run in background for telemetry/latency only |
| | | `off` | Zero network calls or evaluations |

### 5. Documentation Design (`docs/user-guide/decision-engine-guide.md`)

- **Diátaxis Type**: How-to / Reference.
- **Cognitive Load Strategy**:
  - Quick answer / TL;DR upfront.
  - Clear dual-audience path: operators setting up keys vs developers querying policies.
  - Complete CLI command table with flags and defaults.
  - Explicit explanation of safety guarantees (deterministic denial & circuit breakers).
