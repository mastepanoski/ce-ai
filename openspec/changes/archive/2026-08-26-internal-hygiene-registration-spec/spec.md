# Spec Delta

- **WHEN** a new `HarnessKind` variant is added, **THEN** compilation fails
  until `harness::registration::registration_spec` classifies it.
- **WHEN** install or sync runs for cursor, **THEN** no skills tree is
  written under `~/.cursor/skills`.
- **WHEN** ce-ai runs inside a git repository containing `.ce-ai.json`,
  **THEN** model-assignment reads reflect workspace overrides over global
  state, while writes persist globally.
- **WHEN** the crate builds with `-D warnings`, **THEN** zero
  `dead_code` suppressions exist in `src/`.
