.PHONY: e2e test bench build clean hooks lint

test:
	cargo test

build:
	cargo build --release

e2e:
	cargo test --test e2e -- --ignored --nocapture

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
