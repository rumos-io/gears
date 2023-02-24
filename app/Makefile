run-debug:
	RUST_LOG=DEBUG cargo run -- --verbose

run:
	cargo run

test:
	cargo test

install:
	cargo install --path .

# This option only deletes the data and not the gensis file. This is useful since
# the timeout_commit = "10s" param has been manually changed in my genesis file.
tendermint-clean-start: tendermint-reset-unsafe tendermint-start

# This deletes both the data and gensis file first.
tendermint-hard-reset-start: tendermint-reset-init tendermint-start

tendermint-reset-unsafe:
	tendermint unsafe-reset-all

tendermint-reset-init:
	./init-tendermint.sh
	
tendermint-start:
	tendermint start

.PHONY: run run-debug test tendermint-clean-start tendermint-reset-unsafe tendermint-init tendermint-start tendermint-hard-reset