#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  lat-llm-key-helper.sh [--clear] [--print-cache-path]

Environment:
  LAT_LLM_KEY_FETCH_CMD
      Shell command that prints the LLM key to stdout.
  LAT_LLM_KEY_CACHE_BACKEND
      One of: auto, keychain, file. Defaults to auto.
  LAT_LLM_KEY_CACHE_FILE
      Shared cache file path for the file backend.
  LAT_LLM_KEY_CACHE_TTL_SECONDS
      Cache lifetime in seconds. Defaults to 43200 (12 hours). Set to 0 to
      reuse the cache until it is manually cleared.
  LAT_LLM_KEY_KEYCHAIN_SERVICE
      macOS Keychain service name. Defaults to lat.llm-key.

Examples:
  export LAT_LLM_KEY_FETCH_CMD="/opt/homebrew/bin/op item get LAT_LLM_KEY --vault Personal --fields credential --reveal"
  export LAT_LLM_KEY_HELPER="$PWD/scripts/lat-llm-key-helper.sh"
EOF
}

cache_file() {
  if [[ -n "${LAT_LLM_KEY_CACHE_FILE:-}" ]]; then
    printf '%s\n' "$LAT_LLM_KEY_CACHE_FILE"
    return
  fi

  local cache_root="${XDG_CACHE_HOME:-$HOME/.cache}"
  printf '%s\n' "$cache_root/lat/llm-key"
}

clear_cache() {
  local file="$1"
  rm -f "$file" "$file.meta" "$file.meta.state"
}

meta_file() {
  printf '%s.meta\n' "$1"
}

file_mtime() {
  stat -f '%m' "$1"
}

command_hash() {
  printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

cache_valid() {
  local state_path="$1"
  local meta_path="$2"
  local ttl="$3"
  local expected_hash="$4"

  [[ -f "$state_path" && -f "$meta_path" ]] || return 1

  local cached_hash
  cached_hash="$(cat "$meta_path" 2>/dev/null || true)"
  [[ "$cached_hash" == "$expected_hash" ]] || return 1

  if [[ "$ttl" == "0" ]]; then
    return 0
  fi

  local now
  now="$(date +%s)"
  local mtime
  mtime="$(file_mtime "$state_path")"
  (( now - mtime <= ttl ))
}

with_lock() {
  local lock_dir="$1"
  local waited=0

  while ! mkdir "$lock_dir" 2>/dev/null; do
    sleep 0.1
    waited=$((waited + 1))
    if (( waited >= 300 )); then
      echo "timed out waiting for lock: $lock_dir" >&2
      return 1
    fi
  done

  trap "rmdir '$lock_dir' 2>/dev/null || true" EXIT
}

cache_backend() {
  local backend="${LAT_LLM_KEY_CACHE_BACKEND:-auto}"
  if [[ "$backend" != "auto" ]]; then
    printf '%s\n' "$backend"
    return
  fi

  if [[ "$(uname -s)" == "Darwin" ]] && command -v security >/dev/null 2>&1; then
    printf 'keychain\n'
    return
  fi

  printf 'file\n'
}

keychain_service() {
  printf '%s\n' "${LAT_LLM_KEY_KEYCHAIN_SERVICE:-lat.llm-key}"
}

keychain_account() {
  printf '%s\n' "${LAT_LLM_KEY_KEYCHAIN_ACCOUNT:-$USER}"
}

keychain_get() {
  local service="$1"
  local account="$2"
  security find-generic-password -w -a "$account" -s "$service"
}

keychain_set() {
  local service="$1"
  local account="$2"
  local key="$3"
  security add-generic-password -U -a "$account" -s "$service" -T /usr/bin/security -w "$key" >/dev/null
}

keychain_clear() {
  local service="$1"
  local account="$2"
  security delete-generic-password -a "$account" -s "$service" >/dev/null 2>&1 || true
}

main() {
  local mode="print"

  while (($#)); do
    case "$1" in
      --clear)
        mode="clear"
        shift
        ;;
      --print-cache-path)
        mode="path"
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "unknown argument: $1" >&2
        usage >&2
        exit 1
        ;;
    esac
  done

  local file
  file="$(cache_file)"
  local state_path="$file"
  local meta_path
  meta_path="$(meta_file "$file")"
  local backend
  backend="$(cache_backend)"
  local service
  service="$(keychain_service)"
  local account
  account="$(keychain_account)"

  if [[ "$mode" == "path" ]]; then
    if [[ "$backend" == "keychain" ]]; then
      printf 'keychain:%s:%s\n' "$service" "$account"
    else
      printf '%s\n' "$file"
    fi
    exit 0
  fi

  if [[ "$mode" == "clear" ]]; then
    clear_cache "$file"
    keychain_clear "$service" "$account"
    exit 0
  fi

  local fetch_cmd="${LAT_LLM_KEY_FETCH_CMD:-}"
  if [[ -z "$fetch_cmd" ]]; then
    echo "LAT_LLM_KEY_FETCH_CMD is required" >&2
    exit 1
  fi

  local ttl="${LAT_LLM_KEY_CACHE_TTL_SECONDS:-43200}"
  local lock_dir="$file.lock"
  local expected_hash
  expected_hash="$(command_hash "$fetch_cmd")"

  mkdir -p "$(dirname "$file")"
  with_lock "$lock_dir"

  if [[ "$backend" == "keychain" ]]; then
    state_path="$meta_path.state"
  fi

  if cache_valid "$state_path" "$meta_path" "$ttl" "$expected_hash"; then
    if [[ "$backend" == "keychain" ]]; then
      if key="$(keychain_get "$service" "$account" 2>/dev/null)"; then
        printf '%s\n' "$key"
        exit 0
      fi
      rm -f "$state_path" "$meta_path"
    else
      cat "$file"
      exit 0
    fi
  fi

  local key
  key="$(sh -lc "$fetch_cmd")"
  key="${key%$'\n'}"
  if [[ -z "$key" ]]; then
    echo "fetch command returned an empty key" >&2
    exit 1
  fi

  umask 077
  if [[ "$backend" == "keychain" ]]; then
    keychain_set "$service" "$account" "$key"
    printf '%s\n' "$expected_hash" > "$meta_path"
    : > "$state_path"
    printf '%s\n' "$key"
  else
    printf '%s\n' "$key" > "$file"
    printf '%s\n' "$expected_hash" > "$meta_path"
    cat "$file"
  fi
}

main "$@"
