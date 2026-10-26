//! Git diff parser for extracting modified line numbers per file.

use std::collections::HashMap;

/// Parses a unified diff string and returns a map of file path to 1-based modified line numbers.
pub fn parse_unified_diff_lines(diff: &str) -> HashMap<String, Vec<usize>> {
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut current_line: usize = 0;

    for line in diff.lines() {
        if let Some(target) = line.strip_prefix("+++ b/") {
            current_file = Some(target.trim().replace('\\', "/"));
        } else if line.starts_with("+++ /dev/null") {
            current_file = None;
        } else if line.starts_with("@@ ") {
            // Format: @@ -old,count +new_start,new_count @@
            if let Some(plus_idx) = line.find('+') {
                let rest = &line[plus_idx + 1..];
                let num_str = rest.split([' ', ',']).next().unwrap_or("0");
                current_line = num_str.parse::<usize>().unwrap_or(0);
            }
        } else if let Some(file) = &current_file {
            if line.starts_with('+') && !line.starts_with("+++") {
                map.entry(file.clone()).or_default().push(current_line);
                current_line = current_line.saturating_add(1);
            } else if line.starts_with(' ') {
                current_line = current_line.saturating_add(1);
            }
        }
    }

    map
}
