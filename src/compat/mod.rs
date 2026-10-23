//! Typed Compatibility Layer with upstream Compound Engineering contracts,
//! schemas, and paths.

pub mod arch;
pub mod contracts;
pub mod docs;
pub mod release;
pub mod schema;

pub use arch::{
    adrs_dir, arch_dir, archive_dir, contracts_dir, deltas_dir, detect_system_specs_source,
    load_system_manifest, parse_system_manifest, resolve_dependency_status, system_manifest_path,
    system_specs_dir, validate_remote_repo_url, ArchSource, DependencyResolutionStatus,
    SystemDependency, SystemManifest,
};
pub use contracts::{get_skill_contract, CeSkillContract, KNOWN_SKILLS, MODE_RETURN_TO_CALLER};
pub use docs::CeDocsConfig;
pub use release::CeRelease;
pub use schema::{check_solution_frontmatter, is_bug_problem_type, CeSolutionFrontmatter};
