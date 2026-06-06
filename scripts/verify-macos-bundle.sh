#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:-target/release/bundle/osx/Script Kit.app}"
MACOS_DIR="${APP_PATH}/Contents/MacOS"
RESOURCES_DIR="${APP_PATH}/Contents/Resources"
ASSETS_DIR="${RESOURCES_DIR}/assets"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ASSETS_DIR="${REPO_ROOT}/assets"
EXPECTED_BIN="${MACOS_DIR}/script-kit-gpui"
EXPECTED_PI_BIN="${MACOS_DIR}/pi"

echo "bundle_verify begin app=${APP_PATH}"

test -d "${APP_PATH}"
test -d "${MACOS_DIR}"
test -d "${RESOURCES_DIR}"
test -d "${ASSETS_DIR}"
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
echo "bundle_verify resources_dir=${RESOURCES_DIR}"

test -d "${ASSETS_DIR}/icons"
test -d "${ASSETS_DIR}/fonts"

required_resources=(
  "${RESOURCES_DIR}/icon.icns"
  "${ASSETS_DIR}/Info.plist.ext"
  "${ASSETS_DIR}/icon.icns"
  "${ASSETS_DIR}/icon.png"
  "${ASSETS_DIR}/icon@2x.png"
  "${ASSETS_DIR}/logo.svg"
  "${ASSETS_DIR}/icons/file.svg"
  "${ASSETS_DIR}/icons/file_code.svg"
  "${ASSETS_DIR}/icons/folder.svg"
  "${ASSETS_DIR}/icons/folder_open.svg"
  "${ASSETS_DIR}/icons/settings.svg"
  "${ASSETS_DIR}/icons/magnifying_glass.svg"
  "${ASSETS_DIR}/icons/agent_chat.svg"
  "${ASSETS_DIR}/icons/ai_provider_openai.svg"
  "${ASSETS_DIR}/fonts/JetBrainsMono-Regular.ttf"
  "${ASSETS_DIR}/fonts/JetBrainsMono-Bold.ttf"
  "${ASSETS_DIR}/fonts/JetBrainsMono-Italic.ttf"
  "${ASSETS_DIR}/fonts/JetBrainsMono-BoldItalic.ttf"
  "${ASSETS_DIR}/fonts/JetBrainsMono-Medium.ttf"
  "${ASSETS_DIR}/fonts/JetBrainsMono-SemiBold.ttf"
)

for resource in "${required_resources[@]}"; do
  if [[ ! -f "${resource}" ]]; then
    echo "bundle_verify missing_resource=${resource}" >&2
    exit 1
  fi
done

verify_asset_family() {
  local subdir="$1"
  local pattern="$2"
  local src_dir="${SOURCE_ASSETS_DIR}/${subdir}"
  local dst_dir="${ASSETS_DIR}/${subdir}"
  local src_count
  local dst_count

  test -d "${src_dir}"
  test -d "${dst_dir}"

  src_count="$(find "${src_dir}" -maxdepth 1 -type f -name "${pattern}" | wc -l | tr -d ' ')"
  dst_count="$(find "${dst_dir}" -maxdepth 1 -type f -name "${pattern}" | wc -l | tr -d ' ')"

  if [[ "${src_count}" != "${dst_count}" ]]; then
    echo "bundle_verify resource_count_mismatch subdir=${subdir} pattern=${pattern} source=${src_count} bundle=${dst_count}" >&2
    exit 1
  fi

  while IFS= read -r src_file; do
    local rel="${src_file#${SOURCE_ASSETS_DIR}/}"
    local dst_file="${ASSETS_DIR}/${rel}"
    if [[ ! -f "${dst_file}" ]]; then
      echo "bundle_verify missing_bundled_asset=${dst_file}" >&2
      exit 1
    fi
  done < <(find "${src_dir}" -maxdepth 1 -type f -name "${pattern}" | sort)
}

verify_asset_family "icons" "*.svg"
verify_asset_family "fonts" "*.ttf"
verify_asset_family "fonts" "*.txt"

svg_count="$(find "${ASSETS_DIR}/icons" -maxdepth 1 -type f -name '*.svg' | wc -l | tr -d ' ')"
ttf_count="$(find "${ASSETS_DIR}/fonts" -maxdepth 1 -type f -name '*.ttf' | wc -l | tr -d ' ')"
font_text_count="$(find "${ASSETS_DIR}/fonts" -maxdepth 1 -type f -name '*.txt' | wc -l | tr -d ' ')"

echo "bundle_verify resources ok svg_count=${svg_count} ttf_count=${ttf_count} font_text_count=${font_text_count}"
echo "bundle_verify ok app=${APP_PATH}"
