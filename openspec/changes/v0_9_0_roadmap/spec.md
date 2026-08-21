# OpenSpec Requirements: Release v0.9.0 Specifications

## Feature 1: Security Threat Matrix & Path Traversal Verification (ISO 27001 / ISO 27002)

### Requirement 1.1: Path Traversal Rejection
- **WHEN** an archive contains directory traversal relative or absolute paths,
- **THEN** `source::archive::safe_extract` MUST reject the entry prior to writing any bytes to disk.

### Requirement 1.2: Atomic State Write Integrity
- **WHEN** `State::save` or `write_atomic` is invoked,
- **THEN** the system MUST write to a temporary file first and execute an atomic rename, leaving zero temp artifacts.

---

## Feature 2: High-Performance Benchmarks (< 50ms Target)

### Requirement 2.1: Execution Time Limit
- **WHEN** `State::load_with_workspace_overrides` and `InstallManifest::load` are invoked across a repository,
- **THEN** state resolution and diff computation MUST complete in under 50 milliseconds.
