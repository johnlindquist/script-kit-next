#!/usr/bin/env bash
set -euo pipefail

ref_name="${GITHUB_REF_NAME:-${1:-}}"

if [[ -z "$ref_name" ]]; then
  echo "usage: GITHUB_REF_NAME=v1.5.0 bash scripts/verify-release-version.sh" >&2
  echo "   or: bash scripts/verify-release-version.sh v1.5.0" >&2
  exit 64
fi

if [[ ! "$ref_name" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo 'expected a new vMAJOR.MINOR.PATCH release tag' >&2
  exit 1
fi
# Read only exact package sections; repeated or missing entries cannot pass.
version_in_table() {
  awk -v table="$1" ' /^\[/ { active = ($0 == table) } active && /^version *= *"/ {split($0,v,"\""); print v[2]}' Cargo.toml
}
package=$(version_in_table '[package]')
bundle=$(version_in_table '[package.metadata.bundle.bin.script-kit-gpui]')
locked=$(awk '/^\[\[package\]\]/{app=0} /^name = "script-kit-gpui"$/{app=1} app && /^version = "/{split($0,v,"\"");print v[2]}' Cargo.lock)
if [[ "$package" != "${ref_name#v}" || "$bundle" != "$package" || "$locked" != "$package" ]]; then
  echo "version mismatch: tag=$ref_name package=$package bundle=$bundle lock=$locked" >&2
  exit 1
fi
printf 'tag %s matches package, bundle and lockfile\n' "$ref_name"
