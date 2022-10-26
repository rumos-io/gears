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

set:
	curl 'http://127.0.0.1:26657/broadcast_tx_async?tx="somekey=somevalue"'

get:
	curl 'http://127.0.0.1:26657/abci_query?data=0x736f6d656b6579'

.PHONY: run run-debug test tendermint-clean-start tendermint-reset-unsafe tendermint-init tendermint-start set get