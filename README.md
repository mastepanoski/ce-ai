# `ce-ai` — Compound Engineering CLI Orchestrator & Workflow FSM Engine

`ce-ai` is a fast, safe Rust CLI for orchestrating the **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** across 12 AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, and `custom`).

Inspired by the **`gentle-ai`** ecosystem, `ce-ai` governs the **7-stage Compound Engineering Flywheel** via a deterministic Finite State Machine (FSM) engine, providing workspace scope isolation, companion tools management (Engram, CodeGraph, Context7, RTK), atomic POSIX disk writes, and zero-context-loss progress checkpointing.

> [!NOTE]
> `ce-ai` orchestrates distributions of the open-source **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** — a suite of specialized skills, roles, and workflow guidelines for AI coding assistants.

## Features

- **Multi-Harness Native Support**: Supports 12 AI coding harnesses with native config mergers (`opencode`, `claude`, `pi`), Markdown block delimiters (`cursor`, `copilot`), and generic JSON structures (`codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- **Auto-Detection (`--all`)**: Auto-detects installed harnesses on the host system and installs/syncs across all active environments with a single command.
- **Workspace & Global Scope Isolation (`--scope workspace|global`)**: Install skills and configurations isolated strictly to a local Git repository root (`.opencode/`, `.claude/`, `state.json`) or globally across user profiles.
- **Workflow FSM Engine & Checkpoint Recovery (`ce-ai workflow`)**: Governs the 7-stage development cycle (Ideation ➔ OpenSpec ➔ Plan ➔ Work/TDD ➔ Verify ➔ Compound ➔ Ship) with atomic disk savegames (`checkpoint`) and cross-session state re-hydration (`resume`).
- **Companion Tools Manager (`ce-ai tools`)**: Inspect and install essential sidecars and companion tools (**MCP Servers**: Engram persistent memory, CodeGraph blast-radius, Context7 docs; **CLI Token Reducers**: RTK output filter).
- **Multi-Harness Model Sync**: Assign models per agent slot (`ce-brainstorm`, `ce-plan`, `ce-work`) and automatically sync assignments across all installed harness configurations simultaneously.
- **Methodology Flexibility**: Full support for Test-Driven Development (TDD), Code-First + Post-Verification, Behavior-Driven Development (BDD), and R&D Spikes (`ce-ideate`).
- **Safe Extraction & Caching**: Validates tarball structures against path traversal attacks (zip-slip prevention) and maintains SHA256-verified release caches.
- **Model Assignments & Profiles**: Take append-only snapshot profiles (`ce-ai models profile save/load`) and restore role assignments cleanly.
- **Diff & Reconcile (Sync)**: Inspect drift via SHA256 manifest indexing, preview planned updates with `--dry-run`, and repair modified or deleted managed assets.
- **Automatic Backups & Clean Uninstallation**: Pre-mutation configuration is backed up automatically (`~/.ce-ai/backups/`) and `ce-ai uninstall` restores original pre-install configurations cleanly.
- **Health Doctor**: Diagnose configuration errors, hash drift, and state inconsistency (`ce-ai doctor`).

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

### 🚀 Universal One-Line Installer (macOS & Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash
```

### 💻 Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex
```

### 🍺 Homebrew (macOS & Linux)

```bash
brew tap mastepanoski/ce-ai https://github.com/mastepanoski/ce-ai
brew install ce-ai
```

Or install directly using the formula specification:

```bash
brew install Formula/ce-ai.rb
```

### 📦 Build from Source (Cargo)

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

## 📚 User Documentation Directory & Sitemap

`ce-ai` includes comprehensive technical documentation for developers, architects, and newcomers:

| Guide Document | Focus Area | Key Concepts Covered |
| :--- | :--- | :--- |
| 🚀 **[Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md)** | **Beginner & Developer Workflows** | • Greenfield Projects (Scratch Setup)<br>• Building New Features & Enhancements<br>• Fixing Bugs via `ce-debug` (Direct Entry)<br>• Research & Documentation Fast-Tracks<br>• SDD Migration (`gentle-ai` / OpenSpec)<br>• Cross-Session Resumption & Multi-Harness Handoffs<br>• Git Worktree Isolation (`ce-worktree`) |
| 🎓 **[Harnesses, Loops & Context Masterclass](docs/user-guide/harnesses-loops-and-context-masterclass.md)** | **AI Concepts for Beginners** | • What is an AI Harness? (Claude, Cursor, AGY, OpenCode)<br>• MCP Sidecars (Engram/CodeGraph) vs. Token Reducers (RTK)<br>• Autonomous Agent Loops (Read-Evaluate-Act-Reflect)<br>• TDD Feedback Loops & Diagnostic Sub-Loops<br>• Context Compaction, Decay & Token Economics |
| 🏛️ **[Architectural & Conceptual Guide](docs/user-guide/architectural-and-conceptual-guide.md)** | **Systems Engineering & Architecture** | • Multi-Harness Trait Architecture (`HarnessAdapter`)<br>• Scope Isolation Hierarchy (`--scope workspace`)<br>• MCP Sidecars (Engram/CodeGraph) vs. CLI Token Reducers (RTK)<br>• Workflow FSM & Compounding Knowledge Flywheel<br>• POSIX Atomic Write Guarantees (`write_atomic`) |
| 🎮 **[FSM & Checkpoints Masterclass](docs/user-guide/fsm-and-checkpoints-explained.md)** | **FSM Engine & State Persistence** | • 7-Stage Workflow Cycle & Skill Alignment<br>• Savegame Concept & Context Compaction<br>• Sub-Loops (`ce-ideate`, `ce-debug`, `ce-simplify-code`)<br>• FSM Capability Matrix (All Supported Variants) |
| 🔧 **[Installation & Coexistence Guide](docs/user-guide/installation-and-coexistence-mechanisms.md)** | **Installation & Configuration** | • Global vs Workspace Scope Isolation<br>• Non-Destructive User JSON Merging<br>• Manifest SHA256 Indexing & Multi-Harness Discovery |
| 🔄 **[Sync & Upgrade Mechanisms](docs/user-guide/sync-and-upgrade-mechanisms.md)** | **Maintenance & Upgrades** | • Manifest Drift Calculation (Copy, Restore, Remove)<br>• GitHub Release API & Local Source Protection<br>• Atomic Rollbacks & Backup Snapshots |

---

## 🚀 Starting a Project from Scratch (Greenfield Setup)

Starting a new project with `ce-ai` and **Compound Engineering** is straightforward:

```bash
# 1. Create project directory and initialize git
mkdir my-new-project && cd my-new-project
git init

# 2. Install ce-ai scoped to this workspace
ce-ai install --scope workspace

# 3. Define product strategy & architectural vision
# In your AI harness (Claude Code, Cursor, Antigravity, OpenCode):
/ce-strategy

# 4. Create initial OpenSpec scaffold (openspec/changes/001-initial-scaffold/)
# Define proposal.md, spec.md, tasks.md for initial build targets

# 5. Build boilerplate & test pipeline
/ce-plan
/ce-work

# 6. Document initial architecture concepts
/ce-compound

# 7. Ship first commit & PR
/ce-commit-push-pr
```

> 💡 *For detailed step-by-step instructions on greenfield setups, bug fixes, multi-harness handoffs, and worktree workflows, read the **[Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md)**.*

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

> 📖 **Detailed Step-by-Step Guide**: Read the **[Sync & Upgrade User Guide](docs/user-guide/sync-and-upgrade-mechanisms.md)** for a full explanation of how synchronization, SHA256 file hash checks, and GitHub release upgrades work internally.

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

## 🌟 Acknowledgments & Ecosystem Inspiration

`ce-ai` and Compound Engineering take deep inspiration from pioneering open-source work in the **`gentle-ai`** ecosystem:

- **[`gentle-ai`](https://github.com/Gentleman-Programming)**: Foundational ecosystem inspiration for Spec-Driven Development (OpenSpec) and agentic workflows.
- **[`Engram`](https://github.com/Gentleman-Programming/engram)**: Persistent cross-session memory sidecar powered by SQLite + FTS5 full-text search.
- **[`CodeGraph`](https://github.com/colbymchenry/codegraph)**: Codebase structural intelligence sidecar (symbol call-graphs, blast-radius calculation).
- **[`Context7`](https://github.com/upstash/context7)**: Up-to-date documentation retrieval engine for modern frameworks and libraries.
- **[`RTK / Rust Token Killer`](https://github.com/rtk-ai/rtk)**: Terminal output stream filter compressing raw stdio by 60%–90%.
- **[`Sequential Thinking`](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)**: Structured reasoning protocol for step-by-step hypothesis evaluation and reflection.

---

## License

Distributed under the [MIT License](./LICENSE).

