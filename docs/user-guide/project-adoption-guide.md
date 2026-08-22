# Project Adoption Engine Guide: Non-Destructive Multi-Harness Governance

> **Intent**: How-to & Explanation — Learn why the Project Adoption Engine exists, how marker-delimited injection works under the hood, and how to safely adopt or de-adopt projects across real-world development scenarios.

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

<!-- ce-ai:block begin v=2 tier=full sha256=a1b2c3d4e5f6... -->
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

# Re-apply or upgrade block on an already adopted project
ce-ai init-prj --force
```

#### Available Adoption Tiers

| Tier | Purpose | Recommended for |
| :--- | :--- | :--- |
| `full` **(Default)** | Complete 7-stage development cycle, OpenSpec requirements, Single Source of Truth rule for ideation artifacts, DoD checklist, and security policies. | Production repositories, core services, critical products. |
| `minimal` | Core DoD guidelines, atomic write rules, and basic testing verification. | Small libraries, scripts, internal tools. |
| `orchestrator` | Multi-repo orchestration directives and delegation protocols, plus a directive to distill ideation outputs into specs rather than maintaining them in parallel. | Monorepos, parent workspace folders. |

#### Upgrading Adopted Projects (v1 ➔ v2)

Blocks injected by older binaries (`v=1`) remain valid but lack the Single
Source of Truth guidance. To upgrade an adopted project, re-run the command
after upgrading your `ce-ai` binary — the managed block is replaced in place
between markers and all user content around it is preserved:

```bash
ce-ai init-prj /path/to/my-project --tier full
```

> ℹ️ **Expected after upgrading**: `ce-ai doctor` and `ce-ai status` compare
> each adopted project's on-disk block against the current template and will
> report **SHA drift** for v1 adoptions until they are re-adopted. This is the
> intended signal prompting the re-run above, not a regression.

---

### De-adopting a Project (`ce-ai deinit-prj`)

To remove Compound Engineering governance and restore original file state:

```bash
# De-adopt current directory
ce-ai deinit-prj

# De-adopt a specific project path
ce-ai deinit-prj /path/to/my-project
```

- **If `AGENTS.md` pre-existed**: The managed block is removed, leaving your custom pre-existing content intact.
- **If `AGENTS.md` was created by `ce-ai`**: The file and derived `CLAUDE.md` stub are completely deleted.

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

### Interactive TUI Shortcut

In the full-screen interactive dashboard (`ce-ai`), press **`[I]`** at any time to trigger project adoption for the current workspace.

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
