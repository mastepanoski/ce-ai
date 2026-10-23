# Project Adoption Engine Guide: Non-Destructive Multi-Harness Governance

> **Intent**: How-to — Safely adopt or de-adopt projects across real-world development scenarios. For architectural details, see the [Architecture Guide](architectural-and-conceptual-guide.md).

---

## 🎓 Why Does the Project Adoption Engine Exist?

### The Core Adoption Dilemma

Imagine you are a senior developer or technical team lead introducing Compound Engineering AI directives into an existing software repository. You want your AI coding assistants (such as Claude Code, OpenCode, Cursor, or Copilot) to strictly follow the **7-stage development cycle**, enforce **OpenSpec specifications**, and run **TDD verification gates** before making pull requests.

However, your repository already has content! It might contain pre-existing developer documentation in `AGENTS.md`, custom coding style guides, or specialized project setup notes.

This creates a fundamental conflict:

1. **If your adoption tool blindly overwrites files**, it destroys pre-existing user documentation, team guidelines, or custom prompt engineering.
2. **If your adoption tool refuses to touch pre-existing files**, developers must manually copy and paste multi-page instruction blocks, leading to human error, stale rules, and drift.
3. **If your adoption tool duplicates rules into 12 separate harness files** (`.cursorrules`, `CLAUDE.md`, `copilot-instructions.md`), maintaining rules becomes an administrative nightmare.

### The Solution: Non-Destructive, Reversible Adoption

The **Project Adoption Engine** (`ce-ai init-prj` and `ce-ai deinit-prj`) solves this dilemma through **surgical marker injection**:

- **Non-Destructive**: Injects managed instruction blocks inside HTML comment markers (`<!-- ce-ai:block begin ... -->`) without modifying or deleting pre-existing user notes.
- **GFM Compatible & Visually Clean**: HTML comment delimiters render invisibly in GitHub Flavored Markdown (GFM) web views and markdown previewers while remaining 100% visible to AI LLM parsers.
- **Derived Reference Stubs**: Canonical governance rules live exclusively in `AGENTS.md`. Sub-harnesses (such as Claude Code) receive lightweight derived stubs (`CLAUDE.md`) referencing `@AGENTS.md` via native import directives.
- **100% Reversible**: Running `ce-ai deinit-prj` surgically removes only the managed block. If user content pre-existed, it is restored byte-for-byte (including preserving CRLF vs LF line endings). If `ce-ai` created the file from scratch and it becomes empty, the file is automatically removed.
- **Zero Schema Breakage**: Registry state in `~/.config/ce-ai/state.json` uses `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, ensuring total backward compatibility with older `ce-ai` binaries.

---

## 🛠️ How It Works Under the Hood

When you adopt a project via `ce-ai init-prj`, the engine performs the following atomic sequence:

```
[Target Project Directory]
       │
       ├── 1. Read existing AGENTS.md (detect CRLF / LF line endings)
       ├── 2. Render adoption tier template (Full, Minimal, or Orchestrator)
       ├── 3. Compute SHA256 cryptographic digest of block body
       ├── 4. Inject block surrounded by <!-- ce-ai:block begin ... --> markers
       ├── 5. Atomically write AGENTS.md (via write_atomic temporary file rename)
       ├── 6. Create derived CLAUDE.md stub (@AGENTS.md) if missing
       └── 7. Register adoption record atomically in state.json
```

### Managed Block Format

Inside `AGENTS.md`, the injected block takes this exact structure:

```markdown
# Pre-Existing Developer Notes (Preserved Unchanged)
Custom project guidelines written by the developer stay here.

<!-- ce-ai:block begin v=3 tier=full sha256=a1b2c3d4e5f6... -->
# AGENTS.md — AI Agent Operating Directives

## 🛡️ Governance & Compliance Standards
All AI agent operations MUST follow ISO/IEC 27001/27002, ISO/IEC 42001, and NIST AI RMF 1.0.

## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement
[Stage 1: Ideation] ➔ [Stage 2: OpenSpec Definition] ➔ [Stage 3: Execution Plan]
➔ [Stage 4: TDD & Implementation] ➔ [Stage 5: Verification] ➔ [Stage 6: Knowledge Capture]
➔ [Stage 7: Git Shipping]
<!-- ce-ai:block end -->

Developer notes appended after the block also stay untouched.
```

---

## 💻 Command Reference & Usage

### Adopting a Project (`ce-ai init-prj`)

To adopt the current working directory or a target project path:

```bash
# Adopt current directory with Full governance tier (default)
ce-ai init-prj

# Adopt a specific project path with Minimal tier
ce-ai init-prj /path/to/my-project --tier minimal

# Preview changes without modifying disk
ce-ai init-prj --dry-run

# Re-apply or upgrade block on an already adopted project
ce-ai init-prj --force

