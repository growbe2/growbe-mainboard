#! /bin/bash

# https://medium.com/swlh/compiling-rust-for-raspberry-pi-arm-922b55dbb050
# https://stackoverflow.com/questions/68888706/remote-debug-of-rust-program-in-visual-studio-code

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

VSCODE_WS="$1"
SSH_REMOTE="$2"
GDBPORT="$3"

APP="growbe-mainboard"
#TARGET_ARCH="x86_64-unknown-linux-gnu"
TARGET_ARCH="armv7-unknown-linux-gnueabihf"
BUILD_BIN_FILE="${VSCODE_WS}/target/${TARGET_ARCH}/debug/${APP}"
TARGET_USER="wq"
TARGET_BIN_FILE="/home/wq/${APP}"
TARGET_CWD="/home/wq"
RUNNER="docker run --rm  -v $PWD:/usr/src/app -w /usr/src/app docker.pkg.github.com/growbe2/growbe-mainboard/dev"

#ssh "${TARGET_USER}@${SSH_REMOTE}" "killall gdbserver ${APP}"

$RUNNER cargo build --target=${TARGET_ARCH}

if ! rsync -avz "${BUILD_BIN_FILE}" "${TARGET_USER}@${SSH_REMOTE}:${TARGET_BIN_FILE}"; then
    # If rsync doesn't work, it may not be available on target. Fallback to trying SSH copy.
    if ! scp "${BUILD_BIN_FILE}" "${TARGET_USER}@${SSH_REMOTE}:${TARGET_BIN_FILE}"; then
        exit 2
    fi
fi

ssh -f "${TARGET_USER}@${SSH_REMOTE}" "sh -c 'cd ${TARGET_CWD}; nohup gdbserver *:${GDBPORT} ${TARGET_BIN_FILE} > /dev/null 2>&1 &'"
