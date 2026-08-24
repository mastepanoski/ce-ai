# Specification: Grok Adapter Audit Refinements

## Acceptance Criteria

### R1: Thread-Safe Environment Unit Tests
- WHEN running unit tests for `GrokAdapter` in parallel THEN environment variable mutations (`GROK_HOME`) MUST be thread-synchronized via a static `Mutex` guard to prevent test race conditions.

### R2: Elimination of Legacy Generic JSON Mapping
- WHEN inspecting `src/harness/generic_json.rs` THEN `HarnessKind::Grok` MUST NOT be mapped to generic JSON files (`.grok/config.json`), and the module docstring MUST be updated.
