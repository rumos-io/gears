#!/usr/bin/env bash

amount=34
if [[ $AMOUNT != "" ]]; then
  amount=$AMOUNT
fi

set -eux

rm -rf ~/.gaia-rs
cargo run -- init test

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux "$amount"uatom
