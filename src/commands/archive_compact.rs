//! Generational archive compaction and milestone rollups (`ce-ai archive compact`).
//!
//! Compacts aged, completed OpenSpec changes in `openspec/changes/archive/` into
//! consolidated milestone markdown documents and compressed raw tarballs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::commands::workflow::ProbeStatus;
use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::DocHygieneConfig;

#[derive(clap::Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ArchiveSubcommand {
    /// Compact aged archive packages into quarterly or milestone rollups.
    Compact(CompactArgs),
}

#[derive(clap::Args, Debug, Clone, Default, PartialEq, Eq)]
pub struct CompactArgs {
    /// Only compact archives created or dated before this date (YYYY-MM-DD).
    #[arg(long)]
    pub before: Option<String>,

    /// Explicit milestone name (defaults to auto-quarter e.g. '2026-Q3').
    #[arg(long)]
    pub milestone: Option<String>,

    /// Maximum uncompacted package count before compaction is triggered.
    #[arg(long)]
    pub threshold: Option<u32>,

    /// Preview compaction actions without modifying files or removing directories.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Do not generate a compressed tarball of the raw archives.
    #[arg(long, default_value_t = false)]
    pub no_tarball: bool,

    /// Keep loose directories after generating rollup and tarball (do not prune).
    #[arg(long, default_value_t = false)]
    pub keep_loose: bool,
}

/// Finding emitted when loose archive directories exceed the configured threshold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveCompactionFinding {
    pub uncompacted_count: usize,
    pub threshold: u32,
    pub oldest_package: Option<String>,
}

/// Metadata extracted for a candidate archive directory to be compacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionCandidate {
    pub folder_name: String,
    pub feature_name: String,
    pub path: PathBuf,
    pub date: chrono::NaiveDate,
    pub quarter: String,
    pub tasks_progress: (usize, usize),
    pub proposal_summary: String,
    pub spec_summary: String,
}

/// Determines the calendar quarter (e.g. `2026-Q3`) from a NaiveDate.
pub fn date_to_quarter(date: chrono::NaiveDate) -> String {
    let year = date.year();
    let q = match date.month() {
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        _ => 4,
    };
    format!("{}-Q{}", year, q)
}

/// Splits a `YYYY-MM-DD-` prefix from a folder name safely adhering to UTF-8 char boundaries.
pub fn split_date_prefix(folder_name: &str) -> Option<(chrono::NaiveDate, &str)> {
    if folder_name.len() >= 10 && folder_name.is_char_boundary(10) {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&folder_name[..10], "%Y-%m-%d") {
            if folder_name.len() == 10 {
                return Some((d, ""));
            } else if folder_name.as_bytes()[10] == b'-' && folder_name.is_char_boundary(11) {
                return Some((d, &folder_name[11..]));
            }
        }
    }
    None
}

/// Resolves the archive package date using multi-tier fallback:
/// 1. Directory name prefix `YYYY-MM-DD-`
/// 2. Git commit history (`git log -1 --format=%cs -- <path>`)
/// 3. Filesystem modification time (`mtime`)
/// 4. Current UTC date fallback
pub fn resolve_archive_package_date(
    repo_root: &Path,
    pkg_path: &Path,
    folder_name: &str,
) -> chrono::NaiveDate {
    // 1. Folder name prefix: YYYY-MM-DD-
    if let Some((d, _)) = split_date_prefix(folder_name) {
        return d;
    }

    // 2. Git commit date via sanitized git_probe
    if let Some(out) = crate::commands::workflow::git_probe(
        repo_root,
        &["log", "-1", "--format=%cs", "--", folder_name],
    ) {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.len() == 10 && s.is_char_boundary(10) {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                    return d;
                }
            }
        }
    }

    // 3. Filesystem mtime (consolidated stat check)
    let tasks_file = pkg_path.join("tasks.md");
    let meta_res = std::fs::metadata(&tasks_file).or_else(|_| std::fs::metadata(pkg_path));
    if let Ok(meta) = meta_res {
        if let Ok(mtime) = meta.modified() {
            let dt: chrono::DateTime<chrono::Utc> = mtime.into();
            return dt.date_naive();
        }
    }

    // 4. Fallback to current UTC date
    chrono::Utc::now().date_naive()
}

