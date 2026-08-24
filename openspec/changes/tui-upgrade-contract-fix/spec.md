# Spec Delta
- **WHEN** the TUI Upgrade Release action runs, **THEN** it MUST invoke
  `ce-ai upgrade` with arguments accepted by the current CLI contract and
  MUST NOT pass --harness/--force.
- **WHEN** any TUI-spawned vector stops parsing against its subcommand's
  clap surface, **THEN** the crate MUST fail to build tests (anti-drift net).
