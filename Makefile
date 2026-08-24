.PHONY: e2e test bench build clean hooks lint security

test:
	cargo test

build:
	cargo build --release

# Fail-closed containerized gate (#165): Docker absence is a hard failure.
e2e:
	cargo test --test e2e -- --ignored --nocapture

# Local supply-chain gate mirroring the CI Supply Chain Security Audit job
# (#165): installs cargo-audit if missing, then fails on any vulnerability.
security:
	@command -v cargo-audit >/dev/null 2>&1 || { echo "installing cargo-audit (one-time)..."; cargo install --locked cargo-audit; }
	cargo audit

bench:
	cargo test --benches --release

lint:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings

hooks:
	chmod +x .githooks/pre-commit
	git config core.hooksPath .githooks

clean:
	cargo clean
