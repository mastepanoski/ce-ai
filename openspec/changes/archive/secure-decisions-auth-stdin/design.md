# Technical Design: Secure Stdin & Masked Interactive Input for Decisions Auth

## Architectural Overview
This change enhances credential ingestion in `ce-ai decisions auth` by separating interactive secret acquisition from command-line arguments.

```
                  ┌───────────────────────────────┐
                  │   ce-ai decisions auth ...    │
                  └───────────────┬───────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         │                        │                        │
         ▼                        ▼                        ▼
[--stdin provided]      [--key provided]          [No key flags]
         │                        │                        │
  Read directly from       Has value?               Is terminal (TTY)?
  std::io::stdin()        ┌───────┴───────┐        ┌───────┴───────┐
         │                ▼               ▼        ▼               ▼
         │             Yes:            No:       Yes:             No:
         │         Use argument;     Prompt    Interactive      Display
         │         emit warning     masked     prompt with      current
         │         on stderr         input     cancel option     status
         │                │               │        │               │
         └────────────────┼───────────────┴────────┘               │
                          ▼                                        ▼
                  Save key to 0600                        Output status
              credentials.toml & verify                  (no mutations)
```

## Data Contracts & Struct Changes

### 1. Clap Action Enum (`src/commands/decisions.rs`)
```rust
#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    // ...
    /// Configure or test Decision Provider authentication credentials.
    Auth {
        /// API key value to set (if omitted, prompts securely via stdin or displays current key status).
        #[arg(long, num_args = 0..=1)]
        key: Option<Option<String>>,
        /// Read API key value from standard input.
        #[arg(long)]
        stdin: bool,
        /// Verify provider connectivity with the resolved API key.
        #[arg(long)]
        check: bool,
    },
    // ...
}
```

### 2. Secret Ingestion Utilities (`src/decisions/auth.rs`)

```rust
/// Reads an API key from standard input (non-interactive, e.g. piped or redirected).
pub fn read_api_key_from_stdin() -> Result<String, CeError>;

/// Prompts for an API key interactively using crossterm raw mode without character echo.
/// Returns Ok(None) if the operator cancels or submits an empty input.
pub fn prompt_api_key_interactive(prompt: &str) -> Result<Option<String>, CeError>;
```

### 3. RAII Terminal Raw Mode Guard
```rust
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
```

### 4. Interactive Event Handling Contract
When terminal is in raw mode:
- `KeyCode::Enter`: Finishes entry and returns current buffer.
- `KeyCode::Char(c)`: Appends `c` to current buffer.
- `KeyCode::Backspace`: Pops last character from buffer.
- `KeyCode::Esc` | `KeyModifiers::CONTROL` + `KeyCode::Char('c')`:
  Restores raw mode and returns `Err(CeError::Usage("operation cancelled by operator".into()))`.

### 5. `handle_auth` Resolution Flow
1. If `stdin` is true:
   - Call `read_api_key_from_stdin()`.
   - If empty, return `CeError::Usage("API key provided via stdin cannot be empty")`.
   - Save key.
2. Else if `let Some(key_opt) = key`:
   - If `Some(k) = key_opt`:
     - Print security warning to `eprintln!`:
       `warning: passing API key via command-line arguments exposes it in shell history and process lists; prefer interactive prompt or '--stdin'`.
     - Save key `k`.
   - Else (`key_opt` is `None`, i.e. `--key` passed without argument):
     - If interactive: call `prompt_api_key_interactive`. If non-empty, save key.
     - Else (non-interactive): call `read_api_key_from_stdin` and save key.
3. Else (`key` is `None` and `stdin` is `false`):
   - If interactive (`std::io::stdin().is_terminal()`) and `check` is `false`:
     - Prompt interactively: `Enter TypeSafe/Jev API key (press Enter to cancel): `.
     - If non-empty: save key.
     - If empty: print "No changes made." and display current configuration.
4. If `check` is true:
   - Perform health check against the configured provider.
5. If no key was provided and `check` is false in non-interactive environment:
   - Display current configuration status.
