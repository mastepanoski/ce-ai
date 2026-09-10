# Exploration: Architectural Investigation & Tradeoffs for Binary Self-Update

## 1. Running Executable Replacement Mechanics Across OSes

Replacing an actively executing binary is fundamentally different on POSIX systems versus Windows.

### 1.1 POSIX (Linux and macOS)

#### Technical Mechanism:
On Linux and macOS, the filesystem separates directory entries (dentries/file names) from data containers (inodes).
- When a binary is launched, the kernel opens and memory-maps the executable file.
- If a process attempts to open the running file in write mode (`O_WRONLY` or `O_RDWR`), the kernel rejects it with `ETXTBSY` ("Text file busy").
- **However**, `rename(2)` (`std::fs::rename`) does not write to the file. It alters the directory entry in the parent directory.
- Calling `rename("/path/to/.new_ce-ai", "/path/to/ce-ai")` atomically points the name `ce-ai` to the new file's inode.
- The existing running process continues executing from the unlinked inode (whose link count decrements to 0 when the process exits, freeing disk blocks).
- Any subsequent invocation of `ce-ai` loads the new inode immediately.

#### Critical Constraint: Cross-Device Links (`EXDEV`):
If the replacement binary is unpacked in `/tmp` (which is often a `tmpfs` RAM disk or separate partition) and renamed to `~/.ce-ai/bin/ce-ai` (root filesystem), `rename(2)` fails with `EXDEV` ("Invalid cross-device link").
- **Solution**: The temporary replacement file MUST be created in the *same parent directory* as the target binary:
  ```rust
  let parent = current_exe.parent().ok_or(...)?;
  let temp_file = tempfile::Builder::new()
      .prefix(".ce-ai-self-update-")
      .tempfile_in(parent)?;
  ```
- Setting executable permissions (`0o755`) via `std::os::unix::fs::PermissionsExt` before renaming ensures the binary is immediately runnable.

---

### 1.2 Windows (NTFS / ReFS)

#### Technical Mechanism & Win32 Locking:
On Windows, the operating system loader maps the `.exe` into memory using memory-mapped sections (`CreateFileMapping` with `SEC_IMAGE`).
- Attempting to overwrite or delete a running `.exe` fails with `ERROR_ACCESS_DENIED` (error code 5).
- Directly renaming a new file over the running `ce-ai.exe` also fails with `ERROR_ACCESS_DENIED`.

#### Evaluated Approaches for Windows:

| Approach | Mechanics | Tradeoffs & Evaluation |
|---|---|---|
| **Option A: Spawn detached batch / PowerShell script** | Process launches `cmd.exe /c "ping 127.0.0.1 -n 2 >nul & move /y new.exe ce-ai.exe"` and immediately exits. | ❌ **Rejected**: Fragile timing (process may not exit in 2 seconds), creates external shell dependencies, window flickers or fails in restricted environments, difficult to verify or report errors. |
| **Option B: MoveFileEx with `MOVEFILE_DELAY_UNTIL_REBOOT`** | Registers a registry key for Windows to swap files on system restart. | ❌ **Rejected**: Requires administrator privileges, delays update until reboot, terrible developer UX. |
| **Option C: Rename running `.exe` to `.old` + atomic swap + startup cleanup** | Rename `ce-ai.exe` -> `ce-ai.exe.old`, move `ce-ai.new.exe` -> `ce-ai.exe`, clean up `.old` on next run. | ✅ **Selected**: Modern Windows allows renaming open executables within the same directory because the file is opened with `FILE_SHARE_DELETE`. The old path is immediately vacated, allowing the new binary to take its place. Used by `rustup`, Chrome, and Firefox on Windows. |

