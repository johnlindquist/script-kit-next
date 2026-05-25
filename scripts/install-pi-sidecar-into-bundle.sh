#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="${1:-${REPO_ROOT}/target/release/bundle/osx/Script Kit.app}"
SOURCE="${SCRIPT_KIT_PI_SIDECAR_SOURCE:-${REPO_ROOT}/target/pi-sidecar/pi}"
DEST="${APP_PATH}/Contents/MacOS/pi"

echo "pi_sidecar install app=${APP_PATH}"

test -x "${SOURCE}"
test -d "${APP_PATH}/Contents/MacOS"

mkdir -p "$(dirname "${DEST}")"
cp "${SOURCE}" "${DEST}"
chmod 0755 "${DEST}"

echo "pi_sidecar installed dest=${DEST}"
