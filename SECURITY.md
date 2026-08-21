# Security Policy & Information Security Management

`ce-ai` takes security seriously. As a tool responsible for installing and managing plugins, agent prompt policies, and configuration files across AI harnesses, `ce-ai` enforces strict compliance with international security standards:

- **ISO/IEC 27001:2022** — Information Security Management System (ISMS)
- **ISO/IEC 27002:2022** — Code of Practice for Information Security Controls
- **NIST SP 800-53 (Rev. 5)** — Security and Privacy Controls for Information Systems
- **NIST Cybersecurity Framework (CSF 2.0)** — Identify, Protect, Detect, Respond, Recover

---

## 🔒 Security Architecture & Controls

### 1. Cryptographic File Integrity & SHA256 Manifests (ISO 27002 Control 8.9)
Every installed plugin asset (JS loaders, skill markdown definitions, configurations) is indexed into an immutable manifest stored at `manifest.json`.
- Each file entry includes a cryptographic `SHA256` digest.
- `ce-ai sync` and `ce-ai doctor` re-hash managed files on disk and flag any unexpected modification, deletion, or tampering before execution.

### 2. Atomic File Operations & System Restoration (NIST SP 800-53 CP-9, CP-10)
- **Atomic Writes**: State updates (`state.json`) and configuration edits (`opencode.json`) write to temporary files first (`.tmp-XYZ`) and execute atomic renames (`std::fs::rename`) to guarantee zero partial-write corruption.
- **Automated Pre-Install Backups**: Before modifying any harness configuration, `ce-ai` creates a timestamped backup in `~/.ce-ai/backups/`.
- **Zero-Downtime Rollback**: `ce-ai uninstall` restores the original pre-install harness configuration cleanly without leaving dangling artifacts.

### 3. Supply Chain Security & Dependency Auditing (ISO 27002 Control 5.21)
- **Lockfile Pinning**: `Cargo.lock` is committed to version control to pin exact dependency versions.
- **Cargo Audit**: Continuous integration pipelines execute `cargo-audit` to detect known CVE vulnerabilities in third-party crates.
- **Rust TLS**: HTTPS requests (`reqwest`) enforce `rustls-tls` to avoid native OpenSSL dynamic linking vulnerabilities.

### 4. Zero Telemetry & Data Confidentiality (ISO 27001 Clause 7.5)
- `ce-ai` collects **zero analytics, zero telemetry, and zero remote tracking**.
- All state, model profiles, and backups remain 100% local on the user's workstation (`~/.ce-ai` and `~/.config/opencode`).

---

## 🎓 Masterclass: Why We Adopted These Security Guardrails (Teacher's Guide)

