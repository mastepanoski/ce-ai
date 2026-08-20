//! CE source resolution: GitHub releases, tarball cache + SHA256, safe extraction.
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

#![allow(dead_code)]

pub mod archive;
pub mod cache;
pub mod release;
