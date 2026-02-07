# Security Audit (`src/**/*.rs`)

Date: 2026-02-07  
Agent: `codex-security-audit`

## Scope and Method

Audited Rust sources under `src/**/*.rs` with focus on:

- command/script execution paths
- path handling and filesystem writes
- stdin protocol privilege boundaries
- deserialization of untrusted input

High-signal files reviewed in depth:

- `src/app_impl.rs`
- `src/main.rs`
- `src/stdin_commands.rs`
- `src/prompt_handler.rs`
- `src/executor/scriptlet.rs`
- `src/file_search.rs`
- `src/app_actions.rs`

## Severity Rubric

- Critical: Clear remote/local code execution or privilege boundary bypass in common paths.
- High: Injection or arbitrary dangerous operation reachable with realistic attacker control.
- Medium: Privilege expansion or unsafe sink exposure, usually threat-model dependent.
- Low: Hardening gaps or information exposure with lower practical exploitability.

## Executive Summary

- Findings: **1 High, 3 Medium, 3 Low**.
- Highest-risk issue is **AppleScript injection via unescaped path interpolation** in path actions.
- Main architectural risk is that the **stdin command protocol is a high-privilege control surface** (run scripts, simulate keys, open windows, write screenshots) without explicit auth/capability gating.
- No direct shell interpolation vulnerabilities were found in primary script execution (`Command::new(...).arg(...)` is used consistently in runner paths), but there are several hardening gaps around logging, temp files, and unbounded stdin command payloads.

## Findings

### SA-HIGH-001: AppleScript injection via unescaped filesystem paths

- Severity: **High**
- Files:
  - `src/app_impl.rs:5659`
  - `src/app_impl.rs:5661`
  - `src/app_impl.rs:5664`
  - `src/app_impl.rs:5779`
  - `src/app_impl.rs:5781`
  - `src/app_impl.rs:5783`

#### Evidence

Two path actions build AppleScript with raw path interpolation:

1. `open_in_terminal`:
- `do script "cd '{}'"` with `dir_path` inserted directly.

2. `move_to_trash`:
- `delete POSIX file "{}"` with `path_str` inserted directly.

Both are passed to `osascript -e ...`.

#### Risk

A crafted filename/path containing quote characters can break AppleScript/string boundaries and inject additional commands. This is command injection at user privilege level.

#### Recommended Fix

1. Reuse a single escaping helper for AppleScript string literals (already present in `src/app_actions.rs:104`) across all AppleScript call sites.
2. For terminal `cd`, avoid single-quote shell interpolation entirely; prefer AppleScript concatenation with `quoted form of`.
3. Add regression tests using paths containing `'`, `"`, and backslashes.

---

### SA-MED-002: Arbitrary file write sink exposed via stdin `captureWindow`

- Severity: **Medium**
- Files:
  - `src/stdin_commands.rs:105`
  - `src/main.rs:3619`
  - `src/main.rs:3624`

#### Evidence

`ExternalCommand::CaptureWindow { title, path }` accepts an arbitrary `path` string from stdin and writes PNG bytes directly via `std::fs::write(&path, &png_data)`.

#### Risk

Any process controlling stdin can direct the app to write files to arbitrary user-writable locations. In environments where stdin is bridged/proxied unexpectedly, this expands impact from UI automation to filesystem mutation.

#### Recommended Fix

1. Canonicalize and enforce an allowlisted output root (for example `.test-screenshots/` in cwd or `~/.scriptkit/screenshots/`).
2. Reject symlink targets for this command.
3. Return structured error logs including rejected path and policy reason.

---

### SA-MED-003: Stdin protocol is a high-privilege unauthenticated control plane

- Severity: **Medium** (architecture/threat-model dependent)
- Files:
  - `src/stdin_commands.rs:46`
  - `src/main.rs:3037`
  - `src/main.rs:3076`
  - `src/prompt_handler.rs:510`
  - `src/prompt_handler.rs:540`

#### Evidence

The stdin protocol supports privileged operations, including:

- run arbitrary script path (`run`)
- key simulation (`simulateKey`, `simulateAiKey`)
- opening AI/notes windows
- executing fallbacks
- screenshot capture to arbitrary path

`run` flows to `PromptMessage::RunScript { path }` and executes the path directly via `execute_interactive` with no path/domain restriction.

