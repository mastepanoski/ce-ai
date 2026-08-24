# OpenSpec Design: Per-Harness Native Directory Structs

- **Change:** `harness-containment-safety-gate`
- **Issue:** #157 (P0)

---

## 📐 1. Method Signatures

```rust
impl HarnessKind {
    /// Returns native base directory for this harness given home_dir.
    pub fn harness_dir(&self, home_dir: &Path) -> PathBuf {
        match self {
            HarnessKind::Opencode => home_dir.join(".config").join("opencode"),
            HarnessKind::Claude => home_dir.join(".config").join("claude"),
            HarnessKind::Pi => home_dir.join(".pi"),
            HarnessKind::Cursor => home_dir.join(".cursor"),
            HarnessKind::Copilot => home_dir.join(".config").join("github-copilot"),
            HarnessKind::Codex => home_dir.join(".config").join("codex"),
            HarnessKind::Grok => home_dir.join(".config").join("grok"),
            HarnessKind::Kimi => home_dir.join(".config").join("kimi"),
            HarnessKind::Agy => home_dir.join(".gemini").join("antigravity-cli"),
            HarnessKind::Deepseek => home_dir.join(".config").join("deepseek"),
            HarnessKind::Fx => home_dir.join(".config").join("fx"),
            HarnessKind::Custom => home_dir.join(".config").join("custom"),
        }
    }
}
```