/// Extracts feature slug from folder name (stripping `YYYY-MM-DD-` prefix if present).
pub fn extract_feature_slug(folder_name: &str) -> String {
    if let Some((_, slug)) = split_date_prefix(folder_name) {
        if !slug.is_empty() {
            return slug.to_string();
        }
    }
    folder_name.to_string()
}

/// Summarizes a section from a markdown document up to max_len characters.
fn extract_markdown_summary(path: &Path, section_trigger: &str, max_len: usize) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return "Summary not available.".to_string(),
    };

    let trigger_lower = section_trigger.to_lowercase();
    let mut found_section = false;
    let mut collected = Vec::new();
    let mut current_len = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if !found_section {
            if trimmed.starts_with('#') && trimmed.to_lowercase().contains(&trigger_lower) {
                found_section = true;
            }
            continue;
        }

        if trimmed.starts_with('#') {
            break;
        }

        if trimmed.is_empty() {
            if !collected.is_empty() {
                break;
            }
            continue;
        }

        collected.push(trimmed);
        current_len += trimmed.len() + 1;
        if current_len >= max_len {
            break;
        }
    }

    if collected.is_empty() {
        // Fallback: take first non-heading, non-empty lines
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                collected.push(trimmed);
                if collected.len() >= 3 {
                    break;
                }
            }
        }
    }

    let joined = collected.join(" ");
    if joined.is_empty() {
        "Summary not available.".to_string()
    } else if joined.len() > max_len {
        let limit = max_len.saturating_sub(3);
        let boundary = joined
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= limit)
            .last()
            .unwrap_or(limit);
        format!("{}...", &joined[..boundary])
    } else {
        joined
    }
}

/// Collects all candidate archive packages from `openspec/changes/archive/`.
pub fn collect_compaction_candidates(
    repo_root: &Path,
    before: Option<chrono::NaiveDate>,
) -> Vec<CompactionCandidate> {
    let archive_dir = repo_root.join("openspec").join("changes").join("archive");
    let mut candidates = Vec::new();

    let entries = match std::fs::read_dir(&archive_dir) {
        Ok(e) => e,
        Err(_) => return candidates,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if folder_name == "milestones" || folder_name.starts_with('.') {
            continue;
        }

        let date = resolve_archive_package_date(repo_root, &path, &folder_name);
        if let Some(cutoff) = before {
            if date >= cutoff {
                continue;
            }
        }

        let feature_name = extract_feature_slug(&folder_name);
        let tasks_path = path.join("tasks.md");
        let tasks_progress = crate::commands::workflow::count_task_checkboxes(&tasks_path);

        let proposal_summary = extract_markdown_summary(&path.join("proposal.md"), "problem", 280);
        let spec_summary = extract_markdown_summary(&path.join("spec.md"), "scenario", 320);
        let quarter = date_to_quarter(date);

        candidates.push(CompactionCandidate {
            folder_name,
            feature_name,
            path,
            date,
            quarter,
            tasks_progress,
            proposal_summary,
            spec_summary,
        });
    }

    candidates.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| a.folder_name.cmp(&b.folder_name))
    });
    candidates
}

/// Groups candidates by milestone (either explicit custom milestone or by quarter).
pub fn group_candidates_by_milestone(
    candidates: Vec<CompactionCandidate>,
    explicit_milestone: Option<&str>,
) -> BTreeMap<String, Vec<CompactionCandidate>> {
    let mut map = BTreeMap::new();
    for c in candidates {
        let key = match explicit_milestone {
            Some(m) if !m.trim().is_empty() => m.trim().to_string(),
            _ => c.quarter.clone(),
        };
        map.entry(key).or_insert_with(Vec::new).push(c);
    }
    map
}

