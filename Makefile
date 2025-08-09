SHELL=/bin/bash

.PHONY: all api indexer verifier fmt lint run-dev

all: build

build:
	cargo build --workspace

api:
	cargo run -p api

indexer:
	cargo run -p indexer

verifier:
	cargo run -p verifier

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

run-dev: build
	RUST_LOG=info OT_LISTEN_ADDR=127.0.0.1:8080 cargo run -p api
