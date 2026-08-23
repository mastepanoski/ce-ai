# OpenSpec Exploration: Technical Investigation & Architecture Options

- **Change:** `companion-tool-readiness-and-freshness`
- **Issue:** #112
- **Author:** Antigravity AI
- **Date:** 2026-08-22

---

## 🔍 1. Technical Investigation

### A. How Version Extraction Works Across Companion Tools
Each companion tool binary exposes a standard `--version` CLI flag:

1. **Engram (`engram --version`)**: Emits `engram 1.2.0` or `1.2.0`.
2. **CodeGraph (`codegraph --version`)**: Emits `codegraph 0.5.0` or `0.5.0`.
3. **RTK (`rtk --version`)**: Emits `rtk 0.2.1`.
4. **Context7 (MCP Server)**: Checked via configuration entry in `opencode.json` or `claude.json` (`~/.config/opencode/opencode.json` mcpServers map) and binary presence.
5. **`ce-ai` (`ce-ai --version`)**: Evaluates `env!("CARGO_PKG_VERSION")` (`1.6.3`).

### B. Version Parsing & SemVer Comparison
- Use standard SemVer parsing (`semver::Version`) or clean version string token extraction (`split_whitespace()`, stripping `v` prefix).
- Compare `installed_version` against `min_expected_version` and `latest_version`.
- Status Matrix:
  - If binary not in PATH / config missing $\rightarrow$ `Status::Missing`.
  - If `installed_version >= latest_version` $\rightarrow$ `Status::Ok`.
  - If `installed_version < latest_version` $\rightarrow$ `Status::Outdated { current, expected }`.
  - If network check failed / offline $\rightarrow$ `Status::Offline { current }`.

---

## 🏗️ 2. Evaluated Architecture Options

### Option 1: Embedded Registry + Local TTL Cache (Chosen)
- **Concept**: `ce-ai` ships an embedded base manifest (`tools_registry.rs`) containing pinned fallback versions. On `tools status` or `doctor`, it attempts a non-blocking HTTP fetch with a 500ms timeout to GitHub/registry to update `~/.ce-ai/cache/companion-registry.json` (TTL: 24h).
- **Pros**: 100% resilient when offline; instantaneous execution when cached; always up to date when online.
- **Cons**: Requires a small TTL cache manager struct.

### Option 2: Live HTTP Request on Every Execution
- **Concept**: Perform a live HTTP call on every `doctor` or `tools status` invocation.
- **Pros**: Simple code path without disk cache.
- **Cons**: Adds 100ms-500ms network latency to local `ce-ai doctor` runs; fails or slows down in offline/airplane mode environments.

### Option 3: Static Embedded Registry Only
- **Concept**: Embed fixed version strings in binary, updated only when `ce-ai` is compiled/upgraded.
- **Pros**: Zero network overhead.
- **Cons**: Cannot detect new companion tool releases between `ce-ai` release windows.

---

## 💡 3. Architectural Decision

**Decision**: Implement **Option 1 (Embedded Registry + 24h TTL Local Cache)**.
- **Primary Source**: `~/.ce-ai/cache/companion-registry.json`.
- **Fallback**: Embedded `src/source/tools_registry.rs` constants.
- **Network Grace Period**: 500ms timeout; on failure or offline, log `doctor-info: <tool> vX.Y.Z (offline)` without error exit codes.
