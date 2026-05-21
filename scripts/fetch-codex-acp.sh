#!/usr/bin/env bash
set -euo pipefail

TARGET_DIR="${1:-target/release/bundle/osx/Script Kit.app/Contents/MacOS}"
ARCH="${2:-$(uname -m)}"
CODEX_ACP_VERSION="${CODEX_ACP_VERSION:-0.14.0}"

# Map uname -m to npm package architecture identifier
case "$ARCH" in
  x86_64) NPM_ARCH="x64" ;;
  arm64) NPM_ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

PACKAGE_NAME="@zed-industries/codex-acp-darwin-${NPM_ARCH}"
PACKAGE_SPEC="${PACKAGE_NAME}@${CODEX_ACP_VERSION}"
echo "Fetching ${PACKAGE_SPEC} from npm registry..."

# Use a temporary directory for downloading and unpacking
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

TARBALL_BASENAME="$(npm pack "$PACKAGE_SPEC" --pack-destination "$TEMP_DIR" | tail -n 1)"
TARBALL="${TEMP_DIR}/${TARBALL_BASENAME}"
test -f "$TARBALL"
tar -xzf "$TARBALL" -C "$TEMP_DIR"

# Find the binary inside the unpacked package folder
BINARY_PATH=$(find "$TEMP_DIR/package" -type f -name "codex-acp" -print -quit)

if [[ -z "$BINARY_PATH" ]]; then
  echo "Error: codex-acp binary not found in package" >&2
  exit 1
fi

mkdir -p "$TARGET_DIR"
install -m 0755 "$BINARY_PATH" "$TARGET_DIR/codex-acp"

echo "codex_acp_embedded path=$TARGET_DIR/codex-acp"
shasum -a 256 "$TARGET_DIR/codex-acp"
