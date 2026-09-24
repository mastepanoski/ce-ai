<!-- Diátaxis Quadrant: Reference | Audience: Compound Engineering maintainers and contributors -->
# Draft: Discussion Issue for Compound Engineering

> **Use:** This is an English, feedback-first issue draft for [EveryInc/compound-engineering-plugin](https://github.com/EveryInc/compound-engineering-plugin). It is intentionally **not** a pull-request proposal or an integration request. Check current issues for duplicates and follow the upstream issue-first contribution process before posting.

## Title

Discussion: CE-AI — experimenting with Compound Engineering as an agentic engineering harness

## Body

I have been experimenting with applying Compound Engineering beyond a collection of agent skills: treating it as the underlying methodology of an agentic software-engineering harness.

The result is [CE-AI](https://github.com/mastepanoski/ce-ai), an open-source project that explores how Compound Engineering principles can drive an adaptive engineering workflow across agents, tools, project context, and durable artifacts.

My understanding is that Compound Engineering provides the philosophy and the reusable skill loop. CE-AI explores an orchestration layer around that loop: project adoption, recoverable workflow state, formal artifacts when needed, verification visibility, and durable knowledge capture.

```text
Compound Engineering
methodology and skills
          │
     ┌────┴────┐
     │         │
Official     CE-AI
plugin       engineering harness
     │         │
coding-agent hosts   agents / tools / project artifacts
```

CE-AI is not a fork of Compound Engineering and is not intended to compete with the official plugin or with coding-agent hosts such as Claude Code, Codex, or OpenCode. It is an experiment in making the surrounding engineering process more executable and inspectable while using Compound Engineering as the underlying philosophy.

One related experiment is **Organic Driven Development (ODD)**. ODD was created by [Alan Buscaglia](https://github.com/Alan-TheGentleman) through [Gentle AI](https://github.com/Gentleman-Programming/gentle-ai). In CE-AI, it is an adaptive path for lightweight, understood work; a graduation bridge promotes work to formal project artifacts when its scope grows. CE-AI does not claim ownership of ODD.

I am sharing this primarily to get feedback from the Compound Engineering maintainers and community—not to request integration.

In particular, I would value perspectives on:

1. Whether harness-level experimentation like this is aligned with the broader Compound Engineering philosophy.
2. Which boundaries should remain clearly separate from the official multi-host plugin.
3. Whether there are useful, non-invasive integration points or lessons that the two projects could share.

Before posting, I compared the current official repository: it presents Compound Engineering as 36 skills across 14 hosts and includes capabilities such as `ce-explain`, `ce-pov`, Compound Packs, and its evolved multi-host architecture. I would welcome corrections to this comparison and suggestions for where CE-AI’s framing is inaccurate or redundant.

Thank you for building and maintaining the methodology that made this exploration possible.