/// Generates the consolidated milestone markdown rollup document.
pub fn generate_milestone_markdown(
    milestone: &str,
    candidates: &[CompactionCandidate],
    tarball_name: Option<&str>,
) -> String {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let count = candidates.len();
    let mut md = String::new();

    md.push_str(&format!("# Milestone Archive Rollup: {milestone}\n\n"));
    md.push_str(&format!("- **Compaction Date:** {now}\n"));
    md.push_str(&format!("- **Total Changes Compacted:** {count}\n"));
    if let Some(tb) = tarball_name {
        md.push_str(&format!("- **Raw Archive Tarball:** [{tb}]({tb})\n"));
    }
    md.push_str("\n## Compacted Changes Summary\n\n");
    md.push_str("| Feature | Archive Date | Tasks Progress | Proposal Summary |\n");
    md.push_str("| :--- | :--- | :--- | :--- |\n");

    for c in candidates {
        let (completed, total) = c.tasks_progress;
        let prog = if total > 0 {
            format!("{completed}/{total}")
        } else {
            "n/a".to_string()
        };
        let clean_prop = c.proposal_summary.replace('|', "\\|").replace('\n', " ");
        let short_prop = if clean_prop.len() > 80 {
            let boundary = clean_prop
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= 77)
                .last()
                .unwrap_or(77);
            format!("{}...", &clean_prop[..boundary])
        } else {
            clean_prop
        };
        let slug_link = format!("#{}", c.feature_name.to_lowercase().replace(' ', "-"));
        md.push_str(&format!(
            "| [{}]({}) | {} | {} | {} |\n",
            c.feature_name, slug_link, c.date, prog, short_prop
        ));
    }

    md.push_str("\n## Detailed Specifications\n\n");
    for c in candidates {
        let (completed, total) = c.tasks_progress;
        let prog = if total > 0 {
            format!("{completed}/{total}")
        } else {
            "n/a".to_string()
        };
        md.push_str(&format!("### {}\n\n", c.feature_name));
        md.push_str(&format!("- **Original Folder:** `{}`\n", c.folder_name));
        md.push_str(&format!("- **Archived Date:** {}\n", c.date));
        md.push_str(&format!("- **Tasks Progress:** {}\n\n", prog));
        md.push_str("#### Problem Statement\n");
        md.push_str(&c.proposal_summary);
        md.push_str("\n\n#### Acceptance Criteria & Specification\n");
        md.push_str(&c.spec_summary);
        md.push_str("\n\n---\n\n");
    }

    md
}

/// Creates a compressed `.tar.gz` archive containing all candidate package directories.
pub fn create_milestone_tarball(
    tarball_path: &Path,
    candidates: &[CompactionCandidate],
) -> Result<(), CeError> {
    let file = std::fs::File::create(tarball_path)?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    for cand in candidates {
        tar.append_dir_all(&cand.folder_name, &cand.path)?;
    }

    let enc = tar.into_inner()?;
    enc.finish()?;
    Ok(())
}

/// Verifies that the created tarball is non-empty and contains entries for all candidate directories.
pub fn verify_milestone_tarball(
    tarball_path: &Path,
    expected_candidates: &[CompactionCandidate],
) -> Result<(), CeError> {
    let file = std::fs::File::open(tarball_path)
        .map_err(|_| CeError::Verification("tarball does not exist or cannot be opened".into()))?;
    let metadata = file.metadata()?;
    if metadata.len() == 0 {
        return Err(CeError::Verification("tarball file is empty".into()));
    }

    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);

    let mut found_folders = std::collections::HashSet::new();
    for entry_res in archive.entries()? {
        let entry = entry_res?;
        let path = entry.path()?;
        if let Some(std::path::Component::Normal(c)) = path.components().next() {
            if let Some(s) = c.to_str() {
                if !found_folders.contains(s) {
                    found_folders.insert(s.to_string());
                }
            }
        }
    }

    for cand in expected_candidates {
        if !found_folders.contains(&cand.folder_name) {
            return Err(CeError::Verification(format!(
                "tarball missing candidate directory '{}'",
                cand.folder_name
            )));
        }
    }

    Ok(())
}

