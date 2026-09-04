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

`ce-ai` never writes code or talks to an LLM. Its only job is to **install and wire up** the Compound Engineering skills so your AI harness can see and run them. Once that's done, `ce-ai` steps back — you drive everything else by typing `/ce-...` commands *inside your AI tool's chat*, not in the terminal.

> ⚠️ The single most common newbie mistake: typing `/ce-brainstorm` into the terminal. It will fail — it isn't a shell command. It only exists inside your AI harness's chat.

---

## 2. Prerequisites

- One of the [supported AI harnesses](harness-matrix.md) already installed (e.g. Claude Code, Cursor, OpenCode, GitHub Copilot CLI). If you don't have one yet, install any one of them first — `ce-ai` has nothing to plug into otherwise.
- Git installed, and a project directory (an existing repo, or an empty folder you're about to turn into one).

---

## 3. Install the `ce-ai` binary

Pick one (full options in the [README](../../README.md#quick-path)):

```bash
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash
```

---

## 4. Go to your project and install the skills into your harness

```bash
cd my-project   # an existing repo, or: mkdir my-project && cd my-project && git init
ce-ai install --harness all
```

This copies the Compound Engineering skill files (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-compound`, …) into every AI harness `ce-ai` detects on your machine, so they show up as slash commands. Only want one harness? Use its name instead of `all` (e.g. `--harness claude`) — see the [Harness Matrix](harness-matrix.md) for every supported value.

---

## 5. Adopt this specific project

```bash
ce-ai init-prj
```

This is a **separate, complementary step** from `install`: it writes a governance block into this project's `AGENTS.md` (the rules your AI agent follows — the 7-stage cycle, TDD, OpenSpec) and, on harnesses that support it, a startup hook so the agent automatically re-syncs with reality (current branch, uncommitted files, in-progress task) every time a session starts. Nothing here is destructive — see the [Project Adoption Guide](project-adoption-guide.md) if you're curious how.

---

## 6. Verify everything is wired up

```bash
ce-ai doctor
```

A healthy first run looks roughly like this — no `!` warnings, adoption marked `ok`:

```text
== [Harness Health] ==
  claude: installed, skills registered, session-start hook: ok
== [Project Adoption] ==
  AGENTS.md: adopted (tier: full, SHA256 verified)
```

If a harness you expected isn't listed, it means `ce-ai` didn't detect it on this machine — run `ce-ai status` for details, or see [Installation & Coexistence](installation-and-coexistence-mechanisms.md).

---

## 7. Run your first Compound Engineering skill

This is the step that happens **inside your AI harness's chat, not the terminal**. Open Claude Code / Cursor / whichever harness you installed into, **in this same project folder**, and type:

```
/ce-brainstorm add a /health endpoint that returns {"status": "ok"}
```

The agent will ask a couple of clarifying questions and then write a requirements document to `docs/brainstorms/`. That's Stage 1 of the 7-stage cycle — you just started your first Compound Engineering task.

---

## 8. What just happened — the mental model

```
ce-ai install   → puts the skills where your AI harness can find them
ce-ai init-prj  → tells your AI agent the rules to follow in THIS project
ce-ai doctor    → confirms both are wired up correctly
/ce-brainstorm  → the first thing YOU type, inside the chat, to start real work
```

Everything after this point — turning that brainstorm into a spec, a plan, working code, a reviewed PR — is the full 7-stage cycle. Continue with the [Quick Start Workflow Guide](quick-start-workflow-guide.md), which picks up exactly here.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
| :--- | :--- | :--- |
| `/ce-brainstorm` doesn't autocomplete or errors in chat | Skills weren't installed for *this* harness, or the harness needs a restart to pick up new skill files | Re-run `ce-ai install --harness <yours>`, then restart/reload your AI harness |
| `ce-ai doctor` shows a harness as missing | `ce-ai` couldn't find that harness's config on disk | Confirm the harness is actually installed and has been run at least once; see `ce-ai status` |
| `install`/`upgrade` fails with `403 Forbidden` | Unauthenticated GitHub API rate limit | Set `CE_AI_GITHUB_TOKEN` / `GITHUB_TOKEN`, or run `gh auth login` — see the [README](../../README.md) |
| You typed a `/ce-...` command into the terminal by mistake | See Section 1 above | Open your AI harness's chat window instead |

---

*Related reading:*
- [Quick Start Workflow Guide](quick-start-workflow-guide.md) — the full 7-stage cycle: features, bug fixes, resuming work.
- [Compound Workflow Explained](compound-engineering-workflow-explained.md) — how `/ce-strategy`, `/ce-ideate`, `/ce-brainstorm`, OpenSpec, and `/ce-work` fit together without duplicating effort.
- [Harnesses, Loops & Context Masterclass](harnesses-loops-and-context-masterclass.md) — what a "harness" actually is, for readers still fuzzy on that term.
