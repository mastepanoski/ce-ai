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
| 0.1.x   | :white_check_mark: | Active Mainline             |
| < 0.1   | :x:                | Unsupported                 |
