.PHONY: fmt check test build clippy ci

fmt:
	cargo fmt --all

check:
	cargo check --workspace --locked

test:
	./scripts/test.sh

build:
	./scripts/build.sh

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

ci:
	cargo fmt --all -- --check
	cargo check --workspace --locked
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo test --workspace --locked
	./scripts/build.sh