To understand why `ce-ai` implements these specific controls, let's explore the core architectural threats and the exact analogies behind each guardrail:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        🛡️ THE 5 PILLARS OF SECURITY IN CE-AI                           │
├──────────────────────┬──────────────────────┬───────────────────┬──────────────────────┤
│ 1. Path Traversal    │ 2. Atomic Writes     │ 3. #![forbid]     │ 4. Cryptographic     │
│    Sanitizer         │    Scratchpad        │    Memory Safety  │    SHA256 Seals      │
│ (No Trojan Archives) │ (Zero Partial State) │ (No Raw Pointers) │ (Drift Auto-Repair)  │
└──────────────────────┴──────────────────────┴───────────────────┴──────────────────────┘
```

### 1. The Border Guard: Why `safe_extract` Rejects Path Traversal (`../` & `/etc/passwd`)
- **The Threat**: When extracting a remote `.tar.gz` archive, an attacker could craft malicious entries like `../../../../usr/bin/malicious` or `/etc/passwd`. Standard extractors extract files as they read them, which means by the time a malicious path is encountered, half the archive is already written to your disk.
- **The Teacher Analogy**: Imagine a customs officer at an international airport. Instead of letting passengers enter the terminal and checking their passports later, the officer inspects the manifest of the entire plane *at the gate*. If even one passenger lacks a valid visa, the entire plane is turned back before anyone steps onto the tarmac.
- **Our Guarantee**: `safe_extract` scans 100% of archive headers *in memory* before performing a single disk write.

### 2. The Ink Scratchpad: Why `write_atomic` Uses Temporary Files & Renames
- **The Threat**: If a computer loses power, crashes, or runs out of disk space while writing to `state.json` or `opencode.json`, the file gets cut in half, corrupting the user's configurations and active agent model profiles.
- **The Teacher Analogy**: Imagine writing a legal deed with permanent ink directly on the original document. If your pen leaks halfway through, the deed is ruined forever. Instead, you draft on a separate scratchpad first (`.state.json.tmp-12345`). Once every letter is complete and verified, you swap the scratchpad for the final document in one instantaneous, un-interruptible motion (`std::fs::rename`).
- **Our Guarantee**: File states in `ce-ai` are mathematically binary: they are either 100% up-to-date or unchanged. Partial corruption is impossible.

### 3. The Structural Steel Lock: Why Compiler-Enforced `#![forbid(unsafe_code)]`
- **The Threat**: In Rust, `unsafe` blocks allow developers to bypass memory safety checks (e.g. raw pointer arithmetic or manual memory allocation). Over time, un-reviewed `unsafe` code can introduce buffer overflows, use-after-free bugs, or segmentation faults.
- **The Teacher Analogy**: Rust gives you structural steel beams for building a skyscraper safely. Using `unsafe` is like building temporary wooden scaffolding to shortcut a corner. By declaring `#![forbid(unsafe_code)]` at the root of `src/lib.rs` and `src/main.rs`, we physically lock the door to wooden scaffolding: the compiler itself will refuse to compile the binary if any `unsafe` block is introduced anywhere in the codebase.
- **Our Guarantee**: 100% compiler-verified memory safety across the entire application codebase.

### 4. The Self-Contained Vault: Why `rustls-tls` Replaces System OpenSSL
- **The Threat**: Dynamically linking to the host system's OpenSSL dynamic libraries (`libssl.so` or `libssl.dylib`) introduces vulnerability drift. If the user's OS has an unpatched OpenSSL CVE, `ce-ai` becomes vulnerable by association.
- **The Teacher Analogy**: Instead of relying on a shared hotel room lock whose keys might be duplicated by the front desk, you carry your own portable, modern vault key. `rustls-tls` is written in pure, memory-safe Rust and compiled directly into the static binary, eliminating C-library memory corruption vulnerabilities and dynamic library mismatch crashes.

### 5. Digital Tamper Seals: Why Cryptographic SHA256 Manifests Exist
- **The Threat**: Local files, agent skills, or plugin loaders can be accidentally deleted, edited by another tool, or tampered with by malware.
- **The Teacher Analogy**: Think of a tamper-evident seal placed on high-security cargo containers. Every time `ce-ai doctor` or `ce-ai sync` runs, it re-calculates the SHA256 fingerprint of every managed asset and compares it against `install-manifest.json`. If a seal is broken, `ce-ai` flags the exact modified file and automatically repairs it from the pristine cache.

---

## 🛡️ Vulnerability Disclosure Protocol

If you discover a security vulnerability, flaw, or unexpected behavior in `ce-ai`, please report it responsibly:

1. **Report via GitHub Private Security Advisory**: Submit a report directly via [GitHub Security Advisories](https://github.com/mastepanoski/ce-ai/security/advisories/new) (preferred).
2. **Report via GitHub Security Issue**: Alternatively, open a GitHub Issue using our dedicated [Security Report Template](.github/ISSUE_TEMPLATE/security_report.yml).
3. Include:
   - Detailed description of the vulnerability (e.g. path traversal, unsafe extraction, state corruption).
   - Steps to reproduce or proof-of-concept payload.
   - Impact evaluation across operating systems (Linux, macOS, Windows).

### Response Timelines
- **Initial Acknowledgement**: Within 24 hours.
- **Assessment & Triage**: Within 3 business days.
- **Patch & Fix Release**: Critical fixes issued within 7 business days.

---

## 📋 Supported Versions

Security updates are actively applied to the following versions:

| Version | Supported          | Security Maintenance Status |
| ------- | ------------------ | --------------------------- |
| 0.9.x   | :white_check_mark: | Active Mainline (v0.9.0)    |
| 0.8.x   | :white_check_mark: | Supported Maintenance       |
| < 0.8   | :x:                | Deprecated                  |
