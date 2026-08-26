# Proposal: `internal-hygiene-registration-spec`

## Why
Three debts from the v1.20.x sweep: install.rs still carried the nine-arm
copy-paste chain that sync already replaced; the shared table gave cursor a
skills subpath it must not have (v1.19.2 regression: sync polluted
~/.cursor/skills); and 14 dead-code suppressions hid genuinely unused API.

## What Changes
- Promote RegistrationSpec/McpRegistrar/copy_managed_skills to
  harness::registration; install joins sync on the shared table.
- Cursor becomes MCP-only in the spec (skills_subpath None) — fixes the
  pollution + matches its verification-matrix exclusion.
- Delete all #[allow(dead_code)] (incl. three module-wide) and the items
  they hid.
- Wire the documented-but-dormant .ce-ai.json workspace overrides via
  Context.workspace_root into model-assignment readers.

See design.md/spec.md/tasks.md for contracts and test plan.
