//! CE plugin loader placement (OI-3) and skills-path registration (OI-4).
//!
//! RED: tests reference `install_loader` / `skills_path`, which do not exist
//! yet — this file fails to compile until the GREEN implementation.

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::state::diff::sha256_hex;
    use super::*;

    #[test]
    fn copies_loader_into_managed_plugins_dir() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("ce-source");
        let loader_src = source.join(".opencode/plugins/compound-engineering.js");
        std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
        let loader_bytes = b"export default function ceLoader() {}";
        std::fs::write(&loader_src, loader_bytes).unwrap();

        let config_dir = dir.path().join("opencode-config");
        let installed = install_loader(&source, &config_dir).unwrap();

        assert_eq!(installed.path, "plugins/compound-engineering.js");
        assert_eq!(installed.sha256, sha256_hex(loader_bytes));
        let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
        assert_eq!(std::fs::read(&dest).unwrap(), loader_bytes);
    }

    #[test]
    fn skills_path_points_at_managed_skills_dir() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("opencode-config");
        assert_eq!(skills_path(&config_dir), config_dir.join("compound-engineering/skills"));
    }
}