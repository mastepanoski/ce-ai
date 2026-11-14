//! Zero-Dependency Native PII & Secrets Redaction Engine.
//!
//! Complies with ISO/IEC 27001 (Confidentiality), GDPR / CCPA (Data Protection),
//! HIPAA (Health Information), and NIST AI RMF governance controls.
//!
//! Features:
//! - Mathematical Luhn (mod 10) validation for credit/debit card numbers.
//! - ISO 13616 / ISO 7064 Mod 97-10 validation for IBAN numbers.
//! - RFC 5322 email and E.164 / national phone number redaction.
//! - API key, Bearer / JWT token, and PEM private key masking.
//! - Cross-platform filesystem path anonymization (`~` and `<project-root>`).

use std::path::Path;

/// Validates payment card numbers using the standard Luhn (mod 10) checksum algorithm.
///
/// Strips spaces and hyphens. Checks lengths between 13 and 19 digits.
/// Returns true only if all characters are valid digits and the checksum holds.
pub fn is_luhn_valid(number: &str) -> bool {
    let mut clean = Vec::with_capacity(19);
    for c in number.chars() {
        if c.is_ascii_digit() {
            clean.push((c as u8 - b'0') as u32);
        } else if c != ' ' && c != '-' {
            return false;
        }
    }

    if clean.len() < 13 || clean.len() > 19 {
        return false;
    }

    let mut sum = 0;
    for (i, &digit) in clean.iter().rev().enumerate() {
        if i % 2 == 1 {
            let mut doubled = digit * 2;
            if doubled > 9 {
                doubled -= 9;
            }
            sum += doubled;
        } else {
            sum += digit;
        }
    }

    sum > 0 && sum % 10 == 0
}

/// Validates International Bank Account Numbers (IBAN) conforming to ISO 13616 / ISO 7064 Mod 97-10.
///
/// Strips spaces and hyphens. Checks length (15 to 34 chars) and country code.
/// Calculates streaming modulo 97 to prevent 64-bit integer overflow.
pub fn is_iban_valid(iban: &str) -> bool {
    let clean: String = iban
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();

    if clean.len() < 15 || clean.len() > 34 {
        return false;
    }

    let bytes = clean.as_bytes();
    if !bytes[0].is_ascii_alphabetic() || !bytes[1].is_ascii_alphabetic() {
        return false;
    }
    if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
        return false;
    }
    if !clean.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    // Rearrange: move first 4 characters (country code + check digits) to the end
    let rearranged = format!("{}{}", &clean[4..], &clean[..4]);

    // Streaming modulo 97: (remainder * 10 + digit) % 97
    let mut remainder: u32 = 0;
    for ch in rearranged.chars() {
        if ch.is_ascii_digit() {
            let val = (ch as u8 - b'0') as u32;
            remainder = (remainder * 10 + val) % 97;
        } else if ch.is_ascii_alphabetic() {
            let val = (ch.to_ascii_uppercase() as u8 - b'A' + 10) as u32;
            // Letter expands to two digits: tens and ones
            remainder = (remainder * 10 + (val / 10)) % 97;
            remainder = (remainder * 10 + (val % 10)) % 97;
        } else {
            return false;
        }
    }

    remainder == 1
}

/// Checks whether a string contains sensitive PII, credentials, or tokens.
pub fn contains_pii_or_secrets(content: &str) -> bool {
    let redacted = redact_pii_and_secrets(content);
    redacted != content
}

/// Redacts sensitive PII (credit cards, IBANs, emails, phones) and secrets (tokens, keys, passwords).
pub fn redact_pii_and_secrets(content: &str) -> String {
    redact_pii_and_secrets_with_paths(content, None, None)
}

