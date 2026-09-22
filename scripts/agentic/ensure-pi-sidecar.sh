#!/usr/bin/env bash
# Validate or repair the Pi binary selected by the debug runtime.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SIDECAR="${REPO_ROOT}/target/pi-sidecar/pi"
MODE="${1:---auto}"

log() { echo "[ensure-pi-sidecar] $*" >&2; }
usage() {
	echo "usage: scripts/agentic/ensure-pi-sidecar.sh [--check|--repair]" >&2
	exit 2
}

case "${MODE}" in
--auto | --check | --repair) ;;
*) usage ;;
esac
if [[ $# -gt 1 ]]; then
	usage
fi
if [[ "${SCRIPT_KIT_NONINTERACTIVE:-0}" == "1" && "$MODE" == "--auto" ]]; then
  log 'ERROR: agent provisioning requires explicit --repair (inspection never provisions)'
  exit 78
fi

health_receipt() {
	bun "${REPO_ROOT}/scripts/agentic/pi-sidecar-health.ts" "$1"
}

healthy() {
	local candidate="$1"
	[[ -x "${candidate}" ]] && health_receipt "${candidate}"
}

resolved() {
	log "pi available: $1"
	exit 0
}

# Rust's default_pi_binary treats an explicit override as authoritative. Never
# let a healthy repo-local sidecar mask a broken override. Callers (dev.sh /
# dev-cycle) may set SCRIPT_KIT_PI_BINARY to the external dest after ensure;
# this script never writes that override and never copies target/pi-sidecar.
if [[ -n "${SCRIPT_KIT_PI_BINARY:-}" ]]; then
	override="${SCRIPT_KIT_PI_BINARY/#\~/$HOME}"
	if healthy "${override}"; then
		resolved "${override} (SCRIPT_KIT_PI_BINARY)"
	fi
	log "ERROR: explicit SCRIPT_KIT_PI_BINARY=${SCRIPT_KIT_PI_BINARY} is not RPC-healthy"
	log "Unset the override or repair that exact binary; fallback binaries are intentionally ignored."
	exit 1
fi

# SSD human-dev storage retargets the managed sidecar after helper revalidation.
# --check parses read-only inspect JSON; auto/repair eval prepare --shell only.
# Standalone/CI keep the repo-local dest when this root is unset.
if [[ -n "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
	if [[ "${MODE}" == "--check" ]]; then
		_inspect_json="$(python3 -B "${REPO_ROOT}/scripts/agentic/dev-storage.py" --repo "${REPO_ROOT}" inspect)" || exit $?
		_pi_dir="$(
			printf '%s\n' "${_inspect_json}" | python3 -c '
import json, sys
payload = json.load(sys.stdin)
paths = payload.get("paths") or {}
pi = paths.get("SCRIPT_KIT_DEV_PI_DIR") or ""
if payload.get("ok") is not True or not pi:
    raise SystemExit(78)
print(pi)
'
		)" || {
			log "ERROR: helper inspect JSON missing SCRIPT_KIT_DEV_PI_DIR"
			exit 78
		}
		SIDECAR="${_pi_dir}/pi"
		unset _inspect_json _pi_dir
	else
		_dev_storage_exports="$(python3 -B "${REPO_ROOT}/scripts/agentic/dev-storage.py" --repo "${REPO_ROOT}" prepare --shell)" || exit $?
		eval "${_dev_storage_exports}"
		unset _dev_storage_exports
		if [[ -z "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" || -z "${SCRIPT_KIT_DEV_PI_DIR:-}" ]]; then
			log "ERROR: helper prepare did not export SCRIPT_KIT_DEV_STORAGE_ROOT/SCRIPT_KIT_DEV_PI_DIR"
			exit 78
		fi
		SIDECAR="${SCRIPT_KIT_DEV_PI_DIR}/pi"
	fi
fi

# An executable target sidecar is also authoritative in debug Rust resolution.
# Default mode checks first and repairs only when needed; explicit --repair
# deliberately rebuilds the managed pinned candidate.
if [[ "${MODE}" != "--repair" ]] && healthy "${SIDECAR}"; then
	resolved "${SIDECAR}"
fi
if [[ "${MODE}" == "--check" ]]; then
	if [[ -x "${SIDECAR}" ]]; then
		log "ERROR: runtime-selected ${SIDECAR} is executable but unhealthy"
		exit 1
	fi
	if [[ -z "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
		for candidate in "${HOME}/dev/pi_agent_rust/target/release/pi" "${HOME}/dev/pi_agent_rust/target/debug/pi"; do
			if healthy "${candidate}"; then
				resolved "${candidate} (debug fallback)"
			fi
		done
	fi
	log "ERROR: no RPC-healthy Pi binary resolves"
	exit 1
fi

if [[ -x "${SIDECAR}" ]]; then
	log "repairing unhealthy runtime-selected sidecar: ${SIDECAR}"
else
	if [[ -n "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
		log "preparing missing external Pi sidecar"
	else
		log "preparing missing repo-local Pi sidecar"
	fi
fi
bash "${REPO_ROOT}/scripts/prepare-pi-sidecar.sh"
if healthy "${SIDECAR}"; then
	resolved "${SIDECAR} (repaired)"
fi

log "ERROR: prepare-pi-sidecar.sh completed but ${SIDECAR} is not RPC-healthy"
exit 1
