# Specification: Requirements & Acceptance Criteria for Binary Self-Update

## 1. Requirements

### 1.1 Target Triple Resolution
- **R1.1**: The system MUST detect the host architecture and operating system and map it to one of the 6 supported release targets:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
  - `aarch64-pc-windows-msvc`
- **R1.2**: On unsupported host targets, the system MUST return `CeError::Usage` (exit code `2`) with an informative error detailing manual build/install instructions.

### 1.2 Release & Tag Resolution
- **R2.1**: When invoked without `--to`, the system MUST query GitHub releases for `mastepanoski/ce-ai` and identify the latest tagged release.
- **R2.2**: The resolution MUST support unauthenticated web redirects and Atom feed fallbacks to prevent failures from API rate limits (HTTP 403).
- **R2.3**: Resolution MUST NEVER fall back to mutable branches such as `main`.
- **R2.4**: When `--to <tag>` is provided, the system MUST resolve and validate the exact requested tag.

### 1.3 Check-Only Mode (`--check`)
- **R3.1**: WHEN `ce-ai self-update --check` is executed, THEN the system MUST compare the resolved remote version against `env!("CARGO_PKG_VERSION")`.
- **R3.2**: If the remote version is strictly newer, the system MUST print that an update is available and exit with code `0`.
- **R3.3**: If the current version is equal or newer, the system MUST print that `ce-ai` is up to date and exit with code `0`.
- **R3.4**: `--check` MUST NOT download release archives or alter files on disk.

### 1.4 Cryptographic SHA256 Verification & Fail-Closed Semantics
- **R4.1**: The system MUST fetch `SHA256SUMS.txt` for the resolved release.
- **R4.2**: The system MUST calculate the SHA256 hex digest of the downloaded archive bytes.
- **R4.3**: If the calculated digest does not match the digest recorded in `SHA256SUMS.txt`, the system MUST immediately abort, purge temporary files, and exit with `CeError::Verification` (exit code `6`).
- **R4.4**: Under no circumstances shall an unverified or corrupted binary replace the current executable.

### 1.5 Safe Archive Extraction & Zip-Slip Defense
- **R5.1**: Archive extraction MUST validate entry paths before writing any byte to disk.
- **R5.2**: Any archive entry containing `..`, absolute root paths (`/` or `\`), or Windows drive prefixes (`C:`) MUST trigger immediate rejection with `CeError::Runtime`.
- **R5.3**: For Unix targets (`.tar.gz`), extraction MUST extract the `ce-ai` binary.
- **R5.4**: For Windows targets (`.zip`), extraction MUST extract the `ce-ai.exe` binary.

### 1.6 Running Executable Replacement
- **R6.1**: The target executable path MUST be resolved via `std::env::current_exe()`.
- **R6.2 (POSIX)**: On Linux and macOS, the system MUST write the new binary to a temporary file in the *same directory* as `current_exe`, set `0o755` permissions, and invoke `std::fs::rename` to replace the running executable atomically.
- **R6.3 (Windows)**: On Windows, the system MUST rename the running `ce-ai.exe` to `ce-ai.exe.old` in the same directory, rename the replacement binary to `ce-ai.exe`, and attempt deletion of `ce-ai.exe.old`.
- **R6.4**: If the parent directory is not writable (e.g. managed by a system package manager), the system MUST return `CeError::Io` (exit code `4`) with actionable advice.

### 1.7 Dry-Run Mode
- **R7.1**: WHEN `--dry-run` is active, THEN the system MUST resolve the release, download and verify checksums in a sandbox/temp directory, and print what would be replaced without mutating `current_exe`.

### 1.8 Post-Update Health Recommendation
- **R8.1**: Upon successful replacement, the system MUST print the version transition (e.g. `Updated ce-ai from v1.49.0 to v1.50.0`) and recommend running `ce-ai doctor`.

---

## 2. Acceptance Criteria (WHEN / THEN)

1. **AC-1 (Up-To-Date Check)**:
   - `WHEN` running `ce-ai self-update` and the installed version matches the latest release and `--force` is false,
   - `THEN` the command prints that `ce-ai` is already up to date and exits with code `0`.

2. **AC-2 (Check Flag)**:
   - `WHEN` running `ce-ai self-update --check`,
   - `THEN` the command outputs the current version and latest available version without downloading the binary or modifying disk, exiting with code `0`.

3. **AC-3 (Integrity Check Failure)**:
   - `WHEN` the SHA256 checksum of the downloaded archive differs from `SHA256SUMS.txt`,
   - `THEN` the update aborts, deletes temp files, and exits with code `6` (`CeError::Verification`).

4. **AC-4 (Unsupported Architecture)**:
   - `WHEN` running on an architecture not present in the 6 matrix targets,
   - `THEN` the command returns exit code `2` (`CeError::Usage`).

5. **AC-5 (Atomic Replacement & Permission)**:
   - `WHEN` a valid update is applied on Linux/macOS,
   - `THEN` the replacement uses a sibling temp file with atomic rename and `0o755` permissions.

6. **AC-6 (Windows Open File Handling)**:
   - `WHEN` running on Windows,
   - `THEN` the executing binary is renamed to `.old` before the new executable is placed at `ce-ai.exe`.

7. **AC-7 (`ce-ai upgrade --bin`)**:
   - `WHEN` running `ce-ai upgrade --bin`,
   - `THEN` the command executes plugin upgrade followed by binary self-update.
