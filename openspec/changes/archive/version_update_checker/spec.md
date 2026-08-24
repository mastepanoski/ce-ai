# OpenSpec Specification: Version Update Checker

## Specifications

### VC-1: Upstream Release Tag Resolution
- **WHEN** `ce-ai status` or TUI `Status & Harnesses` is loaded,
- **THEN** `ce-ai` MUST report the latest available upstream GitHub release tag alongside installed harness versions.

### VC-2: Local Source Status Clarity
- **WHEN** a harness is installed with `source: local`,
- **THEN** `ce-ai` MUST display `(source: local)` AND show the latest GitHub release tag (e.g. `latest release: v2.5.0`).

### VC-3: Upgrade Recommendation Prompt
- **WHEN** a local source or older release version is detected,
- **THEN** `ce-ai` MUST display a recommendation: `💡 Recommendation: Run 'ce-ai upgrade' to update to latest release vX.Y.Z.`
