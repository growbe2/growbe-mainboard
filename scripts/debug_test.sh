#! /bin/bash

export FILENAME=$(cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]")

tmp=$(mktemp)

jq --arg a "$FILENAME" '.configurations.test.configuration.program = $a' .vimspector.json > "$tmp" && mv "$tmp" .vimspector.json

#echo $BODY | jq > .vimspector.json
