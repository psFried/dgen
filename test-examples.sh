#!/bin/bash

set -ev

echo "Building dgen in debug mode"
cargo build

echo "Running HTTP example"
target/debug/dgen -vv --lib dgen_examples/http/http-lib.dgen -f dgen_examples/http/use-http.dgen

echo "Running json example"
target/debug/dgen -vv dgen_examples/json/json.dgen


echo "Running http with json example"
target/debug/dgen -vv -l dgen_examples/http/http-lib.dgen -l dgen_examples/json/json.dgen -f dgen_examples/http_with_json.dgen
