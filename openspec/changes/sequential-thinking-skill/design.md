# Design: Canonical Sequential-Thinking Skill Integration

## System Architecture & Data Design

### 1. Frontmatter Format Distinction & Unified Schema

There are two related frontmatter conventions operating across the ecosystem:

| Field | Claude Code / Plugin Convention | `ce-ai` `SkillRegistry` (`SkillFrontmatter`) | Purpose / Consumer |
| :--- | :--- | :--- | :--- |
| `name` | Required | Required | Canonical skill identifier (`sequential-thinking`). |
| `description` | Required | Optional (defaults to empty) | Summary shown in skill listings and prompt injection blocks. |
| `argument-hint` | Optional | Ignored by parser (`_ => {}`) | Parameter placeholder hint displayed in slash-command completions. |
| `scope` | Unused | Optional (defaults to tier scope) | Tier boundary (`global` or `project`). |
| `triggers` | Unused | Optional (defaults to empty) | List of regex/fuzzy match keywords for `skills resolve`. |

#### Unified Frontmatter Contract
The authored `SKILL.md` implements a superset format satisfying both systems simultaneously:

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
  - "root cause diagnosis"
---
```

When parsed by [`parse_skill_frontmatter`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/registry.rs#L450-L509):
- `name` -> `"sequential-thinking"`
- `description` -> `"Dynamic, reflective step-by-step problem solving and hypothesis refinement"`
- `scope` -> `"global"`
- `triggers` -> `vec!["complex reasoning", "debugging intricate bugs", ...]`
- `argument-hint` -> safely bypassed by the match default arm without warning or error.

---

### 2. Protocol Content Specification (`skills/sequential-thinking/SKILL.md`)

The markdown body establishes a disciplined, chain-of-thought protocol that guiding models execute natively in their context window:

```markdown
# Sequential Thinking Protocol

Dynamic, reflective step-by-step problem solving and hypothesis refinement. Use this protocol when navigating complex, non-linear reasoning challenges, architectural trade-offs, or intricate debugging.

## Core Operational Mechanics

1. **Step Progression & Thought Tracking**:
   - Maintain explicit progression: `Thought [N] / Estimated [M]`.
   - Update `Estimated [M]` dynamically as problem complexity expands or narrows.
2. **Hypothesis Formulation & Testing**:
   - State hypotheses explicitly before evaluating data.
   - For each hypothesis, identify required supporting evidence and potential falsification criteria.
3. **Dynamic Revision & Branching**:
   - When new evidence invalidates earlier assumptions, explicitly declare a revision:
     `Revision: Revising Thought [K] because [Reason]`.
   - Branch exploration when multiple plausible hypotheses exist, evaluating alternatives systematically.
4. **Negative Evidence & Falsification**:
   - Actively search for contradictory facts before finalizing conclusions.
   - Distinguish verified facts from unverified assumptions.
5. **Synthesis & Convergence**:
   - Transition to synthesis only after all competing hypotheses have been confirmed or falsified.
   - Produce a definitive final conclusion and actionable next steps.
```

---

### 3. File Distribution & Seeding Architecture

```
ce-ai repository
├── skills/
│   └── sequential-thinking/
│       └── SKILL.md                 # Physical file authored in repo
src/
├── source/
│   ├── builtin_skills.rs            # Embedded compile-time fallback constant
│   └── cache.rs                     # managed_tree harvests skills/ prefix
├── commands/
│   ├── install.rs                   # Seeds skill to managed dir; runs sync_registry
│   └── sync.rs                      # Seeds/updates skill; runs sync_registry
└── source/
    └── tools_registry.rs            # is_skill_configured auto-resolves to true
```

#### A. Embedded Fallback Constant (`src/source/builtin_skills.rs`)
```rust
//! Embedded builtin skills packaged directly with the binary.

/// Canonical embedded sequential-thinking skill markdown content.
pub const BUILTIN_SEQUENTIAL_THINKING_SKILL: &str =
    include_str!("../../skills/sequential-thinking/SKILL.md");

/// Relative path within the managed skills hierarchy.
pub const SEQUENTIAL_THINKING_REL_PATH: &str = "skills/sequential-thinking/SKILL.md";
```

#### B. Installation & Synchronization Seeding Logic
In [`src/commands/install.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/install.rs) and [`src/commands/sync.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/sync.rs):
1. When copying managed assets from `managed_tree`:
   - If the source tree contains `skills/sequential-thinking/SKILL.md`, it is copied directly.
   - If the source tree (e.g. an upstream release tarball) does *not* contain it, `ce-ai` seeds it using `crate::source::builtin_skills::BUILTIN_SEQUENTIAL_THINKING_SKILL`.
2. Target path resolution:
   - For OpenCode/standard harnesses: `<config_dir>/compound-engineering/skills/sequential-thinking/SKILL.md`.
   - For Custom harness: `<cfg.skills_dir>/sequential-thinking/SKILL.md`.
   - For global fallback: `<ctx.config_dir>/skills/sequential-thinking/SKILL.md`.
3. All file writes use `crate::state::write_atomic` and respect `if !ctx.dry_run`.

---

### 4. Registry Indexing & Discovery Flow

1. **`SkillRegistry::build(ctx)`**:
   - Scans `ctx.config_dir.join("skills")` (Tier 4 Global) and `ctx.opencode_config_dir.join("compound-engineering").join("skills")`.
   - Encounters subdirectory `sequential-thinking/` containing `SKILL.md`.
   - Calculates SHA256 digest: `sha256 = compute_file_sha256(&path)`.
   - Invokes `parse_skill_frontmatter(&content)`.
   - Inserts `SkillEntry` with mapped paths for all harnesses into `skill_map`.
   - Persists catalog atomically to `~/.ce-ai/skills-registry.json`.
2. **`ce-ai skills resolve sequential-thinking`**:
   - Queries `SkillRegistry` for `"sequential-thinking"`.
   - Verifies physical file exists and SHA256 matches index.
   - Generates dual output:
     - Machine-readable status: `status=paths-injected`.
     - Prompt injection markdown:
       ```markdown
       <!-- ce-ai:skill_resolution status=paths-injected -->
       ## Skills to load before work:
       - **sequential-thinking**: Dynamic, reflective step-by-step problem solving and hypothesis refinement
         Path: `file:///Users/.../skills/sequential-thinking/SKILL.md`
       ```

---

### 5. Diagnostic Integration & Zero-Ad-Hoc Behavior

In [`src/source/tools_registry.rs:348-367`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/source/tools_registry.rs#L348-L367):
```rust
pub fn is_skill_configured(ctx: &Context, name: &str) -> bool {
    if is_mcp_server_configured(ctx, name) {
        return true;
    }
    let registry_path = ctx.config_dir.join("skills-registry.json");
    if let Ok(reg) = crate::source::registry::SkillRegistry::load(&registry_path) {
        if reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
            return true;
        }
    }
    // ...
```

Once `sequential-thinking` is indexed:
1. `reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case("sequential-thinking"))` returns `true`.
2. `is_skill_configured(&ctx, "sequential-thinking")` returns `true`.
3. In [`src/commands/doctor.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/doctor.rs) (`check_companion_tools_freshness`) and [`src/commands/tools.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/tools.rs), the check marks the skill as configured and suppresses the unconfigured diagnostic recommendation.
4. **No code changes are required in `tools_registry.rs`, `doctor.rs`, or `tools.rs`.**
