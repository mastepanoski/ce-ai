//! Syntactic micro-mutator for Rust source code.
//!
//! Generates targeted boolean and relational operator mutations exclusively
//! within diff-touched line numbers to detect cosmetic or hollow tests.

use crate::error::CeError;
use crate::verification::diff::parse_unified_diff_lines;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a single mutation to be applied to a source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationCandidate {
    pub file_path: String,
    pub line_number: usize,
    pub col_offset: usize,
    pub original: String,
    pub replacement: String,
    pub description: String,
}

/// Tokenizer state for stripping comments and string literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Normal,
    LineComment,
    BlockComment(usize),
    StringLiteral,
    CharLiteral,
    RawString(usize),
}

/// Generates targeted mutations for a single source file on specified line numbers.
pub fn generate_mutations_for_source(
    file_path: &str,
    source: &str,
    target_lines: &[usize],
    max_mutants: usize,
) -> Vec<MutationCandidate> {
    if target_lines.is_empty() || max_mutants == 0 {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut state = LexState::Normal;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        if state == LexState::LineComment {
            state = LexState::Normal;
        }

        let is_target = target_lines.contains(&line_num);

        while i < len {
            let b = bytes[i];
            let next = if i + 1 < len {
                Some(bytes[i + 1])
            } else {
                None
            };
            let prev = if i > 0 { Some(bytes[i - 1]) } else { None };

            match state {
                LexState::LineComment => {
                    break;
                }
                LexState::BlockComment(depth) => {
                    if b == b'*' && next == Some(b'/') {
                        if depth <= 1 {
                            state = LexState::Normal;
                        } else {
                            state = LexState::BlockComment(depth - 1);
                        }
                        i += 2;
                        continue;
                    } else if b == b'/' && next == Some(b'*') {
                        state = LexState::BlockComment(depth + 1);
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
                LexState::StringLiteral => {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    } else if b == b'"' {
                        state = LexState::Normal;
                    }
                    i += 1;
                }
                LexState::CharLiteral => {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    } else if b == b'\'' {
                        state = LexState::Normal;
                    }
                    i += 1;
                }
                LexState::RawString(hashes) => {
                    if b == b'"' {
                        let mut valid = true;
                        for h in 0..hashes {
                            if i + 1 + h >= len || bytes[i + 1 + h] != b'#' {
                                valid = false;
                                break;
                            }
                        }
                        if valid {
                            state = LexState::Normal;
                            i += 1 + hashes;
                            continue;
                        }
                    }
                    i += 1;
                }
                LexState::Normal => {
                    if b == b'/' && next == Some(b'/') {
                        state = LexState::LineComment;
                        break;
                    } else if b == b'/' && next == Some(b'*') {
                        state = LexState::BlockComment(1);
                        i += 2;
                        continue;
                    } else if b == b'"' {
                        if i > 0 && bytes[i - 1] == b'r' {
                            state = LexState::RawString(0);
                        } else {
                            state = LexState::StringLiteral;
                        }
                        i += 1;
                        continue;
                    } else if b == b'\'' && next != Some(b'\'') {
                        let is_lifetime = next
                            .map(|c| c.is_ascii_alphanumeric() || c == b'_')
                            .unwrap_or(false)
                            && (i + 2 >= len || bytes[i + 2] != b'\'');
                        if !is_lifetime {
                            state = LexState::CharLiteral;
                        }
                        i += 1;
                        continue;
                    }

                    if is_target {
                        // Check operators
                        if b == b'=' && next == Some(b'=') {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "==".to_string(),
                                replacement: "!=".to_string(),
                                description: "invert == to !=".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 2;
                            continue;
                        } else if b == b'!' && next == Some(b'=') {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "!=".to_string(),
                                replacement: "==".to_string(),
                                description: "invert != to ==".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 2;
                            continue;
                        } else if b == b'>' && next == Some(b'=') {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: ">=".to_string(),
                                replacement: "<".to_string(),
                                description: "invert >= to <".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 2;
                            continue;
                        } else if b == b'<' && next == Some(b'=') {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "<=".to_string(),
                                replacement: ">".to_string(),
                                description: "invert <= to >".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 2;
                            continue;
                        } else if b == b'>'
                            && next != Some(b'=')
                            && next != Some(b'>')
                            && prev != Some(b'-')
                            && prev != Some(b'=')
                            && (next == Some(b' ') || (prev == Some(b' ') && next.is_some()))
                        {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: ">".to_string(),
                                replacement: "<=".to_string(),
                                description: "invert > to <=".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 1;
                            continue;
                        } else if b == b'<'
                            && next != Some(b'=')
                            && next != Some(b'<')
                            && next != Some(b'*')
                            && (next == Some(b' ') || (prev == Some(b' ') && next.is_some()))
                        {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "<".to_string(),
                                replacement: ">=".to_string(),
                                description: "invert < to >=".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 1;
                            continue;
                        } else if is_word(bytes, i, "true") {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "true".to_string(),
                                replacement: "false".to_string(),
                                description: "invert true to false".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 4;
                            continue;
                        } else if is_word(bytes, i, "false") {
                            candidates.push(MutationCandidate {
                                file_path: file_path.to_string(),
                                line_number: line_num,
                                col_offset: i,
                                original: "false".to_string(),
                                replacement: "true".to_string(),
                                description: "invert false to true".to_string(),
                            });
                            if candidates.len() >= max_mutants {
                                return candidates;
                            }
                            i += 5;
                            continue;
                        }
                    }

                    i += 1;
                }
            }
        }
    }

    candidates
}

