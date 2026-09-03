# Technical Design: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent

## 1. Extension Code Implementation

File: `.pi/extensions/compound-engineering.ts`
```typescript
import { execSync } from "node:child_process";

export default function (pi: any) {
  let sessionInitialized = false;

  pi.on("session_start", async () => {
    sessionInitialized = false;
  });

  pi.on("before_agent_start", async (event: any, ctx: any) => {
    if (!sessionInitialized) {
      sessionInitialized = true;
      try {
        const stdout = execSync("ce-ai workflow resume", {
          cwd: ctx?.cwd || process.cwd(),
          encoding: "utf-8",
          timeout: 5000,
        });
        if (stdout && stdout.trim()) {
          return {
            systemPrompt: `${event.systemPrompt}\n\n<!-- CE-AI MANAGED REPOSTATE -->\n${stdout.trim()}`,
          };
        }
      } catch {
        // Fail-open: do not disrupt agent execution loop
      }
    }
  });
}
```

## 2. Harness Interface in `src/harness/pi.rs`

```rust
pub const PI_EXTENSION_FILENAME: &str = "compound-engineering.ts";
pub const PI_EXTENSION_CONTENT: &str = "...";

pub fn has_session_start_hook(extension_path: &Path) -> bool;
pub fn ensure_session_start_hook(extension_path: &Path) -> Result<bool, CeError>;
pub fn remove_session_start_hook(extension_path: &Path) -> Result<bool, CeError>;
```

## 3. Integration Wiring Points

1. `init_prj.rs`:
   When `.pi/` exists:
   `let ext_path = pi_dir.join("extensions").join("compound-engineering.ts");`
   `crate::harness::pi::ensure_session_start_hook(&ext_path)?;`
2. `deinit_prj.rs`:
   `crate::harness::pi::remove_session_start_hook(&ext_path)?;`
3. `doctor.rs`:
   When project has `.pi/`:
   Verify `crate::harness::pi::has_session_start_hook(&ext_path)`.
