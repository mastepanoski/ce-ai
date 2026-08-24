# OpenSpec Exploration: Tool Registration Architecture

- **Change:** `tools-install-real-sidecar-registration`
- **Issue:** #158 (P0)

---

## 🔍 1. Technical Investigation & Options

### Option A: Fake Success Messages (Current - Flawed)
Prints `println!("tools: '{tool}' MCP server registration completed successfully.")` without modifying any file.
- *Verdict*: P0 defect. Unacceptable.

### Option B: Real Registration + Capability Health Probe (Selected)
1. Read existing `opencode.json` using `serde_json`.
2. Insert/update `mcpServers.<tool>` JSON block.
3. Save modified config via `crate::state::write_atomic`.
4. Execute `extract_tool_version(tool)` or probe.
5. If probe returns `Some(version)`, print success. If `None`, print error and return `Err(CeError::Verification(...))`.

---

## 💡 2. Architectural Decision

Adopt Option B. Every `tools install` invocation MUST mutate config athetically via `write_atomic` and verify health before returning success.
