//! Cyclomatic Complexity (CC) static analysis parser for Rust source code.
//!
//! Counts structural decision points according to McCabe and CRAP rules:
//! Base complexity is 1 per function. Each decision branch (`if`, `while`, `for`,
//! `loop`, `=>` match arms, `?` operator, `&&`, `||`) adds 1.

/// Represents the analyzed complexity of a single function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionComplexity {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity: usize,
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

/// Calculates cyclomatic complexity for all top-level and impl functions in a Rust source string.
pub fn calculate_complexity(source: &str) -> Vec<FunctionComplexity> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut current_fn: Option<(String, usize, usize, usize)> = None; // (name, start_line, brace_depth, complexity)
    let mut global_brace_depth: usize = 0;
    let mut skip_depth: Option<usize> = None; // For skipping `mod tests`

    let mut state = LexState::Normal;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        // Reset line comment at end of line
        if state == LexState::LineComment {
            state = LexState::Normal;
        }

        while i < len {
            let b = bytes[i];
            let next = if i + 1 < len {
                Some(bytes[i + 1])
            } else {
                None
            };

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
                        i += 2; // Skip escaped char
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
                        // Check if raw string r#"..."#
                        if i > 0 && bytes[i - 1] == b'r' {
                            state = LexState::RawString(0);
                        } else {
                            state = LexState::StringLiteral;
                        }
                        i += 1;
                        continue;
                    } else if b == b'\'' && next != Some(b'\'') {
                        // Avoid lifetime annotations like 'a, 'static
                        let is_lifetime = next
                            .map(|c| c.is_ascii_alphabetic() || c == b'_')
                            .unwrap_or(false)
                            && (i + 2 >= len || bytes[i + 2] != b'\'');
                        if !is_lifetime {
                            state = LexState::CharLiteral;
                        }
                        i += 1;
                        continue;
                    }

                    // Track brace depth
                    if b == b'{' {
                        global_brace_depth += 1;
                        i += 1;
                        continue;
                    } else if b == b'}' {
                        global_brace_depth = global_brace_depth.saturating_sub(1);

                        // Check if current function ended
                        if let Some((name, start, fn_depth, comp)) = current_fn.take() {
                            if global_brace_depth < fn_depth {
                                results.push(FunctionComplexity {
                                    name,
                                    start_line: start,
                                    end_line: line_num,
                                    complexity: comp,
                                });
                            } else {
                                current_fn = Some((name, start, fn_depth, comp));
                            }
                        }

                        // Check if skip_depth ended
                        if let Some(d) = skip_depth {
                            if global_brace_depth < d {
                                skip_depth = None;
                            }
                        }

                        i += 1;
                        continue;
                    }

                    // Check for mod tests
                    if current_fn.is_none() && skip_depth.is_none() {
                        let rest_bytes = &bytes[i..];
                        if rest_bytes.starts_with(b"mod tests")
                            || rest_bytes.starts_with(b"mod test")
                        {
                            skip_depth = Some(global_brace_depth + 1);
                        }
                    }

                    // Check for function start if not inside another function or test module
                    if current_fn.is_none() && skip_depth.is_none() {
                        if let Some(fn_name) = extract_fn_name(bytes, i) {
                            current_fn = Some((fn_name, line_num, global_brace_depth + 1, 1));
                            i += 2; // advance past "fn"
                            continue;
                        }
                    }

                    // Count decision points if inside function
                    if let Some((_, _, _, ref mut comp)) = current_fn {
                        // Operators: &&, ||, =>, ?
                        if b == b'&' && next == Some(b'&') {
                            *comp += 1;
                            i += 2;
                            continue;
                        } else if b == b'|' && next == Some(b'|') {
                            // In Rust, `||` can be an empty closure parameter list like `ok_or_else(|| ...)`
                            // rather than logical OR. Check if preceding non-whitespace token is delimiter.
                            let mut prev_idx = i;
                            let mut is_closure = false;
                            while prev_idx > 0 {
                                prev_idx -= 1;
                                let pb = bytes[prev_idx];
                                if pb == b' ' || pb == b'\t' {
                                    continue;
                                }
                                if pb == b'('
                                    || pb == b','
                                    || pb == b'['
                                    || pb == b'{'
                                    || pb == b'='
                                    || pb == b';'
                                {
                                    is_closure = true;
                                }
                                break;
                            }
                            if !is_closure {
                                *comp += 1;
                            }
                            i += 2;
                            continue;
                        } else if b == b'=' && next == Some(b'>') {
                            *comp += 1;
                            i += 2;
                            continue;
                        } else if b == b'?' {
                            *comp += 1;
                            i += 1;
                            continue;
                        }

                        // Keywords: if, while, for, loop
                        if is_keyword(bytes, i, b"if")
                            || is_keyword(bytes, i, b"while")
                            || is_keyword(bytes, i, b"for")
                            || is_keyword(bytes, i, b"loop")
                        {
                            *comp += 1;
                            i += 2;
                            continue;
                        }
                    }

                    i += 1;
                }
            }
        }
    }

    // In case function ended at EOF without closing brace
    if let Some((name, start, _, comp)) = current_fn {
        results.push(FunctionComplexity {
            name,
            start_line: start,
            end_line: lines.len(),
            complexity: comp,
        });
    }

    results
}

fn is_keyword(bytes: &[u8], idx: usize, kw: &[u8]) -> bool {
    if !bytes[idx..].starts_with(kw) {
        return false;
    }
    if idx > 0 {
        let prev = bytes[idx - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return false;
        }
    }
    let after_idx = idx + kw.len();
    if after_idx < bytes.len() {
        let after = bytes[after_idx];
        if after.is_ascii_alphanumeric() || after == b'_' {
            return false;
        }
    }
    true
}

fn extract_fn_name(bytes: &[u8], idx: usize) -> Option<String> {
    let rest = &bytes[idx..];
    let prefixes: [&[u8]; 6] = [
        b"pub(crate) async fn ",
        b"pub(crate) fn ",
        b"pub async fn ",
        b"pub fn ",
        b"async fn ",
        b"fn ",
    ];

    let mut name_start = None;
    for prefix in prefixes {
        if rest.starts_with(prefix) {
            name_start = Some(idx + prefix.len());
            break;
        }
    }

    let start = name_start?;
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }

    if end > start {
        String::from_utf8(bytes[start..end].to_vec()).ok()
    } else {
        None
    }
}
