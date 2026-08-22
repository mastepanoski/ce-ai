# `ce-ai` — Compound Engineering CLI Orchestrator & Workflow FSM Engine

`ce-ai` is a fast, safe Rust CLI for orchestrating the **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** across 12 AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, and `custom`). It governs the **7-stage Compound Engineering Flywheel** via a deterministic FSM engine with workspace scope isolation, atomic POSIX disk writes, and zero-context-loss checkpointing.

> [!NOTE]
> `ce-ai` orchestrates distributions of the open-source **[Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering)** — a suite of specialized skills, roles, and workflow guidelines for AI coding assistants.

## Quick Path

**1. Install the binary** (pick one):

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex

# Homebrew (macOS & Linux)
brew tap mastepanoski/ce-ai https://github.com/mastepanoski/ce-ai && brew install ce-ai

# From source
cargo install --path .
```

**2. Preview the plugin installation** (modifies nothing):

```bash
ce-ai install --all --dry-run
```

**3. Verify system health:**

```bash
ce-ai doctor
```

That's it — your harnesses are ready for Compound Engineering. Run your first 7-stage cycle with the [Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md).

## Common Operations

| Command | Purpose | Full guide |
| :--- | :--- | :--- |
| `ce-ai install [--scope workspace\|--scope global]` | Install per harness or workspace-isolated | [Installation & Coexistence](docs/user-guide/installation-and-coexistence-mechanisms.md) |
| `ce-ai status` / `ce-ai doctor` | Inspect installed harnesses and health | — |
| `ce-ai sync` | Reconcile drift against the SHA256 manifest | [Sync & Upgrade Mechanisms](docs/user-guide/sync-and-upgrade-mechanisms.md) |
| `ce-ai upgrade --to <tag>` | Upgrade the plugin to a release tag | [Sync & Upgrade Mechanisms](docs/user-guide/sync-and-upgrade-mechanisms.md) |
| `ce-ai models set/list/profile …` | Assign models per agent slot, snapshot profiles | [Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md) |
| `ce-ai uninstall --harness <name>` | Restore pre-install configuration cleanly | [Backup & Uninstall](docs/user-guide/backup-and-uninstall.md) |
| `ce-ai install --all --dry-run` | Preview any mutation before it touches disk | [Installation & Coexistence](docs/user-guide/installation-and-coexistence-mechanisms.md) |

## Documentation Map

| Document | Audience | Intent |
| :--- | :--- | :--- |
| 🚀 [Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md) | **Beginner** | Tutorial — greenfield setup, first feature, bug fix, resumption |
| 🎓 [Harnesses, Loops & Context Masterclass](docs/user-guide/harnesses-loops-and-context-masterclass.md) | **Beginner** | Explanation — what a harness is, MCP sidecars, token economics |
| 🔧 [Installation & Coexistence](docs/user-guide/installation-and-coexistence-mechanisms.md) | Both | How-to — scopes, non-destructive JSON merging, discovery |
| 🔄 [Sync & Upgrade Mechanisms](docs/user-guide/sync-and-upgrade-mechanisms.md) | Both | How-to — drift repair, upgrades, rollbacks |
| 💾 [Backup & Uninstall](docs/user-guide/backup-and-uninstall.md) | Both | How-to — backups, clean uninstall, state cleanup |
| 🗂️ [Harness Matrix](docs/user-guide/harness-matrix.md) | Reference | Reference — all 12 harnesses, config paths, merge strategies |
| 🏛️ [Architectural & Conceptual Guide](docs/user-guide/architectural-and-conceptual-guide.md) | **Senior** | Explanation — adapter traits, scope hierarchy, atomic writes |
| 🎮 [FSM & Checkpoints Masterclass](docs/user-guide/fsm-and-checkpoints-explained.md) | **Senior** | Explanation — 7-stage cycle, savegames, sub-loops |
| 📐 [OpenSpec specifications](openspec/changes/ce-ai/) | **Senior** | Reference — design, requirements matrix, proposal, roadmap |

## Security, Governance & Quality Gates

- **Security policy**: [`SECURITY.md`](./SECURITY.md) — ISO/IEC 27001/27002, NIST CSF 2.0, vulnerability disclosure.
- **AI governance**: [`AI_POLICY.md`](./AI_POLICY.md) — ISO/IEC 42001, NIST AI RMF 1.0.
- **Contributing**: [`CONTRIBUTING.md`](./CONTRIBUTING.md) · Code of Conduct: [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
- **CI**: GitHub Actions runs build + tests, `clippy -D warnings`, `cargo audit`, and a Docker E2E gate on every PR.
- **Local verification**: `cargo test` (unit/integration) · `make e2e` (containerized E2E).

## Acknowledgments

`ce-ai` takes deep inspiration from the open-source ecosystem: [`gentle-ai`](https://github.com/Gentleman-Programming), [`Engram`](https://github.com/Gentleman-Programming/engram), [`CodeGraph`](https://github.com/colbymchenry/codegraph), [`Context7`](https://github.com/upstash/context7), [`RTK`](https://github.com/rtk-ai/rtk), and [`Sequential Thinking`](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking).

## License

Distributed under the [MIT License](./LICENSE).
