//! CE source resolution: GitHub releases, tarball cache + SHA256, safe extraction.
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

pub mod archive;
pub mod binary_release;
pub mod bug_reporter;
pub mod builtin_skills;
pub mod cache;
pub mod registry;
pub mod release;
pub mod tools_registry;
pub mod update_notifier;
