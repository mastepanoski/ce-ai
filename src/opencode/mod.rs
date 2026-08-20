//! OpenCode config merge, plugin loader placement, and install-manifest I/O (OI-1..OI-5).
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

#![allow(dead_code)]

pub mod config;
pub mod manifest;
pub mod plugins;