#!/usr/bin/env bash
# scripts/agentic/session.sh — Reusable named-pipe session management for Script Kit GPUI.
#
# Usage:
#   session.sh start  [SESSION_NAME]   — Create or resume a session (default: "default")
#   session.sh send   SESSION_NAME CMD — Send a JSON command (fire-and-forget)
#   session.sh rpc    SESSION_NAME CMD [--expect TYPE] [--timeout MS]
#                                      — Send a JSON command and await the response
#   session.sh stop   [SESSION_NAME]   — Stop a session and clean up
#   session.sh status [SESSION_NAME]   — Print session state as JSON
#
# All output on stdout is machine-readable JSON. Diagnostics go to stderr.
# Sessions survive across shells — no fd 3 trick required.

set -euo pipefail

SCHEMA_VERSION=1
SESSION_DIR="${SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions}"
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BINARY="${PROJECT_ROOT}/target/debug/script-kit-gpui"

# --- helpers ----------------------------------------------------------------

log() { echo "[session.sh] $*" >&2; }

json_envelope() {
  local status="$1"; shift
  # Remaining args are key:value pairs injected into the envelope
  local extra=""
  while [ $# -gt 0 ]; do
    extra="${extra},\"${1%%:*}\":${1#*:}"
    shift
  done
  printf '{"schemaVersion":%d,"status":"%s"%s}\n' "$SCHEMA_VERSION" "$status" "$extra"
}

json_error() {
  local code="$1" msg="$2"
  printf '{"schemaVersion":%d,"status":"error","error":{"code":"%s","message":"%s"}}\n' \
    "$SCHEMA_VERSION" "$code" "$msg"
}

session_dir() { echo "${SESSION_DIR}/$1"; }

# --- start ------------------------------------------------------------------

cmd_start() {
  local name="${1:-default}"
  local sdir
  sdir="$(session_dir "$name")"
  local input_fifo="${sdir}/input"
  local log_path="${sdir}/app.log"
  local responses_path="${sdir}/responses.ndjson"

  # Resume if healthy
  if [ -f "${sdir}/pid" ]; then
    local old_pid
    old_pid="$(cat "${sdir}/pid")"
    if kill -0 "$old_pid" 2>/dev/null; then
      log "Resuming existing session '${name}' (pid ${old_pid})"
      json_envelope "ok" \
        "session:\"${name}\"" \
        "pid:${old_pid}" \
        "pipe:\"${input_fifo}\"" \
        "log:\"${log_path}\"" \
        "responses:\"${responses_path}\"" \
        "resumed:true"
      return 0
    else
      log "Stale session '${name}' (pid ${old_pid} dead). Cleaning up."
      rm -rf "${sdir}"
    fi
  fi

  # Check binary
  if [ ! -x "$BINARY" ]; then
    json_error "binary_missing" "Binary not found at ${BINARY}. Run cargo build first."
    return 1
  fi

  # Create session directory and pipe
  mkdir -p "${sdir}"
  local pipe_path="${sdir}/pipe"
  local pid_path="${sdir}/pid"

  rm -f "$pipe_path"
  mkfifo "$pipe_path"

  # Agents send commands by appending to the input FIFO via `session.sh send`.
  # We use a secondary FIFO as an input queue that a background forwarder relays
  # into the app pipe while keeping the write end open across shells.
  rm -f "$input_fifo"
  mkfifo "$input_fifo"

  # Create the responses artifact file
  : > "$responses_path"

  # Background forwarder: reads from input_fifo and writes to pipe.
  # It is started before the app so the app's read-open on the primary FIFO
  # does not block forever waiting for a writer.
  nohup bash -c '
    pipe_path="$1"
    input_fifo="$2"

    exec 9>"$pipe_path"

    # Continuously read from input_fifo, reopening after each writer disconnects.
    # This keeps the primary pipe writer alive for the app process.
    while IFS= read -r line; do
      printf "%s\n" "$line" >&9
    done < <(
      while true; do
        if [ -p "$input_fifo" ]; then
          cat "$input_fifo" 2>/dev/null || true
        else
          break
        fi
      done
    )
  ' _ "$pipe_path" "$input_fifo" </dev/null >/dev/null 2>&1 &
  local fwd_pid=$!
  echo "$fwd_pid" > "${sdir}/fwd_pid"

  # Launch the app reading from the pipe after the forwarder has opened the
  # write end, otherwise the shell can deadlock opening the read end.
  export SCRIPT_KIT_AI_LOG=1
  nohup "$BINARY" < "$pipe_path" > "$log_path" 2>&1 &
  local app_pid=$!
  echo "$app_pid" > "$pid_path"

  # Give the app a moment to start
  sleep 0.5

  # Verify it's still alive
  if ! kill -0 "$app_pid" 2>/dev/null; then
    json_error "start_failed" "App process exited immediately. Check ${log_path}"
    rm -rf "${sdir}"
    return 1
  fi

  log "Started session '${name}' (pid ${app_pid})"
  json_envelope "ok" \
    "session:\"${name}\"" \
    "pid:${app_pid}" \
    "pipe:\"${input_fifo}\"" \
    "log:\"${log_path}\"" \
    "responses:\"${responses_path}\"" \
    "resumed:false"
}

# --- send -------------------------------------------------------------------

cmd_send() {
  local name="${1:-default}"
  local cmd="${2:-}"

  if [ -z "$cmd" ]; then
    json_error "missing_command" "Usage: session.sh send SESSION_NAME JSON_COMMAND"
    return 1
  fi

  local sdir
  sdir="$(session_dir "$name")"
  local input_fifo="${sdir}/input"

  if [ ! -p "$input_fifo" ]; then
    json_error "no_session" "Session '${name}' not found or input FIFO missing."
    return 1
  fi

  # Verify app is alive
  if [ -f "${sdir}/pid" ]; then
    local pid
    pid="$(cat "${sdir}/pid")"
    if ! kill -0 "$pid" 2>/dev/null; then
      json_error "session_dead" "Session '${name}' app process (pid ${pid}) is not running."
      return 1
    fi
  fi

  # Write command to the input FIFO (non-blocking via timeout)
  if printf '%s\n' "$cmd" > "$input_fifo" 2>/dev/null; then
    json_envelope "ok" "session:\"${name}\"" "sent:true"
  else
    json_error "send_failed" "Failed to write to session '${name}' input FIFO."
    return 1
  fi
}

# --- rpc --------------------------------------------------------------------

cmd_rpc() {
  local name="${1:-default}"
  local cmd="${2:-}"
  shift 2 || true

  if [ -z "$cmd" ]; then
    json_error "missing_command" "Usage: session.sh rpc SESSION_NAME JSON_COMMAND [--expect TYPE] [--timeout MS]"
    return 1
  fi

  # Extract requestId from the command JSON (validate early, before session checks)
  local request_id
  request_id="$(
    printf '%s' "$cmd" \
      | sed -nE 's/.*"requestId"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' \
      | head -1 \
      || true
  )"
  if [ -z "$request_id" ]; then
    json_error "missing_request_id" "RPC command must contain a requestId field."
    return 1
  fi

  local sdir
  sdir="$(session_dir "$name")"
  local input_fifo="${sdir}/input"
  local log_path="${sdir}/app.log"
  local responses_path="${sdir}/responses.ndjson"

  if [ ! -p "$input_fifo" ]; then
    json_error "no_session" "Session '${name}' not found or input FIFO missing."
    return 1
  fi

  # Verify app is alive
  if [ -f "${sdir}/pid" ]; then
    local pid
    pid="$(cat "${sdir}/pid")"
    if ! kill -0 "$pid" 2>/dev/null; then
      json_error "session_dead" "Session '${name}' app process (pid ${pid}) is not running."
      return 1
    fi
  fi

  # Parse optional flags
  local expect_type=""
  local timeout_ms="5000"
  while [ $# -gt 0 ]; do
    case "$1" in
      --expect)  expect_type="${2:-}"; shift 2 ;;
      --timeout) timeout_ms="${2:-5000}"; shift 2 ;;
      *)         shift ;;
    esac
  done

  local start_offset="0"
  if [ -f "$log_path" ]; then
    start_offset="$(wc -c < "$log_path" | tr -d '[:space:]')"
  fi

  # Send the command (fire-and-forget to the pipe)
  if ! printf '%s\n' "$cmd" > "$input_fifo" 2>/dev/null; then
    json_error "send_failed" "Failed to write to session '${name}' input FIFO."
    return 1
  fi

  # Await the response using the TypeScript helper
  local await_args=(
    --session "$name"
    --request-id "$request_id"
    --timeout "$timeout_ms"
    --start-offset "$start_offset"
  )
  if [ -n "$expect_type" ]; then
    await_args+=(--expect "$expect_type")
  fi

  local result
  local exit_code=0
  result="$(bun "${PROJECT_ROOT}/scripts/agentic/await-response.ts" "${await_args[@]}")" || exit_code=$?

  # Append to responses artifact
  if [ -n "$result" ]; then
    printf '%s\n' "$result" >> "$responses_path" 2>/dev/null || true
  fi

  printf '%s\n' "$result"
  return $exit_code
}

