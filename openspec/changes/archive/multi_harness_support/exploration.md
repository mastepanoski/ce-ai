# OpenSpec Exploration: Technical Options & Tradeoffs

**Change Identifier:** `multi_harness_support`  

---

## 1. Technical Investigation

### Harness Configuration Categories

1. **Category A: Structured JSON Config Harnesses**
   - *Harnesses*: OpenCode (`opencode.json`), Claude (`.claude.json`), Pi (`.pi/config.json`), Codex, Grok, Kimi, AGY, DeepSeek.
   - *Requirement*: Parse JSON AST, inject/update target arrays (`plugins`, `skills`, `models`), retain all unknown/user keys, and re-serialize formatted JSON atomically.

2. **Category B: Markdown Rule / Prompt Instruction Harnesses**
   - *Harnesses*: Cursor (`.cursorrules`, `.cursor/rules/`), Copilot (`.github/copilot-instructions.md`).
   - *Requirement*: Inject a demarcated comment block containing managed skill/role directives:
     ```markdown
     <!-- CE-AI MANAGED BLOCK BEGIN -->
     # Compound Engineering Directives...
     <!-- CE-AI MANAGED BLOCK END -->
     ```
   - On sync/update: Replace text strictly between `BEGIN` and `END` tags.
   - On uninstall: Strip the demarcated block cleanly.

3. **Category C: Generic Custom Harness Fallback Mode**
   - *Harnesses*: `--harness custom` (e.g. `fx.sh`, custom internal tools).
   - *Requirement*: Require or prompt for `--plugins-dir`, `--skills-dir`, and optional `--rules-file`. Register custom entry in `state.json`.

---

## 2. Evaluated Architecture Options

### Option 1: Modular `HarnessAdapter` Trait (SELECTED)
Define a Rust trait `HarnessAdapter`:
```rust
pub trait HarnessAdapter {
    fn name(&self) -> HarnessKind;
    fn default_config_path(&self, home: &Path) -> PathBuf;
    fn install(&self, ctx: &Context, source: &SourceTree) -> Result<InstallOutcome, CeError>;
    fn sync(&self, ctx: &Context, desired: &Manifest) -> Result<SyncOutcome, CeError>;
    fn set_model(&self, ctx: &Context, slot: &str, model: &str) -> Result<(), CeError>;
    fn uninstall(&self, ctx: &Context) -> Result<(), CeError>;
}
```
- *Pros*: Clean domain separation, robust testing per harness, easy addition of future harnesses.
- *Cons*: Slightly higher initial boilerplate.

### Option 2: Generic Monolithic Rule File Generator
- *Pros*: Simple string templating.
- *Cons*: Fragile, clobbers user settings, violates ISO 27001 data integrity controls.

---

## 3. Decision

Proceed with **Option 1 (Modular `HarnessAdapter` Trait)**.
