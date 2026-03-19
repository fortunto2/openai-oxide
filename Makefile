.PHONY: test clippy fmt check live doc clean

test:
	cargo test

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt -- --check

check: fmt clippy test

live:
	cargo test --features "live-tests"

doc:
	cargo doc --no-deps --open

clean:
	cargo clean

help:
	@echo "test    — run all tests"
	@echo "clippy  — lint (warnings = errors)"
	@echo "fmt     — check formatting"
	@echo "check   — fmt + clippy + test"
	@echo "live    — tests with real API (needs OPENAI_API_KEY)"
	@echo "doc     — generate and open docs"
	@echo "clean   — remove build artifacts"
