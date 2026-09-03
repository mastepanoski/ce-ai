//! Pi native harness adapter implementation for Mario Zechner's pi coding agent.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

/// Harness adapter implementation for the `pi` coding agent (`~/.pi/agent`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PiAdapter;

impl HarnessAdapter for PiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Pi
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("skills") {
            return home.to_path_buf();
        }
        if home.file_name().and_then(|n| n.to_str()) == Some("agent") {
            return home.join("skills");
        }
        if home.file_name().and_then(|n| n.to_str()) == Some(".pi") {
            return home.join("agent").join("skills");
        }
        self.kind().harness_dir(home).join("skills")
    }
}

use crate::error::CeError;
use crate::state::write_atomic;

pub const PI_EXTENSION_FILENAME: &str = "compound-engineering.ts";

pub const PI_EXTENSION_CONTENT: &str = r#"import { execSync } from "node:child_process";

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
"#;

/// Checks if `.pi/extensions/compound-engineering.ts` exists and contains the managed `ce-ai workflow resume` hook.
pub fn has_session_start_hook(extension_path: &Path) -> bool {
    if !extension_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(extension_path) else {
        return false;
    };
    content.contains("ce-ai workflow resume")
}

/// Ensures `.pi/extensions/compound-engineering.ts` exists with the canonical extension content.
/// Idempotent; returns Ok(true) if written/updated, Ok(false) if already present and identical.
pub fn ensure_session_start_hook(extension_path: &Path) -> Result<bool, CeError> {
    if has_session_start_hook(extension_path) {
        return Ok(false);
    }

    if let Some(parent) = extension_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    write_atomic(extension_path, PI_EXTENSION_CONTENT.as_bytes())?;
    Ok(true)
}

/// Surgically removes `.pi/extensions/compound-engineering.ts` if it is managed by ce-ai.
/// Prunes `.pi/extensions` and `.pi` if left empty.
pub fn remove_session_start_hook(extension_path: &Path) -> Result<bool, CeError> {
    if !extension_path.exists() {
        return Ok(false);
    }

    let Ok(content) = std::fs::read_to_string(extension_path) else {
        return Ok(false);
    };

    if !content.contains("ce-ai workflow resume") {
        return Ok(false);
    }

    let _ = std::fs::remove_file(extension_path);

    if let Some(ext_dir) = extension_path.parent() {
        let _ = std::fs::remove_dir(ext_dir);
        if let Some(pi_dir) = ext_dir.parent() {
            let _ = std::fs::remove_dir(pi_dir);
        }
    }

    Ok(true)
}

#[cfg(test)]
#[path = "tests/pi.rs"]
mod tests;
