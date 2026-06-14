#!/usr/bin/env bash
# Ensure the repo-local Pi sidecar is available for dev runs of script-kit-gpui.
#
# Dev runs (./dev.sh, cargo run) execute the bare target binary, so the bundled
# Contents/MacOS/pi sidecar never resolves. Debug builds resolve, in order:
#   1. $SCRIPT_KIT_PI_BINARY
#   2. <repo>/target/pi-sidecar/pi   (built by scripts/prepare-pi-sidecar.sh)
#   3. ~/dev/pi_agent_rust/target/{release,debug}/pi
# (mirrors src/ai/agent_chat/pi/binary.rs::default_pi_binary)
#
# Fast path: exits 0 immediately when the repo-local sidecar is executable. If
# an override or local checkout binary exists, install it into the repo-local
# sidecar path. Otherwise build and install it via scripts/prepare-pi-sidecar.sh.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

log() { echo "[ensure-pi-sidecar] $*" >&2; }

resolved() {
    log "pi available: $1"
    exit 0
}

SIDECAR="${REPO_ROOT}/target/pi-sidecar/pi"

install_existing() {
    local source="$1"
    local label="$2"
    mkdir -p "$(dirname "${SIDECAR}")"
    install -m 0755 "${source}" "${SIDECAR}"
    resolved "${SIDECAR} (installed from ${label}: ${source})"
}

if [[ -x "${SIDECAR}" ]]; then
    resolved "${SIDECAR}"
fi

if [[ -n "${SCRIPT_KIT_PI_BINARY:-}" ]]; then
    expanded="${SCRIPT_KIT_PI_BINARY/#\~/$HOME}"
    if [[ -x "${expanded}" ]]; then
        install_existing "${expanded}" "SCRIPT_KIT_PI_BINARY"
    fi
    log "WARNING: SCRIPT_KIT_PI_BINARY=${SCRIPT_KIT_PI_BINARY} is set but not executable; checking fallbacks"
fi

for candidate in "${HOME}/dev/pi_agent_rust/target/release/pi" "${HOME}/dev/pi_agent_rust/target/debug/pi"; do
    if [[ -x "${candidate}" ]]; then
        install_existing "${candidate}" "local pi_agent_rust"
    fi
done

log "no pi binary resolved; building repo-local sidecar via scripts/prepare-pi-sidecar.sh (first build takes a few minutes)"
if bash "${REPO_ROOT}/scripts/prepare-pi-sidecar.sh"; then
    if [[ -x "${SIDECAR}" ]]; then
        resolved "${SIDECAR}"
    fi
    log "ERROR: prepare-pi-sidecar.sh succeeded but ${SIDECAR} is missing"
    exit 1
fi

log "ERROR: failed to prepare pi sidecar; Agent Chat (cmd+enter) will show 'Pi Agent Chat is unavailable'"
log "Fix: bash scripts/prepare-pi-sidecar.sh, or set SCRIPT_KIT_PI_BINARY to a pi binary"
exit 1
