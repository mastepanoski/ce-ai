# 🎓 Determinism & ce-ai Explained

> **Audience**: Beginner · **Intent**: Explanation
>
> This page answers one question: *"ce-ai says it is deterministic — so why did my AI agent do something different the second time?"* It assumes zero prior knowledge of the concepts involved.

---

## 1. What "deterministic" actually means

A **deterministic** system is one where the same input always produces the same output, no matter when you run it, who runs it, or where.

Think of a vending machine. Press `B4`, insert a coin, and you get the same chocolate bar every single time — today, next week, in another country. The machine does not improvise. That is determinism.

Now think of a barista. Order "a cappuccino" twice and you get two similar but different drinks: more foam today, a leaf drawn in the latte art yesterday. The inputs were identical; the outputs were not. The barista is *skilled*, but *not deterministic*.

Software can be built like the vending machine: a compiler given the same source code produces the same binary. A hash function given the same file produces the same fingerprint forever.

## 2. The three layers of every Compound Engineering run

When you follow a CE workflow (brainstorm → plan → work → ship), three very different layers are at play:

| Layer | What it is | Who controls it | Deterministic? |
| :--- | :--- | :--- | :--- |
| **1. Assets & state** | Skills on disk, plugin files, `state.json`, model assignments, manifests | **ce-ai** | ✅ Yes — by design |
| **2. Execution** | The agent inside your harness reading those skills and writing code | Your harness + the LLM provider | ❌ No |
| **3. The world** | Network answers, file timestamps, other processes editing files | Nobody | ❌ No |

**ce-ai's job is layer 1 only.** It makes sure the *ingredients* are exactly right, byte-for-byte verifiable, and never silently swapped. It cannot control how the *chef* cooks.

## 3. What ce-ai guarantees (layer 1)

These are concrete, verified mechanisms — not promises:

- **Pinned sources.** Installs and upgrades download immutable release tags only. A network failure is a loud error with exit code 5 — ce-ai never quietly substitutes a moving branch ([Sync & Upgrade](sync-and-upgrade-mechanisms.md)).
- **Cryptographic provenance.** Every downloaded archive is hashed (SHA256) and bound to its release tag in `state.json`. Re-running `upgrade --to <tag>` re-verifies those bytes before using them.
- **Ordered, write-free planning.** Drift detection walks files in sorted order and plans actions without touching disk, so planning is repeatable.
- **Validated checkpoint writes.** Every *recorded* stage change passes the FSM gate: `state.json` can never hold an illegal jump such as Stage 2 → Stage 5 — only reset to Stage 1, stay, advance one, or rewind one ([FSM & Checkpoints](fsm-and-checkpoints-explained.md)). Honest boundary: recording checkpoints is opt-in. An LLM that never calls `workflow checkpoint` can move through stages without any technical rejection — the gate validates declarations, not behavior.
- **Atomic writes.** Config files are written via temp-file-and-rename, so a crash mid-write can never leave them half-corrupted.
- **Byte-stable skill resolution.** Resolving the same skills from an unchanged tree emits byte-identical output every time — no hidden clocks stamped into what your agent reads.

## 4. Why the LLM layer cannot be made deterministic

This is the honest part, and it matters that you understand it deeply.

The workflow instructions (skills like `ce-work` or `ce-plan`) are read by a **Large Language Model**. An LLM does not compute an answer the way a hash function does — it generates text token by token by *sampling* from a probability distribution. Even with temperature set to 0:

- Providers run massive distributed systems where batching, hardware differences, and floating-point non-associativity make bit-exact reproducibility across calls impossible.
- Model versions change server-side without your consent or knowledge.
- Context ordering and caching effects shift which tokens influence which.

And beyond sampling, workflows deliberately use **judgment**: review personas argue, sub-agents explore in parallel, tools return live data (docs fetched today differ from docs fetched last month). Two runs over byte-identical assets will diverge — this is expected behavior, not a bug in your setup.

> ⚠️ There is no technical mechanism today — flag, setting, or tool — that makes LLM execution fully deterministic. Anyone claiming otherwise is selling something. The correct engineering response is not to chase impossible run-to-run equality, but to make **inputs reproducible** and **outputs verifiable**. That is exactly what ce-ai does.

## 5. Behaviors that depend on your environment (by design)

Some ce-ai decisions are relative to *your machine's state*. They are consistent for the same machine state, but not portable across time:

| Behavior | Where | Why it varies |
| :--- | :--- | :--- |
| `workflow resume` infers the feature from directory modification times | `openspec/changes/` | Git operations rewrite mtimes |
| Skill resolution re-verifies file hashes at resolution time | managed skills dirs | Files may be edited between commands |
| Registry contents follow workspace precedence | `.ce-ai/skills/` vs global roots | Depends on current directory and `$HOME` |
| Timestamps recorded in `state.json` and backups | metadata fields | Wall clock moves |

None of these corrupt assets — they just mean results reflect *when and where* you ran the command.

## 6. Compensating controls: reproducible inputs, verifiable outputs

Since process determinism is impossible, CE engineering discipline leans on four controls:

1. **Checkpoints & resume** — save progress before context loss; resume re-hydrates the exact stage, task, and OpenSpec feature.
2. **OpenSpec contracts** — `spec.md` requirements in WHEN/THEN form define what *must* be true regardless of how the agent got there.
3. **Drift detection & restore** — SHA256 manifests detect any asset tampering; `sync` restores known-good bytes.
4. **Empirical gates** — `cargo test`, `make e2e`, CI matrices verify outcomes, not intentions.

The mental model: **ce-ai guarantees the sheet music is exact. It cannot guarantee two performances sound identical — but the conductor's checklist catches every wrong note that matters.**

---

## Related

- [Sync & Upgrade Mechanisms](sync-and-upgrade-mechanisms.md) — the fail-loudly source contract in practice
- [FSM & Checkpoints Masterclass](fsm-and-checkpoints-explained.md) — validated transitions and progress recovery
- [Architectural & Conceptual Guide](architectural-and-conceptual-guide.md) — senior-level internals
