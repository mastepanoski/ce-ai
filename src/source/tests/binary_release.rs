use std::io::Write;

use crate::error::CeError;
use crate::source::binary_release::{
    asset_name_for_target, cleanup_stale_update_files, compare_cli_versions, current_target,
    extract_binary, extract_latest_tag_from_cli_atom_feed, extract_tag_from_cli_redirect_url,
    parse_cli_release_payload, parse_sha256sums, replace_executable, verify_checksum,
};

#[test]
fn test_current_target_resolves_known_triples() {
    let target = current_target();
    assert!(
        target.is_some(),
        "Host target must resolve to a supported triple"
    );
    let target_str = target.unwrap();
    let known = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ];
    assert!(
        known.contains(&target_str),
        "Resolved target '{target_str}' must be in official release targets"
    );
}

#[test]
fn test_asset_name_for_target_formats_tar_gz_and_zip() {
    assert_eq!(
        asset_name_for_target("aarch64-apple-darwin"),
        "ce-ai-aarch64-apple-darwin.tar.gz"
    );
    assert_eq!(
        asset_name_for_target("x86_64-unknown-linux-gnu"),
        "ce-ai-x86_64-unknown-linux-gnu.tar.gz"
    );
    assert_eq!(
        asset_name_for_target("x86_64-pc-windows-msvc"),
        "ce-ai-x86_64-pc-windows-msvc.zip"
    );
    assert_eq!(
        asset_name_for_target("aarch64-pc-windows-msvc"),
        "ce-ai-aarch64-pc-windows-msvc.zip"
    );
}

#[test]
fn test_parse_sha256sums_extracts_correct_digest() {
    let sums = r#"
50c9d1dda96e1869b0b9a5e17bfdea6457e82bed080fd1790a00e77f7084cd98  ce-ai-x86_64-apple-darwin.tar.gz
b087b2c925cfb99685d83b34bd41a5812e7a5e371a459b8581ae8c2f92e1e73f  ce-ai-aarch64-apple-darwin.tar.gz
21224ea066400129031a36f6eae90457b1a26fee406be004e8b8202e5b0b7b90  ce-ai-x86_64-pc-windows-msvc.zip
"#;
    assert_eq!(
        parse_sha256sums(sums, "ce-ai-aarch64-apple-darwin.tar.gz"),
        Some("b087b2c925cfb99685d83b34bd41a5812e7a5e371a459b8581ae8c2f92e1e73f".to_string())
    );
    assert_eq!(
        parse_sha256sums(sums, "ce-ai-x86_64-pc-windows-msvc.zip"),
        Some("21224ea066400129031a36f6eae90457b1a26fee406be004e8b8202e5b0b7b90".to_string())
    );
}

#[test]
fn test_parse_sha256sums_handles_missing_asset() {
    let sums = "b087b2c925cfb99685d83b34bd41a5812e7a5e371a459b8581ae8c2f92e1e73f  ce-ai-aarch64-apple-darwin.tar.gz";
    assert_eq!(parse_sha256sums(sums, "nonexistent-asset.tar.gz"), None);
}

#[test]
fn test_verify_checksum_accepts_matching_sha256() {
    let data = b"hello ce-ai self-update";
    use sha2::Digest;
    let expected = format!("{:x}", sha2::Sha256::digest(data));
    let result = verify_checksum(data, &expected);
    assert!(result.is_ok());
}

#[test]
fn test_verify_checksum_rejects_mismatch_with_verification_error() {
    let data = b"tampered payload";
    let bad_expected = "0000000000000000000000000000000000000000000000000000000000000000";
    let err = verify_checksum(data, bad_expected).unwrap_err();
    match err {
        CeError::Verification(ref msg) => {
            assert!(msg.contains("checksum mismatch"));
            assert_eq!(err.exit_code(), 6);
        }
        _ => panic!("Expected CeError::Verification, got {err:?}"),
    }
}

#[test]
fn test_extract_binary_from_tar_gz_success() {
    let payload = b"#!/bin/sh\necho ce-ai-mock\n";
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    {
        let mut tar = tar::Builder::new(&mut enc);
        let mut header = tar::Header::new_gnu();
        header.set_path("ce-ai").unwrap();
        header.set_size(payload.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, &payload[..]).unwrap();
        tar.finish().unwrap();
    }
    let tar_gz_bytes = enc.finish().unwrap();

    let extracted = extract_binary(&tar_gz_bytes, false).unwrap();
    assert_eq!(extracted, payload);
}

#[test]
fn test_extract_binary_from_zip_success() {
    let payload = b"MZ-mock-windows-executable";
    let mut zip_bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("ce-ai.exe", options).unwrap();
        zip.write_all(payload).unwrap();
        zip.finish().unwrap();
    }

    let extracted = extract_binary(&zip_bytes, true).unwrap();
    assert_eq!(extracted, payload);
}

#[test]
fn test_extract_binary_rejects_zip_slip_traversal_tar() {
    let payload = b"evil payload";
    let raw_path = b"../../evil-path/ce-ai";
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    {
        let mut tar = tar::Builder::new(&mut enc);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(payload.len() as u64);
        let name = &mut header.as_old_mut().name;
        name[..raw_path.len()].copy_from_slice(raw_path);
        header.set_cksum();
        tar.append(&header, &payload[..]).unwrap();
        tar.finish().unwrap();
    }
    let tar_gz_bytes = enc.finish().unwrap();

    let err = extract_binary(&tar_gz_bytes, false).unwrap_err();
    assert!(err.to_string().contains("unsafe archive entry path"));
}

