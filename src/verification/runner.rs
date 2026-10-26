//! Sandboxed test runner for diff-scoped mutation verification.
//!
//! Applies mutation candidates one at a time using an RAII FileRestoreGuard
//! to guarantee that source files are 100% restored to their original state
//! even in the event of test failures, process panics, or timeouts.

use crate::error::CeError;
use crate::verification::mutator::{apply_mutation, generate_diff_mutations, MutationCandidate};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// RAII guard ensuring the modified file is reverted to its original content on drop.
pub struct FileRestoreGuard {
    file_path: PathBuf,
    original_content: String,
    active: bool,
}

impl FileRestoreGuard {
    pub fn new(file_path: PathBuf, original_content: String) -> Self {
        Self {
            file_path,
            original_content,
            active: true,
        }
    }

    pub fn restore(&mut self) -> Result<(), CeError> {
        if self.active {
            std::fs::write(&self.file_path, &self.original_content).map_err(CeError::Io)?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for FileRestoreGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = std::fs::write(&self.file_path, &self.original_content);
        }
    }
}

/// The result of evaluating a single mutation candidate against the test suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    pub candidate: MutationCandidate,
    pub killed: bool,
    pub reason: String,
}

/// Comprehensive summary report of a mutation verification run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationReport {
    pub total_mutants: usize,
    pub killed_mutants: usize,
    pub survived_mutants: usize,
    pub results: Vec<MutationResult>,
    pub duration_ms: u128,
}

/// Runs a command with a timeout, returning true if it exited successfully (status 0).
fn execute_test_command(
    repo_root: &Path,
    test_cmd: &str,
    timeout_secs: u64,
) -> Result<bool, CeError> {
    let tokens: Vec<&str> = test_cmd.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(CeError::Usage("empty test command".to_string()));
    }

    let prog = tokens[0];
    let args = &tokens[1..];
    let mut cmd = std::process::Command::new(prog);
    cmd.args(args)
        .current_dir(repo_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Err(CeError::Runtime(format!(
                "failed to spawn test command '{test_cmd}': {e}"
            )));
        }
    };

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let poll_interval = Duration::from_millis(50);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(status.success());
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    // Timed out: infinite loop or deadlock induced by mutant counts as killed
                    return Ok(false);
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                let _ = child.kill();
                return Err(CeError::Runtime(format!(
                    "error waiting for test process: {e}"
                )));
            }
        }
    }
}

/// Executes test suite against a collection of mutation candidates.
pub fn run_mutations_on_candidates(
    repo_root: &Path,
    candidates: &[MutationCandidate],
    test_cmd: Option<&str>,
    timeout_secs: u64,
) -> Result<MutationReport, CeError> {
    let start_total = Instant::now();
    let cmd_str = test_cmd.unwrap_or("cargo test");
    let mut results = Vec::new();
    let mut killed_count = 0;
    let mut survived_count = 0;

    for candidate in candidates {
        let full_path = repo_root.join(&candidate.file_path);
        let original_content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => {
                return Err(CeError::Io(e));
            }
        };

        let mutated = apply_mutation(&original_content, candidate);
        let mut guard = FileRestoreGuard::new(full_path.clone(), original_content);

        // Write mutated source
        if let Err(e) = std::fs::write(&full_path, &mutated) {
            return Err(CeError::Io(e));
        }

        // Run tests
        let test_passed = execute_test_command(repo_root, cmd_str, timeout_secs)?;

        // Restore immediately
        guard.restore()?;

        if test_passed {
            // Tests passed despite mutation -> Mutant SURVIVED (hollow test)
            survived_count += 1;
            results.push(MutationResult {
                candidate: candidate.clone(),
                killed: false,
                reason: "tests passed despite mutation (assertion hollow or missing)".to_string(),
            });
        } else {
            // Tests failed -> Mutant KILLED (assertion caught it)
            killed_count += 1;
            results.push(MutationResult {
                candidate: candidate.clone(),
                killed: true,
                reason: "tests failed as expected".to_string(),
            });
        }
    }

    Ok(MutationReport {
        total_mutants: candidates.len(),
        killed_mutants: killed_count,
        survived_mutants: survived_count,
        results,
        duration_ms: start_total.elapsed().as_millis(),
    })
}

/// Discovers diff mutations and runs verification.
pub fn run_mutation_verification(
    repo_root: &Path,
    test_cmd: Option<&str>,
    max_mutants: usize,
    timeout_secs: u64,
) -> Result<MutationReport, CeError> {
    let candidates = generate_diff_mutations(repo_root, max_mutants)?;
    let report = run_mutations_on_candidates(repo_root, &candidates, test_cmd, timeout_secs)?;
    let _ = save_mutation_receipt(repo_root, &report);
    Ok(report)
}

/// Persists the mutation report to `.mutant-receipt.json` in the repository root.
pub fn save_mutation_receipt(repo_root: &Path, report: &MutationReport) -> Result<(), CeError> {
    let receipt_path = repo_root.join(".mutant-receipt.json");
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| CeError::State(format!("failed to serialize mutation report: {e}")))?;
    std::fs::write(&receipt_path, json).map_err(CeError::Io)?;
    Ok(())
}

/// Reads the existing mutation report from `.mutant-receipt.json` if available.
pub fn read_mutation_receipt(repo_root: &Path) -> Option<MutationReport> {
    let receipt_path = repo_root.join(".mutant-receipt.json");
    let content = std::fs::read_to_string(&receipt_path).ok()?;
    serde_json::from_str(&content).ok()
}
