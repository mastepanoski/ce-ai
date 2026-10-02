# Design: Upstream Coupling Remediation & CE Contract Alignment

## Architectural Design

### 1. `docs_root` Discovery Function (`src/commands/workflow.rs` & `src/commands/doc.rs`)

```rust
/// Resolves the documentation root directory, respecting `.compound-engineering/config.yaml`.
/// Defaults to `PathBuf::from("docs")` if not explicitly relocated.
pub fn resolve_docs_root(repo_root: &Path) -> PathBuf {
    let candidates = [
        repo_root.join(".compound-engineering").join("config.yaml"),
        repo_root.join(".compound-engineering").join("config.local.yaml"),
    ];

    for config_path in candidates {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    if k.trim() == "docs_root" {
                        let clean = v.trim().trim_matches('"').trim_matches('\'');
                        if !clean.is_empty() {
                            return PathBuf::from(clean);
                        }
                    }
                }
            }
        }
    }

    PathBuf::from("docs")
}
```

### 2. Upstream Schema Conformance in `check_solution_frontmatter`

The validator enforces the following rules:
- `module` or `category` must be present.
- `date` must be present.
- `problem_type` must be present.
- `component` (or `components`) must be present.
- `severity` must be present.
- `applies_when` is mandatory ONLY when `problem_type != "bugfix"` and `problem_type != "bug"`. For bugfixes, it is optional.
- `tags` is optional (does not cause missing field error).

### 3. Pi Extension Namespacing in `src/harness/pi.rs`

```rust
pub const PI_EXTENSION_FILENAME: &str = "ce-ai-companion.ts";
pub const PI_LEGACY_EXTENSION_FILENAME: &str = "compound-engineering.ts";
```

In `remove_session_start_hook`:
- Check both `ce-ai-companion.ts` and `compound-engineering.ts`.
- Only delete files containing `"ce-ai workflow resume"`.

### 4. Gate Exemption Hardening in `src/commands/gate.rs`

```rust
let is_ce_debug = clean == "ce-debug"
    || clean.starts_with("ce-debug:")
    || clean.starts_with("ce-debug ")
    || clean.starts_with("ce-debug/");
```
This prevents incidental substring matches from bypassing gate enforcement.
