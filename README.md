# `ce-ai` — Compound Engineering Plugin Manager CLI

`ce-ai` is a fast, safe Rust CLI for installing, syncing, upgrading, and managing model assignments for the **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** across 12 AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, and `custom`).

> [!NOTE]
> `ce-ai` manages distributions of the open-source **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** — a suite of specialized skills, roles, and workflows for AI coding assistants.

## Features

- **Multi-Harness Native Support**: Supports 12 AI coding harnesses with native config mergers (`opencode`, `claude`, `pi`), Markdown block delimiters (`cursor`, `copilot`), and generic JSON structures (`codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- **Auto-Detection (`--all`)**: Auto-detects installed harnesses on the host system and installs/syncs across all active environments with a single command.
- **Multi-Harness Model Sync**: Assign models per agent slot (`ce-brainstorm`, `ce-plan`, etc.) and automatically sync assignments across all installed harness configurations simultaneously.
- **Safe Extraction & Caching**: Validates tarball structures against path traversal attacks (zip-slip prevention) and maintains SHA256-verified release caches.
- **Model Assignments & Profiles**: Take append-only snapshot profiles and restore role assignments cleanly.
- **Diff & Reconcile (Sync)**: Inspect drift, preview planned updates with `--dry-run`, and repair modified or deleted managed assets.
- **Automatic Backups & Clean Uninstallation**: Pre-mutation configuration is backed up automatically (`~/.ce-ai/backups/`) and `ce-ai uninstall` restores original pre-install configurations cleanly.
- **Health Doctor**: Diagnose configuration errors, drift, and state inconsistency.

## Supported Harness Matrix

| Harness Identifier | Config File / Location | Strategy |
| :--- | :--- | :--- |
| `opencode` | `~/.config/opencode/opencode.json` | JSON Array Merger (`plugin` & `skills`) |
| `claude` | `~/.claude.json` / `~/.config/claude/` | JSON Config Merger |
| `pi` | `~/.pi/config.json` / `~/.pi/skills/` | JSON Merger & Native Skill Directory Copy |
| `cursor` | `.cursorrules` / `.cursor/rules/` | Markdown Rule Block Ingestion (`<!-- CE-AI MANAGED BLOCK -->`) |
| `copilot` | `.github/copilot-instructions.md` | Markdown Instruction Block Ingestion |
| `codex` | `~/.codex/config.json` | Generic JSON Config Merger |
| `grok` | `~/.grok/config.json` | Generic JSON Config Merger |
| `kimi` | `~/.kimi/config.json` | Generic JSON Config Merger |
| `agy` | `~/.gemini/antigravity-cli/config.json` | Generic JSON Merger & Skill Copy |
| `deepseek` | `~/.deepseek/config.json` | Generic JSON Config Merger |
| `fx` | `~/.fx/config.json` | Generic JSON Config Merger |
| `custom` | Specified via CLI flags | Fallback Custom Config Mode |

## Installation

Build from source with Cargo:

```bash
cargo build --release
```

Or install binary directly:

```bash
cargo install --path .
```

## Usage

### 1. Install Plugin

Install into a specific harness (e.g. Claude Code, Cursor, or OpenCode):

```bash
ce-ai install --harness claude
ce-ai install --harness cursor
ce-ai install --harness opencode
```

Install into **all auto-detected host harnesses**:

```bash
ce-ai install --all
```

Install from a local source repository or directory:

```bash
ce-ai install --harness claude --source /path/to/compound-engineering-plugin
```

Preview changes without modifying disk:

```bash
ce-ai install --all --dry-run
```

> 📖 **Guía Detallada Paso a Paso**: Lee el documento **[Installation & Coexistence Guide](docs/user-guide/installation-and-coexistence-mechanisms.md)** para entender el mecanismo exacto de instalación, respaldos automáticos y convivencia segura con instalaciones oficiales (Claude Code, Cursor, OpenCode, etc.).

### 2. View Status & Check Health

Check installed harness versions and status:

```bash
ce-ai status
```

Run health checks:

```bash
ce-ai doctor
```

### 3. Reconcile Drift & Upgrade

Reconcile local changes against source manifest:

```bash
ce-ai sync
```

Upgrade plugin to a specific version tag:

```bash
ce-ai upgrade --to compound-engineering-v3.5.0
```

> 📖 **Guía Detallada Paso a Paso**: Lee el documento **[Sync & Upgrade User Guide](docs/user-guide/sync-and-upgrade-mechanisms.md)** para una explicación completa de cómo funcionan internamente la sincronización, la comprobación de hashes SHA256 y las actualizaciones desde GitHub.

### 4. Configure Models & Profiles

Assign a model to a specific slot:

```bash
ce-ai models set ce-brainstorm opencode-go/kimi-k2.6
```

List current model assignments:

```bash
ce-ai models list
```

Save or load named model profiles:

```bash
ce-ai models profile save my-profile
ce-ai models profile load my-profile
```

### 5. Complete Uninstallation Guide

To completely remove `ce-ai` and restore your system:

#### Step 1: Restore Harness Config & Remove Plugin
Restore original harness configurations and remove managed plugin files across installed harnesses:

```bash
ce-ai uninstall --harness claude
ce-ai uninstall --harness opencode
```

#### Step 2: Uninstall `ce-ai` Binary
If you installed `ce-ai` globally via Cargo:

```bash
cargo uninstall ce-ai
```

#### Step 3: Clean Local State & Backups (Optional)
To purge all cached release tarballs, model profile snapshots, and backups:

```bash
rm -rf ~/.ce-ai
```

---

## Backup, Restore & Uninstallation Architecture

`ce-ai` is built with a zero-data-loss guarantee for host harness configurations across all supported harnesses.

### 1. Automatic Pre-Mutation Backups
- Before any file write or configuration update during `install` or `models set`, `ce-ai` checks if pre-existing harness configs (e.g. `opencode.json`, `.claude.json`, `.cursorrules`) exist.
- If present, `ce-ai` creates a timestamped backup copy inside `~/.ce-ai/backups/<utc-timestamp>/`.
- The backup path is registered in the managed manifest (`install-manifest.json`).

### 2. Atomic Uninstallation (`uninstall`)
Running `ce-ai uninstall --harness <name>`:
1. **Restores Original Config**: Locates the latest backup in `~/.ce-ai/backups/` and atomically restores the target config to its exact pre-install content.
2. **Removes Created Files**: If configuration files or Markdown blocks were created by `ce-ai install`, they are cleanly deleted or stripped rather than left behind.
3. **Purges Managed Directory**: Deletes `<harness-config>/compound-engineering/` containing loaders, installed skills, and `install-manifest.json`.
4. **Cleans State**: Updates `~/.ce-ai/state.json` to reflect that the harness is uninstalled.

### 3. Install Manifest (`install-manifest.json`)
Every managed harness directory contains `install-manifest.json` recording:
- Installed plugin version and source (release tag or local path)
- Per-file SHA256 hashes for drift detection
- Config mutation log linking to the exact pre-install backup path

### 4. Model Profile Snapshots
- When saving model assignment profiles (`ce-ai models profile save <name>`), an append-only snapshot is written under `~/.ce-ai/profiles/versions/<name>-<timestamp>.json`.
- Loading a profile (`ce-ai models profile load <name>`) restores previous model assignments while preserving historical snapshots.

---

## Installation Scope: Global (User-Wide) vs Isolated

### Global (User-Wide) by Default
By default, `ce-ai` installs plugins and skills at the **user/harness level** (`~/.config/opencode/opencode.json` and `~/.config/opencode/compound-engineering/`).

- **Universal Availability**: Any project opened in OpenCode automatically gains access to all 200+ Compound Engineering skills and agent model configurations without per-repository setup.
- **Centralized Maintenance**: A single `ce-ai upgrade` or `ce-ai sync` updates the plugin globally across all sessions.

### Custom / Environment Overrides
To isolate configuration for testing or specific directory trees, you can override paths:

- **`CE_AI_OPENCODE_CONFIG`**: Environment variable pointing to a custom OpenCode config directory (e.g., `CE_AI_OPENCODE_CONFIG=/custom/path/opencode ce-ai install --harness opencode`).
- **`--config-dir <DIR>`**: Global CLI flag specifying a custom directory for `ce-ai` internal state, profiles, and backups (defaults to `~/.ce-ai`).

---

## Documentation & OpenSpec Specifications

For complete technical specifications, architecture decisions, and requirement matrices, see the OpenSpec documentation:

- [**Design Architecture (`design.md`)**](openspec/changes/ce-ai/design.md): System architecture, data flow, interfaces, and threat matrix.
- [**Specification Requirements (`spec.md`)**](openspec/changes/ce-ai/spec.md): OpenSpec user requirements, acceptance criteria (OI-1..OI-5, SU-1..SU-5, MM-1..MM-4, CC-1..CC-3, DG-1..DG-3).
- [**Proposal & Scope (`proposal.md`)**](openspec/changes/ce-ai/proposal.md): Project proposal, goals, non-goals, and open items.
- [**Exploration Analysis (`exploration.md`)**](openspec/changes/ce-ai/exploration.md): Harness exploration and direct file write decision rationales.
- [**Implementation Roadmap (`tasks.md`)**](openspec/changes/ce-ai/tasks.md): Complete TDD task list across all 8 development phases.

---

## 🛡️ Security, AI Governance & Standards Compliance

`ce-ai` adheres strictly to international cybersecurity, AI management, and operational frameworks:

| Document | Description & Framework Compliance |
| -------- | ---------------------------------- |
| [**`SECURITY.md`**](./SECURITY.md) | Information Security Management System (**ISO/IEC 27001**, **ISO/IEC 27002**, **NIST CSF 2.0**, **NIST SP 800-53**). Cryptographic checksums, supply chain auditing, and vulnerability disclosure protocol. |
| [**`AI_POLICY.md`**](./AI_POLICY.md) | Artificial Intelligence Management System (**ISO/IEC 42001**) & **NIST AI Risk Management Framework (AI RMF 1.0)**. Govern, Map, Measure, and Manage AI agent integrations. |
| [**`AGENTS.md`**](./AGENTS.md) | AI Agent Guidelines, structural boundaries, and mandatory constraints for autonomous LLM coding assistants. |
| [**`CONTRIBUTING.md`**](./CONTRIBUTING.md) | Development workflow, code review criteria, and security compliance guidelines for contributors. |
| [**`CODE_OF_CONDUCT.md`**](./CODE_OF_CONDUCT.md) | Contributor Covenant Code of Conduct (v2.1). |
| [**`CONTRIBUTORS.md`**](./CONTRIBUTORS.md) | Project maintainers and community contributors. |
| [**`LICENSE`**](./LICENSE) | Official **MIT License**. |

---

## 🧪 CI/CD & Automated Quality Gates

`ce-ai` uses GitHub Actions ([`.github/workflows/ci.yml`](./.github/workflows/ci.yml)) to run automated compliance gates on every push and pull request:
- **Build & Test Matrix**: Validates compilation, formatting (`cargo fmt`), clippy (`cargo clippy -D warnings`), and test suite across Linux & macOS.
- **Supply Chain Audit**: Scans dependencies with `cargo audit` (ISO 27001 / ISO 27002).
- **Docker E2E Gate**: Runs `make e2e` in an isolated Linux container (`Dockerfile.e2e`) (NIST AI RMF & ISO 42001).

---

## Testing

Run unit and CLI integration tests locally:

```bash
cargo test
```

Run Docker containerized E2E gate test:

```bash
make e2e
```

## License

Distributed under the [MIT License](./LICENSE).

