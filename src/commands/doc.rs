//! Solution library clustering, deduplication, and refresh engine (`ce-ai doc`).
//!
//! Analyzes documents in `docs/solutions/`, groups them into semantic consolidation clusters,
//! checks path/frontmatter integrity, and generates scope directives for `/ce-compound-refresh`.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "in", "on", "at", "to", "for", "of", "with", "by", "from", "as",
    "is", "are", "was", "were", "it", "this", "that", "into", "via", "across", "over", "under",
    "between",
];

#[derive(Args, Debug, Clone)]
pub struct DocArgs {
    #[command(subcommand)]
    pub command: DocCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DocCommand {
    /// Analyze solution files and group them into semantic consolidation clusters.
    Cluster {
        /// Minimum number of solutions required to form a cluster.
        #[arg(long, default_value_t = 3)]
        min_size: usize,
        /// Minimum similarity coefficient threshold (0.0 to 1.0).
        #[arg(long, default_value_t = 0.40)]
        threshold: f32,
        /// Output results as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Audit a cluster or specific scope for refresh and consolidation.
    Refresh {
        /// Scope hint (category, module, tag, or cluster name).
        scope: Option<String>,
        /// Dry run preview without suggesting modifications.
        #[arg(long)]
        dry_run: bool,
    },
    /// Audit solution files for missing frontmatter and dead path references.
    Lint {
        /// Fail with non-zero exit code if warnings are found.
        #[arg(long)]
        strict: bool,
        /// Output report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Display inventory statistics for the solution library.
    Stats {
        /// Output statistics as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionMetadata {
    pub file_path: PathBuf,
    pub rel_path: String,
    pub title: String,
    pub category: String,
    pub problem_type: String,
    pub tags: Vec<String>,
    pub components: Vec<String>,
    pub applies_when: String,
    pub date: String,
    #[serde(skip)]
    pub title_tokens: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionClusterMember {
    pub rel_path: String,
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionCluster {
    pub id: String,
    pub name: String,
    pub dominant_tags: Vec<String>,
    pub suggested_refresh_scope: String,
    pub members: Vec<SolutionClusterMember>,
    pub average_similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterReport {
    pub total_solutions: usize,
    pub clusters_count: usize,
    pub clustered_solutions_count: usize,
    pub clusters: Vec<SolutionCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocStatsReport {
    pub total_solutions: usize,
    pub category_distribution: BTreeMap<String, usize>,
    pub top_tags: Vec<(String, usize)>,
    pub clusters_count: usize,
}

/// Dispatches `ce-ai doc` subcommands.
pub fn run_doc(ctx: &Context, args: &DocArgs) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    match &args.command {
        DocCommand::Cluster {
            min_size,
            threshold,
            json,
        } => run_doc_cluster(&repo_root, *min_size, *threshold, *json),
        DocCommand::Refresh { scope, dry_run } => {
            run_doc_refresh(&repo_root, scope.as_deref(), *dry_run)
        }
        DocCommand::Lint { strict, json } => run_doc_lint(&repo_root, *strict, *json),
        DocCommand::Stats { json } => run_doc_stats(&repo_root, *json),
    }
}

/// Tokenizes titles into lowercase normalized keywords filtering stopwords.
pub fn tokenize_title(title: &str) -> HashSet<String> {
    title
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .map(|w| w.trim().to_lowercase())
        .filter(|w| w.len() >= 3 && !STOPWORDS.contains(&w.as_str()))
        .collect()
}

/// Computes the Jaccard similarity coefficient between two sets: |A ∩ B| / |A ∪ B|.
pub fn jaccard_similarity<T: std::hash::Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Calculates multi-dimensional similarity between two solution documents.
pub fn calculate_solution_similarity(a: &SolutionMetadata, b: &SolutionMetadata) -> f32 {
    let a_tags: HashSet<String> = a.tags.iter().cloned().collect();
    let b_tags: HashSet<String> = b.tags.iter().cloned().collect();
    let tag_sim = jaccard_similarity(&a_tags, &b_tags);

    let a_comps: HashSet<String> = a.components.iter().cloned().collect();
    let b_comps: HashSet<String> = b.components.iter().cloned().collect();
    let comp_sim = jaccard_similarity(&a_comps, &b_comps);

    let title_sim = jaccard_similarity(&a.title_tokens, &b.title_tokens);

    let cat_sim = if !a.category.is_empty() && a.category == b.category {
        1.0
    } else {
        0.0
    };

    0.35 * tag_sim + 0.25 * comp_sim + 0.25 * title_sim + 0.15 * cat_sim
}

/// Parses a single solution document and extracts frontmatter metadata.
pub fn parse_solution_file(repo_root: &Path, file_path: &Path) -> Result<SolutionMetadata, String> {
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let rel_path = file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let mut lines = content.trim_start().lines();
    let first = lines.next().ok_or_else(|| "empty file".to_string())?.trim();
    if first != "---" {
        return Err("missing opening YAML delimiter (---)".to_string());
    }

    let mut header_lines = Vec::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        header_lines.push(line);
    }
    if !closed {
        return Err("missing closing YAML delimiter (---)".to_string());
    }

    let mut title = None;
    let mut category = None;
    let mut problem_type = None;
    let mut date = None;
    let mut applies_when = None;
    let mut tags = Vec::new();
    let mut components = Vec::new();

    enum ActiveList {
        Tags,
        Components,
        None,
    }
    let mut active_list = ActiveList::None;

    for line in header_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = v.trim();
                active_list = match key.as_str() {
                    "title" => {
                        title = Some(val.trim_matches('"').trim_matches('\'').to_string());
                        ActiveList::None
                    }
                    "category" | "module" => {
                        category = Some(val.trim_matches('"').trim_matches('\'').to_string());
                        ActiveList::None
                    }
                    "problem_type" => {
                        problem_type = Some(val.trim_matches('"').trim_matches('\'').to_string());
                        ActiveList::None
                    }
                    "date" => {
                        date = Some(val.trim_matches('"').trim_matches('\'').to_string());
                        ActiveList::None
                    }
                    "applies_when" => {
                        applies_when = Some(val.trim_matches('"').trim_matches('\'').to_string());
                        ActiveList::None
                    }
                    "tags" => {
                        if val.starts_with('[') && val.ends_with(']') {
                            let inner = &val[1..val.len() - 1];
                            for item in inner.split(',') {
                                let clean = item.trim().trim_matches('"').trim_matches('\'');
                                if !clean.is_empty() {
                                    tags.push(clean.to_lowercase());
                                }
                            }
                            ActiveList::None
                        } else {
                            ActiveList::Tags
                        }
                    }
                    "components" => {
                        if val.starts_with('[') && val.ends_with(']') {
                            let inner = &val[1..val.len() - 1];
                            for item in inner.split(',') {
                                let clean = item.trim().trim_matches('"').trim_matches('\'');
                                if !clean.is_empty() {
                                    components.push(clean.to_string());
                                }
                            }
                            ActiveList::None
                        } else {
                            ActiveList::Components
                        }
                    }
                    _ => ActiveList::None,
                };
            }
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            let clean = item.trim().trim_matches('"').trim_matches('\'');
            if !clean.is_empty() {
                match active_list {
                    ActiveList::Tags => tags.push(clean.to_lowercase()),
                    ActiveList::Components => components.push(clean.to_string()),
                    ActiveList::None => {}
                }
            }
        }
    }

    let parsed_title = title.unwrap_or_else(|| {
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string()
    });
    let title_tokens = tokenize_title(&parsed_title);

    let parsed_category = category.unwrap_or_else(|| {
        file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("general")
            .to_string()
    });

    Ok(SolutionMetadata {
        file_path: file_path.to_path_buf(),
        rel_path,
        title: parsed_title,
        category: parsed_category,
        problem_type: problem_type.unwrap_or_else(|| "architecture".to_string()),
        tags,
        components,
        applies_when: applies_when.unwrap_or_default(),
        date: date.unwrap_or_default(),
        title_tokens,
    })
}

/// Recursively collects all solution documents under `docs/solutions/`.
pub fn collect_solutions_inventory(repo_root: &Path) -> Vec<SolutionMetadata> {
    let solutions_dir = repo_root.join("docs").join("solutions");
    if !solutions_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_solution_files(&solutions_dir, &mut files);
    files.sort();

    let mut solutions = Vec::new();
    for file in files {
        if let Ok(meta) = parse_solution_file(repo_root, &file) {
            solutions.push(meta);
        }
    }
    solutions
}

fn collect_solution_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_solution_files(&p, files);
            } else if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(p);
            }
        }
    }
}

