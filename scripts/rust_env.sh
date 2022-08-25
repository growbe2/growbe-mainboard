#! /bin/bash

docker run --rm --env COMMIT="$COMMIT" --env PKG_CONFIG_SYSROOT_DIR=/usr/arm-linux-gnueabihf/ -v "$PWD":/usr/src/app -w /usr/src/app docker.pkg.github.com/growbe2/growbe-mainboard/dev "$@"
