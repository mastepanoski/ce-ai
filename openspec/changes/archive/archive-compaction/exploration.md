# Exploration: Generational Archive Compaction Architecture & Strategy

## 1. Technical Context & Investigation

### Current Archive Scale
Inspection of `openspec/changes/archive/` reveals:
- **109 package folders** containing over 540 markdown files.
- Two distinct naming styles:
  1. **Date-prefixed:** e.g. `2026-08-26-canonical-skills-adoption`, `2026-09-11-openspec-archive-command`.
  2. **Legacy slug-only:** e.g. `adoption-block-ssot-v2`, `blocking-gate-check`, `backup_restore_management`.
- In addition, `openspec/changes/archive/README.md` acts as the project's living archive ledger.

### Impact of Archive Sprawl
When an AI agent or developer runs codebase searches (`grep_search`, `find_by_name`, `ripgrep`), tool results are overwhelmed by historical files in `archive/*/tasks.md` and `archive/*/exploration.md`. This consumes precious LLM context window tokens and introduces false positives during codebase refactoring.

## 2. Technical Options Evaluated

### Option A: External Archive Branch or Submodule
- **Approach:** Move all historical archives to an orphaned git branch `archive` or git submodule.
- **Tradeoffs:**
  - *Pros:* Completely empties the working tree.
  - *Cons:* Extremely high friction. Breaks offline access, breaks git blame and direct links from PRs, requires extra git tooling, and complicates CI checks.
- **Verdict:** **Rejected**.

### Option B: Raw Deletion of Scaffolding Files (`exploration.md`, `tasks.md`)
- **Approach:** In-place delete `exploration.md` and `tasks.md`, retaining only `spec.md` and `proposal.md` loose.
- **Tradeoffs:**
  - *Pros:* Simple to implement.
  - *Cons:* Destroys original task checklists and exploration records (violates NIST/ISO audit preservation). Still leaves 109 directories in `openspec/changes/archive/`.
- **Verdict:** **Rejected**.

### Option C: Generational Milestone Rollup with Scaffolding Tarball (Recommended)
- **Approach:**
  1. Group older archives into milestone windows (e.g. quarterly `2026-Q1`, `2026-Q2`, `2026-Q3` or named `--milestone`).
  2. Synthesize a structured rollup document: `openspec/changes/archive/milestones/<milestone>.md` containing executive summaries, acceptance criteria, and task completion metrics.
  3. Compress all raw directories into a single gzipped tarball: `openspec/changes/archive/milestones/archive-<milestone>.tar.gz`.
  4. Atomically remove the loose directories from `openspec/changes/archive/` once the tarball and rollup are verified.
  5. Update `openspec/changes/archive/README.md` with milestone links.
- **Tradeoffs:**
  - *Pros:* 0 data loss. Reduces 100+ directories to 1-2 clean files. Restores instant grep/file-search cleanliness. Fully auditable and reproducible.
  - *Cons:* Requires safe tarball extraction and careful path handling.
- **Verdict:** **Selected**.

## 3. Date Resolution Architecture

Compaction requires assigning a calendar date to each archive package to determine age and quarter. We implement a multi-tier fallback:
1. **Directory Name Parsing (Fast Tier):** If the directory name begins with `YYYY-MM-DD-` (10 chars + hyphen), parse as `chrono::NaiveDate`. This handles the majority of packages in 0 milliseconds.
2. **Git Commit History (Reliable Tier):** For legacy slug-only directories, execute `git log -1 --format=%cs -- <dir_path>` to retrieve the exact date when the package was archived/committed.
3. **Filesystem Metadata (No-Git Tier):** In environments lacking `.git`, inspect directory or `tasks.md` `mtime` as a fallback.

## 4. Compaction Algorithm & Rollup Structure

```
openspec/changes/archive/
├── milestones/
│   ├── 2026-Q3.md                 # Consolidated feature catalog & specs
│   └── archive-2026-Q3.tar.gz    # Compressed raw packages (zero data loss)
├── 2026-09-12-doc-debt-engine/   # Recent active archives remain loose (< 30d)
└── README.md                     # Living ledger updated with milestone links
```

### Milestone Rollup Document Format (`milestones/YYYY-QX.md`)
- **Header:** `# Milestone Archive Rollup: YYYY-QX`
- **Metadata:** Compaction timestamp, candidate count, raw archive tarball link.
- **Summary Matrix:** Markdown table listing feature, date, completion ratio, and primary domain.
- **Feature Sections:** For each feature:
  - `## <feature-slug>`
  - Proposal Problem Statement summary.
  - Core Acceptance Criteria (extracted from `spec.md`).
  - Tasks summary (completed/total).

## 5. Safety & Atomic Guarantees
- **Verification Before Deletion:** Loose directories are NEVER deleted until:
  1. `milestones/<milestone>.md` is written and non-empty.
  2. `milestones/archive-<milestone>.tar.gz` is written and verified by reading back tar entries and matching candidate count.
- **Dry Run Support:** `--dry-run` performs full candidate resolution, candidate grouping, and preview reporting without mutating the filesystem.
