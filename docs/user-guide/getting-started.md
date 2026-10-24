<!-- Diátaxis Quadrant: Tutorial | Audience: Absolute beginner -->
# 🌱 Getting Started: Your First 10 Minutes with `ce-ai`

This guide assumes **zero prior knowledge**. If you've installed `ce-ai`, run `doctor`, and still don't know what to actually *do*, you're in the right place.

---

## 1. Two different things share this project's name

Before touching a terminal, get this distinction straight — it is the #1 source of confusion:

| | What it is | Where you use it |
| :--- | :--- | :--- |
| **`ce-ai`** | A command-line program (a Rust binary) | Your **terminal** (bash/zsh/PowerShell) |
| **Compound Engineering** | A methodology — a set of skills like `/ce-brainstorm`, `/ce-plan`, `/ce-work` | Your **AI chat window** (Claude Code, Cursor, OpenCode, …) |

`ce-ai` does not replace your coding agent or independently generate code. It installs and adopts the Compound Engineering workflow, then keeps project-level workflow state, artifacts, and health checks visible across sessions. You still drive the coding agent by typing `/ce-...` commands *inside your AI tool's chat*, not in the terminal.

> ⚠️ The single most common newbie mistake: typing `/ce-brainstorm` into the terminal. It will fail — it isn't a shell command. It only exists inside your AI harness's chat.

---

## 2. Prerequisites & Companion Tools

Before adopting the workflow, make sure you have:
1. **One supported AI harness** installed (e.g. Claude Code, Cursor, OpenCode, Codex, Pi, GitHub Copilot CLI). See the [Harness Matrix](harness-matrix.md).
2. **Git** and a project repository.
3. **Companion Developer Tools (Memory, Structural Graph & Token Compressor)**:
   - **CodeGraph**: AST symbol, call-graph, and blast-radius indexer (`codegraph_explore`). Gives coding agents structural repository maps, symbol hierarchies, callers, and callees in a single query rather than guessing with blind `grep`/`find`. Initialized per project via `codegraph init`.
   - **Engram**: SQLite + FTS5 persistent memory server (`mem_save`, `mem_context`, `mem_search`). Persists architecture decisions, bug fixes, patterns, and user preferences across agent sessions and context window compactions so the agent never starts blind.
   - **RTK (Rust Token Killer)**: Smart terminal pre-processor and token reducer. Intercepts verbose CLI outputs (`cargo test`, `git status`, `docker ps`, linter errors) and shrinks stdio token volume by 60%–90% before sending text to the LLM. Automatically configured via session hooks during project adoption (`ce-ai init-prj`).

---

## 3. Installation by Operating System & Package Manager

Install `ce-ai` and the companion tools using the standard package manager or installer for your platform:

### 🐧 Linux (Debian, Ubuntu, Fedora, Arch, Alpine, etc.)

Linux developers can choose between direct standalone scripts, language package managers (`cargo`, `go`), or Homebrew (Linuxbrew):

```bash
# 1. Install ce-ai (official multi-platform shell installer)
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash

# 2. Install CodeGraph (self-contained Node.js runtime, zero dependencies required)
curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh

# 3. Install Engram (persistent memory)
# Option A: via Go
go install github.com/Gentleman-Programming/engram/cmd/engram@latest
# Option B: via direct binary release (x86_64 / aarch64 musl)
curl -fsSL https://github.com/Gentleman-Programming/engram/releases/latest/download/engram-linux-amd64.tar.gz | tar -xz -C ~/.local/bin

# 4. Install RTK (terminal token compressor)
# Option A: via Cargo
cargo install rtk
# Option B: via standalone installer script
curl -fsSL https://www.rtk-ai.app/install.sh | sh

# Alternative for Linuxbrew users:
# brew install mastepanoski/ce-ai/ce-ai gentleman-programming/tap/engram rtk
```

---

### 🪟 Windows (PowerShell, Winget, Go & Cargo)

On Windows, use native PowerShell one-liners, `cargo`, and `go`:

