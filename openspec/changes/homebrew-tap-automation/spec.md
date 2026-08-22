# Spec: Automated Homebrew Tap Updates

## ADDED Requirements

### Requirement: Automatic tap bump on release

The system SHALL automatically update the Homebrew tap formula when a
GitHub release is published.

#### Scenario: Release published triggers formula update

- **WHEN** a GitHub release for tag `vX.Y.Z` is published in
  `mastepanoski/ce-ai`
- **THEN** the `bump-homebrew` workflow runs, renders
  `Formula/ce-ai.rb` with version `X.Y.Z`, four download URLs pointing at
  the release assets, and their SHA256 digests
- **AND** pushes the formula to `mastepanoski/homebrew-ce-ai` branch `main`

### Requirement: Checksum integrity before publish

The workflow SHALL verify all four platform assets are downloadable and
hashable before writing anything to the tap repository.

#### Scenario: Missing asset aborts update

- **WHEN** one of the four platform tarballs is missing from the release
- **THEN** the workflow fails
- **AND** the tap repository is left unchanged

### Requirement: Manual re-run support

The workflow SHALL support manual invocation with an explicit tag.

#### Scenario: Workflow dispatch with tag input

- **WHEN** an operator runs `bump-homebrew` via `workflow_dispatch` with
  input `tag: v1.0.9`
- **THEN** the workflow updates the formula for that exact tag

### Requirement: Idempotent re-runs

Re-running the workflow for an already-published version SHALL NOT fail.

#### Scenario: Formula already at target version

- **WHEN** the rendered formula is byte-identical to the tap's current
  formula
- **THEN** no new commit is created and the run succeeds
