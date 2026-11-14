use std::path::PathBuf;

use super::*;

#[test]
fn test_luhn_validation_valid_cards() {
    // Valid Visa 16 digits
    assert!(is_luhn_valid("4532015012345671"));
    // Valid Visa with spaces and hyphens
    assert!(is_luhn_valid("4532 0150 1234 5671"));
    assert!(is_luhn_valid("4532-0150-1234-5671"));

    // Valid Diners Club 14 digits
    assert!(is_luhn_valid("30569309025904"));

    // Valid Amex 15 digits
    assert!(is_luhn_valid("378282246310005"));
    assert!(is_luhn_valid("3782-822463-10005"));

    // Valid Mastercard 16 digits
    assert!(is_luhn_valid("5424180123456789"));
}

#[test]
fn test_luhn_validation_invalid_cards() {
    // Bad check digit
    assert!(!is_luhn_valid("4532015012345677"));
    assert!(!is_luhn_valid("4532 0150 1234 5677"));

    // Too short (< 13 digits)
    assert!(!is_luhn_valid("123456789012"));

    // Too long (> 19 digits)
    assert!(!is_luhn_valid("12345678901234567890"));

    // All zeroes
    assert!(!is_luhn_valid("0000000000000000"));

    // Characters that are letters or symbols
    assert!(!is_luhn_valid("453201501234567a"));
    assert!(!is_luhn_valid("4532.0150.1234.5676"));
}

#[test]
fn test_iban_validation_valid() {
    // Valid German IBAN
    assert!(is_iban_valid("DE89370400440532013000"));
    assert!(is_iban_valid("DE89 3704 0044 0532 0130 00"));

    // Valid Spanish IBAN
    assert!(is_iban_valid("ES9121000418450200051332"));
    assert!(is_iban_valid("ES91-2100-0418-4502-0005-1332"));

    // Valid British IBAN
    assert!(is_iban_valid("GB82WEST12345698765432"));
    assert!(is_iban_valid("GB82 WEST 1234 5698 7654 32"));
}

#[test]
fn test_iban_validation_invalid() {
    // Bad check digits
    assert!(!is_iban_valid("DE88370400440532013000"));

    // Too short (< 15 chars)
    assert!(!is_iban_valid("DE8912345678"));

    // Non-country code prefix
    assert!(!is_iban_valid("1289370400440532013000"));

    // Non-digit check digits
    assert!(!is_iban_valid("DEXX370400440532013000"));
}

#[test]
fn test_redact_credit_cards_and_preserve_dates() {
    let input = "Order 4532-0150-1234-5671 processed on 2026-11-14 with id 9999888877776666";
    let redacted = redact_pii_and_secrets(input);

    assert!(
        redacted.contains("[REDACTED_CREDIT_CARD]"),
        "expected card redacted, got: {redacted}"
    );
    assert!(!redacted.contains("4532-0150-1234-5671"));
    // Date must NOT be redacted
    assert!(
        redacted.contains("2026-11-14"),
        "expected date preserved, got: {redacted}"
    );
    // Non-luhn 16 digit number must NOT be redacted as credit card
    assert!(redacted.contains("9999888877776666"));
}

#[test]
fn test_redact_ibans() {
    let input = "Send payment to DE89 3704 0044 0532 0130 00 immediately.";
    let redacted = redact_pii_and_secrets(input);

    assert!(
        redacted.contains("[REDACTED_IBAN]"),
        "expected IBAN redacted, got: {redacted}"
    );
    assert!(!redacted.contains("DE89 3704 0044 0532 0130 00"));
}

#[test]
fn test_redact_emails_and_preserve_git_ssh() {
    let input = "User alice.smith@example.com cloned git@github.com:mastepanoski/ce-ai.git and emailed bob+test@corp.co.uk";
    let redacted = redact_pii_and_secrets(input);

    assert!(
        redacted.contains("[REDACTED_EMAIL]"),
        "expected emails redacted, got: {redacted}"
    );
    assert!(!redacted.contains("alice.smith@example.com"));
    assert!(!redacted.contains("bob+test@corp.co.uk"));
    // Git SSH url must NOT be redacted
    assert!(
        redacted.contains("git@github.com:mastepanoski/ce-ai.git"),
        "expected git ssh clone preserved, got: {redacted}"
    );
}