```powershell
# 1. Install ce-ai (official PowerShell installer)
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex

# 2. Install CodeGraph (official PowerShell installer)
irm https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.ps1 | iex

# 3. Install Engram (persistent memory)
# Option A: via Go (recommended)
go install github.com/Gentleman-Programming/engram/cmd/engram@latest
# Option B: download engram.exe from GitHub Releases and place in your PATH:
# https://github.com/Gentleman-Programming/engram/releases

# 4. Install RTK (terminal token compressor)
cargo install rtk
```

---

### 🍏 macOS (Standalone Scripts, Go/Cargo, or Homebrew)

Choose between standalone scripts (zero package managers required) or Homebrew:

#### Option A: Without Homebrew (Standalone Scripts, Go & Cargo)

```bash
# 1. Install ce-ai (official shell installer)
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash

# 2. Install CodeGraph (standalone script)
curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh

# 3. Install Engram (via Go or GitHub release binary)
go install github.com/Gentleman-Programming/engram/cmd/engram@latest

# 4. Install RTK (via Cargo or install script)
cargo install rtk
```

#### Option B: With Homebrew

```bash
# Install ce-ai, Engram, and RTK
brew install mastepanoski/ce-ai/ce-ai gentleman-programming/tap/engram rtk

# Install CodeGraph
curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh
```

---

### 📊 Package Manager & Tool Matrix Summary

| Platform / Tool | `ce-ai` | `codegraph` | `engram` | `rtk` |
| :--- | :--- | :--- | :--- | :--- |
| **Linux (Standalone)** | `curl .../install.sh \| bash` | `curl .../install.sh \| sh` | `go install` or release tarball | `cargo install rtk` or curl script |
| **Linux (Linuxbrew)** | `brew install .../ce-ai` | `curl .../install.sh \| sh` | `brew install .../engram` | `brew install rtk` |
| **Windows** | `irm .../install.ps1 \| iex` | `irm .../install.ps1 \| iex` | `go install` or `engram.exe` | `cargo install rtk` |
| **macOS (Standalone)** | `curl .../install.sh \| bash` | `curl .../install.sh \| sh` | `go install` or release tarball | `cargo install rtk` |
| **macOS (Homebrew)** | `brew install .../ce-ai` | `curl .../install.sh \| sh` | `brew install .../engram` | `brew install rtk` |

---

## 4. Go to your project, index the codebase, and adopt the repository

```bash
cd my-project   # an existing repo, or: mkdir my-project && cd my-project && git init

# 1. Initialize CodeGraph index for structural codebase navigation
codegraph init

# 2. Adopt this repository: configure rules, session hooks, and RTK token compression
ce-ai init-prj
```

This step:
- **Builds the `.codegraph/` index**: enables coding agents to query symbols, callers, and callees via the `codegraph_explore` MCP tool instead of broad filesystem searches.
- **Adopts the project (`ce-ai init-prj`)**: writes non-destructive Compound Engineering rules into `AGENTS.md` (and `CLAUDE.md`), configures session-start hooks so agents automatically recover git status and active tasks, and wires RTK terminal output compression. Nothing here clobbers user settings — see the [Project Adoption Guide](project-adoption-guide.md).

---

## 5. Govern and align your AI agent fleet

```bash
# Pin to the latest upstream Compound Engineering release (or an explicit tag like v3.30.5)
ce-ai fleet pin latest

# Synchronize all installed harnesses to the pinned version
ce-ai fleet sync
```

Instead of extracting raw tarballs or scraping files, **`ce-ai fleet` acts as a cross-host version governance engine**:
- It inspects each installed coding agent harness on your machine (`OpenCode`, `Claude Code`, `Pi`, `Codex`, `Cursor`).
- It drives each harness's **native package manager or configuration system** (`opencode.json` plugin arrays, Claude Code plugin updater, `~/.pi/agent/settings.json`, `.codex/config.json`, `.cursor/settings.json`).
- You can check multi-harness parity at any time with `ce-ai fleet status`.

---

## 6. Verify everything is wired up

```bash
ce-ai doctor
ce-ai tools status
ce-ai fleet status
```

A healthy first run confirms your harnesses, project adoption, companion tools, and fleet version parity:

