# Proposal: Grok Adapter Audit Refinements

- **Goal**: Address audit findings for the xAI Grok Build CLI native harness adapter.

## Audit Findings Addressed
1. **Flaky Test Environment Race Fix**: In `src/harness/grok.rs`, synchronize environment variable access (`GROK_HOME`) in unit tests using a static `Mutex` lock to prevent intermittent test failures caused by parallel thread execution.
2. **Dead Code Cleanup in `generic_json.rs`**: Remove dead legacy `HarnessKind::Grok` mapping (`.grok/config.json`) from `src/harness/generic_json.rs` and update associated unit tests.
