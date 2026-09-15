use std::collections::HashSet;
use std::fs;
use tempfile::tempdir;

use super::*;

#[test]
fn test_tokenize_title() {
    let title = "Fixing the SQLite Race Condition and Lock Contention via Mutex!";
    let tokens = tokenize_title(title);

    assert!(tokens.contains("fixing"));
    assert!(tokens.contains("sqlite"));
    assert!(tokens.contains("race"));
    assert!(tokens.contains("condition"));
    assert!(tokens.contains("lock"));
    assert!(tokens.contains("contention"));
    assert!(tokens.contains("mutex"));

    // Stopwords and short words should be filtered
    assert!(!tokens.contains("the"));
    assert!(!tokens.contains("and"));
    assert!(!tokens.contains("via"));
    assert!(!tokens.contains("a"));
}

#[test]
fn test_jaccard_similarity() {
    let mut set_a: HashSet<String> = HashSet::new();
    let mut set_b: HashSet<String> = HashSet::new();

    // Empty sets
    assert_eq!(jaccard_similarity(&set_a, &set_b), 0.0);

    // Identical sets
    set_a.insert("tag1".into());
    set_a.insert("tag2".into());
    set_b.insert("tag1".into());
    set_b.insert("tag2".into());
    assert!((jaccard_similarity(&set_a, &set_b) - 1.0).abs() < f32::EPSILON);

    // Disjoint sets
    let mut set_c: HashSet<String> = HashSet::new();
    set_c.insert("tag3".into());
    set_c.insert("tag4".into());
    assert_eq!(jaccard_similarity(&set_a, &set_c), 0.0);

    // Partial overlap: intersection = 1 ("tag1"), union = 3 ("tag1", "tag2", "tag3") -> 1/3
    let mut set_d: HashSet<String> = HashSet::new();
    set_d.insert("tag1".into());
    set_d.insert("tag3".into());
    let sim = jaccard_similarity(&set_a, &set_d);
    assert!((sim - (1.0 / 3.0)).abs() < 1e-4);
}

#[test]
fn test_calculate_solution_similarity() {
    let doc_a = SolutionMetadata {
        file_path: PathBuf::from("docs/solutions/architecture/sqlite-locking.md"),
        rel_path: "docs/solutions/architecture/sqlite-locking.md".to_string(),
        title: "SQLite Locking Issues and Concurrency".to_string(),
        category: "architecture".to_string(),
        problem_type: "architecture".to_string(),
        tags: vec!["sqlite".into(), "database".into(), "locking".into()],
        components: vec!["db.rs".into()],
        applies_when: "concurrency errors".to_string(),
        date: "2026-09-01".to_string(),
        title_tokens: tokenize_title("SQLite Locking Issues and Concurrency"),
    };

    let doc_b = SolutionMetadata {
        file_path: PathBuf::from("docs/solutions/architecture/sqlite-wal-mode.md"),
        rel_path: "docs/solutions/architecture/sqlite-wal-mode.md".to_string(),
        title: "SQLite WAL Mode and Concurrency Locking".to_string(),
        category: "architecture".to_string(),
        problem_type: "architecture".to_string(),
        tags: vec!["sqlite".into(), "database".into(), "wal".into()],
        components: vec!["db.rs".into()],
        applies_when: "slow transactions".to_string(),
        date: "2026-09-02".to_string(),
        title_tokens: tokenize_title("SQLite WAL Mode and Concurrency Locking"),
    };

    let doc_c = SolutionMetadata {
        file_path: PathBuf::from("docs/solutions/workflow/spec-validation.md"),
        rel_path: "docs/solutions/workflow/spec-validation.md".to_string(),
        title: "OpenSpec Living Spec Validation".to_string(),
        category: "workflow".to_string(),
        problem_type: "process".to_string(),
        tags: vec!["openspec".into(), "validation".into()],
        components: vec!["spec.rs".into()],
        applies_when: "linting specs".to_string(),
        date: "2026-09-10".to_string(),
        title_tokens: tokenize_title("OpenSpec Living Spec Validation"),
    };

    let sim_ab = calculate_solution_similarity(&doc_a, &doc_b);
    let sim_ac = calculate_solution_similarity(&doc_a, &doc_c);

    // doc_a and doc_b share category, components, tags ("sqlite", "database"), and title tokens ("sqlite", "locking", "concurrency")
    assert!(sim_ab > 0.60, "sim_ab was {sim_ab}");
    // doc_a and doc_c share no category, no components, no tags, and no title tokens
    assert_eq!(sim_ac, 0.0);
}

#[test]
fn test_parse_solution_file_valid() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("solution-1.md");
    let content = r#"---
title: "Fix Deadlock in Worker Pool"
category: "concurrency"
problem_type: "bugfix"
date: "2026-09-12"
applies_when: "threads hang on channel send"
tags:
  - worker
  - deadlock
  - threading
components:
  - src/worker.rs
---

