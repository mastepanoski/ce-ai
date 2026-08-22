# Disclaimer

**AI-Co-Created & Experimental Software**

## Nature of This Project

`ce-ai` is **co-created with AI**: the vast majority of its code, documentation, and design artifacts were produced through human-AI collaboration, where a human engineer directs, reviews, and validates the output of AI coding agents.

The project is **experimental software under active development**. It is being continuously tested and refined, and its behavior, APIs, CLI surface, and documentation may change without notice between releases.

## Origin & Scope Evolution

This tool was **originally created as a personal utility** to keep the [Compound Engineering Plugin](https://github.com/Every-One-AI/compound-engineering) installation and its linked tools up to date across multiple AI harnesses. It has since been **extended beyond its original scope**, incorporating ideas and patterns discovered in other environments and ecosystems — including [`gentle-ai`](https://github.com/Gentleman-Programming), open research lines, and community tooling practices.

As a result, some capabilities may reflect opinions tuned for specific workflows rather than general-purpose solutions.

## What This Means for You

- **No warranty of fitness**: `ce-ai` modifies files under your AI harness configuration directories (`opencode.json`, `AGENTS.md`, plugin folders). While every mutation is backed up and reversible (`--dry-run`, `ce-ai uninstall`), you use this tool at your own risk.
- **Verify before trusting**: Always review diffs before applying changes, especially on machines with hand-tuned agent configurations.
- **Human accountability remains yours**: Per our [AI Policy](./AI_POLICY.md), the human operator retains ultimate authority and responsibility over all changes applied by or through AI agents.
- **Version numbers do not imply stability**: although `ce-ai` follows SemVer and has passed the 1.0 mark, it remains experimental software; breaking changes may land in any release regardless of whether the version bump is major or minor. Review the [CHANGELOG](./CHANGELOG.md) before upgrading.

## Transparency Statement

We disclose the degree of AI involvement in this project following the [AI Assessment Scale](https://www.skills.sh/mastepanoski/claude-skills/ai-assessment-scale): this project operates at the highest levels of the scale (**Full AI Generation → Human-Directed AI Collaboration**), with continuous human oversight, review gates (CI, code review personas, TDD verification), and final human approval on every merged change.