/// Updates `openspec/changes/archive/README.md` with the newly compacted milestone details.
pub fn update_archive_readme_ledger(
    readme_path: &Path,
    milestone: &str,
    count: usize,
    tarball_name: Option<&str>,
) -> Result<(), CeError> {
    if !readme_path.exists() {
        return Ok(());
    }

    let original = std::fs::read_to_string(readme_path)?;
    let md_link = format!("[milestones/{milestone}.md](milestones/{milestone}.md)");
    let tb_link = if let Some(tb) = tarball_name {
        format!("[milestones/{tb}](milestones/{tb})")
    } else {
        "*(none)*".to_string()
    };

    let new_row = format!("| {milestone} | {count} | {md_link} | {tb_link} |\n");

    let updated = if original.contains("## Compacted Milestones") {
        if original.contains(&format!("| {milestone} |")) {
            // Already present, leave intact
            return Ok(());
        }
        // Insert into existing table after header
        let marker = "| :--- | :--- | :--- | :--- |";
        if let Some(pos) = original.find(marker) {
            let after_marker = &original[pos + marker.len()..];
            let newline_offset = if let Some(idx) = after_marker.find('\n') {
                idx + 1
            } else {
                0
            };
            let split_idx = pos + marker.len() + newline_offset;
            format!(
                "{}{}{}",
                &original[..split_idx],
                new_row,
                &original[split_idx..]
            )
        } else {
            original
        }
    } else {
        // Create new Compacted Milestones section above Triage
        let table_section = format!(
            "## Compacted Milestones\n\n| Milestone | Changes Compacted | Rollup Summary | Raw Archive |\n| :--- | :--- | :--- | :--- |\n{new_row}\n"
        );
        if let Some(pos) = original.find("## Triage") {
            format!("{}{}{}", &original[..pos], table_section, &original[pos..])
        } else {
            format!("{}\n\n{}", original.trim_end(), table_section)
        }
    };

    crate::state::write_atomic(readme_path, updated.as_bytes())?;
    Ok(())
}

/// Diagnostic probe checking if loose uncompacted archive packages exceed the threshold.
pub fn probe_archive_compaction(
    repo_root: &Path,
    config: &DocHygieneConfig,
) -> ProbeStatus<ArchiveCompactionFinding> {
    let archive_dir = repo_root.join("openspec").join("changes").join("archive");
    if !archive_dir.is_dir() {
        return ProbeStatus::Clean;
    }

    let entries = match std::fs::read_dir(&archive_dir) {
        Ok(e) => e,
        Err(_) => return ProbeStatus::Clean,
    };

    let mut eligible = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if name == "milestones" || name.starts_with('.') {
            continue;
        }
        eligible.push((path, name));
    }

    let count = eligible.len();
    if count <= config.archive_compaction_threshold as usize {
        return ProbeStatus::Clean;
    }

    let mut oldest_date = None;
    let mut oldest_pkg = None;

    for (path, name) in &eligible {
        let date = resolve_archive_package_date(repo_root, path, name);
        if oldest_date.is_none() || Some(date) < oldest_date {
            oldest_date = Some(date);
            oldest_pkg = Some(name.clone());
        }
    }

    ProbeStatus::Debt(ArchiveCompactionFinding {
        uncompacted_count: count,
        threshold: config.archive_compaction_threshold,
        oldest_package: oldest_pkg,
    })
}

