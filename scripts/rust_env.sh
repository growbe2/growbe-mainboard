#! /bin/bash

docker run --rm --env COMMIT="$COMMIT" -v "$PWD":/usr/src/app -w /usr/src/app docker.pkg.github.com/growbe2/growbe-mainboard/dev "$@"
