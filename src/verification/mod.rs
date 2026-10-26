//! Verification and test integrity engine.
//!
//! Provides diff-scoped micro-mutator capabilities to smoke-test test suites
//! against cosmetic or hollow assertions during Stage 5 Verification.

pub mod diff;
pub mod mutator;
pub mod runner;

#[cfg(test)]
mod tests;