# Fix Deadlock in Worker Pool
Details here.
"#;
    fs::write(&file, content).unwrap();

    let meta = parse_solution_file(dir.path(), &file).expect("should parse valid solution");
    assert_eq!(meta.title, "Fix Deadlock in Worker Pool");
    assert_eq!(meta.category, "concurrency");
    assert_eq!(meta.problem_type, "bugfix");
    assert_eq!(meta.date, "2026-09-12");
    assert_eq!(meta.tags, vec!["worker", "deadlock", "threading"]);
    assert_eq!(meta.components, vec!["src/worker.rs"]);
    assert!(meta.title_tokens.contains("deadlock"));
    assert!(meta.title_tokens.contains("worker"));
    assert!(meta.title_tokens.contains("pool"));
}

#[test]
fn test_parse_solution_file_bracketed_arrays() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("solution-2.md");
    let content = r#"---
title: "Refactor Cache Expiration"
module: "caching"
tags: [cache, redis, ttl]
components: ["src/cache.rs", "src/store.rs"]
---

# Body
"#;
    fs::write(&file, content).unwrap();

    let meta = parse_solution_file(dir.path(), &file).expect("should parse bracketed arrays");
    assert_eq!(meta.title, "Refactor Cache Expiration");
    assert_eq!(meta.category, "caching");
    assert_eq!(meta.tags, vec!["cache", "redis", "ttl"]);
    assert_eq!(meta.components, vec!["src/cache.rs", "src/store.rs"]);
}

#[test]
fn test_parse_solution_file_missing_delimiter() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("invalid.md");
    fs::write(&file, "No frontmatter at all\n").unwrap();

    let err = parse_solution_file(dir.path(), &file).unwrap_err();
    assert!(err.contains("missing opening YAML delimiter"));
}

#[test]
fn test_parse_solution_file_unclosed_delimiter() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("unclosed.md");
    fs::write(&file, "---\ntitle: Unclosed\n").unwrap();

    let err = parse_solution_file(dir.path(), &file).unwrap_err();
    assert!(err.contains("missing closing YAML delimiter"));
}

#[test]
fn test_cluster_solutions_grouping() {
    let mut solutions = Vec::new();

    // Group 1: 3 SQLite / Database solutions
    for i in 1..=3 {
        let title = format!("SQLite Concurrency and Locking Issue Part {i}");
        solutions.push(SolutionMetadata {
            file_path: PathBuf::from(format!("docs/solutions/architecture/sqlite-{i}.md")),
            rel_path: format!("docs/solutions/architecture/sqlite-{i}.md"),
            title: title.clone(),
            category: "architecture".to_string(),
            problem_type: "architecture".to_string(),
            tags: vec!["sqlite".into(), "database".into(), "concurrency".into()],
            components: vec!["src/db.rs".into()],
            applies_when: "sqlite errors".into(),
            date: format!("2026-09-0{i}"),
            title_tokens: tokenize_title(&title),
        });
    }

    // Group 2: 2 OpenSpec solutions (should NOT form cluster if min_size = 3)
    for i in 1..=2 {
        let title = format!("OpenSpec Living Specifications Validation Step {i}");
        solutions.push(SolutionMetadata {
            file_path: PathBuf::from(format!("docs/solutions/workflow/spec-{i}.md")),
            rel_path: format!("docs/solutions/workflow/spec-{i}.md"),
            title: title.clone(),
            category: "workflow".to_string(),
            problem_type: "process".to_string(),
            tags: vec!["openspec".into(), "specification".into()],
            components: vec!["src/spec.rs".into()],
            applies_when: "spec checks".into(),
            date: format!("2026-09-1{i}"),
            title_tokens: tokenize_title(&title),
        });
    }

    // Cluster with min_size = 3, threshold = 0.40
    let clusters = cluster_solutions(&solutions, 0.40, 3);
    assert_eq!(clusters.len(), 1, "Expected 1 cluster with min_size 3");
    let c = &clusters[0];
    assert_eq!(c.members.len(), 3);
    assert!(c.dominant_tags.contains(&"sqlite".to_string()));
    assert_eq!(c.suggested_refresh_scope, "concurrency");

    // Cluster with min_size = 2, threshold = 0.40 -> should find 2 clusters
    let clusters_2 = cluster_solutions(&solutions, 0.40, 2);
    assert_eq!(clusters_2.len(), 2, "Expected 2 clusters with min_size 2");
}

#[test]
fn test_probe_solution_clusters() {
    let dir = tempdir().unwrap();
    let sol_dir = dir
        .path()
        .join("docs")
        .join("solutions")
        .join("architecture");
    fs::create_dir_all(&sol_dir).unwrap();

    for i in 1..=3 {
        let content = format!(
            r#"---
title: "Database Mutex Lock Part {i}"
category: "architecture"
tags: [database, mutex, locking]
components: [db.rs]
date: "2026-09-0{i}"
---

# Solution {i}
"#
        );
        fs::write(sol_dir.join(format!("db-{i}.md")), content).unwrap();
    }

    let warnings = probe_solution_clusters(dir.path());
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("1 dense topic cluster(s) detected"));
    assert!(warnings[0].contains("ce-ai doc cluster"));
}
