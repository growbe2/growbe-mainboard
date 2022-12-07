#! /bin/bash

mkdir -p ./target/coverage/html
grcov . --binary-path ./target/cov/debug/deps -s . -t html --branch --ignore-not-existing --ignore '*vendor*' --ignore '*proto*' -o target/coverage/html
grcov . --binary-path ./target/cov/debug/deps -s . -t cobertura --branch --ignore-not-existing --ignore '*vendor*' --ignore '*proto*' -o target/coverage/cobertura.xml