/// Redacts PII, secrets, and filesystem paths (project root and user home).
pub fn redact_pii_and_secrets_with_paths(
    raw: &str,
    home: Option<&Path>,
    project_root: Option<&Path>,
) -> String {
    let mut result = raw.to_string();

    // 1. Anonymize project root paths across OS path variants
    if let Some(root) = project_root {
        replace_path_variants(&mut result, root, "<project-root>");
    }

    // Anonymize current working directory if available
    if let Ok(cwd) = std::env::current_dir() {
        if cwd != Path::new("/") && cwd != Path::new(".") {
            replace_path_variants(&mut result, &cwd, "<project-root>");
        }
    }

    // 2. Anonymize home directory paths across OS path variants
    if let Some(h) = home {
        replace_path_variants(&mut result, h, "~");
    }

    // 3. Multi-line PEM private keys
    redact_pem_private_keys(&mut result);

    // 4. Line-by-line token, PII, and secret redaction
    let lines: Vec<String> = result.lines().map(redact_line).collect();
    let mut processed = lines.join("\n");

    // Preserve trailing newline if present in input
    if raw.ends_with('\n') && !processed.ends_with('\n') {
        processed.push('\n');
    }

    processed
}

fn redact_pem_private_keys(text: &mut String) {
    while let Some(start) = text.find("-----BEGIN") {
        if let Some(end_marker) = text[start..].find("-----END") {
            let tail = &text[start + end_marker..];
            if let Some(end_close) = tail.find("-----") {
                let abs_end = start + end_marker + end_close + 5;
                text.replace_range(start..abs_end, "[REDACTED_PRIVATE_KEY]");
                continue;
            }
        }
        break;
    }
}

