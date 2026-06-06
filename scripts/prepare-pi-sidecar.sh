#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_LOCAL_PI_REPO="${REPO_ROOT}/../pi_agent_rust"
if [[ -n "${PI_AGENT_RUST_DIR:-}" ]]; then
  PI_REPO="${PI_AGENT_RUST_DIR}"
elif [[ -f "${DEFAULT_LOCAL_PI_REPO}/Cargo.toml" ]]; then
  PI_REPO="${DEFAULT_LOCAL_PI_REPO}"
elif [[ -n "${RUNNER_TEMP:-}" ]]; then
  PI_REPO="${RUNNER_TEMP}/pi_agent_rust-src"
else
  PI_REPO="${REPO_ROOT}/../pi_agent_rust-src"
fi
PI_AGENT_RUST_URL="${PI_AGENT_RUST_URL:-https://github.com/Dicklesworthstone/pi_agent_rust.git}"
PI_AGENT_RUST_REF="${PI_AGENT_RUST_REF:-3d1a3950c16ffdb10cd81780b26921c75c180770}"
PI_BIN="${PI_REPO}/target/release/pi"
DEST="${REPO_ROOT}/target/pi-sidecar/pi"

echo "pi_sidecar prepare repo=${PI_REPO}"

if [[ ! -f "${PI_REPO}/Cargo.toml" ]]; then
  mkdir -p "$(dirname "${PI_REPO}")"
  git clone --filter=blob:none "${PI_AGENT_RUST_URL}" "${PI_REPO}"
  git -C "${PI_REPO}" checkout "${PI_AGENT_RUST_REF}"
fi

test -f "${PI_REPO}/Cargo.toml"
git -C "${PI_REPO}" rev-parse HEAD
cargo build --manifest-path "${PI_REPO}/Cargo.toml" --locked --release --bin pi

mkdir -p "$(dirname "${DEST}")"
cp "${PI_BIN}" "${DEST}"
chmod 0755 "${DEST}"

echo "pi_sidecar ready dest=${DEST}"
