#!/usr/bin/env bash
# Build the pinned goose ACP sidecar that powers Script Kit's AI surfaces.
#
# goose (Apache-2.0, https://github.com/aaif-goose/goose) is built from an
# exact pinned commit with only the features Script Kit needs: rustls TLS and
# no self-update. The default goose build also embeds llama.cpp, V8, voice,
# telemetry and bundled MCP servers; none of those ship inside Script Kit.
#
# A destination binary stamped with the pinned inputs is reused as-is. Human
# dev runs also adopt a stamped sidecar from the repo-local dest or any other
# SSD storage root before building. `--rebuild` skips both and always builds.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
REBUILD=0
case "${1:-}" in
  "") ;;
  --rebuild) REBUILD=1 ;;
  *) echo "usage: scripts/prepare-goose-sidecar.sh [--rebuild]" >&2; exit 2 ;;
esac
for worker_setting in CARGO_BUILD_JOBS RUST_TEST_THREADS; do
  worker_count="${!worker_setting:-2}"
  [[ "$worker_count" =~ ^[12]$ ]] || { echo "goose_sidecar invalid ${worker_setting}" >&2; exit 78; }
  export "${worker_setting}=${worker_count}"
done
native_workers="${CMAKE_BUILD_PARALLEL_LEVEL:-1}"
[[ "$native_workers" == "1" ]] || { echo 'goose_sidecar native worker budget must be one inside Cargo' >&2; exit 78; }
export CMAKE_BUILD_PARALLEL_LEVEL=1
GOOSE_SIDECAR_URL="${GOOSE_SIDECAR_URL:-https://github.com/aaif-goose/goose.git}"
source "${REPO_ROOT}/scripts/goose-sidecar-pin.sh"
# Only the Rust workspace is materialized; docs, UI and evals never build here.
GOOSE_SIDECAR_PATHS=(Cargo.toml Cargo.lock rust-toolchain.toml .cargo crates vendor)
if [[ -n "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" ]]; then
  _dev_storage_exports="$(python3 -B "${REPO_ROOT}/scripts/agentic/dev-storage.py" --repo "${REPO_ROOT}" prepare --shell)" || exit $?
  eval "${_dev_storage_exports}"
  unset _dev_storage_exports
  if [[ -z "${SCRIPT_KIT_DEV_STORAGE_ROOT:-}" || -z "${SCRIPT_KIT_DEV_GOOSE_DIR:-}" || -z "${GOOSE_SIDECAR_CACHE_DIR:-}" || -z "${GOOSE_SIDECAR_TARGET_DIR:-}" ]]; then
    echo 'goose_sidecar helper prepare did not export required storage paths' >&2
    exit 78
  fi
  CACHE_ROOT="${GOOSE_SIDECAR_CACHE_DIR}"
  GOOSE_TARGET_DIR="${GOOSE_SIDECAR_TARGET_DIR}"
  DEST_DIR="${SCRIPT_KIT_DEV_GOOSE_DIR}"
  lease="${SCRIPT_KIT_DEV_STORAGE_ROOT}/.locks/goose-sidecar-source.lock"
else
  CACHE_ROOT="${GOOSE_SIDECAR_CACHE_DIR:-${REPO_ROOT}/target/goose-sidecar/cache}"
  GOOSE_TARGET_DIR="${GOOSE_SIDECAR_TARGET_DIR:-${CACHE_ROOT}/cargo-target}"
  DEST_DIR="${REPO_ROOT}/target/goose-sidecar"
  lease="${REPO_ROOT}/target-agent/.locks/goose-sidecar-source.lock"
fi
OBJECT_CACHE="${CACHE_ROOT}/goose.git"
SOURCE_DIR="${CACHE_ROOT}/source-${GOOSE_SIDECAR_REF}"
GOOSE_BIN="${GOOSE_TARGET_DIR}/release/goose"
DEST="${DEST_DIR}/goose"

log() { echo "goose_sidecar $*"; }
if [[ "${SCRIPT_KIT_NONINTERACTIVE:-0}" == "1" ]]; then
  [[ "$GOOSE_SIDECAR_URL" == 'https://github.com/aaif-goose/goose.git' \
    && "$CACHE_ROOT" == "${REPO_ROOT}/target/goose-sidecar/cache" \
    && "$GOOSE_TARGET_DIR" == "${REPO_ROOT}/target/goose-sidecar/cache/cargo-target" ]] \
    || { echo 'goose_sidecar unregistered agent source/cache/target override' >&2; exit 78; }
fi
for path in "$CACHE_ROOT" "$OBJECT_CACHE" "$SOURCE_DIR" "$GOOSE_TARGET_DIR" "$DEST_DIR" "$DEST" "$DEST.stamp"; do
  [[ ! -L "$path" ]] || { echo 'goose_sidecar refuses symlink ownership' >&2; exit 78; }
