# Proposal: Google Antigravity (AGY) Adapter Audit Refinements

## Problem Statement
Audit findings across the native harness adapters identified specific documentation and behavior clarifications for the Google Antigravity (`agy`) adapter:
1. `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` are custom `ce-ai` relocation extensions not present in official Google Antigravity documentation.
2. Workspace-level `.agents/rules/*.md` loading is an extension convention; `GEMINI.md` remains the primary instruction file.
3. Managed server name collisions convert any pre-existing remote `serverUrl` entry under managed names (`codegraph`, `engram`) to stdio command definitions (`server_url = None`).
4. `HarnessAdapter` trait abstraction (`canonical_instruction_file`, `derived_stub_files`) evolved cleanly as an earned abstraction.

## Scope Boundaries
- **In Scope**:
  - Formally document `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` extension conventions in OpenSpec `design.md`.
  - Formally document project rules architecture and `serverUrl` collision policy in OpenSpec `design.md`.
  - Explicitly preserve non-colliding remote MCP servers while resetting `serverUrl` to `None` only on name collision with managed stdio tool names (`codegraph`, `engram`).
  - Formally document `HarnessAdapter` zero-argument trait signatures (`canonical_instruction_file`, `derived_stub_files`).
  - Add explicit unit tests verifying `serverUrl` resetting on name collision and environment variable extension fallback order.
- **Out of Scope**:
  - Modifying Google Antigravity official binary or breaking existing native configuration schemas.
