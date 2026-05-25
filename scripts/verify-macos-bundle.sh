#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:-target/release/bundle/osx/Script Kit.app}"
MACOS_DIR="${APP_PATH}/Contents/MacOS"
EXPECTED_BIN="${MACOS_DIR}/script-kit-gpui"
EXPECTED_PI_BIN="${MACOS_DIR}/pi"

echo "bundle_verify begin app=${APP_PATH}"

test -d "${APP_PATH}"
test -d "${MACOS_DIR}"
test -x "${EXPECTED_BIN}"
test -x "${EXPECTED_PI_BIN}"

echo "bundle_verify macos_dir_listing"
find "${MACOS_DIR}" -maxdepth 1 -type f -print | sort

if command -v file >/dev/null 2>&1; then
  file "${EXPECTED_BIN}"
  file "${EXPECTED_PI_BIN}"
fi

UNEXPECTED="$(find "${MACOS_DIR}" -maxdepth 1 -type f ! -name 'script-kit-gpui' ! -name 'pi' -print || true)"
if [[ -n "${UNEXPECTED}" ]]; then
  echo "bundle_verify unexpected=${UNEXPECTED}" >&2
  exit 1
fi

echo "bundle_verify sidecar=${EXPECTED_PI_BIN}"
echo "bundle_verify ok app=${APP_PATH}"
