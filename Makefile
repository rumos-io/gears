run-debug:
	RUST_LOG=DEBUG cargo run -- run --verbose

run:
	cargo run -- run

test:
	cargo test

install:
# "cargo install --path" ignores the lockfile, so we need to use "--locked" to ensure we use the same versions as in the lockfile
# see https://github.com/rust-lang/cargo/issues/6983
	cargo install --path ./gaia-rs --locked

init:
	./gaia-rs/scripts/init.sh

tendermint-start:
	tendermint start --home ~/.gaia-rs

.PHONY: run run-debug test install init tendermint-start