fn redact_line(line: &str) -> String {
    let mut l = line.to_string();

    // 1. Strip GitHub Personal Access Tokens (classic & fine-grained)
    for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
        while let Some(idx) = l.find(prefix) {
            let token_len = l[idx..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .count();
            if token_len >= 20 {
                l.replace_range(idx..idx + token_len, "[REDACTED_GH_TOKEN]");
            } else {
                break;
            }
        }
    }

    while let Some(idx) = l.find("github_pat_") {
        let token_len = l[idx..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .count();
        if token_len >= 40 {
            l.replace_range(idx..idx + token_len, "[REDACTED_GH_TOKEN]");
        } else {
            break;
        }
    }

    // 2. Strip AWS Access Keys (AKIA...)
    let mut aws_search_idx = 0;
    while let Some(pos) = l[aws_search_idx..].find("AKIA") {
        let abs_pos = aws_search_idx + pos;
        let token_len = l[abs_pos..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .count();
        if token_len == 20 {
            l.replace_range(abs_pos..abs_pos + token_len, "[REDACTED_AWS_KEY]");
            aws_search_idx = abs_pos + "[REDACTED_AWS_KEY]".len();
        } else {
            aws_search_idx = abs_pos + 4;
        }
    }

    // 3. Strip sk- style API keys (OpenAI, Anthropic, etc.) and ts_ tokens
    for prefix in ["sk-", "ts_"] {
        let mut search_idx = 0;
        while let Some(pos) = l[search_idx..].find(prefix) {
            let abs_pos = search_idx + pos;
            let token_len = l[abs_pos..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .count();
            let min_len = if prefix == "sk-" { 16 } else { 8 };
            if token_len >= min_len {
                l.replace_range(abs_pos..abs_pos + token_len, "[REDACTED]");
                search_idx = abs_pos + "[REDACTED]".len();
            } else {
                search_idx = abs_pos + prefix.len();
            }
        }
    }

    // 4. Strip Authorization Bearer headers
    if let Some(pos) = l.find("Bearer ") {
        let after = &l[pos + 7..];
        let token_len = after
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'' && *c != ';' && *c != ',')
            .count();
        if token_len > 0 && !after[..token_len].starts_with("[REDACTED") {
            l.replace_range(pos + 7..pos + 7 + token_len, "[REDACTED]");
        }
    }

    // 5. Strip JWT tokens (eyJ...)
    let mut jwt_search = 0;
    while let Some(pos) = l[jwt_search..].find("eyJ") {
        let abs_pos = jwt_search + pos;
        let candidate = &l[abs_pos..];
        let token_len = candidate
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .count();
        let token = &candidate[..token_len];
        // Standard JWT has three dot-separated base64url segments
        let dots = token.matches('.').count();
        if dots == 2 && token.len() >= 30 {
            l.replace_range(abs_pos..abs_pos + token_len, "[REDACTED_JWT]");
            jwt_search = abs_pos + "[REDACTED_JWT]".len();
        } else {
            jwt_search = abs_pos + 3;
        }
    }

    // 6. Strip generic secret/api key key-value pairs (key=val or "key": "val")
    for keyword in [
        "password",
        "pass",
        "token",
        "secret",
        "api_key",
        "apikey",
        "api-key",
        "auth_token",
        "access_token",
        "refresh_token",
        "client_secret",
        "private_key",
    ] {
        for delim in [':', '='] {
            let mut search_idx = 0;
            while let Some(pos) = l[search_idx..].to_lowercase().find(keyword) {
                let abs_keyword_pos = search_idx + pos;
                let rem = &l[abs_keyword_pos + keyword.len()..];
                // Check if delimiter follows (ignoring whitespace and optional quote after keyword)
                let delim_pos = rem.find(delim);
                if let Some(dp) = delim_pos {
                    let between = &rem[..dp]
                        .trim_matches(|c: char| c.is_whitespace() || c == '"' || c == '\'');
                    if between.is_empty() {
                        let after_delim = &rem[dp + 1..];
                        let val_start = after_delim.find(|c: char| !c.is_whitespace());
                        if let Some(vs) = val_start {
                            let val_content = &after_delim[vs..];
                            let trimmed_val = val_content.trim_start_matches(['"', '\'']);
                            if trimmed_val.starts_with("[REDACTED") {
                                search_idx = abs_keyword_pos
                                    + keyword.len()
                                    + dp
                                    + 1
                                    + vs
                                    + "[REDACTED".len();
                                continue;
                            }
                            let val_len = val_content
                                .chars()
                                .take_while(|c| !c.is_whitespace() && *c != ',' && *c != ';')
                                .count();
                            let effective_len =
                                val_content[..val_len].trim_end_matches(['"', '\'']).len();
                            if effective_len >= 4 {
                                let abs_val_start = abs_keyword_pos + keyword.len() + dp + 1 + vs;
                                let is_quoted =
                                    val_content.starts_with('"') || val_content.starts_with('\'');
                                let quote_char = if is_quoted { &val_content[..1] } else { "" };
                                let replacement = if is_quoted {
                                    format!("{quote_char}[REDACTED]{quote_char}")
                                } else {
                                    "[REDACTED]".to_string()
                                };
                                l.replace_range(
                                    abs_val_start..abs_val_start + val_len,
                                    &replacement,
                                );
                                search_idx = abs_val_start + replacement.len();
                                continue;
                            }
                        }
                    }
                }
                search_idx = abs_keyword_pos + keyword.len();
            }
        }
    }

    // 7. Redact Credit Card Numbers (Luhn validated)
    l = redact_credit_cards(&l);

    // 8. Redact IBANs (ISO 7064 Mod 97-10 validated)
    l = redact_ibans(&l);

    // 9. Redact Emails
    l = redact_emails(&l);

    // 10. Redact Phone Numbers
    l = redact_phone_numbers(&l);

    l
}

fn redact_credit_cards(text: &str) -> String {
    let mut result = text.to_string();
    let bytes = result.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for start of a digit sequence
        if bytes[i].is_ascii_digit() && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let start = i;
            let mut j = i;
            let mut digit_count = 0;
            while j < len {
                if bytes[j].is_ascii_digit() {
                    digit_count += 1;
                    j += 1;
                } else if (bytes[j] == b' ' || bytes[j] == b'-')
                    && j + 1 < len
                    && bytes[j + 1].is_ascii_digit()
                {
                    j += 1;
                } else {
                    break;
                }
            }

            // Word boundary check at the end
            let at_boundary = j == len || !bytes[j].is_ascii_alphanumeric();

            if at_boundary && (13..=19).contains(&digit_count) {
                let candidate = &result[start..j];
                if is_luhn_valid(candidate) {
                    result.replace_range(start..j, "[REDACTED_CREDIT_CARD]");
                    return redact_credit_cards(&result);
                }
            }
            i = j.max(start + 1);
        } else {
            i += 1;
        }
    }

    result
}

