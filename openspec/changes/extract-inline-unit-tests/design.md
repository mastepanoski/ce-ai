# Technical Design & Directory Architecture (#265)

## 1. Directory & File Organization Model

For any functional source file `src/<domain>/<file>.rs`, its extracted unit test file will live at:
`src/<domain>/tests/<file>.rs`

And the functional source file `src/<domain>/<file>.rs` will contain at the bottom:
```rust
#[cfg(test)]
#[path = "tests/<file>.rs"]
mod tests;
```

For top-level single files like `src/error.rs`:
```rust
#[cfg(test)]
#[path = "tests/error.rs"]
mod tests;
```
Living at `src/tests/error.rs`.

For `mod.rs` files (e.g. `src/state/mod.rs` or `src/harness/mod.rs`):
```rust
#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
```
Living at `src/<domain>/tests/mod_tests.rs`.

## 2. Domain Inventory & Target File Mapping

### A. State Domain (`src/state/`)
| Source File | Extracted Test File |
| :--- | :--- |
| `src/state/state.rs` | `src/state/tests/state.rs` |
| `src/state/diff.rs` | `src/state/tests/diff.rs` |
| `src/state/ports.rs` | `src/state/tests/ports.rs` |
| `src/state/backups.rs` | `src/state/tests/backups.rs` |
| `src/state/journal.rs` | `src/state/tests/journal.rs` |
| `src/state/profiles.rs` | `src/state/tests/profiles.rs` |
| `src/state/mod.rs` | `src/state/tests/mod_tests.rs` |

### B. OpenCode Domain (`src/opencode/`)
| Source File | Extracted Test File |
| :--- | :--- |
| `src/opencode/config.rs` | `src/opencode/tests/config.rs` |
| `src/opencode/manifest.rs` | `src/opencode/tests/manifest.rs` |
| `src/opencode/plugins.rs` | `src/opencode/tests/plugins.rs` |

### C. Source Domain (`src/source/`)
| Source File | Extracted Test File |
| :--- | :--- |
| `src/source/cache.rs` | `src/source/tests/cache.rs` |
| `src/source/tools_registry.rs` | `src/source/tests/tools_registry.rs` |
| `src/source/registry.rs` | `src/source/tests/registry.rs` |
| `src/source/release.rs` | `src/source/tests/release.rs` |
| `src/source/archive.rs` | `src/source/tests/archive.rs` |

### D. Harness Domain (`src/harness/`)
| Source File | Extracted Test File |
| :--- | :--- |
| `src/harness/agents.rs` | `src/harness/tests/agents.rs` |
| `src/harness/pi.rs` | `src/harness/tests/pi.rs` |
| `src/harness/claude.rs` | `src/harness/tests/claude.rs` |
| `src/harness/copilot.rs` | `src/harness/tests/copilot.rs` |
| `src/harness/grok.rs` | `src/harness/tests/grok.rs` |
| `src/harness/codex.rs` | `src/harness/tests/codex.rs` |
| `src/harness/custom.rs` | `src/harness/tests/custom.rs` |
| `src/harness/agy.rs` | `src/harness/tests/agy.rs` |
| `src/harness/cursor.rs` | `src/harness/tests/cursor.rs` |
| `src/harness/fx.rs` | `src/harness/tests/fx.rs` |
| `src/harness/kimi.rs` | `src/harness/tests/kimi.rs` |
| `src/harness/mod.rs` | `src/harness/tests/mod_tests.rs` |

### E. Commands Domain (`src/commands/`)
| Source File | Extracted Test File |
| :--- | :--- |
| `src/commands/upgrade.rs` | `src/commands/tests/upgrade.rs` |
| `src/commands/tools.rs` | `src/commands/tests/tools.rs` |
| `src/commands/audit.rs` | `src/commands/tests/audit.rs` |
| `src/commands/models.rs` | `src/commands/tests/models.rs` |
| `src/commands/sync.rs` | `src/commands/tests/sync.rs` |
| `src/commands/doctor.rs` | `src/commands/tests/doctor.rs` |
| `src/commands/init_prj.rs` | `src/commands/tests/init_prj.rs` |
| `src/commands/workflow.rs` | `src/commands/tests/workflow.rs` |
| `src/commands/install.rs` | `src/commands/tests/install.rs` |
| `src/commands/guard.rs` | `src/commands/tests/guard.rs` |

### F. TUI & Core Root
| Source File | Extracted Test File |
| :--- | :--- |
| `src/error.rs` | `src/tests/error.rs` |
| `src/tui/mod.rs` | `src/tui/tests/mod_tests.rs` |

## 3. Structural Conventions Inside Extracted Test Files

Each extracted file will start with standard imports:
```rust
use super::*;
use tempfile::{tempdir, TempDir};
```
And contain the exact test functions without any alteration of test logic, names, assertions, or fixtures.
