//! Typed Compatibility Layer with upstream Compound Engineering contracts,
//! schemas, and paths.

pub mod contracts;
pub mod docs;
pub mod release;
pub mod schema;

pub use contracts::{get_skill_contract, CeSkillContract, KNOWN_SKILLS, MODE_RETURN_TO_CALLER};
pub use docs::CeDocsConfig;
pub use release::CeRelease;
pub use schema::{check_solution_frontmatter, CeSolutionFrontmatter};
