#!/bin/bash

set -ev

cd "$(git rev-parse --show-toplevel)"

ARTIFACT_NAME="dgen-${TRAVIS_TAG}-${TRAVIS_OS_NAME}.exe"

cargo build --release
mkdir -p target/artifacts/
cp target/release/dgen.exe target/artifacts/${ARTIFACT_NAME}

