//! Advisory workflow observation subsystem.
//!
//! Provides non-blocking, artifact-derived observation of repository reality:
//! git branch, working tree changes, markdown plans, review receipts,
//! handoff artifacts, knowledge capture solutions, and optional OpenSpec packages.

pub mod state;

#[cfg(test)]
mod tests;

pub use state::{
    HandoffObservation, KnowledgeObservation, ObservableWorkflowState, OpenSpecObservation,
    PlanObservation, VerificationObservation,
};
