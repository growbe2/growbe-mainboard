#! /bin/bash

mkdir -p ./target/coverage/html
grcov . --binary-path ./target/cov/debug/deps -s . -t html --branch --ignore-not-existing --ignore '*vendor*' -o target/coverage/html
grcov . --binary-path ./target/cov/debug/deps -s . -t cobertura --branch --ignore-not-existing --ignore '*vendor*' -o target/coverage/cobertura.xml