/// Main execution logic for `ce-ai archive compact`.
pub fn run_archive_compact(ctx: &Context, args: &CompactArgs) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    let archive_dir = repo_root.join("openspec").join("changes").join("archive");

    if !archive_dir.is_dir() {
        println!(
            "archive compact: no archive directory found at {}",
            archive_dir.display()
        );
        return Ok(());
    }

    let before_date = if let Some(b) = &args.before {
        match chrono::NaiveDate::parse_from_str(b.trim(), "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                return Err(CeError::Usage(format!(
                    "invalid date format for --before: '{}'. Expected YYYY-MM-DD",
                    b
                )));
            }
        }
    } else {
        None
    };

    let candidates = collect_compaction_candidates(&repo_root, before_date);
    if candidates.is_empty() {
        println!("archive compact: no loose archive packages match the compaction criteria");
        return Ok(());
    }

    if let Some(thresh) = args.threshold {
        if candidates.len() <= thresh as usize {
            println!(
                "archive compact: {} candidate packages below threshold ({}) — no compaction needed",
                candidates.len(),
                thresh
            );
            return Ok(());
        }
    }

    let milestone_groups = group_candidates_by_milestone(candidates, args.milestone.as_deref());

    if args.dry_run {
        let total_cands: usize = milestone_groups.values().map(|v| v.len()).sum();
        println!(
            "dry-run: would compact {} package(s) into {} milestone rollup(s):",
            total_cands,
            milestone_groups.len()
        );

        for (milestone, cands) in &milestone_groups {
            let earliest = cands
                .first()
                .map(|c| c.date.to_string())
                .unwrap_or_default();
            let latest = cands.last().map(|c| c.date.to_string()).unwrap_or_default();
            let tb_note = if args.no_tarball { " (no tarball)" } else { "" };
            let loose_note = if args.keep_loose {
                " (keep loose)"
            } else {
                " (remove loose)"
            };

            println!(
                "  - Milestone '{}': {} package(s) [{}..{}]{}{}",
                milestone,
                cands.len(),
                earliest,
                latest,
                tb_note,
                loose_note
            );
            println!(
                "      rollup: openspec/changes/archive/milestones/{}.md",
                milestone
            );
            if !args.no_tarball {
                println!(
                    "      tarball: openspec/changes/archive/milestones/archive-{}.tar.gz",
                    milestone
                );
            }
        }
        return Ok(());
    }

    let milestones_dir = archive_dir.join("milestones");
    std::fs::create_dir_all(&milestones_dir)?;

    let readme_path = archive_dir.join("README.md");
    let mut total_compacted = 0;

    for (milestone, cands) in &milestone_groups {
        let rollup_path = milestones_dir.join(format!("{milestone}.md"));
        let tarball_name = if !args.no_tarball {
            Some(format!("archive-{milestone}.tar.gz"))
        } else {
            None
        };

        // 1. Write milestone markdown rollup
        let markdown = generate_milestone_markdown(milestone, cands, tarball_name.as_deref());
        crate::state::write_atomic(&rollup_path, markdown.as_bytes())?;

        // 2. Write and verify tarball if enabled
        if let Some(tb_name) = &tarball_name {
            let tb_path = milestones_dir.join(tb_name);
            create_milestone_tarball(&tb_path, cands)?;
            verify_milestone_tarball(&tb_path, cands)?;
        }

        // 3. Remove loose candidate directories ONLY if tarball was generated and not --keep-loose
        if !args.keep_loose && !args.no_tarball {
            for c in cands {
                if c.path.is_dir() {
                    std::fs::remove_dir_all(&c.path)?;
                }
            }
        }

        // 4. Update archive README ledger
        update_archive_readme_ledger(
            &readme_path,
            milestone,
            cands.len(),
            tarball_name.as_deref(),
        )?;

        total_compacted += cands.len();
        println!(
            "archive compact: compacted {} package(s) into milestone '{}'",
            cands.len(),
            milestone
        );
    }

    println!(
        "archive compact: successfully completed compaction ({} packages across {} milestone(s))",
        total_compacted,
        milestone_groups.len()
    );

    Ok(())
}

#[path = "tests/archive_compact.rs"]
#[cfg(test)]
mod tests;
