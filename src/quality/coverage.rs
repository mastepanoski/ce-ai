//! LCOV coverage report parser.
//!
//! Parses standard LCOV (.info) format files to extract line coverage ratios per file.

use std::collections::HashMap;

/// Parses an LCOV format content string and returns a map of relative file path to coverage ratio (0.0 to 1.0).
pub fn parse_lcov(content: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut lines_found: usize = 0;
    let mut lines_hit: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("SF:") {
            current_file = Some(path.replace('\\', "/"));
            lines_found = 0;
            lines_hit = 0;
        } else if let Some(lf_str) = trimmed.strip_prefix("LF:") {
            lines_found = lf_str.parse::<usize>().unwrap_or(0);
        } else if let Some(lh_str) = trimmed.strip_prefix("LH:") {
            lines_hit = lh_str.parse::<usize>().unwrap_or(0);
        } else if trimmed == "end_of_record" {
            if let Some(file) = current_file.take() {
                let ratio = if lines_found > 0 {
                    (lines_hit as f64) / (lines_found as f64)
                } else {
                    0.0
                };
                map.insert(file, ratio);
            }
        }
    }

    map
}
