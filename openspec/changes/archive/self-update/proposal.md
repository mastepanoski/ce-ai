# Proposal: Native CLI Binary Self-Update (`ce-ai self-update`)

## 1. Problem Statement

`ce-ai upgrade` (and the TUI Upgrade Release flow) exclusively upgrades the compound-engineering plugin across registered harnesses (`~/.config/opencode/compound-engineering/`, etc.). There is currently no in-tool command to update the `ce-ai` CLI binary itself.

On Linux and headless systems, users running an outdated `ce-ai` binary must manually re-pipe a remote shell script:
```bash
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash
```
Similarly on Windows:
```powershell
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex
```

While `ce-ai status` or version update recommendations can detect when a newer release is published (e.g. via GitHub API), acting on it requires manual out-of-band intervention. This is prone to user error, inconvenient in headless/CI environments, unverifiable mid-flight, and inconsistent with the polished in-tool upgrade UX provided for the plugin.

This proposal implements **Issue #341**: a native, in-tool self-update capability allowing `ce-ai` to query `mastepanoski/ce-ai` for new releases, verify cryptographic SHA256 integrity, unpack the platform asset safely, and atomically replace the running executable across Linux, macOS, and Windows.

---

## 2. In-Scope / Out-of-Scope Boundaries

### In-Scope:
1. **Dedicated CLI Subcommand `ce-ai self-update`**:
   - `--check`: Query and report whether a new CLI release is available without applying it.
   - `--to <tag>`: Explicit target release tag (e.g. `v1.50.0`), bypassing latest release resolution.
   - `--force`: Force reinstallation/replacement even if the current version matches or is newer.
   - `--dry-run`: Global flag (`-d` / `--dry-run`) previewing planned download and swap without mutating disk.
2. **Flag in `ce-ai upgrade --bin`**:
   - Allows users running `ce-ai upgrade --bin` to upgrade both the plugin and the CLI binary in a single command.
3. **Platform Target Detection**:
   - Maps current OS and architecture at compile-time/runtime to the 6 official release targets:
     - `x86_64-unknown-linux-gnu`
     - `aarch64-unknown-linux-gnu`
     - `x86_64-apple-darwin`
     - `aarch64-apple-darwin`
     - `x86_64-pc-windows-msvc`
     - `aarch64-pc-windows-msvc`
   - Rejects unsupported platforms with clear error messaging before any download.
4. **Provenance & Release Resolution**:
   - Queries `mastepanoski/ce-ai` latest release using pinned tag resolution (never mutable branches like `main`).
   - Includes zero-friction unauthenticated web fallback (`/releases/latest` redirect & Atom feed) matching `src/source/release.rs` to withstand GitHub API rate limits.
5. **Cryptographic SHA256 Integrity Verification (Fail-Closed)**:
   - Fetches and parses official `SHA256SUMS.txt` published alongside release assets.
   - Calculates the SHA256 digest of the downloaded archive before extraction.
   - On checksum mismatch: immediately aborts, purges temp files, and exits with `CeError::Verification` (exit code `6`).
6. **Safe Archive Extraction (Zip-Slip Defense)**:
   - For `.tar.gz` (Unix): Validates all entries via safe relative path checks before writing; extracts binary `ce-ai`.
   - For `.zip` (Windows): Validates zip entries against path traversal; extracts binary `ce-ai.exe`.
7. **OS-Aware Executable Replacement Engine**:
   - Uses `std::env::current_exe()` (canonicalized) to target the active binary.
   - **POSIX (Linux/macOS)**: Writes temporary file in the same parent directory, applies `0o755` permissions, and executes atomic `std::fs::rename` (safe inode swap while running).
   - **Windows (NTFS)**: Renames locked running `ce-ai.exe` to `ce-ai.exe.old`, atomically renames replacement to `ce-ai.exe`, attempts immediate delete, and registers silent startup cleanup in `main.rs` for any remaining `.old` files.
8. **Permissions & Managed Package Warning**:
   - Detects if `current_exe` resides in a package-manager managed location (e.g. Homebrew Cellar) or is unwritable, advising the appropriate command (e.g. `brew upgrade ce-ai`).

### Out-of-Scope:
- Auto-updating in the background as a daemon or cron task (self-update is user- or agent-triggered).
- Supporting platforms outside the 6 official matrix targets.
- Maintaining a custom Homebrew formula (distribution is owned by `mastepanoski/homebrew-ce-ai`).
- Overriding user configurations or state during update.

---

## 3. Risk Evaluation & Mitigations

- **Process In-Use File Locking on Windows**:
  - *Risk*: Windows denies write access (`ERROR_ACCESS_DENIED`) when overwriting a running executable.
  - *Mitigation*: Windows NTFS allows *renaming* an open executable. Rename `ce-ai.exe` -> `ce-ai.exe.old`, place new `ce-ai.exe`, and clean up `.old` on startup.
- **Cross-Mount / Cross-Device Move Failure (`EXDEV`)**:
  - *Risk*: Creating temp files in `/tmp` (e.g. `tmpfs`) and renaming to `~/.ce-ai/bin` (root disk) fails across filesystem boundaries.
  - *Mitigation*: Temporary files are always created in the *same parent directory* as `current_exe`.
- **Zip-Slip & Path Traversal in Downloaded Archives**:
  - *Risk*: Malicious or corrupted archive contains `../../bin/` paths overwriting arbitrary system files.
  - *Mitigation*: Reject all entries containing `..`, root `/`, or Windows drive letters before writing any byte.
- **Corrupted / Partial Download Replacement**:
  - *Risk*: Network drop yields truncated binary; overwriting leaves user with a broken tool.
  - *Mitigation*: Checksum verification against `SHA256SUMS.txt` occurs *prior* to extraction and replacement. If hash fails, exit code 6 is returned and disk is untouched.
- **Half-Installed State on Interruption**:
  - *Risk*: Ctrl+C during file swap leaves no executable.
  - *Mitigation*: Atomic rename is instantaneous (a single directory entry update in the kernel).

---

## 4. Success Criteria

1. `ce-ai self-update --check` accurately identifies whether an update is available without disk mutations.
2. `ce-ai self-update` successfully updates the binary, displaying old and new versions, and recommending `ce-ai doctor`.
3. Checksum mismatch triggers immediate fail-closed abort with exit code `6` (`CeError::Verification`).
4. Cross-platform replacement works safely on POSIX (atomic rename) and Windows (rename `.old` + swap).
5. 100% test pass on unit tests and integration tests (`cargo test`).
6. Zero clippy warnings with `-D warnings` and compliant `cargo fmt`.