```text
== [Companion Tools, Memory Sidecars & Token Reducers Status] ==
  ✅ codegraph    [MCP Server & CLI] (CodeGraph Codebase Indexer) : v1.4.1 (ok)
  ✅ engram       [MCP Server      ] (Engram Persistent Memory Server) : v2.2.1 (ok)
  ✅ rtk          [CLI Pre-Processor] (RTK CLI Token Reduction Engine) : v0.50.0 (ok)

== [Harness Health] ==
  claude: installed, skills registered, session-start hook: ok
  opencode: installed, plugins registered: ok

== [Fleet Version Governance] ==
  Pinned Version: v3.30.5
  Fleet Aligned:  yes

== [Project Adoption] ==
  AGENTS.md: adopted (tier: full, SHA256 verified)
```

If a tool, harness, or adoption needs attention, `ce-ai doctor` and `ce-ai tools status` provide exact copy-paste remediation commands, or you can run `ce-ai doctor --fix` for automated repair. See [Installation & Coexistence](installation-and-coexistence-mechanisms.md).

---

## 7. Run your first Compound Engineering skill

This is the step that happens **inside your AI harness's chat, not the terminal**. Open Claude Code / Cursor / OpenCode / whichever harness you installed into, **in this same project folder**, and type:

```
/ce-brainstorm add a /health endpoint that returns {"status": "ok"}
```

The agent will ask a couple of clarifying questions and then write a requirements document to `docs/brainstorms/`. That's Stage 1 of the 7-stage cycle — you just started your first Compound Engineering task.

---

## 8. What just happened — the mental model

```
codegraph init        → builds structural AST index for callers, callees, and symbol refs
ce-ai init-prj       → adopts the repo: injects AGENTS.md rules, session hooks, and RTK
ce-ai fleet pin/sync → aligns Compound Engineering version across all your AI harnesses
ce-ai doctor         → confirms project adoption, fleet alignment, and companion readiness
/ce-brainstorm       → typed inside your AI tool's chat to begin real engineering work
```

Everything after this point — turning that brainstorm into a spec, a plan, working code, a reviewed PR, and reusable learning — is the full 7-stage cycle. For the agent/plugin/harness distinction, see [CE-AI positioning](ce-ai-positioning.md). Continue with the [Quick Start Workflow Guide](quick-start-workflow-guide.md), which picks up exactly here.

---

## 9. How to de-adopt or uninstall cleanly

If you ever need to roll back or remove the setup:
- **De-adopt this project**: run `ce-ai deinit-prj` in your project folder. This surgically removes the managed `AGENTS.md` block and session hooks without touching your code.
- **Uninstall from all coding agent harnesses**: run `ce-ai uninstall --harness all --yes` to restore pre-mutation configuration backups.
- For complete binary and companion tool removal, see the [Backup & Uninstall Guide](backup-and-uninstall.md).

---

## Troubleshooting

| Symptom | Likely cause | Fix |
| :--- | :--- | :--- |
| `/ce-brainstorm` doesn't autocomplete or errors in chat | Plugin/skills were not synchronized for *this* harness, or the harness needs a restart | Run `ce-ai fleet sync`, then restart/reload your AI harness |
| `ce-ai fleet status` shows a harness as divergent | An individual harness is lagging behind the pinned version | Run `ce-ai fleet sync` (use `--dry-run` to preview actions) |
| `ce-ai doctor` shows a harness as missing | `ce-ai` couldn't find that harness's config on disk | Confirm the harness is actually installed and has been run at least once; see `ce-ai status` |
| `install`/`upgrade` fails with `403 Forbidden` | Unauthenticated GitHub API rate limit | Set `CE_AI_GITHUB_TOKEN` / `GITHUB_TOKEN`, or run `gh auth login` — see the [README](../../README.md) |
| You typed a `/ce-...` command into the terminal by mistake | See Section 1 above | Open your AI harness's chat window instead |

---

*Related reading:*
- [Quick Start Workflow Guide](quick-start-workflow-guide.md) — the full 7-stage cycle: features, bug fixes, resuming work.
- [Compound Workflow Explained](compound-engineering-workflow-explained.md) — how `/ce-strategy`, `/ce-ideate`, `/ce-brainstorm`, Living Architecture, and `/ce-work` fit together without duplicating effort.
- [Harnesses, Loops & Context Masterclass](harnesses-loops-and-context-masterclass.md) — what a "harness" actually is, for readers still fuzzy on that term.