# --- status -----------------------------------------------------------------

cmd_status() {
  local name="${1:-default}"
  local sdir
  sdir="$(session_dir "$name")"

  if [ ! -d "$sdir" ]; then
    json_envelope "not_found" "session:\"${name}\"" "alive:false"
    return 0
  fi

  local pid="0"
  local alive="false"
  local pipe_path="${sdir}/input"
  local log_path="${sdir}/app.log"
  local responses_path="${sdir}/responses.ndjson"
  local pipe_writable="false"

  if [ -f "${sdir}/pid" ]; then
    pid="$(cat "${sdir}/pid")"
    if kill -0 "$pid" 2>/dev/null; then
      alive="true"
    fi
  fi

  if [ -p "$pipe_path" ]; then
    pipe_writable="true"
  fi

  json_envelope "ok" \
    "session:\"${name}\"" \
    "pid:${pid}" \
    "alive:${alive}" \
    "pipe:\"${pipe_path}\"" \
    "pipeWritable:${pipe_writable}" \
    "log:\"${log_path}\"" \
    "responses:\"${responses_path}\""
}

# --- stop -------------------------------------------------------------------

cmd_stop() {
  local name="${1:-default}"
  local sdir
  sdir="$(session_dir "$name")"

  if [ ! -d "$sdir" ]; then
    json_envelope "ok" "session:\"${name}\"" "wasRunning:false"
    return 0
  fi

  # Kill forwarder
  if [ -f "${sdir}/fwd_pid" ]; then
    local fwd_pid
    fwd_pid="$(cat "${sdir}/fwd_pid")"
    kill "$fwd_pid" 2>/dev/null || true
    wait "$fwd_pid" 2>/dev/null || true
  fi

  # Kill app
  if [ -f "${sdir}/pid" ]; then
    local pid
    pid="$(cat "${sdir}/pid")"
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
      log "Stopped session '${name}' (pid ${pid})"
    fi
  fi

  # Clean up FIFOs and directory
  rm -f "${sdir}/pipe" "${sdir}/input"
  rm -rf "${sdir}"

  json_envelope "ok" "session:\"${name}\"" "wasRunning:true"
}

# --- main -------------------------------------------------------------------

subcmd="${1:-}"
shift || true

case "$subcmd" in
  start)  cmd_start "$@" ;;
  send)   cmd_send "$@" ;;
  rpc)    cmd_rpc "$@" ;;
  stop)   cmd_stop "$@" ;;
  status) cmd_status "$@" ;;
  *)
    json_error "unknown_command" "Usage: session.sh {start|send|rpc|stop|status} [SESSION_NAME] [ARGS]"
    exit 1
    ;;
esac
