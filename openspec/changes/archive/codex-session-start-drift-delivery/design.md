# Technical Design: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI

## 1. System Architecture

```
┌────────────────────────────────────────────────────────┐
│               OpenAI Codex CLI Session                 │
└──────────────────────────┬─────────────────────────────┘
                           │ 1. Parses .codex/config.toml
                           │    Triggers [[hooks.SessionStart]]
                           ▼
┌────────────────────────────────────────────────────────┐
│               ce-ai workflow resume                    │
│           (Generates live RepoState)                   │
└──────────────────────────┬─────────────────────────────┘
                           │ 2. Emits formatted text on stdout
                           ▼
┌────────────────────────────────────────────────────────┐
│             Codex Developer Context Injection          │
│       (Injected before model generates response)       │
└────────────────────────────────────────────────────────┘
```

## 2. Interface Definitions in `src/harness/codex.rs`

```rust
pub const CODEX_RESUME_COMMAND: &str = "ce-ai workflow resume";

pub fn has_session_start_hook(config_path: &Path) -> bool;
pub fn ensure_session_start_hook(config_path: &Path) -> Result<bool, CeError>;
pub fn remove_session_start_hook(config_path: &Path) -> Result<bool, CeError>;
```

### Hook Schema in `.codex/config.toml`:
```toml
[[hooks.SessionStart]]
matcher = "startup|resume|compact"

[[hooks.SessionStart.hooks]]
type = "command"
command = "ce-ai workflow resume"
statusMessage = "Loading ce-ai workflow state"
```

## 3. Integration Wiring Points

1. `init_prj.rs`:
   When `.codex/` exists:
   Ensure `.codex/config.toml` has the `SessionStart` hook via `crate::harness::codex::ensure_session_start_hook`.
2. `deinit_prj.rs`:
   Clean up `.codex/config.toml` via `crate::harness::codex::remove_session_start_hook`.
3. `doctor.rs`:
   Audit adopted projects with `.codex` to ensure `config.toml` contains `SessionStart`.
