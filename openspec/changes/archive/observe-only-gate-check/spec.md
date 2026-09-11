# Specification: Spike Observe-Only — Medir (Sin Bloquear) Escritura en ce-work Sin OpenSpec Aprobado

## Formal Requirements

### Requirement 1: Kill-Switch Short-Circuit
- **WHEN** the environment variable `CE_AI_DISABLE_GATE_CHECK=1` or `CE_AI_GATE_CHECK_DISABLED=1` is present, OR the `--disabled` flag is provided to `ce-ai gate check`,
- **THEN** `ce-ai gate check` MUST exit immediately with code `0`, performing no state loading, no filesystem traversal, and no log writes.

### Requirement 2: Write Target Scope Filtering
- **WHEN** `ce-ai gate check` receives an invocation,
- **AND** the tool name is not `Write` or `Edit` (case-insensitive), OR the target file path does not descend from `src/`,
- **THEN** the command MUST exit immediately with code `0`, without recording a gate decision or evaluating OpenSpec contracts.

### Requirement 3: Undetermined Checkpoint Resolution
- **WHEN** the target write is under `src/**`,
- **AND** `state.json` is missing, unparseable, unadopted for the workspace, or contains no workflow checkpoint for the active `(workspace, branch)` tuple,
- **THEN** the decision MUST be `undetermined`,
- **AND** the event MUST be logged to `~/.ce-ai/gate-events.jsonl` with `decision: "undetermined"`,
- **AND** the process MUST exit with code `0`.

### Requirement 4: Edge Case Bucket — Mtime Fallback
- **WHEN** the active workflow checkpoint has `resolution` set to `Some(FeatureResolution::MtimeFallback)`,
- **THEN** the decision MUST be `edge_case` with `edge_case: "mtime_fallback"`,
- **AND** this decision MUST NOT be categorized as `would-block` or `pass`,
- **AND** the process MUST log the event and exit with code `0`.

### Requirement 5: Edge Case Bucket — Worktree Uncommitted Spec
- **WHEN** the active workflow checkpoint references a feature whose `openspec/changes/<feature>/` directory contains uncommitted modifications or untracked files as detected by `probe_openspec_has_uncommitted`,
- **THEN** the decision MUST be `edge_case` with `edge_case: "worktree_uncommitted"`,
- **AND** this decision MUST NOT be categorized as `would-block` or `pass`,
- **AND** the process MUST log the event and exit with code `0`.

### Requirement 6: Edge Case Bucket — Stale Cycle Guard
- **WHEN** the active workflow task string contains `"(nuevo ciclo detectado)"` (indicating a reset to Stage 1 on the same branch),
- **THEN** the decision MUST be `edge_case` with `edge_case: "stale_cycle_guard"`,
- **AND** this decision MUST NOT be categorized as `would-block` or `pass`,
- **AND** the process MUST log the event and exit with code `0`.

### Requirement 7: Stage 4 Missing OpenSpec Evaluation (`would-block`)
- **WHEN** none of the edge case conditions apply,
- **AND** the declared workflow stage in the active checkpoint is Stage 4 (`WorkflowStage::WorkTdd`),
- **AND** any of `proposal.md`, `spec.md`, or `tasks.md` are absent or empty under `openspec/changes/<feature>/`,
- **THEN** the decision MUST be `would-block`,
- **AND** the event MUST be appended to `~/.ce-ai/gate-events.jsonl` detailing the specific missing files,
- **AND** the process MUST exit with code `0` (the write is never prevented).

### Requirement 8: Stage 4 Complete Contract & Other Stages (`pass`)
- **WHEN** none of the edge case conditions apply,
- **AND** either:
  1. The declared stage is Stage 4 (`WorkflowStage::WorkTdd`) AND `proposal.md`, `spec.md`, and `tasks.md` all exist and are non-empty under `openspec/changes/<feature>/`, OR
  2. The declared stage is any stage other than Stage 4 (e.g., Stages 1, 2, 3, 5, 6, 7),
- **THEN** the decision MUST be `pass`,
- **AND** the event MUST be appended to `~/.ce-ai/gate-events.jsonl`,
- **AND** the process MUST exit with code `0`.

### Requirement 9: Dual Stdin and Argument Ingestion
- **WHEN** `ce-ai gate check` is executed with CLI flags `--tool <TOOL> --path <PATH>`,
- **THEN** the tool and path MUST be parsed directly from the CLI flags.
- **WHEN** the CLI flags are absent,
- **THEN** `ce-ai gate check` MUST read `stdin`, parsing JSON matching Claude Code `PreToolUse` (`tool_name`/`tool` and `tool_input.path`/`tool_input.file_path`/`input.path`/`input.file_path`).

### Requirement 10: Claude Code Hook Configuration Lifecycle
- **WHEN** `ce-ai init-prj` or `ce-ai install` configures Claude Code hooks,
- **THEN** it MUST ensure `.claude/settings.json` contains a `PreToolUse` hook matcher `Write|Edit` executing `ce-ai gate check`.
- **WHEN** `ce-ai deinit-prj` or `ce-ai uninstall` removes hooks,
- **THEN** it MUST surgically remove the `PreToolUse` hook without removing unrelated user hooks.

### Requirement 11: Privacy & Metrics Telemetry
- **WHEN** `ce-ai status` or `ce-ai doctor` is executed,
- **THEN** it MUST read `~/.ce-ai/gate-events.jsonl` and display aggregate counts:
  `gate-check: <total> observed (<would_block> would-block, <pass> pass, <undetermined> undetermined, <edge_case> edge-case: <mtime> mtime_fallback, <uncommitted> worktree_uncommitted, <stale> stale_cycle_guard)`.
- **AND** it MUST NOT print or expose any file contents, code buffers, or modifications.
