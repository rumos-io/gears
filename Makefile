run-debug:
	RUST_LOG=DEBUG cargo run -- run --min-gas-prices 0uatom

run-backtrace:
	RUST_BACKTRACE=1 cargo run -- run --min-gas-prices 0uatom

run:
	cargo run -- run --min-gas-prices 0uatom

test:
	RUST_BACKTRACE=1 cargo test --features=macros_test,it  --no-fail-fast 

install:
# "cargo install --path" ignores the lockfile, so we need to use "--locked" to ensure we use the same versions as in the lockfile
# see https://github.com/rust-lang/cargo/issues/6983
	cargo install --path ./gaia-rs --locked --no-default-features --features=rocksdb

install-sled:
	cargo install --path ./gaia-rs --locked

init:
	AMOUNT=$(AMOUNT) ./gaia-rs/scripts/init.sh

tendermint-start:
	tendermint start --home ~/.gaia-rs

init-second:
	./gaia-rs/scripts/init_second.sh

tendermint-start-second:
	tendermint start --home ~/.gaia-rs-second  --p2p.laddr tcp://0.0.0.0:26659 --rpc.laddr tcp://127.0.0.1:26660 --proxy_app tcp://127.0.0.1:26661

run-second:
	cargo run -- run --home ~/.gaia-rs-second --address "127.0.0.1:26661" --rest-listen-addr "127.0.0.1:1318" --min-gas-prices 0uatom

.PHONY: run run-debug test install install-sled init tendermint-start init-second tendermint-start-second run-second
