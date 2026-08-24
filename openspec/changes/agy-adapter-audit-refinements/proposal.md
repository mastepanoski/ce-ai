# Proposal: Antigravity (AGY) Adapter Audit Refinements

## Problem Statement
Audit of Google Antigravity (`agy`) Native Harness Adapter identified three documentation and specification clarifications:
1. `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` environment variables are `ce-ai` extension conventions for custom harness directory overrides. They should be documented as `ce-ai` extensions in `design.md`.
2. Project rule adoption targets `.agents/rules/compound-engineering.md` and `GEMINI.md`. Their relationship should be explicitly detailed in `design.md`.
3. The name collision policy in `register_agy_mcp_server` (converting an existing `serverUrl` entry to `stdio` command when registered with the same name) should be formally documented.

## In-Scope
- Update `openspec/changes/agy-native-harness-adapter/design.md` with explicit sections for environment variable extensions, project rule locations, and `serverUrl` collision policy.
- Create OpenSpec change set for AGY audit refinements.

## Out-of-Scope
- Code changes (current code already correctly implements all behavior).
