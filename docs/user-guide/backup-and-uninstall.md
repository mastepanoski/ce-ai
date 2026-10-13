# Backup & Uninstall

> **Intent**: How-to — back up, restore, de-adopt projects, and cleanly remove `ce-ai`, harnesses, and companion tools without data loss. For installation, see the [Installation & Coexistence Guide](installation-and-coexistence-mechanisms.md).

`ce-ai` provides safeguards before modifying any configuration: it creates timestamped pre-mutation backups in `~/.ce-ai/backups/` and uses atomic writes (`write_atomic`) to guarantee configurations are never corrupted.

---

## 1. Automatic Pre-Mutation Backups

Before modifying any existing harness configuration file (e.g. `opencode.json`, `.claude.json`, `.cursorrules`), `ce-ai`:

1. Checks whether pre-existing configuration files exist.
2. Creates an immutable, timestamped copy inside `~/.ce-ai/backups/<utc-timestamp>/`.
3. Records the backup pointer in `install-manifest.json` and `state.json`.

You can inspect existing backups at any time:
```bash
# List all recorded backups
ce-ai backups list
```

---

## 2. De-Adopting a Specific Project (`ce-ai deinit-prj`)

If you want to remove `ce-ai` governance from a specific repository without affecting your global tools or other projects:

```bash
cd your-project
ce-ai deinit-prj
```

What `ce-ai deinit-prj` does:
- **Surgically strips managed blocks**: removes the delimited Compound Engineering block from `AGENTS.md` (or deletes the file if `ce-ai` created it).
- **Cleans derived stubs**: removes managed blocks from `CLAUDE.md`, `.cursor/rules/`, and `.github/copilot-instructions.md`.
- **Removes session hooks**: unregisters session-start auto-resume hooks and RTK terminal hooks for the repository.
- **Updates project registry**: unregisters the repository from `~/.ce-ai/state.json`.
- **Zero code loss**: user source code, commits, and custom rules outside the managed block remain 100% untouched.

---

## 3. Uninstalling from Coding Agent Harnesses (`ce-ai uninstall`)

To remove Compound Engineering configurations and restore original harness settings:

### Uninstall from a Single Harness
```bash
ce-ai uninstall --harness claude
# Or:
ce-ai uninstall --harness opencode
```

### Uninstall from All Harnesses in One Step
```bash
ce-ai uninstall --harness all --yes
```

What `ce-ai uninstall` does:
1. **Restores original config**: locates the pre-install backup in `~/.ce-ai/backups/` and atomically restores your pre-existing configuration.
2. **Removes managed assets**: deletes managed loaders and skill directories (`<harness-config>/compound-engineering/`).
3. **Preserves user settings**: custom user plugins, skills, and API keys outside the managed block are retained.
4. **Updates global state**: marks the harness as uninstalled in `~/.ce-ai/state.json`.

---

## 4. Removing Companion Tools (`codegraph`, `engram`, `rtk`)

If you also wish to clean up companion tools:

### CodeGraph
```bash
# In your repository: remove local index
rm -rf .codegraph/

# On macOS/Linux: remove binary
rm -f $(which codegraph 2>/dev/null)
```

### Engram (Persistent Memory)
```bash
# Remove Engram database and memory cache
rm -rf ~/.engram

# Remove binary (if installed via Homebrew or Go)
brew uninstall engram 2>/dev/null || rm -f $(which engram 2>/dev/null)
```

### RTK (Token Reducer)
```bash
# If installed via Cargo or Homebrew:
cargo uninstall rtk 2>/dev/null || brew uninstall rtk 2>/dev/null || rm -f $(which rtk 2>/dev/null)
```

---

## 5. Complete Removal of `ce-ai`

To completely remove `ce-ai` from your machine:

```bash
# 1. Restore all harness configurations cleanly
ce-ai uninstall --harness all --yes

# 2. Remove the ce-ai CLI binary by platform:

# macOS (Homebrew):
brew uninstall ce-ai

# Linux / Unix (Standalone):
rm -f ~/.local/bin/ce-ai ~/.ce-ai/bin/ce-ai

# Windows (PowerShell):
# Remove-Item "$env:USERPROFILE\.ce-ai\bin\ce-ai.exe"

# 3. Purge local configuration and backups (optional)
rm -rf ~/.ce-ai
```

