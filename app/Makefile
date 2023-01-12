run-debug:
	RUST_LOG=DEBUG cargo run -- --verbose

run:
	cargo run

test:
	cargo test

tendermint-clean-start: tendermint-reset-unsafe tendermint-init tendermint-start

tendermint-reset-unsafe:
	tendermint unsafe-reset-all

tendermint-init:
	tendermint init validator
	
tendermint-start:
	tendermint start

.PHONY: run run-debug test tendermint-clean-start tendermint-reset-unsafe tendermint-init tendermint-start set get