#!/usr/bin/env bash

set -xe

RUSTC_VERSION=1.57.0;
PACKAGE="sherpax-runtime";
BUILD_OPTS=$BUILD_OPTS;

docker run --rm -it -e PACKAGE=$PACKAGE -e BUILD_OPTS="$BUILD_OPTS" -v $PWD:/build -v $TMPDIR/cargo:/cargo-home -v ~/docker-cargo:/root/.cargo paritytech/srtool:$RUSTC_VERSION $*

