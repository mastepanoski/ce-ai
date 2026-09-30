# Specification: Cargo Build Profile Optimization

## Requirements

### Requirement 1: Compact Debug Profile
- **WHEN** building or testing in `dev` or `test` profile,
- **THEN** Cargo MUST emit line-tables-only debug information (`debug = "line-tables-only"`), reducing artifact size while keeping stack trace file and line numbers intact.

### Requirement 2: Incremental Cache Prevention
- **WHEN** running development and test builds,
- **THEN** Cargo MUST NOT generate cumulative session directories in `target/debug/incremental/` (`incremental = false`).

### Requirement 3: Sweep Maintenance Rule
- **WHEN** a developer executes `make sweep`,
- **THEN** it MUST ensure `cargo-sweep` is installed and execute `cargo sweep --time 14` to prune dormant artifacts without wiping the active build.