#[test]
fn test_redact_phone_numbers() {
    let input = "Call support at +1-800-555-0199 or (555) 123-4567, or EU +34 612 34 56 78";
    let redacted = redact_pii_and_secrets(input);

    assert!(
        redacted.contains("[REDACTED_PHONE]"),
        "expected phones redacted, got: {redacted}"
    );
    assert!(!redacted.contains("+1-800-555-0199"));
    assert!(!redacted.contains("(555) 123-4567"));
    assert!(!redacted.contains("+34 612 34 56 78"));
}

#[test]
fn test_redact_tokens_and_keys() {
    let fake_ghp = format!("{}{}", "ghp_", "123456789012345678901234567890123456");
    let fake_pat = format!(
        "{}{}",
        "github_pat_", "11AAAAAAA0123456789abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQR"
    );
    let fake_openai = format!("{}{}", "sk-", "proj-12345678901234567890");
    let fake_aws = format!("{}{}", "AKIA", "IOSFODNN7EXAMPLE");
    let fake_jwt = format!(
        "{}{}{}",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.",
        "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.",
        "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
    );

    let input = format!(
        "Tokens:\n\
        ghp={fake_ghp}\n\
        pat={fake_pat}\n\
        openai=\"{fake_openai}\"\n\
        aws={fake_aws}\n\
        auth=Bearer {fake_jwt}\n\
        secret_password = mysecretpassword123;\n"
    );

    let redacted = redact_pii_and_secrets(&input);

    assert!(redacted.contains("[REDACTED_GH_TOKEN]"));
    assert!(redacted.contains("[REDACTED]"));
    assert!(redacted.contains("[REDACTED_AWS_KEY]"));
    assert!(redacted.contains("Bearer [REDACTED]"));
    assert!(!redacted.contains(&fake_ghp));
    assert!(!redacted.contains(&fake_pat));
    assert!(!redacted.contains(&fake_openai));
    assert!(!redacted.contains(&fake_aws));
    assert!(!redacted.contains("mysecretpassword123"));
}

#[test]
fn test_redact_pem_private_keys() {
    let begin_marker = "-----BEGIN RSA PRIVATE KEY-----";
    let end_marker = "-----END RSA PRIVATE KEY-----";
    let input = format!(
        "Certificate check:\n{begin_marker}\nMIIEowIBAAKCAQEA0Y3y\nABCDEF123456\n{end_marker}\nDone."
    );

    let redacted = redact_pii_and_secrets(&input);
    assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(!redacted.contains("MIIEowIBAAKCAQEA0Y3y"));
    assert!(!redacted.contains("ABCDEF123456"));
    assert!(redacted.contains("Certificate check:"));
    assert!(redacted.contains("Done."));
}

#[test]
fn test_redact_with_paths() {
    let home = PathBuf::from("/Users/alice");
    let project = PathBuf::from("/Users/alice/projects/topsecret");

    let raw =
        "Crash in /Users/alice/projects/topsecret/src/lib.rs: file /Users/alice/.ce-ai/state.json";
    let redacted = redact_pii_and_secrets_with_paths(raw, Some(&home), Some(&project));

    assert!(redacted.contains("<project-root>/src/lib.rs"));
    assert!(redacted.contains("~/.ce-ai/state.json"));
    assert!(!redacted.contains("/Users/alice"));
}

#[test]
fn test_contains_pii_or_secrets() {
    assert!(contains_pii_or_secrets("Contact user@example.com for info"));
    assert!(contains_pii_or_secrets("api_key = secret1234"));
    assert!(contains_pii_or_secrets("Card: 4532 0150 1234 5671"));
    assert!(!contains_pii_or_secrets("Running build clean"));
}
