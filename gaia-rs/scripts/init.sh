#!/usr/bin/env bash

set -eux

rm -rf ~/.gaia-rs
cargo run -- init test

echo "Generating deterministic account - alice"
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | cargo run -- keys add alice --recover --keyring-backend=test

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 1000000000000uatom

cargo run -- gentx 10000000000uatom --from-key alice --min-self-delegation 1 --moniker test --account-number 0 --sequence 0 --keyring-backend=test

echo "Collecting genesis txs..."
cargo run -- collect-gentxs
