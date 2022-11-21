#! /bin/bash

# https://medium.com/swlh/compiling-rust-for-raspberry-pi-arm-922b55dbb050
# https://stackoverflow.com/questions/68888706/remote-debug-of-rust-program-in-visual-studio-code

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

VSCODE_WS=.
SSH_REMOTE="$1"
GDBPORT="17777"
COMMAND="${2:=run}"

APP="growbe-mainboard"
TARGET_ARCH="armv7-unknown-linux-gnueabihf"
TARGET_CC="arm-linux-gnueabihf-gcc"
BUILD_BIN_FILE="${VSCODE_WS}/target/${TARGET_ARCH}/release/${APP}"
#BUILD_BIN_FILE="./drivers/mainboard_driver"
TARGET_USER="${TARGET_USER:=pi}"
TARGET_BIN_FILE="/opt/growbe/${APP}"
#TARGET_BIN_FILE="./mainboard_driver"
TARGET_CWD="/opt/growbe/"
RUNNER="./scripts/rust_env.sh"

COMMAND_RUN="killall growbe-mainboard; sudo systemctl stop growbe-mainboard@dev.service; sh -c 'cd ${TARGET_CWD};${TARGET_BIN_FILE} ./dev.json'"
COMMAND_RESTART="sudo systemctl restart growbe-mainboard@dev.service"

$RUNNER make -C ./drivers CC=${TARGET_CC}
COMMIT=$(git rev-parse --short HEAD) $RUNNER cargo build --target=${TARGET_ARCH} --features pi --release

if ! rsync -avz "${BUILD_BIN_FILE}" "./mainboard_config.json" "${TARGET_USER}@${SSH_REMOTE}:${TARGET_CWD}"; then
    # If rsync doesn't work, it may not be available on target. Fallback to trying SSH copy.
    if ! scp "${BUILD_BIN_FILE}" "${TARGET_USER}@${SSH_REMOTE}:${TARGET_BIN_FILE}"; then
        exit 2
    fi
fi

if [[ $COMMAND == "run" ]]; then
    ssh "${TARGET_USER}@${SSH_REMOTE}" "$COMMAND_RUN"
else
    ssh "${TARGET_USER}@${SSH_REMOTE}" "$COMMAND_RESTART"
fi
