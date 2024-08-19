#!/usr/bin/env bash

set -eux

rm -rf ~/.gaia-rs
cargo run -- init test

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 1000000000uatom
