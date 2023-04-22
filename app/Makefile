run-debug:
	RUST_LOG=DEBUG cargo run -- run --verbose

run:
	cargo run -- run

test:
	cargo test

install:
	cargo install --path .

init:
	./scripts/init.sh

tendermint-start:
	tendermint start --home ~/.gears

.PHONY: run run-debug test install init tendermint-start