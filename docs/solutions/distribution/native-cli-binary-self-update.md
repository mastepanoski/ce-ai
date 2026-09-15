---
title: "Native CLI Binary Self-Update & Cross-Platform In-Use Executable Replacement"
category: "distribution"
date: "2026-09-09"
tags:
  - self-update
  - distribution
  - cross-platform
  - integrity-verification
  - windows-file-locking
components:
  - commands
  - source
  - binary_release
applies_when: "Implementing in-tool self-updating or in-use executable binary replacement across POSIX and Windows"
problem_type: distribution
---

# Native CLI Binary Self-Update & Cross-Platform In-Use Executable Replacement

## Context

`ce-ai upgrade` previously only updated the compound-engineering plugin across registered harnesses (`~/.config/opencode/compound-engineering/`). Users running outdated CLI binaries had to manually re-pipe `install.sh` or `install.ps1`. Issue #341 introduced native CLI binary self-update (`ce-ai self-update` and `ce-ai upgrade --bin`).

Replacing an actively executing binary presents fundamental operating system divergence:
- **POSIX (Linux/macOS)**: Replaces directory entries pointing to new inodes; running processes keep their open file descriptor to the unlinked inode until termination. Sibling temporary files prevent cross-device (`EXDEV`) link failures.
- **Windows (NTFS)**: Directly overwriting or deleting an executing PE binary returns `ERROR_ACCESS_DENIED`. However, Windows NTFS allows *renaming* an open executable because files are opened with `FILE_SHARE_DELETE`.

---

## Architectural Patterns

### 1. In-Use Executable Replacement

#### POSIX Pattern:
1. Identify parent directory of `std::env::current_exe()`.
2. Write new binary to a temporary file in the *same parent directory* (preventing `EXDEV` across mount points).
3. Set permissions to `0o755` (`chmod +x`).
4. Atomically rename temporary file over the running binary path (`std::fs::rename`).

#### Windows Pattern (NTFS Open File Swap):
1. Clean up any previous `ce-ai.exe.old` from prior runs if unlocked.
2. Extract replacement binary to sibling temporary file `ce-ai.exe.new`.
3. Rename running `ce-ai.exe` -> `ce-ai.exe.old`.
4. Rename `ce-ai.exe.new` -> `ce-ai.exe`.
5. Best-effort delete of `ce-ai.exe.old` (if still held by current process, ignore).
6. Silent startup hook in `src/main.rs`: runs `cleanup_stale_update_files` to remove leftover `*.old` files on subsequent invocations.

### 2. Cryptographic Integrity & Fail-Closed Semantics
- Published releases include `SHA256SUMS.txt`.
- The archive is downloaded, and its SHA256 digest is computed and compared against `SHA256SUMS.txt` *prior* to extraction.
- Any mismatch immediately purges temp files and returns `CeError::Verification` (exit code `6`).

### 3. Safe Archive Extraction (Zip-Slip Defense)
- Validates all entries in `.tar.gz` and `.zip` archives.
- Entries containing `..`, root `/`, or Windows drive prefixes (`C:`) are rejected before writing any byte.