#[test]
fn test_extract_binary_rejects_zip_slip_traversal_zip() {
    let payload = b"evil payload";
    let mut zip_bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("../evil/ce-ai.exe", options).unwrap();
        zip.write_all(payload).unwrap();
        zip.finish().unwrap();
    }

    let err = extract_binary(&zip_bytes, true).unwrap_err();
    assert!(err.to_string().contains("unsafe archive entry path"));
}

#[test]
fn test_extract_binary_rejects_empty_archive() {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    {
        let mut tar = tar::Builder::new(&mut enc);
        tar.finish().unwrap();
    }
    let tar_gz_bytes = enc.finish().unwrap();

    let err = extract_binary(&tar_gz_bytes, false).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_replace_executable_posix_atomic_swap_and_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let bin_path = dir.path().join("ce-ai");
    std::fs::write(&bin_path, b"old version").unwrap();

    let new_data = b"new binary version 1.50.0";
    let res = replace_executable(&bin_path, new_data);
    assert!(res.is_ok());

    let updated_content = std::fs::read(&bin_path).unwrap();
    assert_eq!(updated_content, new_data);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&bin_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}

#[test]
fn test_replace_executable_fails_on_unwritable_parent_directory() {
    let non_existent_dir = tempfile::tempdir().unwrap().path().join("does-not-exist");
    let bin_path = non_existent_dir.join("ce-ai");
    let res = replace_executable(&bin_path, b"bytes");
    assert!(res.is_err());
}

#[test]
fn test_cleanup_stale_update_files_removes_old_and_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let temp_file = dir.path().join(".ce-ai-self-update-123");
    let old_file = dir.path().join("ce-ai.old");
    let keep_file = dir.path().join("ce-ai");

    std::fs::write(&temp_file, b"temp").unwrap();
    std::fs::write(&old_file, b"old").unwrap();
    std::fs::write(&keep_file, b"active").unwrap();

    cleanup_stale_update_files(dir.path());

    assert!(!temp_file.exists(), "temp file should be removed");
    assert!(!old_file.exists(), "old file should be removed");
    assert!(keep_file.exists(), "active file should be kept");
}

#[test]
fn test_compare_cli_versions() {
    use std::cmp::Ordering;
    assert_eq!(compare_cli_versions("v1.49.0", "v1.49.0"), Ordering::Equal);
    assert_eq!(compare_cli_versions("1.49.0", "v1.49.0"), Ordering::Equal);
    assert_eq!(
        compare_cli_versions("v1.50.0", "v1.49.0"),
        Ordering::Greater
    );
    assert_eq!(compare_cli_versions("v1.49.0", "v1.50.0"), Ordering::Less);
    assert_eq!(compare_cli_versions("v1.10.0", "v1.9.0"), Ordering::Greater);
    assert_eq!(
        compare_cli_versions("v2.0.0", "v1.99.99"),
        Ordering::Greater
    );
}

#[test]
fn test_parse_cli_release_payload() {
    let payload = br#"{
        "tag_name": "v1.49.0",
        "assets": [
            {
                "name": "ce-ai-aarch64-apple-darwin.tar.gz",
                "browser_download_url": "https://github.com/mastepanoski/ce-ai/releases/download/v1.49.0/ce-ai-aarch64-apple-darwin.tar.gz"
            },
            {
                "name": "SHA256SUMS.txt",
                "browser_download_url": "https://github.com/mastepanoski/ce-ai/releases/download/v1.49.0/SHA256SUMS.txt"
            }
        ]
    }"#;

    let release = parse_cli_release_payload(payload).unwrap();
    assert_eq!(release.tag, "v1.49.0");
    assert_eq!(release.version, "1.49.0");
    assert_eq!(release.assets.len(), 2);
    assert_eq!(release.assets[0].name, "ce-ai-aarch64-apple-darwin.tar.gz");
    assert_eq!(
        release.assets[0].download_url,
        "https://github.com/mastepanoski/ce-ai/releases/download/v1.49.0/ce-ai-aarch64-apple-darwin.tar.gz"
    );
}

#[test]
fn test_extract_tag_from_cli_redirect_url() {
    assert_eq!(
        extract_tag_from_cli_redirect_url(
            "https://github.com/mastepanoski/ce-ai/releases/tag/v1.50.0"
        ),
        Some("v1.50.0".to_string())
    );
    assert_eq!(
        extract_tag_from_cli_redirect_url(
            "https://github.com/mastepanoski/ce-ai/releases/tag/v1.50.0/"
        ),
        Some("v1.50.0".to_string())
    );
    assert_eq!(
        extract_tag_from_cli_redirect_url(
            "https://github.com/mastepanoski/ce-ai/releases/tag/invalid"
        ),
        None
    );
}

#[test]
fn test_extract_latest_tag_from_cli_atom_feed() {
    let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>tag:github.com,2008:Repository/123/v1.48.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/mastepanoski/ce-ai/releases/tag/v1.48.0"/>
    <title>v1.48.0</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/123/v1.50.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/mastepanoski/ce-ai/releases/tag/v1.50.0"/>
    <title>v1.50.0</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/123/v1.49.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/mastepanoski/ce-ai/releases/tag/v1.49.0"/>
    <title>v1.49.0</title>
  </entry>
</feed>"#;

    assert_eq!(
        extract_latest_tag_from_cli_atom_feed(feed),
        Some("v1.50.0".to_string())
    );
}
