use std::path::PathBuf;

use tempfile::tempdir;

use super::*;
use crate::commands::Context;
use crate::source::cache::{record_tarball_provenance, Cache};

fn ctx_in(dir: &Path) -> Context {
    Context {
        config_dir: dir.to_path_buf(),
        opencode_config_dir: dir.join(".config/opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    }
}

/// Seeds state.json with provenance for `tag` and caches matching bytes.
fn seed_cache(ctx: &Context, tag: &str, bytes: &[u8]) {
    let (_, hex) = Cache::new(ctx.config_dir.join("cache"))
        .cache_tarball(bytes)
        .unwrap();
    record_tarball_provenance(
        &ctx.config_dir.join("state.json"),
        ReleaseProvenance {
            tag: tag.into(),
            url: format!("https://example.test/ce-{tag}.tar.gz"),
            archive_sha256: hex,
            extraction_path: ctx.config_dir.join("cache/trees").join(tag),
        },
    )
    .unwrap();
}

#[test]
fn to_tag_mismatch_fails_without_relabeling() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    seed_cache(&ctx, "v1.0.0", b"release-v1-archive");

    let err = cached_tarball_for(&ctx, "v9.9.9").unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("v1.0.0"));
    assert!(err.to_string().contains("v9.9.9"));

    // State must still bind the artifact to v1.0.0 — never relabelled.
    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    assert_eq!(
        state.release_provenance.as_ref().map(|p| p.tag.as_str()),
        Some("v1.0.0")
    );
}

#[test]
fn missing_provenance_is_a_usage_error() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    let err = cached_tarball_for(&ctx, "v1.0.0").unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("no release provenance"));
}

#[test]
fn tampered_cache_fails_closed() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    seed_cache(&ctx, "v1.0.0", b"original-bytes");

    // Tamper: rewrite the cached archive in place.
    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    let digest_hex = state
        .release_provenance
        .as_ref()
        .unwrap()
        .archive_sha256
        .clone();
    let tarball_path = ctx
        .config_dir
        .join("cache")
        .join(format!("ce-{digest_hex}.tar.gz"));
    std::fs::write(&tarball_path, b"tampered-payload").unwrap();

    let err = cached_tarball_for(&ctx, "v1.0.0").unwrap_err();
    assert_eq!(err.exit_code(), 6);
    assert!(err.to_string().contains("integrity check failed"));
    assert!(err.to_string().contains("tampered") || err.to_string().contains("expected"));
}

#[test]
fn digest_provenance_divergence_fails_verification() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    seed_cache(&ctx, "v1.0.0", b"release-v1-archive");

    // Corrupt only the digest bookkeeping, not the archive.
    let mut state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    state
        .managed_asset_digest
        .insert("tarball".into(), "sha256:stale".into());
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    let err = cached_tarball_for(&ctx, "v1.0.0").unwrap_err();
    assert_eq!(err.exit_code(), 6);
    assert!(err.to_string().contains("mismatch"));
}

#[test]
fn matching_tag_resolves_intact_cache() {
    let dir = tempdir().unwrap();
    let ctx = ctx_in(dir.path());
    let bytes = b"release-v1-archive";
    seed_cache(&ctx, "v1.0.0", bytes);

    let resolved = cached_tarball_for(&ctx, "v1.0.0").unwrap();
    assert_eq!(std::fs::read(&resolved).unwrap(), bytes);
}

#[derive(clap::Parser)]
struct Cli {
    #[command(flatten)]
    args: Args,
}

use clap::Parser as _;

#[test]
fn dead_flags_are_rejected_with_usage_errors() {
    // --harness/-t and --force/-f were accepted-and-ignored before #161;
    // removing them makes clap reject them as unknown arguments (exit 2).
    assert!(Cli::try_parse_from(["upgrade", "--harness", "claude"]).is_err());
    assert!(Cli::try_parse_from(["upgrade", "-t", "claude"]).is_err());
    assert!(Cli::try_parse_from(["upgrade", "--force"]).is_err());
    assert!(Cli::try_parse_from(["upgrade", "-f"]).is_err());

    let ok = Cli::try_parse_from(["upgrade", "--to", "v1.2.3"]).unwrap();
    assert_eq!(ok.args.to.as_deref(), Some("v1.2.3"));
    assert_eq!(ok.args.source, None::<PathBuf>);
}
