#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod capture;
pub mod commands;
pub mod compat;
pub mod decisions;
pub mod error;
pub mod fleet;
pub mod harness;
pub mod opencode;
pub mod source;
pub mod state;