fn is_word(bytes: &[u8], idx: usize, word: &str) -> bool {
    let word_bytes = word.as_bytes();
    if !bytes[idx..].starts_with(word_bytes) {
        return false;
    }
    if idx > 0 {
        let prev = bytes[idx - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return false;
        }
    }
    let after_idx = idx + word_bytes.len();
    if after_idx < bytes.len() {
        let after = bytes[after_idx];
        if after.is_ascii_alphanumeric() || after == b'_' {
            return false;
        }
    }
    true
}

/// Applies a mutation candidate to a source string, returning the mutated source.
pub fn apply_mutation(source: &str, candidate: &MutationCandidate) -> String {
    let mut output = String::with_capacity(source.len() + 16);

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;
        if idx > 0 {
            output.push('\n');
        }
        if line_num == candidate.line_number {
            let bytes = line.as_bytes();
            let orig_bytes = candidate.original.as_bytes();
            let col = candidate.col_offset;
            if col + orig_bytes.len() <= bytes.len()
                && &bytes[col..col + orig_bytes.len()] == orig_bytes
            {
                output.push_str(&line[..col]);
                output.push_str(&candidate.replacement);
                output.push_str(&line[col + orig_bytes.len()..]);
            } else if let Some(found_idx) = line.find(&candidate.original) {
                output.push_str(&line[..found_idx]);
                output.push_str(&candidate.replacement);
                output.push_str(&line[found_idx + candidate.original.len()..]);
            } else {
                output.push_str(line);
            }
        } else {
            output.push_str(line);
        }
    }

    if source.ends_with('\n') {
        output.push('\n');
    }

    output
}

/// Scans the repository git diff and generates mutation candidates.
pub fn generate_diff_mutations(
    repo_root: &Path,
    max_total_mutants: usize,
) -> Result<Vec<MutationCandidate>, CeError> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "HEAD"]).current_dir(repo_root);

    let output = match cmd.output() {
        Ok(out) => out,
        Err(e) => {
            return Err(CeError::Verification(format!(
                "failed to execute git diff: {e}"
            )));
        }
    };

    let diff_str = String::from_utf8_lossy(&output.stdout);
    let diff_map = parse_unified_diff_lines(&diff_str);

    let mut all_mutants = Vec::new();

    for (file_rel, touched_lines) in diff_map {
        // Skip non-rust files and test files
        if !file_rel.ends_with(".rs")
            || file_rel.contains("/tests/")
            || file_rel.ends_with("_test.rs")
            || file_rel.starts_with("tests/")
        {
            continue;
        }

        let full_path = repo_root.join(&file_rel);
        if let Ok(source) = std::fs::read_to_string(&full_path) {
            let remaining = max_total_mutants.saturating_sub(all_mutants.len());
            if remaining == 0 {
                break;
            }
            let file_mutants =
                generate_mutations_for_source(&file_rel, &source, &touched_lines, remaining);
            all_mutants.extend(file_mutants);
            if all_mutants.len() >= max_total_mutants {
                break;
            }
        }
    }

    Ok(all_mutants)
}
