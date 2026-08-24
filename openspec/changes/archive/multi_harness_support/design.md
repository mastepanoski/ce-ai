# OpenSpec Design: Multi-Harness Architecture & Data Schemas

**Change Identifier:** `multi_harness_support`  

---

## 1. System Architecture & Module Boundaries

```
src/
├── harness/
│   ├── mod.rs             # HarnessKind enum, HarnessAdapter trait & dispatch registry
│   ├── opencode.rs        # OpenCode JSON adapter
│   ├── claude.rs          # Claude Code JSON adapter
│   ├── pi.rs              # Pi JSON & skills directory adapter
│   ├── cursor.rs          # Cursor Markdown block adapter (.cursorrules)
│   ├── copilot.rs         # Copilot Markdown block adapter (.github/copilot-instructions.md)
│   ├── generic_json.rs    # Generic JSON adapter for Codex, Grok, Kimi, AGY, DeepSeek
│   └── custom.rs          # Custom fallback adapter (--harness custom)
```

---

## 2. Core Data Structs & Enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessKind {
    Opencode,
    Claude,
    Pi,
    Cursor,
    Copilot,
    Codex,
    Grok,
    Kimi,
    Agy,
    Deepseek,
    Fx,
    Custom,
}
```

### Custom Harness Schema in `state.json`
```json
{
  "harnesses": {
    "custom": {
      "plugins_dir": "/path/to/custom/plugins",
      "skills_dir": "/path/to/custom/skills",
      "rules_file": "/path/to/custom/rules.md"
    }
  }
}
```

---

## 3. CLI Interface Contract

```bash
# Install specific harness
ce-ai install --harness claude|pi|cursor|copilot|codex|grok|kimi|agy|deepseek|fx

# Install across all detected host harnesses
ce-ai install --all

# Custom harness installation
ce-ai install --harness custom --plugins-dir ~/.my-harness/plugins --skills-dir ~/.my-harness/skills

# Sync all installed harnesses
ce-ai sync --all

# Set model for all active harnesses
ce-ai models set ce-brainstorm = claude-3-7-sonnet
```
