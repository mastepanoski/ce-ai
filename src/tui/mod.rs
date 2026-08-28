//! Full-screen Ratatui TUI dashboard for ce-ai.
//! Provides a modern, rich, split-panel terminal interface with live status,
//! keyboard navigation, model slot tables, and one-key action execution.

pub mod app;
pub mod handlers;
pub mod render;
pub mod runner;
pub mod spawn;
pub mod tabs;

pub use app::App;
pub use runner::run_interactive;
pub use tabs::MenuTab;

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
