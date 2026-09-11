# Exploration: Canonical Sequential-Thinking Skill Integration

## Technical Investigation & Context

### 1. Issue #309 & Parent #299 Context

Issue #299 sequenced three sub-issues to address companion tool and harness integration:
- Issue #307: Closed registration parity gap for `engram` and `codegraph` across Custom and OpenCode harnesses (delivered in v1.41.0).
- Issue #308: Auto-configured `rtk` hook injection for natively supported harnesses (delivered in v1.42.0).
- Issue #309: Evaluated the integration model for `sequential-thinking`.

Unlike `engram` (persistent SQLite state) or `codegraph` (SCIP AST index querying), `sequential-thinking` has no native binary, local database, or OS-level state. The official reference implementation (`@modelcontextprotocol/server-sequential-thinking`) is an in-memory Node.js state machine whose sole function is accepting JSON objects with thought numbers, hypotheses, and revision tags, returning the same thought back to the LLM.

### 2. Decision Analysis: Option (a) vs Option (b)

- **Option (a): Promote to Registered MCP Server**:
  - *Drawbacks*: Requires installing `@modelcontextprotocol/server-sequential-thinking` via `npm`/`npx`, introducing Node.js runtime dependencies across all environments. Spawns an external daemon per agent session that consumes memory and process table slots. Incurs IPC overhead on every thought step. Context window bloat from JSON tool-call schema serialization and RPC payloads. Violates the strict No-MCP architectural constraint of the Pi harness.
  - *Empirical Cognitive Findings*: LLM reasoning benchmarks show modern foundation models (Gemini 2.5/3, Claude 3.5/3.7, GPT-4o) do not benefit from round-tripping thoughts through an external JSON RPC boundary compared to structured reasoning instructions directly in context.
- **Option (b): Maintain On-Demand Skill Model**:
  - *Benefits*: Zero external daemons, zero Node.js dependency, zero process spawning. Universal compatibility across all 12 supported harnesses, including Pi (No-MCP). Cognitive instructions load directly into model context as a system prompt / guidance block.
  - *Veredict*: Option (b) was formally approved by user consensus.

### 3. Resolution Gap Analysis: Sub-option 1 vs Sub-option 2

Upon reproducing `ce-ai skills resolve sequential-thinking`, the command was found to degrade to `status=none` with an empty resolution list. Two approaches were explored to resolve this correctness gap:

- **Sub-option 2 (Manual Setup Instructions / JSON Snippets)**:
  - Returning JSON snippets or instructions in `skills resolve` violates the purpose of the command, which is to return `file://` URIs for prompt injection. It creates a dead-end for headless agent orchestrators and pushes the user toward the external Node.js daemon previously rejected.
- **Sub-option 1 (Canonical `SKILL.md` Distribution)**:
  - Deliver a real `skills/sequential-thinking/SKILL.md` file through `ce-ai`'s native asset pipeline. When resolved, it outputs `status=paths-injected` with a verified `file://` URI to an actionable reasoning protocol.
  - Selected approach: Sub-option 1.

### 4. Code Architecture Investigation

#### A. Skill Registry Pipeline
In [`src/source/registry.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs):
- `SkillRegistry::build`: Scans configured skill directories (`~/.ce-ai/skills/`, `~/.config/opencode/compound-engineering/skills/`, workspace roots).
- `scan_skill_directory`: Identifies `<dir>/<name>/SKILL.md` or `<dir>/SKILL.md`.
- `process_skill_file`: Computes SHA256 of the file, parses YAML frontmatter via `parse_skill_frontmatter`, maps paths across all harnesses, and stores entries in `skills-registry.json`.
- `SkillRegistry::resolve`: Searches for matching skill name/description/triggers. Checks `path.exists()` and matches current SHA256 against indexed digest. If verified, emits `status=paths-injected` and `file://<path>`.

#### B. Asset Harvesting via `managed_tree`
In [`src/source/cache.rs:64-89`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/cache.rs#L64-L89):
- Scans `source_root` for paths starting with `skills/` or `.opencode/skills/`.
- Files matching `skills/<name>/...` are collected with their relative path and SHA256.
- During `ce-ai install` and `ce-ai sync`:
  - OpenCode/standard harnesses copy files into `<config_dir>/compound-engineering/skills/<name>/...`.
  - Custom harness copies files into `<cfg.skills_dir>/<name>/...`.
  - `SkillRegistry::sync_registry(ctx)` builds and saves `skills-registry.json`.

#### C. Built-in Fallback Seeding Pattern
In [`src/opencode/plugins.rs:17`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/opencode/plugins.rs#L17), `ce-ai` embeds `BUILTIN_LOADER` via `include_str!`. When the source tree lacks the physical loader, `install_loader` falls back to the embedded content.
Mirroring this pattern for `sequential-thinking`:
- Embed `skills/sequential-thinking/SKILL.md` in `src/source/builtin_skills.rs` (or `registry.rs`) via `pub const BUILTIN_SEQUENTIAL_THINKING_SKILL: &str = include_str!("../../../skills/sequential-thinking/SKILL.md");`.
- During `install` and `sync`, if `sequential-thinking/SKILL.md` is not present in the resolved `source_path`, seed it into `~/.ce-ai/skills/sequential-thinking/SKILL.md` (or the target harness skill dir) using `write_atomic`.
- This ensures hermetic reliability across development, local testing, and production homebrew distribution without relying on upstream release latency.

### 5. Frontmatter Differences & Harmonization

- **Claude Code / Plugin Harness Convention**:
  - Uses `name`, `description`, and `argument-hint` (e.g. `argument-hint: "[thought or problem to analyze]"`).
  - Used by slash-command dispatchers to display interactive parameter hints.
- **`ce-ai`'s `SkillRegistry` Convention ([`SkillFrontmatter`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L48-L54))**:
  - Specifically parses: `name`, `description`, `scope`, `triggers`.
  - Used for fuzzy keyword matching, lifecycle precedence tiering, and discovery.
- **Harmonization Strategy**:
  - The frontmatter parser in `parse_skill_frontmatter` ignores unknown keys safely.
  - The authored `SKILL.md` will supply **both sets of fields**:
    ```yaml
    ---
    name: sequential-thinking
    description: "Dynamic, reflective step-by-step problem solving and hypothesis refinement"
    argument-hint: "[thought or problem to analyze]"
    scope: "global"
    triggers:
      - "complex reasoning"
      - "debugging intricate bugs"
      - "architectural analysis"
      - "sequential thought"
      - "hypothesis testing"
    ---
    ```
  - This guarantees 100% compliance with `SkillFrontmatter` indexing in `ce-ai` while simultaneously presenting clean metadata to plugin-level slash command parsers.
