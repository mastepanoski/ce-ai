# OpenSpec Design: Release v0.7.0 System Architecture

## Struct & Schema Extensions

### 1. Workspace Configuration Merging (`src/state/state.rs`)
```rust
impl State {
    pub fn load_with_workspace_overrides(global_path: &Path, workspace_root: Option<&Path>) -> Result<Self, CeError> {
        let mut state = Self::load(global_path)?;
        if let Some(ws_root) = workspace_root {
            let local_config = ws_root.join(".ce-ai.json");
            if local_config.exists() {
                let local_state = Self::load(&local_config)?;
                state.merge_overrides(local_state);
            }
        }
        Ok(state)
    }

    pub fn merge_overrides(&mut self, overrides: State) {
        for (slot, assignment) in overrides.model_assignments {
            self.model_assignments.insert(slot, assignment);
        }
    }
}
```

### 2. Multi-Harness Uninstall Extension (`src/commands/uninstall.rs` & `src/harness/mod.rs`)
```rust
pub struct UninstallArgs {
    pub harness: Option<String>,
    pub all: bool,
    pub yes: bool,
}

pub trait HarnessAdapter {
    fn uninstall(&self, ctx: &Context, all: bool) -> Result<(), CeError>;
}
```

---

## Data Flow & Control Sequence

```
User CLI Command: ce-ai uninstall --harness all --all --yes
       │
       ▼
Parse UninstallArgs in src/main.rs
       │
       ▼
src/commands/uninstall.rs::run()
       │
       ├─► Probe installed harnesses (or target specific harness)
       │
       ├─► Check --yes flag; if false, prompt interactive confirmation
       │
       ├─► Iterate HarnessKind target adapters
       │     └─► Call adapter.uninstall(ctx, all)
       │           ├─► Restore backup if available
       │           └─► If all=true, delete managed loader scripts & skill manifests
       │
       └─► Update state.json & print confirmation report
```