# Upgrade or re-adopt all registered projects across your machine
ce-ai init-prj --all
```

#### CLI Options & Flags

| Flag | Description |
| :--- | :--- |
| `[PATH]` | Target project directory path (default: current working directory). |
| `--tier <tier>` | Adoption tier: `full` (default), `minimal`, or `orchestrator`. |
| `--all` | Batch upgrade or re-adopt all registered projects in `state.json`. |
| `--dry-run` | Preview planned file modifications without writing to disk. |
| `--force` | Force overwrite of modified managed blocks when drift is detected. |
| `--skip-rtk` | Skip auto-configuring RTK (Run-Time Kit) hook injection. |
| `--skip-companions` | Skip configuring all companion tools (both MCP servers and hooks). |

#### Available Adoption Tiers

| Tier | Purpose | Recommended for |
| :--- | :--- | :--- |
| `full` **(Default)** | Complete 7-stage development cycle, OpenSpec requirements, Single Source of Truth rule (ideation artifacts are retained as raw history — "disposable" never means deleting them), DoD checklist, and security policies. | Production repositories, core services, critical products. |
| `minimal` | Core DoD guidelines, atomic write rules, and basic testing verification. | Small libraries, scripts, internal tools. |
| `orchestrator` | Multi-repo orchestration directives and delegation protocols, plus a directive to distill ideation outputs into specs rather than maintaining them in parallel — and retain them as raw history instead of deleting them. | Monorepos, parent workspace folders. |

#### Companion Tool Integration (CodeGraph, Engram, RTK)

During `ce-ai init-prj`, `ce-ai` automatically configures and integrates the 3 foundational companion tools:

1. **CodeGraph (Deterministic Code Intelligence & Call Graphs)**:
   - When the `codegraph` CLI is installed on your workstation, `ce-ai init-prj` automatically initializes the local `.codegraph/` repository index (`init_codegraph_if_available`).
   - Sibling Git worktrees (`<repo>-worktrees/<name>`) each receive their own independent index.
   - Sentinel lines are added to `.gitignore` so `.codegraph/` is never accidentally tracked in Git.
2. **Engram (Cross-Session Persistent Memory)**:
   - Registered as an MCP server across active host harnesses (`~/.claude.json`, `opencode.json`, `~/.cursor/mcp.json`).
   - Observations, architectural decisions, and bug fixes are tagged with the active project path so memory remains partitioned cleanly between workspaces.
3. **RTK (Run-Time Kit Token Optimization & Output Redirection)**:
   - Unless `--skip-rtk` or `--skip-companions` is specified, `ce-ai init-prj` reconciles RTK command hooks in supported harnesses (`reconcile_rtk_hooks_if_supported`).
   - Filters verbose build, test, and package manager outputs to preserve agent context windows.

#### Upgrading Adopted Projects (v2 ➔ v3) & Batch Maintenance

Blocks injected by older binaries (`v=2`) remain valid but predate the
retention clarification of the Single Source of Truth rule. You can upgrade
individual projects or batch-upgrade all registered projects:

```bash
# Upgrade single project in place
ce-ai init-prj /path/to/my-project --tier full

# Batch upgrade all registered projects on your machine
ce-ai init-prj --all

# Batch reconcile drift and repair issues across all registered projects
ce-ai doctor --fix --all-projects
```

> ℹ️ **Expected after upgrading**: `ce-ai doctor` and `ce-ai status` compare
> each adopted project's on-disk block against the current template and will
> report **stale block version** hints for older-version adoptions until they
> are re-adopted — for tiers whose block body changed (`full`,
> `orchestrator`). `minimal`-tier adoptions classify as healthy because their
> body is byte-identical across these versions (the SHA match short-circuits
> the version check), even though `state.json` keeps the older
> `block_version`. These hints are the intended signal prompting the re-run
> above, not a regression.

#### Migrating Legacy AGENTS.md Directives (`--migrate`)

Repositories adopted with early versions of CE-AI may contain rigid, unmanaged workflow directives embedded directly in `AGENTS.md` outside the managed block. These include obsolete write-blocking gates (such as rigid Invariant 6, mandatory ASCII FSM diagrams, pre-decoupling Stage 2 constraints, and deprecated `ce-ai models` or `ce-ai decisions` commands).

To surgically modernize legacy directives into decoupled Compound Engineering guidance while strictly preserving 100% of host project domain rules and custom governance policies:

```bash
# Preview planned modernization without writing changes (non-destructive)
ce-ai init-prj /path/to/my-project --migrate --dry-run

# Execute surgical migration (automatically creates /path/to/my-project/AGENTS.md.bak)
ce-ai init-prj /path/to/my-project --migrate

