# Proposal: Grok Build CLI Turn-0 Drift Delivery Evaluation

## Problem Statement
Evaluate whether Grok Build CLI (`grok`) supports native Turn-0 conversational context injection via `SessionStart` hooks (`~/.grok/hooks/*.json` or `.grok/hooks/*.json`) to eliminate Turn-0 drift automatically.

## In-Scope Boundaries
- Empirical investigation of Grok CLI lifecycle hooks subsystem, stdin/stdout schemas, and context injection capabilities.
- Formal architectural capture of negative finding if stdout context injection is unsupported.
- Confirmation of existing text-based fallback mechanism in `AGENTS.md` and `.grok/rules/compound-engineering.md`.

## Out-of-Scope Boundaries
- Implementing a shell hook that executes with discarded output.
- Modifying Grok CLI runtime or proprietary xAI binaries.

## Success Criteria
- Documented findings explaining the technical reason `SessionStart` hooks cannot inject context in Grok CLI.
- No dummy or non-functional hooks installed into user environments.
