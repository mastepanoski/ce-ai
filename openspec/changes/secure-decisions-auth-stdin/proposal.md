# Proposal: Secure Stdin & Masked Interactive Input for Decisions Auth

## Problem Statement
When operators configure credentials for the Decision Engine via `ce-ai decisions auth --key <KEY>`, the raw API key is provided as a command-line argument. This introduces serious credential leakage vectors:
1. **Shell History Pollution**: The secret is logged in cleartext in `~/.bash_history`, `~/.zsh_history`, or shell history databases.
2. **Process Table Inspection**: Any local unprivileged process can observe the full argument list and raw API key in cleartext via `ps aux`, `/proc`, or activity monitoring tools.
3. **CI/Shell Log Leakage**: Execution trace logs or shell snapshots may capture terminal commands including the `--key` parameter.

To adhere to ISO/IEC 27001/27002 cryptographic and secret hygiene standards, `ce-ai decisions auth` must support:
- Secure, non-echoing (masked) interactive terminal input for `ce-ai decisions auth` and `ce-ai decisions auth --key`.
- Direct stdin streaming via `--stdin` (`cat key.txt | ce-ai decisions auth --stdin` or `echo "$KEY" | ce-ai decisions auth --stdin`) for automated pipelines and password managers.
- Clear security deprecation warning when `--key <VALUE>` is passed on the command line.

## In-Scope
1. **Interactive Masked Stdin Ingestion**:
   - In interactive terminals (`std::io::stdin().is_terminal()`), running `ce-ai decisions auth` (without `--check`) or `ce-ai decisions auth --key` prompts the operator securely without echoing characters to screen.
   - Operators can press Enter without input to cancel or inspect current configuration.
2. **Standard Input Flag (`--stdin`)**:
   - Support `ce-ai decisions auth --stdin` to read the key directly from standard input until EOF or newline.
   - Support empty/whitespace validation with explicit exit codes (`CeError::Usage`).
3. **CLI Argument Deprecation Warning**:
   - Retain `ce-ai decisions auth --key <VALUE>` for backwards compatibility, but emit a stderr security warning advising operators of process table and shell history risks.
4. **Documentation & Help Strings**:
   - Update `ce-ai decisions auth --help`, command guidance in `src/commands/decisions.rs`, and related documentation.
5. **Quality Gates & Release**:
   - 100% test coverage including unit tests and CLI integration tests with mocked/piped stdin.
   - Pass `cargo clippy`, `cargo fmt`, `cargo test`, `make e2e`.
   - Bump SemVer minor (`v1.64.0`) in `Cargo.toml` and update `CHANGELOG.md`.

## Out-of-Scope
1. Changing the underlying credential storage location (`~/.config/ce-ai/credentials.toml`) or permissions (`0600`).
2. Storing keys in OS keychains (macOS Keychain, Linux Secret Service).
3. Modifying Jev API client protocol or HTTP request headers.

## Risk Evaluation & Mitigation
- **Risk (Terminal Raw Mode Deadlock / Crash)**: A panic or unexpected abort while terminal is in raw mode could leave the operator's terminal in an unreadable state.
  - *Mitigation*: Use RAII drop guard for raw mode management and catch `Ctrl+C` / `Escape` to gracefully restore terminal state before exiting.
- **Risk (CI/Pipeline Non-Interactive Stall)**: Running `ce-ai decisions auth` in non-interactive environments (where stdin is not a TTY and neither `--key` nor `--stdin` is set) could hang waiting for input.
  - *Mitigation*: Check `std::io::stdin().is_terminal()`. In non-interactive environments, omit prompting and default to status display unless `--stdin` is explicitly passed.
