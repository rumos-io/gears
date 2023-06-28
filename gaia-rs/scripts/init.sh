#!/usr/bin/env bash

set -eux

rm -rf ~/.gaia-rs
cargo run -- init test
