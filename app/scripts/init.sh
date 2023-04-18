#!/usr/bin/env bash

set -eux

rm -rf ~/.gears
cargo run -- init test
