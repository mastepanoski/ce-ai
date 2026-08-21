# OpenSpec Exploration: Technical Options

1. **Release Automation**: Use GitHub Actions with `cargo-release` or custom tag publisher to build release binaries (`cargo build --release`) and publish via `gh release create`.
2. **Sync Verification**: Enhance `sync_with` to return `SyncAuditReport` containing `(harness, file_path, action, sha256_match: bool)` and print clean itemized verification tables.
