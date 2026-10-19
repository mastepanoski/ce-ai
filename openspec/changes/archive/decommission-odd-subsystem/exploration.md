# Exploration: Decommission ODD Subsystem and Align Compound Agile Fast-Path

## Background & Historical Context

ODD was adapted from Gentle AI (Alan Buscaglia) to prevent ceremonial fatigue. In theory, it offered a "Speedboat" track (`odd/tasks/<feature>.md`) contrasting with the formal "Aircraft Carrier" track (`openspec/changes/<feature>/`), bridged by `ce-ai workflow graduate`.

In practice:
1. **Developers and Agents Bypassed It**: Coding agents (Claude Code, Cursor, OpenCode, Antigravity) use native Compound Engineering workflows (`/ce-plan`, `/ce-work`, `/ce-debug`).
2. **Artifact Proliferation**: Adding `odd/tasks/` created a parallel specification standard that clashed with Compound Engineering's canonical `docs/plans/` and OpenSpec's `openspec/changes/`.
3. **UX Friction**: Developers inspecting `workflow status` were presented with banners instructing them to create files in `odd/tasks/` that no downstream tool consumed.

## Evaluated Options

### Option A: Retain ODD as-is with Documentation Updates
- *Pros:* No code changes.
- *Cons:* Leaves dead commands, misleading CLI banners, and continues violating *No Semantic Authority*.

### Option B: Abrupt Hard Deletion of All ODD Code and Types
- *Pros:* Complete eradication of ~500 LOC.
- *Cons:* Breaks deserialization for any existing `state.json` with `"execution_mode": "organic"` and breaks CLI invocations expecting `ce-ai graduate`.

### Option C: Deprecate `odd/tasks/` and `graduate`, Re-Align Banners to Compound Agile Fast-Path (Selected)
- *Pros:*
  - Preserves enum deserialization in `state.json`.
  - Re-aligns `workflow status` and `guide` to the reality of Compound Engineering (`docs/plans/` + direct execution).
  - Emits clear, non-blocking deprecation notices if `graduate` is invoked.
  - Leaves the repository in a clean, self-explaining, and faithful state.
