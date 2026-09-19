# Exploration: Secure Stdin & Masked Input for Decisions Auth

## Architectural Context
The Decision Engine (`ce-ai decisions`) connects to TypeSafe / Jev providers for probabilistic routing, risk evaluation, and readiness advisories. Authentication credentials are saved in `~/.config/ce-ai/credentials.toml` with `0600` permissions.
Currently, `ce-ai decisions auth` accepts credentials only via the `--key <VALUE>` CLI argument.

## Technical Options & Tradeoffs

### 1. Terminal Masking Mechanism: `crossterm` vs `rpassword`
- **Option A: Add `rpassword` dependency**:
  - *Pros*: Industry standard single-purpose crate for reading passwords without echo.
  - *Cons*: Adds another third-party crate dependency, requires cargo audit and potential supply-chain review.
- **Option B: Utilize existing `crossterm` dependency (Selected)**:
  - *Pros*: `crossterm = "0.28"` is already an approved, core dependency of `ce-ai` (used by Ratatui TUI). It natively provides `enable_raw_mode()` and event polling cross-platform (macOS, Linux, Windows).
  - *Cons*: Requires implementing a minimal terminal event loop (~25 LOC) with an RAII raw-mode cleanup guard.
  - *Decision*: Option B. Avoids dependency bloat, preserves deterministic build artifacts, and ensures uniform behavior across all targets.

### 2. CLI Argument Schema & Clap Representation
- **Option A: Replace `--key` completely with `--stdin`**:
  - *Cons*: Breaking change for existing scripts and documentation.
- **Option B: Retain `--key` with `num_args = 0..=1` and add `--stdin` (Selected)**:
  - *Design*:
    ```rust
    /// API key value to set (if omitted, prompts securely via stdin or displays status).
    #[arg(long, num_args = 0..=1)]
    key: Option<Option<String>>,
    /// Read API key value from standard input.
    #[arg(long)]
    stdin: bool,
    /// Verify provider connectivity with the resolved API key.
    #[arg(long)]
    check: bool,
    ```
  - *Behavior*:
    - `ce-ai decisions auth --key <VAL>`: sets key, emits security warning on stderr.
    - `ce-ai decisions auth --key`: prompts securely via masked stdin.
    - `ce-ai decisions auth --stdin`: reads from stdin stream (no interactive prompt).
    - `ce-ai decisions auth` (interactive TTY): prompts securely; if Enter is pressed with blank input, keeps existing key and displays status.
    - `ce-ai decisions auth` (non-interactive pipe, no flags): displays status without hanging.

### 3. Visual Feedback during Interactive Input
- **Option A: Silent typing (Unix `sudo` / `ssh` style)**:
  - Characters are consumed silently with no visual echo.
  - *Pros*: Maximum visual shoulder-surfing privacy.
  - *Cons*: Operators occasionally wonder if terminal input is frozen.
- **Option B: Silent typing with clear prompt instruction (Selected)**:
  - Prompt: `Enter TypeSafe/Jev API key (input hidden, press Enter to cancel): `
  - Pressing `Backspace` cleanly updates buffer.
  - Pressing `Ctrl+C` or `Escape` aborts with `CeError::Usage("operation cancelled")`.

### 4. Cross-Platform & Non-Interactive Safety
- When `!std::io::stdin().is_terminal()`, raw mode must NOT be enabled (it would fail on pipes).
- In piped contexts (`cat secret | ce-ai decisions auth --stdin`), standard buffered line reading `std::io::stdin().read_line(&mut buf)` is used.
- Trailing newline `\r\n` or `\n` is automatically stripped.
