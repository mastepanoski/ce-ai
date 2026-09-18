//! Credential resolution and secure storage for the Decision Engine.
//!
//! Enforces zero-leak security invariants:
//! - Keys are resolved from environment variables first (`TYPESAFE_API_KEY`, `JEV_API_KEY`).
//! - Fallback reads user-scoped `~/.config/ce-ai/credentials.toml`.
//! - File writes enforce Unix `0600` permissions.
//! - Secret masking helper ensures unmasked keys are never printed in logs or terminal displays.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;

/// Schema of the private credentials file (`credentials.toml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CredentialsFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typesafe_api_key: Option<String>,
}

/// Returns default user credentials path (`~/.config/ce-ai/credentials.toml`).
pub fn default_credentials_path() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("CE_AI_CREDENTIALS_PATH") {
        let clean = custom.trim();
        if !clean.is_empty() {
            return Some(PathBuf::from(clean));
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;

    Some(
        PathBuf::from(home)
            .join(".config")
            .join("ce-ai")
            .join("credentials.toml"),
    )
}

/// Resolves the API key using tiered precedence:
/// 1. `TYPESAFE_API_KEY` process environment variable.
/// 2. `JEV_API_KEY` process environment variable.
/// 3. `credentials.toml` file (custom path or default `~/.config/ce-ai/credentials.toml`).
pub fn resolve_api_key(custom_path: Option<&Path>) -> Option<String> {
    // 1. Process environment: TYPESAFE_API_KEY
    if let Ok(key) = std::env::var("TYPESAFE_API_KEY") {
        let clean = key.trim();
        if !clean.is_empty() {
            return Some(clean.to_string());
        }
    }

    // 2. Alias environment: JEV_API_KEY
    if let Ok(key) = std::env::var("JEV_API_KEY") {
        let clean = key.trim();
        if !clean.is_empty() {
            return Some(clean.to_string());
        }
    }

    // 3. User global credentials file
    let path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => default_credentials_path()?,
    };

    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(creds) = toml::from_str::<CredentialsFile>(&content) {
                if let Some(key) = creds.typesafe_api_key {
                    let clean = key.trim();
                    if !clean.is_empty() {
                        return Some(clean.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Saves an API key safely to the credentials file with `0600` permissions.
pub fn save_api_key(key: &str, custom_path: Option<&Path>) -> Result<PathBuf, CeError> {
    let clean_key = key.trim();
    if clean_key.is_empty() {
        return Err(CeError::Usage("cannot save empty API key".into()));
    }

    let path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => default_credentials_path().ok_or_else(|| {
            CeError::State(
                "cannot determine user credentials directory (HOME/USERPROFILE not set)".into(),
            )
        })?,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut creds = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<CredentialsFile>(&content).unwrap_or_default()
    } else {
        CredentialsFile::default()
    };

    creds.typesafe_api_key = Some(clean_key.to_string());

    let serialized = toml::to_string_pretty(&creds)
        .map_err(|e| CeError::State(format!("failed to serialize credentials: {e}")))?;

    // Atomic write via tempfile + rename
    crate::state::write_atomic(&path, serialized.as_bytes())?;

    // Enforce 0600 on Unix platforms
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(&path, perms);
    }

    Ok(path)
}

/// Masks an API key for safe display in status and diagnostics.
/// Examples:
/// - `ts-1234567890abcdef` ➔ `ts****...****cdef`
/// - `short` ➔ `******`
pub fn mask_api_key(key: &str) -> String {
    let clean = key.trim();
    if clean.len() <= 6 {
        "******".to_string()
    } else {
        let prefix = &clean[..2];
        let suffix = &clean[clean.len() - 4..];
        format!("{prefix}****...****{suffix}")
    }
}

#[cfg(test)]
#[path = "tests/auth_tests.rs"]
mod tests;
