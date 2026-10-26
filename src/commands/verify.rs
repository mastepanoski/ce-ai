//! Implementation of the `ce-ai verify` subcommand.
//!
//! Executes diff-scoped micro-mutations on touched lines to verify test suite
//! integrity and detect cosmetic/hollow assertions before shipping.

use crate::commands::Context;
use crate::error::CeError;
use crate::verification::runner::run_mutation_verification;
use clap::Args as ClapArgs;
use std::path::Path;

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    /// Custom test command to execute against each mutant (default: cargo test)
    #[arg(long)]
    pub test_cmd: Option<String>,

    /// Maximum number of mutation candidates to evaluate (default: 3)
    #[arg(long, default_value_t = 3)]
    pub max_mutants: usize,

    /// Per-mutant test execution timeout in seconds (default: 10)
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Enforce zero surviving mutants: exit with code 6 (Verification) on hollow assertions
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let repo_root = ctx
        .workspace_root
        .as_deref()
        .unwrap_or_else(|| Path::new("."));

    let report = run_mutation_verification(
        repo_root,
        args.test_cmd.as_deref(),
        args.max_mutants,
        args.timeout,
    )?;

    if report.total_mutants == 0 {
        if !ctx.quiet {
            println!("✓ No diff mutation targets found (clean or non-relational diff)");
        }
        return Ok(());
    }

    let secs = (report.duration_ms as f64) / 1000.0;

    if report.survived_mutants == 0 {
        if !ctx.quiet {
            println!(
                "✓ Test integrity verified: {}/{} diff mutants killed in {:.2}s (no hollow assertions detected)",
                report.killed_mutants, report.total_mutants, secs
            );
        }
        Ok(())
    } else {
        eprintln!(
            "⚠ Test integrity alert: {}/{} diff mutants survived ({:.2}s) — hollow assertions detected:",
            report.survived_mutants, report.total_mutants, secs
        );
        for res in &report.results {
            if !res.killed {
                eprintln!(
                    "  → {}:{}: {} survived (tests still passed)",
                    res.candidate.file_path, res.candidate.line_number, res.candidate.description
                );
            }
        }

        if args.strict {
            Err(CeError::Verification(format!(
                "test integrity gate failed: {} surviving mutant(s) detected",
                report.survived_mutants
            )))
        } else {
            eprintln!("  (hint: run with --strict in CI or Stage 5 Verification to enforce gate)");
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "tests/verify.rs"]
mod tests;
