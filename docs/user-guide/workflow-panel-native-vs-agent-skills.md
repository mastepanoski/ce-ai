# Why the Workflow Panel Only Runs Native Commands

> **Quadrant:** Explanation — this page explains *why* the dashboard behaves the way it does. For a hands-on walkthrough, see the [Quick Start Workflow Guide](quick-start-workflow-guide.md); for FSM internals, see [FSM & Checkpoints Explained](fsm-and-checkpoints-explained.md).

If you open `ce-ai` and press `2` to reach the **Workflow (FSM)** panel, you will notice something deliberate: some actions run right there inside the dashboard, while others are shown only as *pointers* to your agent session. This page explains that split so it never feels arbitrary.

---

## The two kinds of rows: `[run]` vs `skill:`

Every row in the panel carries one of two text markers:

| Marker | Meaning | Example |
| --- | --- | --- |
| `[run]` | Executable here, by the `ce-ai` binary itself | `[run] [Enter]  Query workflow status` |
| `skill:` | Belongs to an agent session; the dashboard shows you *what* to run, not run it | `skill: [1: Ideation]   ➔ /ce-brainstorm · /ce-ideate · /ce-strategy` |

The markers are plain text on purpose — you can tell them apart even in a terminal without color support.

## What runs natively

Exactly three things are executable from the panel, because they are real subcommands of the `ce-ai` binary:

- **`[Enter]` — query workflow status.** Runs `ce-ai workflow status` and shows its actual output in a results window.
- **`[1-7]` — save stage checkpoints.** Records which of the 7 stages you are in, directly into `state.json`.

That is the complete list. Nothing else is advertised as clickable, and everything advertised works.

## Why the 7 stages are guide-only

The panel lists all seven stages of the Compound Engineering cycle, each with its mapped skills. These skills (`/ce-brainstorm`, `/ce-plan`, `/ce-work`, `/ce-compound`, `/ce-commit-push-pr`) are **not programs** — they are instructions that an AI agent harness (OpenCode, Claude, Cursor, ...) loads and follows.

Two reasons the dashboard chooses not to launch them:

1. **Delegation would be different for every harness.** There is no single command like "run ce-brainstorm". OpenCode, Claude, and the ten other supported harnesses each have their own way of invoking agent sessions. A dashboard cannot guess which one you use.
2. **It would break the single-dashboard experience.** Agent sessions produce long, interactive output. Spawning them behind the dashboard would lose the visibility that makes the dashboard useful in the first place.

So the panel does the honest thing: it tells you exactly which skill belongs to each stage, and lets you run it where skills actually live — your agent session.

> 💡 Skill names above follow the **OpenCode** convention (`/ce-...`). Other harnesses may name or file these skills differently; mapping every harness's naming is out of scope for this page.

## Why there is no "resume" key

You might expect a button to resume from your last checkpoint. Today, `workflow resume` is a placeholder: it re-prints status rather than restoring anything, and pressing `[1-7]` already records stage transitions reversibly. Binding a key to it would show a success message without doing real work — exactly what an *honest* dashboard must never do. If checkpoint recovery ever grows beyond stage transitions (for example, restoring model assignments), it deserves its own feature.

## The mental model

Think of the dashboard as a **control tower**, not an airplane:

- It monitors state (`status`), records positions (`checkpoints`) — things it owns.
- It hands off flying to the pilots — the agent sessions that execute each stage.
