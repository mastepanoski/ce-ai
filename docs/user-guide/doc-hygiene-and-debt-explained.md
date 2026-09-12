# Documentation Technical Debt & Hygiene Explained: A Professor's Walkthrough

> **Audience:** Beginner / Newcomer  
> **Intent (Diátaxis):** Explanation  
> **Prerequisites:** Basic familiarity with Git and Markdown files.

---

## 1. The Core Problem: The "Rotting Map" Analogy

Imagine you are a traveler visiting a historic city for the first time. The tourist office hands you a printed guidebook with walking maps. 

As you follow the map, you run into trouble:
- **Map says**: *"Turn left on Mill Street and cross the wooden footbridge."*
- **Reality on the ground**: The wooden footbridge was torn down five years ago and replaced by a subway tunnel. Mill Street is now a pedestrian mall.

What happened? The map was accurate on the day it was printed, but the city kept growing and evolving. Because nobody updated the guidebook, the map suffered from **drift**. If you blindly trust the old map, you will wander into dead ends, waste time, and get lost.

In modern software development with AI coding agents (such as Claude Code, GitHub Copilot, Gemini, Cursor, OpenCode, and Antigravity), **documentation is the map that AI agents use to navigate your codebase**.

When documentation falls behind your code, we call this **Documentation Technical Debt**.

```
[Repository Code & Architecture] ──── evolves rapidly ───► [New Reality]
                                                                  ▲
                                                                  │ (DRIFT!)
[OpenSpecs & Solution Notes]    ──── left untouched ─────────────┘
                                     (outdated map)
```

---

## 2. The Three Common Forms of Documentation Debt

In projects powered by the Compound Engineering methodology, work moves through structured specifications (`openspec/changes/`) and persistent learnings (`docs/solutions/`). Over time, three specific types of documentation debt naturally accumulate:

### A. Stranded OpenSpecs (Desynchronization)
An engineer or an AI agent implements a feature, merges the branch into `main`, and ships the release to production. But they forgot to check off the remaining checkboxes in `openspec/changes/<feature>/tasks.md`.

- **The Danger**: When the next AI agent arrives and checks the repository state, it sees open checkboxes and assumes the feature is unfinished. It may waste hours and thousands of tokens attempting to rewrite code that is already in production!
- **How `ce-ai` Spots It**: The diagnostic engine notices that all parent tasks are complete while subtasks remain open, or that git commits modifying the feature already landed on `main`, or that the version tag cited in the spec was surpassed by `Cargo.toml`.

### B. Stale or Abandoned Specs (Inactivity)
Someone starts an ambitious brainstorm or technical design for an experimental feature, writes a spec, and then priorities change. The change folder remains in `openspec/changes/` untouched for months.

- **The Danger**: Active planning folders represent in-flight commitments. Lingering dormant specs clutter the workspace and confuse contributors about what is actually being built.
- **How `ce-ai` Spots It**: The inactivity watchdog calculates the elapsed days since the last commit or filesystem modification. If an incomplete spec sits idle for more than 21 days (configurable), `ce-ai` flags it as stale.

### C. Solution Library Drift (Dead Paths & Missing Metadata)
`docs/solutions/` contains curated post-mortem learnings explaining how tricky architectural bugs were solved. But months later, an engineer refactors a module (e.g. moving `src/tui.rs` into a module directory `src/tui/`).

- **The Danger**: An AI agent searching the solution library for advice reads an obsolete snippet referencing `src/tui.rs`. It tries to import or modify the non-existent file, triggering hallucinations and compiler errors.
- **How `ce-ai` Spots It**: The solution drift probe extracts all backticked code paths (`src/**/*.rs`, `tests/**/*.rs`), verifies their existence against the real filesystem, and checks that standard YAML frontmatter fields (`title`, `category`, `problem_type`, `tags`, `applies_when`) are present.

---

## 3. Why Documentation Debt Hurts AI Agents More Than Humans

Human engineers have institutional memory. When a human developer reads an old spec that says *"Implement widget X"*, they might think: *"Wait, didn't Sarah build widget X last month? Let me check git log."*

AI agents do **not** have human intuition or informal hallway conversations. AI agents are **probabilistic inference engines constrained by context windows**:

