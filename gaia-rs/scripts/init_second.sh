#!/usr/bin/env bash

set -eux

GAIA_HOME=~/.gaia-rs-second

rm -rf $GAIA_HOME
cargo run -- init test_second --home $GAIA_HOME --chain-id test-chain-1

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 34uatom --home $GAIA_HOME
