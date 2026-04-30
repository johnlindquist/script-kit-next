#!/usr/bin/env bash
set -euo pipefail

ref_name="${GITHUB_REF_NAME:-${1:-}}"

if [[ -z "$ref_name" ]]; then
  echo "usage: GITHUB_REF_NAME=v1.5.0 bash scripts/verify-release-version.sh" >&2
  echo "   or: bash scripts/verify-release-version.sh v1.5.0" >&2
  exit 64
fi

tag_version="${ref_name#v}"
cargo_version="$(grep -E '^version[[:space:]]*=' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"

if [[ "$tag_version" != "$cargo_version" ]]; then
  echo "tag $ref_name does not match Cargo.toml version $cargo_version; bump Cargo.toml or retag" >&2
  exit 1
fi

echo "tag $ref_name matches Cargo.toml version $cargo_version"
