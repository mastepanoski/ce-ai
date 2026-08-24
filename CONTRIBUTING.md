# Contributing to `ce-ai`

First off, thank you for considering contributing to `ce-ai`! It is through contributions like yours that `ce-ai` remains a secure, high-quality, and robust plugin manager for AI coding harnesses.

---

## 🛡️ Governance & Security Compliance Standards

All contributions to `ce-ai` must comply with our foundational security and AI management standards:

- **ISO/IEC 27001 & 27002**: Strict Information Security Management, access controls, supply chain security, and cryptographic integrity.
- **NIST Cybersecurity Framework (CSF) & SP 800-53**: Secure software development lifecycle, continuous vulnerability monitoring, and atomic system restoration.
- **ISO/IEC 42001**: Artificial Intelligence Management System (AIMS) governance for AI tool delivery and agent model management.
- **NIST AI Risk Management Framework (AI RMF 1.0)**: Systematic Risk Mapping, Measurement, Governance, and Management for AI integration plugins.

Refer to [`SECURITY.md`](./SECURITY.md) and [`AI_POLICY.md`](./AI_POLICY.md) for full compliance directives.

---

## 🛠️ Development Setup & Workflow

### Prerequisites
- **Rust toolchain** (latest stable release): `rustup update stable`
- **Docker Engine** (required for containerized E2E gate testing)
- **Node.js 20+** & `npm`

### Local Setup
```bash
# Clone the repository
git clone git@github.com:mastepanoski/ce-ai.git
cd ce-ai

# Build the debug binary
cargo build

# Run unit and integration test suite
cargo test

# Run formatting and clippy linter checks
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Docker E2E Gate Execution
Before submitting a pull request, you MUST verify that the containerized end-to-end gate passes cleanly:
```bash
make e2e
```
This builds an isolated Linux container (`Dockerfile.e2e`) and validates `ce-ai install`, `sync`, `models`, `status`, and `uninstall` flows against OpenCode.

---

## 📜 Pull Request Guidelines

1. **PR Template**: Open all pull requests using the repository [PR template](./.github/PULL_REQUEST_TEMPLATE.md); every verification gate and checklist item must be satisfied before review.
2. **Branch Naming**: Use descriptive branch names: `feature/description`, `fix/issue-name`, or `docs/update`.
3. **Conventional Commits**: Commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
   - `feat(models): add support for custom profile export`
   - `fix(sync): resolve atomic rename lock race`
   - `docs(security): update ISO 27001 risk matrix`
4. **100% Test Coverage**: New features or bug fixes must include unit or CLI integration tests.
5. **No Breaking Changes**: Preserve existing CLI contracts and JSON schema formats unless explicitly documented and reviewed.
6. **Security Audit**: Code must pass `cargo audit` without open vulnerability alerts.

---

## 💬 Community Code of Conduct

Please note that this project is released with a Contributor Code of Conduct. By participating in this project you agree to abide by its terms. See [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
