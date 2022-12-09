#! /bin/bash
export FILENAME=$(cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]")


BODY=$(cat <<-END
{
  "configurations": {
    "Rust - Test": {
      "adapter": "CodeLLDB",
      "configuration": {
        "request": "launch",
        "program": "${FILENAME}"
      }
    }
  }
}
	END
	)

echo $BODY > .vimspector.json