# Batch migrate all registered projects across your local fleet
ce-ai init-prj --all --migrate
```

**Safety Guarantees**:
1. **Host Rule Preservation**: Non-CE-AI content (domain rules, tech stack linters, security protocols) remains byte-for-byte untouched.
2. **Automatic Backup**: A clean `AGENTS.md.bak` snapshot is written prior to mutating disk.
3. **Doctor Auto-Repair**: `ce-ai doctor` flags unmanaged legacy directives during routine audits, and `ce-ai doctor --fix` automatically migrates them.

#### Migrating Legacy OpenSpec to Living Architecture (`--migrate-arch`)

Repositories maintaining specifications under legacy `openspec/` can be migrated to living architecture (`docs/arch/`) safely using `--migrate-arch`:

```bash
# Preview planned migration without moving assets
ce-ai init-prj /path/to/my-project --migrate-arch --dry-run

# Execute migration (requires a clean Git working tree)
ce-ai init-prj /path/to/my-project --migrate-arch

# Or repair via doctor during environment audits
ce-ai doctor /path/to/my-project --fix --migrate-arch
```

**What it does**:
1. Verifies the Git working tree is clean to prevent accidental asset clobbering.
2. Migrates `openspec/specs/` to living domain specifications in `docs/arch/system/`.
3. Migrates `openspec/changes/archive/` to historical milestones in `docs/arch/archive/`.
4. Cleans up empty legacy directories and updates the managed block in `AGENTS.md` to version 9.
5. Preserves unmigrated repositories seamlessly via the dual-path compatibility layer.

---

### De-adopting a Project (`ce-ai deinit-prj`)

To remove Compound Engineering governance and restore original file state cleanly:

```bash
# Preview what de-adoption would remove without touching disk
ce-ai deinit-prj --dry-run

# De-adopt current directory
ce-ai deinit-prj

# De-adopt a specific project path
ce-ai deinit-prj /path/to/my-project
```

- **If `AGENTS.md` pre-existed**: The managed block between `<!-- ce-ai:block begin -->` and `<!-- ce-ai:block end -->` is surgically removed, preserving all pre-existing developer content byte-for-byte (including CRLF/LF line endings).
- **If `AGENTS.md` was created by `ce-ai`**: The file and derived `CLAUDE.md` stub are completely deleted.
- **Project unregistration**: The project path is removed from `state.json` atomically.

---

### Diagnostic Probes (`status` & `doctor`)

You can inspect the health and integrity of all adopted projects across your system at any time:

```bash
# Inspect overall system status & project adoption state
ce-ai status
```

*Example Output:*
```text
installed: opencode (v1.0.8, source: github)
drift: none
projects: 1 adopted
  - /Users/dev/projects/web-api (tier: Full, file: AGENTS.md, status: OK)
```

```bash
# Run automated health probes (checks missing files & SHA block drift)
ce-ai doctor
```

If an adopted project's `AGENTS.md` is deleted or manually tampered with, `ce-ai doctor` will immediately flag the issue:
```text
project-adoption: block SHA drift detected at '/Users/dev/projects/web-api'
doctor found 1 finding(s)
```

---

## 🎯 Real-World Scenarios

### Scenario A: Adopting a Brand New (Greenfield) Project

**Goal**: You just ran `cargo new my-service` or `npm init` and want to establish AI governance from day one.

1. Navigate to the new project folder:
   ```bash
   cd my-service
   ```
2. Run project adoption:
   ```bash
   ce-ai init-prj
   ```
3. **Result**: `ce-ai` creates `AGENTS.md` containing the Full 7-stage cycle block, creates `CLAUDE.md` with `@AGENTS.md`, and registers the project in `state.json`.

---

### Scenario B: Adopting an Established Legacy Repository

**Goal**: You have a 5-year-old codebase with a detailed `AGENTS.md` file containing team onboarding notes and deployment commands. You cannot risk losing these notes.

1. Run adoption on the legacy repo:
   ```bash
   ce-ai init-prj /path/to/legacy-repo
   ```
2. **Result**: `ce-ai` reads your existing `AGENTS.md`, detects line endings, appends the managed `<!-- ce-ai:block begin ... -->` block at the bottom, and saves the file atomically. Your original onboarding notes remain 100% intact above the block.

---

### Scenario C: Team Working Across Multiple AI Harnesses

**Goal**: Developers on your team use different AI coding assistants — Alice uses Claude Code, Bob uses OpenCode, and Charlie uses Cursor.

1. Adopt the repository once:
   ```bash
   ce-ai init-prj
   ```
2. **Result**:
   - **OpenCode & Cursor**: Read the root `AGENTS.md` directly.
   - **Claude Code**: Reads `CLAUDE.md`, which imports `@AGENTS.md`.
   - All team members operate under identical governance directives without duplicating rule files.

---

### Scenario D: Temporary Adoption for a Code Audit or Refactor

**Goal**: You want to enforce strict 7-stage verification rules while completing a sensitive security refactor, but want to return the repository to its clean state afterward.

1. Adopt the repository before starting work:
   ```bash
   ce-ai init-prj
   ```
2. Execute your refactor under full AI agent governance.
3. Once merged and complete, de-adopt the repository:
   ```bash
   ce-ai deinit-prj
   ```
4. **Result**: The managed block is removed, returning the workspace to its exact original state.