fn redact_ibans(text: &str) -> String {
    let mut result = text.to_string();
    let bytes = result.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // IBAN starts with 2 uppercase letters followed by 2 digits
        if i + 4 <= len
            && bytes[i].is_ascii_uppercase()
            && bytes[i + 1].is_ascii_uppercase()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
        {
            let start = i;
            let mut j = i + 4;
            while j < len {
                if bytes[j].is_ascii_uppercase()
                    || bytes[j].is_ascii_digit()
                    || ((bytes[j] == b' ' || bytes[j] == b'-')
                        && j + 1 < len
                        && (bytes[j + 1].is_ascii_uppercase() || bytes[j + 1].is_ascii_digit()))
                {
                    j += 1;
                } else {
                    break;
                }
            }

            let at_boundary = j == len || !bytes[j].is_ascii_alphanumeric();
            if at_boundary {
                let candidate = &result[start..j];
                if is_iban_valid(candidate) {
                    result.replace_range(start..j, "[REDACTED_IBAN]");
                    return redact_ibans(&result);
                }
            }
            i = j.max(start + 1);
        } else {
            i += 1;
        }
    }

    result
}

fn redact_emails(text: &str) -> String {
    let mut result = text.to_string();
    let mut search_idx = 0;

    while let Some(at_pos) = result[search_idx..].find('@') {
        let abs_at = search_idx + at_pos;

        // Skip if inside [REDACTED...]
        if let Some(open_bracket) = result[..abs_at].rfind('[') {
            if result[open_bracket..abs_at].starts_with("[REDACTED") {
                if let Some(close_bracket) = result[abs_at..].find(']') {
                    search_idx = abs_at + close_bracket + 1;
                    continue;
                }
            }
        }

        // Find local part start
        let before = &result[..abs_at];
        let local_start = before
            .rfind(|c: char| {
                !c.is_ascii_alphanumeric()
                    && c != '.'
                    && c != '_'
                    && c != '%'
                    && c != '+'
                    && c != '-'
            })
            .map(|i| i + 1)
            .unwrap_or(0);

        let local_part = &result[local_start..abs_at];

        // Find domain part end
        let after = &result[abs_at + 1..];
        let domain_len = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
            .count();
        let domain_part = &after[..domain_len];

        // Check if git SSH clone URL (e.g. git@github.com:org/repo.git)
        let is_git_ssh = (local_part == "git" || local_part == "root")
            && (domain_part == "github.com"
                || domain_part == "gitlab.com"
                || domain_part == "bitbucket.org");

        if !local_part.is_empty()
            && domain_part.contains('.')
            && !is_git_ssh
            && !domain_part.ends_with('.')
        {
            let tld = domain_part.split('.').next_back().unwrap_or("");
            if tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()) {
                let abs_end = abs_at + 1 + domain_len;
                result.replace_range(local_start..abs_end, "[REDACTED_EMAIL]");
                search_idx = local_start + "[REDACTED_EMAIL]".len();
                continue;
            }
        }

        search_idx = abs_at + 1;
    }

    result
}

fn redact_phone_numbers(text: &str) -> String {
    let mut result = text.to_string();
    let bytes = result.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Case A: International format starting with '+'
        if bytes[i] == b'+'
            && (i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b'(')
        {
            let start = i;
            let mut j = i + 1;
            let mut digits = 0;
            while j < len
                && (bytes[j].is_ascii_digit()
                    || bytes[j] == b' '
                    || bytes[j] == b'-'
                    || bytes[j] == b'('
                    || bytes[j] == b')'
                    || bytes[j] == b'.')
            {
                if bytes[j].is_ascii_digit() {
                    digits += 1;
                }
                j += 1;
            }
            while j > start && !bytes[j - 1].is_ascii_digit() {
                j -= 1;
            }
            if (7..=15).contains(&digits) {
                result.replace_range(start..j, "[REDACTED_PHONE]");
                return redact_phone_numbers(&result);
            }
            i = j;
            continue;
        }

        // Case B: National formatted phone (e.g. 555-123-4567 or (555) 123-4567)
        // Avoid dates like 2026-11-14 (which have 4 digits in first block)
        if (bytes[i].is_ascii_digit() || bytes[i] == b'(')
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
        {
            let start = i;
            let mut j = i;
            let mut digits = 0;
            let mut first_block_digits = 0;
            let mut in_first_block = true;

            while j < len
                && (bytes[j].is_ascii_digit()
                    || bytes[j] == b' '
                    || bytes[j] == b'-'
                    || bytes[j] == b'('
                    || bytes[j] == b')'
                    || bytes[j] == b'.')
            {
                if bytes[j].is_ascii_digit() {
                    digits += 1;
                    if in_first_block {
                        first_block_digits += 1;
                    }
                } else if bytes[j] == b'-' || bytes[j] == b'/' {
                    in_first_block = false;
                }
                j += 1;
            }
            while j > start && !bytes[j - 1].is_ascii_digit() {
                j -= 1;
            }

            // Exclude dates (4 digits in year) and ensure format has separators and 10-11 digits
            let candidate = &result[start..j];
            let has_separator =
                candidate.contains('-') || candidate.contains('(') || candidate.contains(' ');
            if has_separator && (digits == 10 || digits == 11) && first_block_digits != 4 {
                result.replace_range(start..j, "[REDACTED_PHONE]");
                return redact_phone_numbers(&result);
            }
            i = j;
            continue;
        }

        i += 1;
    }

    result
}

