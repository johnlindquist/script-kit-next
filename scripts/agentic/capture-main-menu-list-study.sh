#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: scripts/agentic/capture-main-menu-list-study.sh <variant-id>" >&2
  exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VARIANT_ID="$1"
SURFACE="main-menu-list-studies"
SCREENSHOT_DIR="${PROJECT_ROOT}/test-screenshots"
CAPTURE_PATH="${SCREENSHOT_DIR}/${SURFACE}-${VARIANT_ID}.png"
PIPE="$(mktemp -u /tmp/sk-main-menu-list-study.XXXXXX)"
LOG_PATH="$(mktemp /tmp/sk-main-menu-list-study-log.XXXXXX)"
APP_PID=""

cleanup() {
  if [ -n "${APP_PID}" ] && kill -0 "${APP_PID}" 2>/dev/null; then
    kill "${APP_PID}" 2>/dev/null || true
    wait "${APP_PID}" 2>/dev/null || true
  fi
  exec 3>&- 2>/dev/null || true
  rm -f "${PIPE}"
}

trap cleanup EXIT

mkdir -p "${SCREENSHOT_DIR}"

cd "${PROJECT_ROOT}"

cargo build --features storybook --bin script-kit-gpui >/dev/null
cargo run --features storybook --bin storybook -- --adopt --story main-menu-raycast-weight-studies --variant "${VARIANT_ID}" >/dev/null

mkfifo "${PIPE}"
export SCRIPT_KIT_AI_LOG=1
export SCRIPT_KIT_STORYBOOK_MAIN_MENU_LIST_STUDY=1
"${PROJECT_ROOT}/target/debug/script-kit-gpui" < "${PIPE}" > "${LOG_PATH}" 2>&1 &
APP_PID="$!"

exec 3>"${PIPE}"
sleep 1
printf '{"type":"show"}\n' >&3
sleep 1.5
printf '{"type":"captureWindow","title":"","path":"%s"}\n' "${CAPTURE_PATH}" >&3
sleep 1

bun scripts/agentic/write-storybook-fixture.ts \
  "${CAPTURE_PATH}" \
  "${SURFACE}" \
  "${VARIANT_ID}" \
  main \
  scriptList
