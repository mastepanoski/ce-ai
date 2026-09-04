<!-- Diátaxis Quadrant: Explanation | Audience: Beginner -->
# 🎓 Checkpoints vs. Memory Writes & Planning Files Explained

> **Audience**: Beginner · **Intent**: Explanation
>
> This page answers a question that comes up often once a team already has Engram (persistent memory) or a generic file-based planning skill installed: *"Couldn't we just use that instead of `ce-ai workflow checkpoint`?"* Short answer: no — they solve different problems, and the difference is exactly what makes `ce-ai`'s progress tracking trustworthy. If you got asked this in a review, you can quote this page.

---

## 1. The three objections, answered up front

| Objection | Short answer |
| :--- | :--- |
| *"Can't Engram + a handoff note build a good checkpoint?"* | No. A checkpoint is a **validated FSM state transition**, written by a CLI binary. A handoff note is **freeform prose written by an LLM** — nothing stops it from being wrong, inconsistent, or skipping a stage. |
| *"Doesn't that burn a lot of tokens?"* | The checkpoint write itself doesn't — it's a normal CLI invocation with two short string arguments, not a document the model has to compose. What burns tokens is generating and re-reading a long narrative handoff — which is exactly what this mechanism avoids. |
| *"What about a file-based plan, like `task_progress.md`?"* | That pattern is for open-ended tasks that have **no fixed methodology**. `ce-ai`'s 7-stage cycle is the opposite: a fixed, shared methodology every adopted project follows, so its progress needs one canonical, schema-validated ledger — not a bespoke markdown file whose shape can drift per session. |

The rest of this page explains *why*, in detail.

---

## 2. What `ce-ai workflow checkpoint` actually is

It is a **CLI subcommand**, not something an AI agent composes from scratch:

```bash
ce-ai workflow checkpoint --stage 4 --task "4.2 Implementing Unit 2"
```

Three properties fall out of that, and none of them are available to a memory write or a markdown file:

1. **Validated, not just recorded.** Every write is checked against the FSM's legal-transition rules before it touches disk — reset to Stage 1, stay, advance exactly one stage, or rewind exactly one. Stage 2 → Stage 5 is rejected with exit code 2, full stop. See [FSM & Checkpoints Masterclass](fsm-and-checkpoints-explained.md) for the full transition table, and [Determinism & ce-ai Explained](determinism-explained.md) §3 for why this is one of the few things `ce-ai` can actually guarantee.
2. **One canonical location, fixed schema.** State lives in `~/.ce-ai/state.json` as a small typed struct (stage, task, feature name, timestamp) — atomically written, machine-parseable (`--json`), diffable, and independent of which harness or which agent wrote it.
3. **Re-probed live, not replayed from memory.** `ce-ai workflow resume` doesn't just print back what was last saved — it re-runs git, the manifest drift check, and the OpenSpec task checklist **at the moment you resume**, every time. A handoff note or a memory entry is a snapshot frozen at write time; it goes stale the second the disk changes underneath it.

---

## 3. Why Engram is a different (complementary) tool, not a substitute

Engram's `mem_search` / `mem_context` are for **durable, semantic knowledge** — "we hit this bug before," "this pattern works," "the user prefers X" — retrieved by *meaning*, across sessions and even across projects. That is genuinely valuable, and it already has a place in the 7-stage cycle: **Stage 6 (`ce-compound`)** is where a finished cycle's discoveries get written to `docs/solutions/` and enrich Engram memory.

What Engram (or a "handoff" note built on top of it) cannot do is answer "is it legal for this project to be at Stage 5 right now?" — because that requires validating against the FSM's transition rules, and a freeform memory write has no schema to validate against. Nothing stops an agent from writing a handoff that says "ready to ship" while `tasks.md` shows two items still unchecked. Stage 6 knowledge capture happens **after** a checkpoint records *where* you are — it captures *why* things happened, not *where* you are in the cycle.

---

## 4. Why it doesn't meaningfully burn tokens

The checkpoint write is two short string arguments passed to a compiled binary — comparable in cost to a one-line commit message, not to writing a report. No LLM call is required to *produce* `state.json`; the CLI just validates and serializes what it was given.

Compare the read side too: `ce-ai workflow resume` prints a **bounded** set of status lines — current stage, active task, git branch, drift count, OpenSpec checklist progress — pulled fresh from disk probes each time. A handoff document or memory entry meant to fully capture "everything needed to resume" has no such bound: it tends to grow with how much the session accumulated, and its cost is paid **twice** — once to generate it, again in full every time it's read back in a future session. The checkpoint mechanism's cost stays flat regardless of how long the surrounding conversation gets.

---

## 5. Why file-based planning (`task_progress.md`) isn't the same job

Generic file-based planning skills exist for tasks with **no fixed methodology** — arbitrary, open-ended, multi-step work where the shape of "progress" is different every time and the agent should be free to structure its own plan file however it wants. That freedom is the whole point of that pattern, and it's genuinely the right tool for, say, an ad hoc research survey (see [Quick Start Guide, Scenario 3: fast-tracking](quick-start-workflow-guide.md#-scenario-3-bypassing--fast-tracking-the-workflow) for tasks that legitimately bypass the 7-stage cycle entirely).

`ce-ai`'s 7-stage cycle is the opposite kind of problem: it is a **fixed, shared methodology** that every project adopting Compound Engineering follows the same way, and the whole point is that progress through it must mean the same thing across every harness, every agent, and every session. That needs one canonical, schema-validated ledger that rejects illegal states — not a bespoke checklist file whose format, field names, and honesty are whatever the agent that last touched it decided.

---

## 6. Side-by-side

| | `ce-ai workflow checkpoint` | Engram + handoff note | File-based planning (`task_progress.md`) |
| :--- | :--- | :--- | :--- |
| **Written by** | CLI binary | LLM (freeform prose) | LLM (freeform markdown) |
| **Format** | Fixed schema (`state.json`) | Unstructured | Unstructured, ad hoc |
| **Validated against illegal states?** | Yes — FSM transition gate | No | No |
| **Cost pattern** | Flat — two short arguments in, bounded status lines out | Grows with session; paid on write and full re-read | Grows with session; paid on write and full re-read |
| **Read back how?** | Re-probed live from disk/git each time | Recalled from memory, as last written | Re-read verbatim |
| **Best for** | Position in a fixed, shared 7-stage methodology | Durable cross-session/cross-project knowledge (Stage 6) | Open-ended tasks with no fixed methodology |

---

## Related

- [FSM & Checkpoints Masterclass](fsm-and-checkpoints-explained.md) — the full transition table and internals
- [Determinism & ce-ai Explained](determinism-explained.md) — what `ce-ai` guarantees and what it honestly cannot
- [Zero-Step Drift Recovery Explained](zero-step-drift-recovery-explained.md) — how `resume` re-probes live disk state at Turn-0
- [Quick Start Workflow Guide](quick-start-workflow-guide.md) — when to use the full cycle vs. a fast-track bypass