done
source "${REPO_ROOT}/scripts/agentic/cargo-cache-locks.sh"
generation="$(python3 -c 'import uuid; print(uuid.uuid4())')"
cargo_cache_lease acquire "$lease" "$$" "$generation" 600000 >/dev/null
source_tmp=""; candidate=""
cleanup() {
  [[ -z "$source_tmp" ]] || rm -rf -- "$source_tmp"
  [[ -z "$candidate" ]] || rm -f -- "$candidate" "$candidate.stamp"
  cargo_cache_lease release "$lease" "$$" "$generation" >/dev/null
}
trap cleanup EXIT

mkdir -p "${DEST_DIR}"

health() { bun "${REPO_ROOT}/scripts/agentic/goose-sidecar-health.ts" "$1"; }

# Probe a candidate in the destination directory, then use same-directory
# rename so observers see either the previous healthy binary or the new one.
# The stamp lands after the binary, so an interrupted install reads as stale.
install_sidecar() {
  candidate="${DEST_DIR}/.goose.candidate.${generation}"
  install -m 0755 "$1" "${candidate}" || return 1
  health "${candidate}" || return 1
  mv -f "${candidate}" "${DEST}" || return 1
  goose_sidecar_stamp > "${candidate}.stamp" || return 1
  mv -f "${candidate}.stamp" "${DEST}.stamp" || return 1
  candidate=""
}

if [[ "${REBUILD}" == "0" ]]; then
  if goose_sidecar_stamp_matches "${DEST}" && health "${DEST}" >/dev/null; then
    log "reuse dest=${DEST} ref=${GOOSE_SIDECAR_REF}"
    exit 0
  fi
  if [[ "${SCRIPT_KIT_NONINTERACTIVE:-0}" != "1" ]]; then
    for seed in "${REPO_ROOT}/target/goose-sidecar/goose" /Volumes/*/sk/*/goose-sidecar/goose; do
      [[ "${seed}" != "${DEST}" && ! -L "${seed}" ]] || continue
      goose_sidecar_stamp_matches "${seed}" || continue
      log "adopting stamped sidecar seed=${seed}"
      if install_sidecar "${seed}"; then
        log "ready dest=${DEST} ref=${GOOSE_SIDECAR_REF} (adopted)"
        exit 0
      fi
      rm -f -- "${candidate}" "${candidate}.stamp"; candidate=""
    done
  fi
fi

mkdir -p "${CACHE_ROOT}"

if [[ ! -d "${OBJECT_CACHE}" ]]; then
  log "initializing repo-managed object cache=${OBJECT_CACHE}"
  git init --bare "${OBJECT_CACHE}"
  git --git-dir="${OBJECT_CACHE}" remote add origin "${GOOSE_SIDECAR_URL}"
fi

# Offline first: only contact the remote when the pinned commit is absent.
if ! git --git-dir="${OBJECT_CACHE}" cat-file -e "${GOOSE_SIDECAR_REF}^{commit}" 2>/dev/null; then
  log "pinned ref missing from cache; fetching ref=${GOOSE_SIDECAR_REF}"
  git --git-dir="${OBJECT_CACHE}" fetch --depth 1 --filter=blob:none --no-tags origin "${GOOSE_SIDECAR_REF}"
fi
actual_ref="$(git --git-dir="${OBJECT_CACHE}" rev-parse "${GOOSE_SIDECAR_REF}^{commit}")"
if [[ "${actual_ref}" != "${GOOSE_SIDECAR_REF}" ]]; then
  echo "goose_sidecar expected ${GOOSE_SIDECAR_REF}, got ${actual_ref}" >&2
  exit 1
fi

# Materialize an immutable source snapshot from the exact cached commit. This
# never reads or mutates an adjacent developer checkout.
if [[ ! -f "${SOURCE_DIR}/Cargo.toml" ]]; then
  source_tmp="${CACHE_ROOT}/.source-${GOOSE_SIDECAR_REF}-${generation}"
  mkdir "${source_tmp}"
  git --git-dir="${OBJECT_CACHE}" archive "${GOOSE_SIDECAR_REF}" -- "${GOOSE_SIDECAR_PATHS[@]}" | tar -x -C "${source_tmp}"
  mv "${source_tmp}" "${SOURCE_DIR}"
  source_tmp=""
fi

python3 "${REPO_ROOT}/scripts/agentic/goose-sidecar-source.py" "$OBJECT_CACHE" "$GOOSE_SIDECAR_REF" "$SOURCE_DIR" "${GOOSE_SIDECAR_PATHS[@]}"

log "build source=${SOURCE_DIR} ref=${actual_ref} target_dir=${GOOSE_TARGET_DIR}"
if [[ "${SCRIPT_KIT_NONINTERACTIVE:-0}" == "1" ]]; then
  bash "${REPO_ROOT}/scripts/agentic/agent-cargo.sh" goose-sidecar-build
else
  # Build with Script Kit's pinned toolchain (rust-toolchain.toml at the repo
  # root), the same one CI installs and the managed agent build uses.
  CARGO_TARGET_DIR="${GOOSE_TARGET_DIR}" cargo build \
    --manifest-path "${SOURCE_DIR}/Cargo.toml" \
    --locked --release -p goose-cli --bin goose \
    --no-default-features --features "${GOOSE_SIDECAR_FEATURES}"
fi

install_sidecar "${GOOSE_BIN}"

log "ready dest=${DEST} ref=${actual_ref}"
