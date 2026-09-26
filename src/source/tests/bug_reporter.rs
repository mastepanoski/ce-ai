use std::path::PathBuf;

use super::*;

#[test]
fn test_url_encode() {
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(url_encode("a/b?c=d&e=f"), "a%2Fb%3Fc%3Dd%26e%3Df");
    assert_eq!(url_encode("simple-test_1.0~"), "simple-test_1.0~");
}

#[test]
fn test_sanitize_paths() {
    let home = PathBuf::from("/Users/testuser");
    let project = PathBuf::from("/Users/testuser/projects/secretapp");

    let raw = "Error in /Users/testuser/projects/secretapp/src/main.rs: config at /Users/testuser/.ce-ai/state.json corrupt";
    let sanitized = sanitize_text(raw, Some(&home), Some(&project));

    assert!(
        sanitized.contains("<project-root>/src/main.rs"),
        "expected project root replaced, got: {sanitized}"
    );
    assert!(
        sanitized.contains("~/.ce-ai/state.json"),
        "expected home replaced with ~, got: {sanitized}"
    );
    assert!(!sanitized.contains("/Users/testuser"));
}

#[test]
fn test_sanitize_tokens_and_secrets() {
    let fake_ghp = format!("{}{}", "ghp_", "123456789012345678901234567890123456");
    let fake_pat = format!(
        "{}{}",
        "github_pat_", "11AAAAAAA0123456789abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQR"
    );
    let fake_openai = format!("{}{}", "sk-", "proj-12345678901234567890");
    let raw = format!(
        r#"
        API call failed with Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.t-ID5_OUeUsW Campus
        github token: {fake_ghp}
        fine-grained: {fake_pat}
        openai_key: "{fake_openai}"
        secret_password = supersecretpass123;
    "#
    );

    let sanitized = sanitize_text(&raw, None, None);

    assert!(sanitized.contains("Bearer [REDACTED]"));
    assert!(sanitized.contains("github token: [REDACTED_GH_TOKEN]"));
    assert!(sanitized.contains("fine-grained: [REDACTED_GH_TOKEN]"));
    assert!(sanitized.contains("openai_key: \"[REDACTED]\""));
    assert!(sanitized.contains("secret_password = [REDACTED];"));
    assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    assert!(!sanitized.contains(&fake_ghp));
    assert!(!sanitized.contains(&fake_openai));
    assert!(!sanitized.contains("supersecretpass123"));
}

#[test]
fn test_sanitize_private_key_block() {
    let begin_marker = format!("-----{}-----", "BEGIN RSA PRIVATE KEY");
    let end_marker = format!("-----{}-----", "END RSA PRIVATE KEY");
    let raw = format!(
        "Config loaded\n{begin_marker}\nMIIEowIBAAKCAQEA0Y3y\nABCDEF123456\n{end_marker}\nDone"
    );
    let sanitized = sanitize_text(&raw, None, None);

    assert!(sanitized.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(!sanitized.contains("MIIEowIBAAKCAQEA0Y3y"));
    assert!(!sanitized.contains("ABCDEF123456"));
}

#[test]
fn test_format_github_issue_body() {
    let bundle = BugReportBundle {
        ce_version: "1.70.0".to_string(),
        os: "macos".to_string(),
        arch: "aarch64".to_string(),
        target_harness: "opencode".to_string(),
        command_invoked: "ce-ai sync".to_string(),
        error_message: "manifest checksum verification failed".to_string(),
        logs_sanitized: "error: verification failed at ~/.config/opencode".to_string(),
    };

    let body = format_github_issue_body(&bundle);
    assert!(body.contains("### Bug Description\nmanifest checksum verification failed"));
    assert!(body.contains("### Operating System\nmacos (aarch64)"));
    assert!(body.contains("### Target Harness\nopencode"));
    assert!(body.contains("### Steps To Reproduce\n1. Invoked command: `ce-ai sync`"));
    assert!(body.contains("### CLI Logs & Error Output\n```shell\nerror: verification failed at ~/.config/opencode\n```"));
}

#[test]
fn test_generate_web_issue_url() {
    let url = generate_web_issue_url("[BUG]: Test Error", "Body text with spaces & special");
    assert!(url.starts_with("https://github.com/mastepanoski/ce-ai/issues/new?title="));
    assert!(url.contains("%5BBUG%5D%3A%20Test%20Error"));
    assert!(url.contains("&labels=bug"));
}

#[test]
fn test_install_instructions() {
    assert!(install_instructions("macos").contains("brew install gh"));
    assert!(install_instructions("linux").contains("apt install gh"));
    assert!(install_instructions("windows").contains("winget install"));
}

#[test]
fn test_sanitize_windows_paths_and_verbatim_prefixes() {
    let project = PathBuf::from(r"C:\Users\runneradmin\project\testapp");
    let raw_fwd = "Error in C:/Users/runneradmin/project/testapp/src/main.rs: crash";
    let sanitized_fwd = sanitize_text(raw_fwd, None, Some(&project));
    assert!(sanitized_fwd.contains("<project-root>/src/main.rs"));

    let raw_bwd = r"Error in C:\Users\runneradmin\project\testapp\src\main.rs: crash";
    let sanitized_bwd = sanitize_text(raw_bwd, None, Some(&project));
    assert!(sanitized_bwd.contains(r"<project-root>\src\main.rs"));

    let raw_lower = "Error in c:/users/runneradmin/project/testapp/src/main.rs: crash";
    let sanitized_lower = sanitize_text(raw_lower, None, Some(&project));
    assert!(sanitized_lower.contains("<project-root>/src/main.rs"));
}

#[test]
fn test_sanitize_git_msys_windows_paths() {
    let msys_project = PathBuf::from("/c/Users/runneradmin/project/testapp");
    let raw = r"Error in C:\Users\runneradmin\project\testapp\src\main.rs";
    let sanitized = sanitize_text(raw, None, Some(&msys_project));
    assert!(sanitized.contains(r"<project-root>\src\main.rs"));
}
