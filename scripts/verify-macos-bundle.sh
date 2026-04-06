#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:-target/release/bundle/osx/Script Kit.app}"
MACOS_DIR="${APP_PATH}/Contents/MacOS"
EXPECTED_BIN="${MACOS_DIR}/script-kit-gpui"

echo "bundle_verify begin app=${APP_PATH}"

test -d "${APP_PATH}"
test -d "${MACOS_DIR}"
test -x "${EXPECTED_BIN}"

echo "bundle_verify macos_dir_listing"
find "${MACOS_DIR}" -maxdepth 1 -type f -print | sort

UNEXPECTED="$(find "${MACOS_DIR}" -maxdepth 1 -type f ! -name 'script-kit-gpui' -print || true)"
if [[ -n "${UNEXPECTED}" ]]; then
  echo "bundle_verify unexpected=${UNEXPECTED}" >&2
  exit 1
fi

echo "bundle_verify ok app=${APP_PATH}"
