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

const KEYRING_SERVICE: &str = "ce-ai";
const KEYRING_USER: &str = "typesafe";

/// Looks up the API key in the OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service).
pub fn keyring_get() -> Option<String> {
    if std::env::var("CE_AI_CREDENTIALS_PATH").is_ok() {
        return None;
    }

    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if let Ok(key) = entry.get_password() {
            let clean = key.trim().to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }

    // Interoperability with jevkit: check if the key was stored under the jevkit service name
    if let Ok(entry) = keyring::Entry::new("jevkit", KEYRING_USER) {
        if let Ok(key) = entry.get_password() {
            let clean = key.trim().to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }

    None
}

/// Stores the API key securely in the OS keyring.
pub fn keyring_set(key: &str) -> Result<(), CeError> {
    if std::env::var("CE_AI_CREDENTIALS_PATH").is_ok() {
        return Ok(());
    }

    let clean = key.trim();
    if clean.is_empty() {
        return Err(CeError::Usage("cannot save empty API key".into()));
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| CeError::Runtime(format!("failed to open OS keyring: {e}")))?;
    entry
        .set_password(clean)
        .map_err(|e| CeError::Runtime(format!("failed to write to OS keyring: {e}")))?;
    Ok(())
}

/// Removes the API key from the OS keyring.
pub fn keyring_delete() -> Result<bool, CeError> {
    if std::env::var("CE_AI_CREDENTIALS_PATH").is_ok() {
        return Ok(false);
    }

    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| CeError::Runtime(format!("failed to open OS keyring: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(CeError::Runtime(format!(
            "failed to delete from OS keyring: {e}"
        ))),
    }
}

/// Resolves the API key using tiered precedence:
/// 1. `TYPESAFE_API_KEY` process environment variable.
/// 2. `JEV_API_KEY` process environment variable.
/// 3. OS Keyring (`~/.keychain` / Windows Credential Manager / Secret Service) when no custom path is given.
/// 4. `credentials.toml` file (custom path or default `~/.config/ce-ai/credentials.toml`).
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

    // If an explicit custom path is provided (e.g. isolated test harness), resolve only from that file.
    if let Some(path) = custom_path {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
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
        return None;
    }

    // 3. OS Keyring
    if let Some(key) = keyring_get() {
        return Some(key);
    }

    // 4. Default user global credentials file
    let path = default_credentials_path()?;
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

/// Saves an API key safely to the OS keyring and credentials file with `0600` permissions.
pub fn save_api_key(key: &str, custom_path: Option<&Path>) -> Result<PathBuf, CeError> {
    let clean_key = key.trim();
    if clean_key.is_empty() {
        return Err(CeError::Usage("cannot save empty API key".into()));
    }

    // Attempt to write to OS Keyring only when targeting global user credentials (not isolated tests)
    if custom_path.is_none() {
        let _ = keyring_set(clean_key);
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

/// Reads an API key from an arbitrary reader, trimming whitespace and newlines.
pub fn read_api_key_from_reader<R: std::io::BufRead>(mut reader: R) -> Result<String, CeError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let clean = line.trim().to_string();
    if clean.is_empty() {
        return Err(CeError::Usage(
            "API key provided via stdin cannot be empty".into(),
        ));
    }
    Ok(clean)
}

/// Reads an API key from standard input (non-interactive stream, e.g. piped or redirected).
pub fn read_api_key_from_stdin() -> Result<String, CeError> {
    read_api_key_from_reader(std::io::stdin().lock())
}

/// Prompts for an API key interactively using rpassword without character echo.
/// Returns Ok(None) if the operator cancels or submits an empty input.
pub fn prompt_api_key_interactive(prompt: &str) -> Result<Option<String>, CeError> {
    use std::io::IsTerminal;

    // If stdin is not a terminal (e.g. piped in tests or CI), fall back to standard line reading.
    if !std::io::stdin().is_terminal() {
        let key = read_api_key_from_stdin()?;
        return Ok(Some(key));
    }

    use std::io::Write;
    print!("{prompt}");
    let _ = std::io::stdout().flush();

    let key = rpassword::read_password()
        .map_err(|e| CeError::Runtime(format!("failed to read API key: {e}")))?;

    let n = key.chars().count();
    if n > 0 {
        println!("\x1b[2m{} \x1b[0m({n} chars)\x1b[0m", "*".repeat(n));
    }

    let clean = key.trim().to_string();
    if clean.is_empty() {
        Ok(None)
    } else {
        Ok(Some(clean))
    }
}

#[cfg(test)]
#[path = "tests/auth_tests.rs"]
mod tests;
