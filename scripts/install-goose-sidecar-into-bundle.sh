#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="${1:-${REPO_ROOT}/target/release/bundle/osx/Script Kit.app}"
SOURCE="${SCRIPT_KIT_GOOSE_SIDECAR_SOURCE:-${REPO_ROOT}/target/goose-sidecar/goose}"
DEST="${APP_PATH}/Contents/MacOS/goose"

echo "goose_sidecar install app=${APP_PATH}"

test -x "${SOURCE}"
test -d "${APP_PATH}/Contents/MacOS"

mkdir -p "$(dirname "${DEST}")"
cp "${SOURCE}" "${DEST}"
# Release bundles ship the stripped engine; debug symbols stay in the build cache.
strip -x "${DEST}"
chmod 0755 "${DEST}"

echo "goose_sidecar installed dest=${DEST}"