#### Risk

If an untrusted component can send data into process stdin (or if deployment wrappers expose stdin to third parties), this becomes a full local automation and code-execution channel.

#### Recommended Fix

1. Gate stdin-command mode behind explicit startup opt-in (env flag/CLI flag) for non-test builds.
2. Require command authentication (bearer token / one-time nonce) when enabled.
3. Add capability scoping (example: `capture_only`, `ui_only`, `run_disabled`).

---

### SA-MED-004: Unbounded stdin line size enables memory DoS pressure

- Severity: **Medium**
- Files:
  - `src/stdin_commands.rs:179`
  - `src/stdin_commands.rs:183`

#### Evidence

Listener uses `BufRead::lines()` and parses each full line as JSON without size checks. Very large lines allocate proportionally before parse/validation.

#### Risk

Attacker controlling stdin can push oversized JSON lines causing high memory pressure or process instability.

#### Recommended Fix

1. Enforce max line length (example 64KB or 256KB) before parse.
2. Drop and log oversized payloads with safe truncated preview.
3. Add tests for oversized line rejection.

---

### SA-LOW-005: Raw stdin payloads are logged verbatim (potential secret leakage)

- Severity: **Low**
- Files:
  - `src/stdin_commands.rs:182`

#### Evidence

`logging::log("STDIN", &format!("Received: {}", line));` records full incoming JSON command content.

#### Risk

If future commands carry secrets/tokens, logs persist sensitive data. This contrasts with safer truncation behavior implemented in `src/protocol/io.rs`.

#### Recommended Fix

1. Replace full-line logging with bounded preview (same strategy as `protocol/io.rs`).
2. Consider field-level redaction for known sensitive keys.

---

### SA-LOW-006: Predictable temp filenames for scriptlet execution

- Severity: **Low**
- Files:
  - `src/executor/scriptlet.rs:239`
  - `src/executor/scriptlet.rs:375`
  - `src/executor/scriptlet.rs:446`
  - `src/app_impl.rs:5880`
  - `src/app_impl.rs:5940`

#### Evidence

Multiple temp files use deterministic names based on PID and optional scriptlet name.

#### Risk

Predictable names increase collision and local race/symlink attack surface in shared temp directories (especially same-user adversary scenarios).

#### Recommended Fix

1. Use `tempfile::NamedTempFile` / `Builder` with random suffix and exclusive create semantics.
2. Keep restrictive file permissions where possible.

---

### SA-LOW-007: Inconsistent AppleScript string escaping across modules

- Severity: **Low**
- Files:
  - `src/app_actions.rs:104`
  - `src/file_search.rs:912`
  - `src/file_search.rs:942`
  - `src/app_impl.rs:5659`
  - `src/app_impl.rs:5779`

#### Evidence

Different modules apply different escaping strategies (`escape_applescript_string` helper exists in one place, ad hoc replacement in others, none in vulnerable app_impl paths).

#### Risk

Inconsistent escaping creates regression risk and makes future injection bugs likely.

#### Recommended Fix

1. Introduce shared `escape_applescript_string` utility in a common module.
2. Refactor all AppleScript call sites to use it.
3. Add centralized tests for escaping edge cases.

## Positive Security Observations

- Primary script runner paths use `Command::new(...).arg(...)` rather than shell interpolation, reducing classic command injection risk (`src/executor/runner.rs`).
- Some AppleScript paths already attempt escaping (`src/app_actions.rs:104`, `src/file_search.rs:157`).
- Debug-only local command file watcher is gated with `#[cfg(debug_assertions)]` (`src/main.rs:2955`).

## Prioritized Remediation Plan

1. Fix SA-HIGH-001 immediately (shared AppleScript escaping + regression tests).
2. Constrain stdin filesystem write and add protocol gating/auth (SA-MED-002/003).
3. Add stdin payload size limits and log redaction (SA-MED-004, SA-LOW-005).
4. Migrate temp file creation to secure random/exclusive APIs (SA-LOW-006).

## Known Gaps / Assumptions

- This audit is static (code review + dataflow tracing), not dynamic fuzzing.
- Stdin threat severity depends on deployment model (trusted local automation vs exposed bridge).
- macOS-specific AppleScript behavior was evaluated from source construction semantics; exploitability should be validated with integration tests after patching.
