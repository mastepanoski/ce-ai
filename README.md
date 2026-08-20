# `ce-ai` — Compound Engineering Plugin Manager CLI

`ce-ai` is a fast, safe Rust CLI for installing, syncing, upgrading, and managing model assignments for the **compound-engineering plugin** across AI agent harnesses (starting with OpenCode v1).

## Features

- **Direct OpenCode Integration**: Installs CE loader and registers skills without clobbering existing configuration (`~/.config/opencode/opencode.json`).
- **Safe Extraction & Caching**: Validates tarball structures against path traversal attacks (zip-slip prevention) and maintains SHA256-verified release caches.
- **Model Assignments & Profiles**: Set models per agent slot (`sdd-explore`, `sdd-design`, etc.) and take append-only snapshot profiles.
- **Diff & Reconcile (Sync)**: Inspect drift, preview planned updates with `--dry-run`, and repair modified or deleted managed assets.
- **Atomic Rollback & Backups**: Pre-mutation state is automatically backed up (`~/.ce-ai/backups/`) and clean uninstallation restores pre-install configuration.
- **Health Doctor**: Diagnose configuration errors, drift, and state inconsistency.

## Installation

Build from source with Cargo:

```bash
cargo build --release
```

Or install binary directly:

```bash
cargo install --path .
```

## Usage

### 1. Install Plugin

Install from the latest GitHub release (default):

```bash
ce-ai install --harness opencode
```

Install from a local source repository or directory:

```bash
ce-ai install --harness opencode --source /path/to/compound-engineering-plugin
```

Preview changes without modifying disk:

```bash
ce-ai install --harness opencode --dry-run
```

### 2. View Status & Check Health

Check installed harness versions and status:

```bash
ce-ai status
```

Run health checks:

```bash
ce-ai doctor
```

### 3. Reconcile Drift & Upgrade

Reconcile local changes against source manifest:

```bash
ce-ai sync
```

Upgrade plugin to a specific version tag:

```bash
ce-ai upgrade --to compound-engineering-v3.5.0
```

### 4. Configure Models & Profiles

Assign a model to a specific slot:

```bash
ce-ai models set sdd-explore opencode-go/kimi-k2.6
```

List current model assignments:

```bash
ce-ai models list
```

Save or load named model profiles:

```bash
ce-ai models profile save my-profile
ce-ai models profile load my-profile
```

### 5. Uninstall

Restore original configuration and remove managed plugin files:

```bash
ce-ai uninstall --harness opencode
```

## Testing

Run unit and CLI integration tests:

```bash
cargo test
```

Run Docker containerized E2E gate test:

```bash
make e2e
```

## License

MIT / Apache-2.0
