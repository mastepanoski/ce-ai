.PHONY: e2e test build clean

test:
	cargo test

build:
	cargo build --release

e2e:
	cargo test --test e2e -- --nocapture

clean:
	cargo clean
