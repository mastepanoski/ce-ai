# Step-by-Step Guide: Sync & Reconcile and Upgrade Release Mechanisms

This guide explains in detail the step-by-step internal execution flow of **Sync & Reconcile** (`ce-ai sync`) and **Upgrade Release** (`ce-ai upgrade`) in `ce-ai`.

---

## 1. Sync & Reconcile Mechanism (`ce-ai sync`)

The `ce-ai sync` command (or the `Sync & Reconcile` tab in the TUI) is responsible for **guaranteeing managed asset integrity** and repairing any accidental modifications or deleted files (drift) across your AI tools (`OpenCode`, `Claude Code`, `Pi`, `Cursor`, `Copilot`, `Kimi`, `Antigravity`, etc.).

### 🛠️ Step-by-Step Sync Workflow

```mermaid
flowchart TD
    A[Start: ce-ai sync] --> B[Step 1: Read Manifest & Target Source Tree]
    B --> C[Step 2: Compare SHA256 File Hashes]
    C --> D[Step 3: Plan & Inspect with --dry-run]
    D --> E[Step 4: Atomic Disk File Writes]
    E --> F[Step 5: Propagate Across Active Host Harnesses]
    F --> G[Step 6: Output SHA256 Sync Verification Matrix]
```

#### Step 1: Read Manifest & Target Source Tree
- `ce-ai` reads the installation manifest (`install-manifest.json`) located in `~/.config/opencode/` to determine the registered source tree (a GitHub release tarball or a local source directory).
- Scans skill files (`skills/`) and loaders (`plugins/compound-engineering.js`) in the source.

#### Step 2: Compare SHA256 Hashes (Drift Detection)
- Computes the **SHA256** hash for each managed asset on disk and compares it against the desired source:
  - **Copy**: If a managed file is missing, it is marked to be copied.
  - **Restore**: If a file was locally modified or corrupted, it is marked for restoration.
  - **Remove**: If a file is stale or deleted in the target version, it is marked for removal.

#### Step 3: Dry-Run Inspection (`--dry-run`)
- When running `ce-ai sync --dry-run`, `ce-ai` outputs the exact planned diff actions without writing any data to disk.

#### Step 4: Safe Atomic Disk Writes
- Disk updates use an **atomic write pattern** (`write_atomic`): writes to a temporary file first and performs an atomic `rename`, ensuring system crashes or power interruptions never leave corrupt partial files on disk.

#### Step 5: Propagate Across All Active Host Harnesses
- Merges and maintains skill paths and plugin entries in the configuration of **all installed harnesses on your machine** (`opencode.json`, `claude.json`, `config.json`, `antigravity.json`, `.cursorrules`, etc.).

#### Step 6: Output SHA256 Sync Verification Matrix
- Upon completion, `ce-ai` outputs an itemized integrity verification audit table:
  ```text
  == [Sync Verification Matrix] ==
  version: v0.4.0
  source: github-release
    ✓ harness 'opencode': synced & verified (12 files, SHA256 integrity match)
    ✓ harness 'claude': synced & verified (12 files, SHA256 integrity match)
    ✓ harness 'agy': synced & verified (12 files, SHA256 integrity match)
    ✓ harness 'kimi': synced & verified (12 files, SHA256 integrity match)
  reconciliation status: 100% Verified (0 drift)
  ```

---

## 2. Upgrade Release Mechanism (`ce-ai upgrade`)

The `ce-ai upgrade` command (or the `Upgrade Release` tab in the TUI) fetches the **latest official release of the Compound Engineering Plugin published on GitHub** and safely updates all active host harnesses.

### 🚀 Step-by-Step Upgrade Workflow

```mermaid
flowchart TD
    A[Start: ce-ai upgrade / TUI Upgrade Release] --> B[Step 1: Query GitHub Release API]
    B --> C[Step 2: Download & Cache SHA256 Release Tarball]
    C --> D[Step 3: Extract Tarball Safely Anti Zip-Slip]
    D --> E[Step 4: Convert Local Sources to GitHub Release]
    E --> F[Step 5: Run Sync Engine Across All Active Harnesses]
    F --> G[Step 6: Update Global State File state.json]
```

#### Step 1: Query GitHub Release API
- `ce-ai` queries GitHub API to resolve the latest release tag of `everyinc/compound-engineering-plugin` (or the specific tag supplied via `--to <tag>`).

#### Step 2: Download & Cache Tarball (`~/.ce-ai/cache/`)
- Downloads the official `.tar.gz` archive and computes its SHA256 digest.
- Caches the file at `~/.ce-ai/cache/ce-<sha256>.tar.gz` to enable fast offline re-installs and test runs.

#### Step 3: Zip-Slip Safe Extraction
- Inspects each entry inside the `.tar.gz` archive before unpacking.
- **Security Check**: Rejects any entry containing path traversal sequences (`../`, absolute paths `/etc/`), preventing arbitrary file overwrite attacks.

#### Step 4: Transparent Conversion from Local Sources (`source: local`)
- If a harness was previously installed from a local development tree (`dev`), `ce-ai upgrade` displays a notice:
  `notice: upgrading harnesses with local source to latest GitHub release.`
- Automatically converts the installation from local source code to the latest official stable GitHub release tag.

#### Step 5: Execute Sync Engine Across All Active Harnesses
- Triggers the `sync` engine (detailed in Section 1) to update skills, loaders, and configuration entries across **all** active AI agent harnesses installed on your system.

#### Step 6: Update Global State (`state.json`)
- Updates `~/.ce-ai/state.json` recording the new release tag, synchronization timestamp (`last_synced_at`), and manifest index.

---

## 📊 Comparison Matrix: When to use Sync vs Upgrade?

| Feature | 🔄 Sync & Reconcile (`ce-ai sync`) | 🚀 Upgrade Release (`ce-ai upgrade`) |
| :--- | :--- | :--- |
| **Purpose** | Repair drift, recover deleted or modified files. | Update plugin to a newer GitHub release. |
| **Data Source** | Currently registered source tree (local or cache). | Queries and downloads latest GitHub Releases. |
| **TUI Usage** | Press **`[Enter]`** in `Sync & Reconcile` tab. | Press **`[Enter]`** in `Upgrade Release` tab. |
| **Result** | Files 100% identical to currently installed version. | Files updated to latest official GitHub release tag. |
