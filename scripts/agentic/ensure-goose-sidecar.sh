#!/usr/bin/env bash
# Validate or repair the goose ACP sidecar selected by the debug runtime.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SIDECAR="${REPO_ROOT}/target/goose-sidecar/goose"
MODE="${1:---auto}"

log() { echo "[ensure-goose-sidecar] $*" >&2; }
usage() {
	echo "usage: scripts/agentic/ensure-goose-sidecar.sh [--check|--repair]" >&2
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
	bun "${REPO_ROOT}/scripts/agentic/goose-sidecar-health.ts" "$1"
}

healthy() {
	local candidate="$1"
	[[ -x "${candidate}" ]] && health_receipt "${candidate}"
}

# The managed sidecar must also carry the stamp of the pinned build inputs.
source "${REPO_ROOT}/scripts/goose-sidecar-pin.sh"
managed_healthy() {
	goose_sidecar_stamp_matches "$1" && healthy "$1"
}

resolved() {
	log "goose available: $1"
	exit 0
}

# Rust's default_goose_binary treats an explicit override as authoritative.
# Never let a healthy repo-local sidecar mask a broken override. Callers
# (dev.sh / dev-cycle) may set SCRIPT_KIT_GOOSE_BINARY to the external dest
# after ensure; this script never writes that override and never copies
# target/goose-sidecar.
if [[ -n "${SCRIPT_KIT_GOOSE_BINARY:-}" ]]; then
	override="${SCRIPT_KIT_GOOSE_BINARY/#\~/$HOME}"
	if healthy "${override}"; then
		resolved "${override} (SCRIPT_KIT_GOOSE_BINARY)"
	fi
	log "ERROR: explicit SCRIPT_KIT_GOOSE_BINARY=${SCRIPT_KIT_GOOSE_BINARY} does not answer ACP initialize"
	log "Unset the override or repair that exact binary; fallback binaries are intentionally ignored."
	exit 1
fi

# SSD human-dev storage retargets the managed sidecar after helper revalidation.
# --check parses read-only inspect JSON; auto/repair eval prepare --shell only.
# Standalone/CI keep the repo-local dest when this root is unset.
if [[ -n "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
	if [[ "${MODE}" == "--check" ]]; then
		_inspect_json="$(python3 -B "${REPO_ROOT}/scripts/agentic/dev-storage.py" --repo "${REPO_ROOT}" inspect)" || exit $?
		_goose_dir="$(
			printf '%s\n' "${_inspect_json}" | python3 -c '
import json, sys
payload = json.load(sys.stdin)
paths = payload.get("paths") or {}
goose = paths.get("SCRIPT_KIT_DEV_GOOSE_DIR") or ""
if payload.get("ok") is not True or not goose:
    raise SystemExit(78)
print(goose)
'
		)" || {
			log "ERROR: helper inspect JSON missing SCRIPT_KIT_DEV_GOOSE_DIR"
			exit 78
		}
		SIDECAR="${_goose_dir}/goose"
		unset _inspect_json _goose_dir
	else
		_dev_storage_exports="$(python3 -B "${REPO_ROOT}/scripts/agentic/dev-storage.py" --repo "${REPO_ROOT}" prepare --shell)" || exit $?
		eval "${_dev_storage_exports}"
		unset _dev_storage_exports
		if [[ -z "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" || -z "${SCRIPT_KIT_DEV_GOOSE_DIR:-}" ]]; then
			log "ERROR: helper prepare did not export SCRIPT_KIT_DEV_STORAGE_ROOT/SCRIPT_KIT_DEV_GOOSE_DIR"
			exit 78
		fi
		SIDECAR="${SCRIPT_KIT_DEV_GOOSE_DIR}/goose"
	fi
fi

# An executable target sidecar is also authoritative in debug Rust resolution.
# Default mode checks first and repairs only when needed; explicit --repair
# deliberately rebuilds the managed pinned candidate.
if [[ "${MODE}" != "--repair" ]] && managed_healthy "${SIDECAR}"; then
	resolved "${SIDECAR}"
fi
if [[ "${MODE}" == "--check" ]]; then
	if [[ -x "${SIDECAR}" ]] && ! goose_sidecar_stamp_matches "${SIDECAR}"; then
		log "ERROR: ${SIDECAR} is not stamped with the pinned build ($(goose_sidecar_stamp)); run scripts/agentic/ensure-goose-sidecar.sh"
	elif [[ -x "${SIDECAR}" ]]; then
		log "ERROR: runtime-selected ${SIDECAR} is executable but does not answer ACP initialize"
	else
		log "ERROR: no goose sidecar at ${SIDECAR}; run scripts/agentic/ensure-goose-sidecar.sh --repair"
	fi
	exit 1
fi

prepare_args=()
if [[ "${MODE}" == "--repair" ]]; then
	prepare_args=(--rebuild)
	log "rebuilding managed sidecar: ${SIDECAR}"
elif [[ -x "${SIDECAR}" ]] && ! goose_sidecar_stamp_matches "${SIDECAR}"; then
	log "replacing sidecar not stamped with the pinned build: ${SIDECAR}"
elif [[ -x "${SIDECAR}" ]]; then
	log "repairing unhealthy runtime-selected sidecar: ${SIDECAR}"
else
	if [[ -n "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
		log "preparing missing external goose sidecar"
	else
		log "preparing missing repo-local goose sidecar"
	fi
fi
bash "${REPO_ROOT}/scripts/prepare-goose-sidecar.sh" ${prepare_args[@]+"${prepare_args[@]}"}
if managed_healthy "${SIDECAR}"; then
	resolved "${SIDECAR} (repaired)"
fi

log "ERROR: prepare-goose-sidecar.sh completed but ${SIDECAR} does not answer ACP initialize"
exit 1