#### Detailed Windows Swap Flow:
1. Resolve `current_exe` (e.g. `C:\Users\name\.ce-ai\bin\ce-ai.exe`).
2. If `ce-ai.exe.old` exists (from previous run), attempt to delete it (`std::fs::remove_file`). If in use, generate unique suffix `ce-ai.exe.old.<timestamp>`.
3. Extract new binary to sibling temporary file `ce-ai.exe.new` in the same directory.
4. Rename `ce-ai.exe` to `ce-ai.exe.old` (`std::fs::rename`).
5. Rename `ce-ai.exe.new` to `ce-ai.exe` (`std::fs::rename`).
6. Attempt best-effort deletion of `ce-ai.exe.old` (may fail if still held by current process; safely ignored).
7. On startup in `src/main.rs`, run `cleanup_stale_update_files` to silently remove any leftover `*.old` files.

---

## 2. Platform Target Detection

The release pipeline (`.github/workflows/release.yml`) builds and publishes binaries for 6 specific target triples:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`

### Evaluated Options for Target Resolution:

1. **Option 1: Build script (`build.rs`) exporting `env!("TARGET")`**
   - *Pros*: Directly uses cargo compile target.
   - *Cons*: Adds build script compilation overhead to every dev build; requires maintaining build.rs.
2. **Option 2: Compile-time `cfg!` matching**
   - *Pros*: Pure Rust, zero build-time overhead, exact matching to known targets:
     ```rust
     pub fn current_target() -> Option<&'static str> {
         if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
             Some("x86_64-unknown-linux-gnu")
         } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
             Some("aarch64-unknown-linux-gnu")
         } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
             Some("x86_64-apple-darwin")
         } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
             Some("aarch64-apple-darwin")
         } else if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
             Some("x86_64-pc-windows-msvc")
         } else if cfg!(all(target_arch = "aarch64", target_os = "windows")) {
             Some("aarch64-pc-windows-msvc")
         } else {
             None
         }
     }
     ```
   - *Selected*: Option 2 is deterministic, robust, and zero-overhead.

---

## 3. Archive Format & Extraction

### Release Asset Matrix:
- Unix (Linux, macOS): `ce-ai-<target>.tar.gz` containing single executable `ce-ai`.
- Windows: `ce-ai-<target>.zip` containing single executable `ce-ai.exe`.

### Extraction Implementation:
- **Unix (`.tar.gz`)**:
  - `flate2` and `tar` are already in `Cargo.toml`.
  - Reuses the safe path verification pattern established in `src/source/archive.rs` (`is_safe_relative_path`).
- **Windows (`.zip`)**:
  - Adding `zip = { version = "2.2", default-features = false, features = ["deflate"] }` to `Cargo.toml`.
  - Uses `flate2` (already vendored) under the hood with zero external C dependencies.
  - Verifies entry names to prevent Zip-Slip before writing.
  - Works consistently across unit tests and platforms.

---

## 4. Cryptographic SHA256 Integrity Verification

### Requirements & Flow:
1. Every release publishes a `SHA256SUMS.txt` asset containing lines formatted as:
   `<sha256_hex>  <asset_name>`
2. Self-update downloads `SHA256SUMS.txt` alongside the target archive.
3. Parses the expected SHA256 hex string corresponding to the target asset name.
4. Computes SHA256 digest of the downloaded archive bytes (`sha2::Sha256`).
5. Constant-time / exact string comparison.
6. If mismatch occurs:
   - Logs error: `verification error: SHA256 digest mismatch (expected ..., got ...)`.
   - Deletes temporary download files.
   - Exits with `CeError::Verification` (mapped to exit code `6`).

---

## 5. Command Interface & Ergonomics

### Command Structure:
- `ce-ai self-update` is introduced as a top-level subcommand in `src/commands/registry.rs`.
- `ce-ai upgrade --bin` is added as an optional flag to `src/commands/upgrade.rs`. When passed, `upgrade` updates both the plugin and runs the binary self-update sequence.
- Status and doctor commands display actionable suggestions:
  - If a newer version is known, suggest: `Run 'ce-ai self-update' to upgrade the CLI binary.`