fn replace_path_variants(text: &mut String, path: &Path, replacement: &str) {
    let mut candidates = Vec::new();
    let direct = path.to_string_lossy().to_string();
    if !direct.is_empty() {
        candidates.push(direct.clone());
        // Handle Git for Windows MSYS path format: /c/Users/... -> C:/Users/...
        if direct.len() >= 3
            && direct.starts_with('/')
            && direct.as_bytes()[2] == b'/'
            && direct.as_bytes()[1].is_ascii_alphabetic()
        {
            let drive = direct.as_bytes()[1] as char;
            let win_path = format!("{}:/{}", drive.to_uppercase(), &direct[3..]);
            if let Ok(canon) = Path::new(&win_path).canonicalize() {
                let canon_str = canon.to_string_lossy().to_string();
                if !canon_str.is_empty() {
                    candidates.push(canon_str);
                }
            }
            candidates.push(win_path);
        }
    }
    if let Ok(canon) = path.canonicalize() {
        let canon_str = canon.to_string_lossy().to_string();
        if !canon_str.is_empty() {
            candidates.push(canon_str);
        }
    }

    let mut expanded = Vec::new();
    for c in candidates {
        expanded.push(c.clone());
        if let Some(stripped) = c.strip_prefix(r"\\?\") {
            expanded.push(stripped.to_string());
        }
        if let Some(stripped) = c.strip_prefix("/private") {
            expanded.push(stripped.to_string());
        }
    }

    let mut final_variants = Vec::new();
    for item in expanded {
        let fwd = item.replace('\\', "/");
        let bwd = item.replace('/', "\\");
        for variant in [item, fwd, bwd] {
            if variant.is_empty() {
                continue;
            }
            final_variants.push(variant.clone());
            // Drive letter case variations on Windows (e.g. C: vs c:)
            if let Some(first) = variant.chars().next() {
                if variant.len() >= 2 && variant.as_bytes()[1] == b':' {
                    let upper = format!("{}{}", first.to_uppercase(), &variant[first.len_utf8()..]);
                    let lower = format!("{}{}", first.to_lowercase(), &variant[first.len_utf8()..]);
                    final_variants.push(upper);
                    final_variants.push(lower);
                }
            }
        }
    }

    // Sort descending by length so child paths are sanitized before parent dirs
    final_variants.sort_by_key(|b| std::cmp::Reverse(b.len()));
    final_variants.dedup();

    for v in final_variants {
        if !v.is_empty() {
            replace_ignore_ascii_case(text, &v, replacement);
        }
    }
}

fn replace_ignore_ascii_case(text: &mut String, pattern: &str, replacement: &str) {
    if pattern.is_empty() {
        return;
    }
    let lower_pattern = pattern.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;
    let text_lower = text.to_ascii_lowercase();

    while let Some(pos) = text_lower[last_end..].find(&lower_pattern) {
        let abs_pos = last_end + pos;
        result.push_str(&text[last_end..abs_pos]);
        result.push_str(replacement);
        last_end = abs_pos + pattern.len();
    }
    result.push_str(&text[last_end..]);
    *text = result;
}

#[cfg(test)]
#[path = "tests/pii_tests.rs"]
mod tests;
