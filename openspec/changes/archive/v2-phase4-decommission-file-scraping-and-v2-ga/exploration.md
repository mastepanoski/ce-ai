# Exploration: Phase 4 — Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0 GA)

## Technical Investigation & Context

### 1. The File Scraping Problem
In CE-AI v1, when a user invoked `ce-ai install` or `ce-ai upgrade`, the tool would:
1. Query GitHub releases for `EveryInc/compound-engineering-plugin`.
2. Download the source tarball (`*.tar.gz`).
3. Extract the tarball into `<config-dir>/cache/trees/<tag>/`.
4. Copy `plugins/compound-engineering.js` and `skills/` directly into harness directories (`~/.config/opencode/compound-engineering`, Claude, Codex, etc.).
5. Generate a `manifest.json` indexing all extracted files by SHA-256 hash.
6. Run `sync --watch` to poll these directories and overwrite any user edits or native updates that drifted from the manifest.

### 2. Maintainer Feedback & Structural Impact
As stated by the Compound Engineering maintainer:
> *"Copying `skills/` out of the release tarball skips the per-target rewrites the converter and native packaging do, so some hosts end up with content that isn't what we ship for them. Restoring user edits on a watch loop and running alongside a native install (which ce-ai already reports as `external-duplicate`) gives users two sources of truth. If you want a fleet or pinning layer, I'd have it drive each host's native installer and compare released versions rather than hash our files."*

With Phase 2 having delivered `src/fleet/` (with drivers for Claude, OpenCode, Codex, and Pi) and Phase 3 having delivered the read-only Advisory Workflow Observer, the file-scraping pipeline is completely obsolete.

### 3. Evaluated Options for CLI Transition

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| **Option A: Hard Delete `install` and `sync`** | Eliminates legacy code immediately; clean CLI surface. | Breaks legacy scripts and user habits abruptly; harsh upgrade shock. | **Ruled out** |
| **Option B: Silent Aliasing (redirect `sync` to `fleet sync`)** | Zero breakage for existing workflows. | Hides the paradigm shift; users don't learn native host installation semantics. | **Ruled out** |
| **Option C: Explicit Deprecation & Forwarding Guidance** | Clear user guidance towards `fleet` and `init-prj`; maintains backwards compatibility for dry-run inspection; clean upgrade path. | Requires maintaining deprecation adapters during v2.x. | **Selected** |

### 4. Binary Self-Update Distinction
`ce-ai self-update` also uses archive extraction, but it extracts the compiled `ce-ai` binary release for its own distribution, not Compound Engineering skills. The security utilities (`is_safe_relative_path` and `extract_safe`) in `src/source/archive.rs` protect against directory traversal (zip-slip) and must be preserved for `binary_release.rs` and security compliance.
