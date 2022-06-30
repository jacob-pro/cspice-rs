.PHONY: test
test:
	cargo fmt -- --check
	cargo-sort --check --workspace
	cargo clippy --all-features --workspace -- -D warnings
	cargo test --all-features --workspace -- --test-threads=1

.PHONY: format
format:
	cargo fmt
	cargo-sort --workspace
