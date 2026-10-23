---
title: "Decommission ODD Subsystem and Align Compound Agile Fast-Path"
domain: gate
---

# Specification: Decommission ODD Subsystem and Align Compound Agile Fast-Path

### WHEN `ce-ai workflow status` is executed on a tactical branch or in Organic mode
- **THEN** it MUST display `[Workflow Mode: Compound Agile Fast-Path (Direct / docs/plans/)]`.
- **THEN** it MUST emphasize direct `/ce-debug` / `/ce-work` execution and lightweight `docs/plans/` planning.
- **THEN** it MUST NOT prompt the user to author `odd/tasks/<feature>.md`.

### WHEN `ce-ai workflow graduate <feature>` (or `ce-ai graduate <feature>`) is executed
- **THEN** it MUST emit a deprecation warning advising that `odd/tasks/` is deprecated in favor of `docs/plans/` and direct `openspec/changes/`.
- **THEN** if a legacy `odd/tasks/<feature>.md` file exists, it MUST migrate the tasks losslessly into `openspec/changes/<feature>/` for backward compatibility.
- **THEN** if no `odd/tasks/<feature>.md` file exists, it MUST return a helpful usage error guiding the user.

### WHEN `ce-ai workflow guide` is executed
- **THEN** it MUST describe Path 1 as the **Compound Agile Fast-Path (Tactical & Bug Fixes)**.
- **THEN** it MUST omit `ce-ai workflow graduate` from the recommended core workflow commands.

### WHEN `ce-ai gate check` evaluates an organic task exceeding 200 LOC
- **THEN** it MUST advise formalizing an OpenSpec change package or planning in `docs/plans/` without referencing `ce-ai workflow graduate`.