/// Partitions solutions into connected clusters based on similarity threshold.
pub fn cluster_solutions(
    solutions: &[SolutionMetadata],
    threshold: f32,
    min_size: usize,
) -> Vec<SolutionCluster> {
    let n = solutions.len();
    if n < min_size {
        return Vec::new();
    }

    // Build adjacency list
    let mut adj = vec![Vec::new(); n];
    let mut pairwise_sims = BTreeMap::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let sim = calculate_solution_similarity(&solutions[i], &solutions[j]);
            pairwise_sims.insert((i, j), sim);
            if sim >= threshold {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    let mut visited = vec![false; n];
    let mut raw_clusters = Vec::new();

    for i in 0..n {
        if !visited[i] {
            let mut component = Vec::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(i);
            visited[i] = true;

            while let Some(curr) = queue.pop_front() {
                component.push(curr);
                for &neighbor in &adj[curr] {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        queue.push_back(neighbor);
                    }
                }
            }

            if component.len() >= min_size {
                raw_clusters.push(component);
            }
        }
    }

    let mut clusters = Vec::new();

    for (cluster_idx, member_indices) in raw_clusters.iter().enumerate() {
        let mut members = Vec::new();
        let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut total_sim = 0.0;
        let mut pair_count = 0;

        for &idx in member_indices {
            let s = &solutions[idx];
            members.push(SolutionClusterMember {
                rel_path: s.rel_path.clone(),
                title: s.title.clone(),
                date: s.date.clone(),
                tags: s.tags.clone(),
            });

            for tag in &s.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        // Calculate average similarity
        for i in 0..member_indices.len() {
            for j in (i + 1)..member_indices.len() {
                let idx1 = member_indices[i];
                let idx2 = member_indices[j];
                let (min_idx, max_idx) = if idx1 < idx2 {
                    (idx1, idx2)
                } else {
                    (idx2, idx1)
                };
                if let Some(&s) = pairwise_sims.get(&(min_idx, max_idx)) {
                    total_sim += s;
                    pair_count += 1;
                }
            }
        }

        let avg_sim = if pair_count > 0 {
            total_sim / (pair_count as f32)
        } else {
            1.0
        };

        let mut sorted_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let dominant_tags: Vec<String> = sorted_tags
            .iter()
            .take(3)
            .map(|(tag, _)| tag.clone())
            .collect();

        let top_tag = dominant_tags
            .first()
            .cloned()
            .unwrap_or_else(|| "general".to_string());
        let second_tag = dominant_tags.get(1).cloned();

        let cluster_name = match second_tag {
            Some(second) => format!("{top_tag}-{second}"),
            None => top_tag.clone(),
        };

        let suggested_scope = top_tag;

        // Sort members by date descending
        members.sort_by(|a, b| {
            b.date
                .cmp(&a.date)
                .then_with(|| a.rel_path.cmp(&b.rel_path))
        });

        clusters.push(SolutionCluster {
            id: format!("cluster-{}", cluster_idx + 1),
            name: cluster_name,
            dominant_tags,
            suggested_refresh_scope: suggested_scope,
            members,
            average_similarity: avg_sim,
        });
    }

    // Sort clusters by size descending, then similarity descending
    clusters.sort_by(|a, b| {
        b.members.len().cmp(&a.members.len()).then_with(|| {
            b.average_similarity
                .partial_cmp(&a.average_similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    clusters
}

fn run_doc_cluster(
    repo_root: &Path,
    min_size: usize,
    threshold: f32,
    json: bool,
) -> Result<(), CeError> {
    let solutions = collect_solutions_inventory(repo_root);
    let clusters = cluster_solutions(&solutions, threshold, min_size);

    let clustered_count: usize = clusters.iter().map(|c| c.members.len()).sum();
    let report = ClusterReport {
        total_solutions: solutions.len(),
        clusters_count: clusters.len(),
        clustered_solutions_count: clustered_count,
        clusters,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!(
        "== [Solution Library Clustering (Total: {}, Clustered: {}, Clusters: {})] ==",
        report.total_solutions, report.clustered_solutions_count, report.clusters_count
    );

    if report.clusters.is_empty() {
        println!(
            "doc cluster: no clusters found with >= {} solutions (threshold: {:.2})",
            min_size, threshold
        );
        return Ok(());
    }

    for c in &report.clusters {
        println!();
        println!(
            "Cluster {} · {} ({} solutions, avg similarity: {:.2})",
            c.id,
            c.name,
            c.members.len(),
            c.average_similarity
        );
        println!("  Dominant Tags: [{}]", c.dominant_tags.join(", "));
        println!(
            "  Suggested Scope: /ce-compound-refresh {}",
            c.suggested_refresh_scope
        );
        println!("  Member Solutions:");
        for m in &c.members {
            let date_str = if m.date.is_empty() {
                String::new()
            } else {
                format!(" ({})", m.date)
            };
            println!("    • {}{}: {}", m.rel_path, date_str, m.title);
        }
    }

    Ok(())
}

fn run_doc_refresh(repo_root: &Path, scope: Option<&str>, dry_run: bool) -> Result<(), CeError> {
    let solutions = collect_solutions_inventory(repo_root);

    let matching_docs: Vec<&SolutionMetadata> = match scope {
        Some(query) => {
            let q = query.to_lowercase();
            solutions
                .iter()
                .filter(|s| {
                    s.category.to_lowercase().contains(&q)
                        || s.rel_path.to_lowercase().contains(&q)
                        || s.title.to_lowercase().contains(&q)
                        || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
                })
                .collect()
        }
        None => {
            // Find highest density cluster
            let clusters = cluster_solutions(&solutions, 0.40, 3);
            if let Some(top_cluster) = clusters.first() {
                let member_paths: HashSet<&str> = top_cluster
                    .members
                    .iter()
                    .map(|m| m.rel_path.as_str())
                    .collect();
                solutions
                    .iter()
                    .filter(|s| member_paths.contains(s.rel_path.as_str()))
                    .collect()
            } else {
                Vec::new()
            }
        }
    };

    if matching_docs.is_empty() {
        println!(
            "doc refresh: no solutions matched scope '{}'",
            scope.unwrap_or("(auto-detected top cluster)")
        );
        return Ok(());
    }

    let resolved_scope = scope.unwrap_or_else(|| {
        matching_docs
            .first()
            .and_then(|s| s.tags.first())
            .map(|s| s.as_str())
            .unwrap_or("architecture")
    });

    println!(
        "== [Solution Refresh & Consolidation Plan (Scope: '{}', Matches: {})] ==",
        resolved_scope,
        matching_docs.len()
    );

    if dry_run {
        println!("mode: dry-run (preview only, no external commands triggered)");
    }

    println!("\nCandidate Documents for Review:");
    for doc in &matching_docs {
        println!("  • {} ({}) — {}", doc.rel_path, doc.date, doc.title);
    }

    println!("\nSuggested Agent Skill Action:");
    println!("  /ce-compound-refresh {resolved_scope}");
    println!("\nThis invokes the compound-engineering refresh workflow to assess Keep/Update/Consolidate/Replace/Delete.");

    Ok(())
}

fn run_doc_lint(repo_root: &Path, strict: bool, json: bool) -> Result<(), CeError> {
    let config = crate::state::state::DocHygieneConfig {
        check_solution_paths: true,
        require_solution_frontmatter: true,
        ..Default::default()
    };

    let status = crate::commands::workflow::probe_solution_drift(repo_root, &config);

    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        if strict && status.is_debt() {
            return Err(CeError::Verification(
                "solution library drift findings detected in strict mode".to_string(),
            ));
        }
        return Ok(());
    }

    match status {
        crate::commands::workflow::ProbeStatus::Clean => {
            println!("doc lint: all solutions have valid frontmatter and resolvable source paths");
            Ok(())
        }
        crate::commands::workflow::ProbeStatus::Debt(findings) => {
            eprintln!(
                "doc lint: {} finding(s) detected in docs/solutions/:",
                findings.len()
            );
            for f in &findings {
                for dead_path in &f.dead_paths {
                    eprintln!(
                        "  [dead-path] {}: references non-existent path '{dead_path}'",
                        f.solution_path
                    );
                }
                if !f.missing_frontmatter_fields.is_empty() {
                    eprintln!(
                        "  [missing-frontmatter] {}: missing required field(s): {}",
                        f.solution_path,
                        f.missing_frontmatter_fields.join(", ")
                    );
                }
            }

            if strict {
                Err(CeError::Verification(
                    "solution library drift findings detected in strict mode".to_string(),
                ))
            } else {
                Ok(())
            }
        }
        crate::commands::workflow::ProbeStatus::Unknown => {
            println!("doc lint: solutions check indeterminate");
            Ok(())
        }
    }
}

fn run_doc_stats(repo_root: &Path, json: bool) -> Result<(), CeError> {
    let solutions = collect_solutions_inventory(repo_root);
    let mut category_distribution: BTreeMap<String, usize> = BTreeMap::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();

    for s in &solutions {
        *category_distribution.entry(s.category.clone()).or_insert(0) += 1;
        for t in &s.tags {
            *tag_counts.entry(t.clone()).or_insert(0) += 1;
        }
    }

    let mut top_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_tags.truncate(10);

    let clusters = cluster_solutions(&solutions, 0.40, 3);
    let report = DocStatsReport {
        total_solutions: solutions.len(),
        category_distribution,
        top_tags,
        clusters_count: clusters.len(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("== [Solution Library Inventory & Statistics] ==");
    println!("Total Solutions: {}", report.total_solutions);
    println!("Dense Clusters:  {}", report.clusters_count);
    println!("\nCategories:");
    for (cat, count) in &report.category_distribution {
        println!("  • {:<20} {:>3} file(s)", cat, count);
    }
    println!("\nTop 10 Tags:");
    for (tag, count) in &report.top_tags {
        println!("  • {:<20} {:>3} occurrences", tag, count);
    }

    Ok(())
}

/// Non-blocking probe for `ce-ai doctor` reporting dense solution clusters.
pub fn probe_solution_clusters(repo_root: &Path) -> Vec<String> {
    let solutions = collect_solutions_inventory(repo_root);
    if solutions.len() < 3 {
        return Vec::new();
    }

    let clusters = cluster_solutions(&solutions, 0.40, 3);
    if clusters.is_empty() {
        return Vec::new();
    }

    let top_name = &clusters[0].name;
    vec![format!(
        "solution library: {} dense topic cluster(s) detected (e.g. '{}') — run 'ce-ai doc cluster' for consolidation suggestions",
        clusters.len(),
        top_name
    )]
}

#[cfg(test)]
#[path = "tests/doc.rs"]
mod tests;
