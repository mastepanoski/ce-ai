# Exploration: Skills Resolve Positional Query Support

## 1. Technical Investigation
Tracing the execution path of `ce-ai doctor` and `ce-ai tools status`:
1. `src/source/tools_registry.rs`:
   Companion tool definitions define `resolve_cmd: "ce-ai skills resolve sequential-thinking".into()`.
2. `src/commands/doctor.rs` & `src/commands/tools.rs`:
   When companion tools are unconfigured or missing, the command suggestions display `companion.resolve_cmd` directly to the user.
3. `src/commands/skills.rs`:
   ```rust
   Resolve {
       #[arg(long, default_value = "opencode")]
       harness: String,

       #[arg(long)]
       query: String,

       #[arg(long, default_value_t = false)]
       json: bool,
   }
   ```
   Clap matches `Resolve` and looks for flags. When invoked as `ce-ai skills resolve sequential-thinking`, `sequential-thinking` is parsed as an unexpected positional argument, triggering a clap error and returning exit code 2.

## 2. Evaluated Options

### Option A: Change `resolve_cmd` in `tools_registry.rs` to use `--query`
- **Mechanism**: Change `"ce-ai skills resolve sequential-thinking"` to `"ce-ai skills resolve --query sequential-thinking"`.
- **Drawbacks**:
  - Does not fix user expectation: `ce-ai skills resolve <skill>` is the natural CLI UX (similar to `apt install <pkg>` or `cargo add <crate>`).
  - Stale caches: Existing installations have cached `companion-registry.json` (24h TTL) in `~/.ce-ai/cache/` pointing to `ce-ai skills resolve sequential-thinking`. Users would continue to get the broken recommendation until cache invalidation.
  - Fragile ergonomics.

### Option B: Support both Positional Argument and `--query` Flag in `ce-ai skills resolve` (Recommended)
- **Mechanism**:
  Declare `query_pos: Option<String>` as a positional parameter:
  ```rust
  Resolve {
      #[arg(long, default_value = "opencode")]
      harness: String,

      /// Search query as positional argument (e.g. ce-ai skills resolve sequential-thinking)
      #[arg(value_name = "QUERY")]
      query_pos: Option<String>,

      /// Search query via named flag
      #[arg(long)]
      query: Option<String>,

      #[arg(long, default_value_t = false)]
      json: bool,
  }
  ```
  Resolve the effective query:
  ```rust
  let search_query = query_pos.as_deref().or(query.as_deref()).unwrap_or("");
  ```
- **Benefits**:
  - Fulfills the exact command printed by `doctor` and `tools status`.
  - Works with existing cached registry files.
  - Keeps 100% backward compatibility for `--query`.
  - Natural CLI UX.

## 3. Tradeoff Analysis
Option B is strictly superior because it handles both UX expectations and backward compatibility while fixing the issue for both fresh and cached environments.
