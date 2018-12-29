#!/bin/bash

set -ev

cd "$(git rev-parse --show-toplevel)"

ARTIFACT_NAME="dgen-${TRAVIS_TAG:-dev}-${TRAVIS_OS_NAME:-$OSTYPE}"

COMMAND="zip"
EXTENSION=""
if [[ "$TRAVIS_OS_NAME" == "windows" ]]; then
    COMMAND="7z"
    EXTENSION=".exe"
fi

cargo build --release

# remove any old artifacts if they are remaining
rm -rf target/artifacts/*
mkdir -p target/artifacts/${ARTIFACT_NAME}

# for some reason, mv is giving an "are the same file" error so we'll just use cp for now
cp "target/release/dgen${EXTENSION}" "target/artifacts/${ARTIFACT_NAME}/"

eval ${COMMAND} -r target/artifacts/${ARTIFACT_NAME}.zip target/artifacts/${ARTIFACT_NAME}/