1. **Context Window Waste**: Every kilobyte of obsolete planning text fed to an LLM consumes valuable prompt space, pushing out recent code context.
2. **Hallucination Amplification**: When an LLM reads contradictory instructions (e.g., an outdated solution recommending pattern A versus code using pattern B), its confidence drops, leading to confused suggestions.
3. **Zombie Work Loops**: An agent asked to "continue current work" will latch onto the first open checkbox it discovers, happily re-implementing solved problems.

---

## 4. How `ce-ai` Solves This: The Diagnostic Engine

`ce-ai` introduces an automated **Documentation Technical Debt Diagnostic Engine** built directly into the workflow substrate and health system.

```
                  ┌───────────────────────────────┐
                  │    ce-ai doctor / Turn-0      │
                  └──────────────┬────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         ▼                       ▼                       ▼
 ┌───────────────┐       ┌───────────────┐       ┌───────────────┐
 │    Probe 1    │       │    Probe 2    │       │    Probe 5    │
 │OpenSpec Desync│       │ Stale Pending │       │Solution Drift │
 └───────┬───────┘       └───────┬───────┘       └───────┬───────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────────────┐
                    │      DocDebtReport      │
                    │   (Tri-State Analysis)  │
                    └─────────────────────────┘
```

### The Substrate Tri-State (`Clean`, `Debt`, `Unknown`)
To prevent false confidence, every probe reports one of three distinct statuses:
- **`Clean`**: The check ran successfully and found zero documentation debt.
- **`Debt(findings)`**: The check ran and found specific items requiring attention.
- **`Unknown`**: The check requires a substrate that is unavailable (for example, git commit history inside an unpacked zip archive or container with no `.git`). 

> [!IMPORTANT]
> In `ce-ai`, a missing Git substrate **never defaults to `Clean`**. It reports `Unknown` with an honest `[git: n/a]` indicator so that neither humans nor AI agents are misled.

### Turn-0 Workflow Visibility
Whenever you run `ce-ai workflow status` or `ce-ai workflow resume`, `ce-ai` includes a high-signal, single-line summary under Environment State:

```bash
== [Environment State & Drift Status] ==
  git branch: feat/my-feature (HEAD: a1b2c3d)
  working tree: clean (0 uncommitted changes)
  manifest integrity: clean (0 drifted files)
  openspec ledger: clean (0 pending archival)
  doc debt: 2 unarchived (code merged), 1 stale spec (34d), 2 dead solution links
```

When no documentation debt exists, it reports:
```bash
  doc debt: clean
```

---

## 5. Resolving Documentation Debt

`ce-ai` never just complains—it gives you **copy-pasteable commands** to resolve debt instantly when running `ce-ai doctor`.

| Diagnostic Output | What it means | How to fix |
| :--- | :--- | :--- |
| `doctor-warn: openspec change 'feat' is complete with open subtasks` | Code is finished, but checklist has open subtasks. | Run `ce-ai archive <feat> --auto-mark` |
| `doctor-warn: openspec change 'feat' has been pending for 34 days...` | Spec has been inactive for longer than `stale_spec_days`. | Resume the work, shelve it, or run `ce-ai archive <feat> --status "superseded"` |
| `doctor-warn: solution '...' references non-existent path 'src/...'` | A solution file references a deleted or moved source file. | Update the path in the solution or run `/ce-compound-refresh` |
| `doctor-warn: solution '...' missing required YAML frontmatter: ...` | Solution file is missing required metadata fields. | Add the missing YAML key (`title`, `category`, `problem_type`, `tags`, `applies_when`) |

---

## 6. Configuring Repository Hygiene (`.ce-ai.json`)

By default, `ce-ai` applies battle-tested hygiene defaults with zero configuration required. If your team has specific preferences, you can configure them in `.ce-ai.json` at your project root:

```json
{
  "doc_hygiene": {
    "stale_spec_days": 14,
    "check_solution_paths": true,
    "require_solution_frontmatter": true
  }
}
```

- **`stale_spec_days`**: Inactivity threshold before flagging pending specs (default: `21`).
- **`check_solution_paths`**: Validate backticked file existence on disk (default: `true`).
- **`require_solution_frontmatter`**: Enforce structured frontmatter on solution files (default: `true`).

---

## 7. The Golden Rule of Document Hygiene

> **Documentation is code for AI agents.**  
> Just as a compiler checks your types and imports, `ce-ai` checks your specifications and solution library. Keeping your documentation debt clean ensures your AI teammates work with laser precision, zero hallucinations, and maximum token efficiency.
