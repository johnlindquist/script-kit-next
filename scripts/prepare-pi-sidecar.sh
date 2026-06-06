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
PI_TARGET_DIR="${PI_AGENT_RUST_TARGET_DIR:-${PI_REPO}/target}"
PI_BIN="${PI_TARGET_DIR}/release/pi"
DEST="${REPO_ROOT}/target/pi-sidecar/pi"
STRICT_REF="${PI_AGENT_RUST_STRICT_REF:-${CI:-0}}"

echo "pi_sidecar prepare repo=${PI_REPO}"
echo "pi_sidecar target_dir=${PI_TARGET_DIR}"

if [[ ! -d "${PI_REPO}/.git" ]]; then
  rm -rf "${PI_REPO}"
  mkdir -p "$(dirname "${PI_REPO}")"
  git clone --filter=blob:none --no-checkout "${PI_AGENT_RUST_URL}" "${PI_REPO}"
  STRICT_REF=1
fi

if [[ "${STRICT_REF}" == "1" || "${STRICT_REF}" == "true" ]]; then
  git -C "${PI_REPO}" fetch --filter=blob:none origin "${PI_AGENT_RUST_REF}"
  git -C "${PI_REPO}" checkout --detach "${PI_AGENT_RUST_REF}"
fi

test -f "${PI_REPO}/Cargo.toml"
actual_ref="$(git -C "${PI_REPO}" rev-parse HEAD)"
echo "pi_sidecar ref=${actual_ref}"
if [[ "${STRICT_REF}" == "1" || "${STRICT_REF}" == "true" ]]; then
  if [[ "${actual_ref}" != "${PI_AGENT_RUST_REF}" ]]; then
    echo "pi_sidecar expected ${PI_AGENT_RUST_REF}, got ${actual_ref}" >&2
    exit 1
  fi
fi

CARGO_TARGET_DIR="${PI_TARGET_DIR}" cargo build --manifest-path "${PI_REPO}/Cargo.toml" --locked --release --bin pi

mkdir -p "$(dirname "${DEST}")"
install -m 0755 "${PI_BIN}" "${DEST}"
chmod 0755 "${DEST}"

echo "pi_sidecar ready dest=${DEST}"
