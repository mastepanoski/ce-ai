# OpenSpec Technical Exploration: Project Adoption Engine

## Technical Investigation & Architectural Tradeoffs

### 1. Marker-Delimited Managed Block Architecture
To inject instruction rules into project markdown files (`AGENTS.md`) without destroying existing user content, we evaluated three options:

- **Option A (Full File Management)**: `ce-ai` owns the entire `AGENTS.md` file.
  - *Tradeoff*: Destroys user customizations and custom project instructions. **REJECTED**.
- **Option B (HTML Comment Markers)**: Delimit managed blocks with versioned HTML comments:
  ```markdown
  <!-- ce-ai:block begin v=1 tier=full sha256=... -->
  ## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement
  [ce-brainstorm] ➔ [OpenSpec definition] ➔ [ce-plan] ➔ [TDD & ce-work]
  ➔ [Verification] ➔ [ce-compound] ➔ [ce-commit-push-pr]
  <!-- ce-ai:block end -->
  ```
  - *Tradeoff*: Standard markdown renderers hide HTML comments from visual output while AI agent parsers process the inner markdown text cleanly. **ACCEPTED**.

### 2. Instruction File Standardization Across Harnesses
- **Canonical Instruction File**: `AGENTS.md` is the cross-harness standard supported by OpenCode, Claude Code, Cursor, Codex, Copilot, and Pi.
- **Derived Reference Stubs**: Harnesses like Claude Code expect `CLAUDE.md`. Rather than duplicating full instruction content (which guarantees content drift), `CLAUDE.md` is generated as a thin reference stub:
  ```markdown
  @AGENTS.md
  ```
- **HarnessAdapter Extensions**: Extending `HarnessAdapter` trait in `src/harness/mod.rs` guarantees that adding new harnesses in the future automatically handles project instruction files without branching in `commands/init_prj.rs`.

### 3. Adoption Registry Data Schema in `state.json`
Storing adopted projects in global state (`~/.config/ce-ai/state.json` or `~/.ce-ai/state.json`) allows CLI commands (`status`, `doctor`, `uninstall`) to track adopted projects across the workstation:

```json
{
  "projects": [
    {
      "path": "/Users/mastepanoski/projects/web/ai/ce-ai",
      "file": "AGENTS.md",
      "tier": "full",
      "block_version": 1,
      "block_sha256": "a3f5...",
      "created_file": true,
      "adopted_at": "2026-08-22T00:20:00Z"
    }
  ]
}
```

### 4. Reversibility & Idempotency Guarantee
- **Idempotency**: Running `ce-ai init-prj` twice on an adopted repo produces zero file mutations and returns exit code 0.
- **Byte-for-Byte Restoration**: `ce-ai deinit-prj` strips only text between `<!-- ce-ai:block begin -->` and `<!-- ce-ai:block end -->`. If `created_file` is true and the file becomes empty after extraction, the file is deleted. If user content existed before init, the file is restored to its exact original byte sequence.
