#!/usr/bin/env bash

amount=1000000000000
if [[ $AMOUNT != "" ]]; then
  amount=$AMOUNT
fi

set -eux

rm -rf ~/.gaia-rs
cargo run -- init test

echo "Generating deterministic account - alice"
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | gaia-rs keys add alice --recover --keyring-backend=test

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux "$amount"uatom

echo "Collecting genesis txs..."
cargo run -- gentx 10uatom --from-key alice --account-number 2 --sequence 0 --keyring-backend=test

cargo run -- collect-gentxs
