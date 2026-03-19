.PHONY: test clippy fmt check

test:
	cargo test

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt -- --check

check: test clippy fmt

help:
	@grep -E '^##' Makefile | sed 's/## //'
