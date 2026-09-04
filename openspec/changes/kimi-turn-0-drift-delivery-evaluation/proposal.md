# Proposal: Kimi Code CLI Turn-0 Drift Delivery Evaluation

## Problem Statement
Evaluate whether Kimi Code CLI (`kimi`) supports native project-level Turn-0 conversational context injection via `SessionStart` hooks (`[[hooks]]` in `config.toml`) to automatically eliminate Turn-0 drift on project adoption.

## In-Scope Boundaries
- Empirical investigation of Kimi Code CLI lifecycle hooks subsystem, configuration file discovery paths, and context injection mechanisms.
- Formal architectural capture of negative finding if project-level hook configuration is absent.
- Confirmation of existing text-based fallback mechanism in `AGENTS.md` and `.kimi-code/AGENTS.md`.

## Out-of-Scope Boundaries
- Mutating user-global configuration files (`~/.kimi-code/config.toml`) during project-level adoption (`init-prj`).
- Modifying Kimi Code CLI runtime or proprietary binaries.

## Success Criteria
- Documented findings explaining the technical reason `SessionStart` hooks cannot be applied per-project in Kimi Code CLI.
- Preservation of per-project isolation without global configuration contamination.
