/// Upstream Compound Engineering integration contracts, documented mode tokens,
/// and canonical skill capabilities.
pub const MODE_RETURN_TO_CALLER: &str = "mode:return-to-caller";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeSkillContract {
    pub name: &'static str,
    pub return_to_caller_supported: bool,
    pub description: &'static str,
}

pub const KNOWN_SKILLS: &[CeSkillContract] = &[
    CeSkillContract {
        name: "ce-brainstorm",
        return_to_caller_supported: false,
        description: "Explore vague or ambitious ideas into right-sized requirements",
    },
    CeSkillContract {
        name: "ce-plan",
        return_to_caller_supported: false,
        description: "Create structured execution plans for multi-step work",
    },
    CeSkillContract {
        name: "ce-work",
        return_to_caller_supported: true,
        description: "Execute a plan or concrete work prompt end-to-end",
    },
    CeSkillContract {
        name: "ce-resolve-pr-feedback",
        return_to_caller_supported: true,
        description: "Resolve PR review feedback comments and threads",
    },
    CeSkillContract {
        name: "ce-simplify-code",
        return_to_caller_supported: false,
        description: "Simplify recently changed code for clarity and quality",
    },
    CeSkillContract {
        name: "ce-compound",
        return_to_caller_supported: false,
        description: "Document a recently solved problem or durable vocabulary",
    },
    CeSkillContract {
        name: "ce-compound-refresh",
        return_to_caller_supported: false,
        description: "Audit docs/solutions learnings against codebase drift",
    },
    CeSkillContract {
        name: "ce-commit-push-pr",
        return_to_caller_supported: false,
        description: "Commit, push, and open or describe a Pull Request",
    },
    CeSkillContract {
        name: "ce-debug",
        return_to_caller_supported: false,
        description: "Diagnosis and fix loop for bugs and failing behavior",
    },
];

/// Returns the static skill contract for a given canonical skill name, if known.
pub fn get_skill_contract(skill_name: &str) -> Option<&'static CeSkillContract> {
    KNOWN_SKILLS.iter().find(|s| s.name == skill_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_skill_contracts() {
        let work = get_skill_contract("ce-work").expect("ce-work should be registered");
        assert!(work.return_to_caller_supported);

        let resolve_pr = get_skill_contract("ce-resolve-pr-feedback")
            .expect("ce-resolve-pr-feedback should be registered");
        assert!(resolve_pr.return_to_caller_supported);

        let brainstorm =
            get_skill_contract("ce-brainstorm").expect("ce-brainstorm should be registered");
        assert!(!brainstorm.return_to_caller_supported);

        assert!(get_skill_contract("unknown-skill").is_none());
    }
}
