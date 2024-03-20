#!/usr/bin/env bash

cd ../../

docker build -t gaia-test  -f ./gaia-rs/tests/Dockerfile .

docker container rm gaia

docker run --name gaia -it gaia-test /gears/gaia-rs/tests/inner.sh