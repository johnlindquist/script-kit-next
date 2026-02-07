# Crypto + Secrets Security Audit

## Scope
- Repository: `script-kit-gpui`
- Audit date: 2026-02-07
- Requested crates: `age` (`0.11`), `hostname` (`0.4`), `hex` (`0.4.3`)
- Files reviewed: `Cargo.toml`, `src/secrets.rs`, `src/ai/config.rs`, `src/prompts/env.rs`, `src/ai/window.rs`, `src/clipboard_history/blob_store.rs`, and related callsites.

## Dependency Baseline
- `age = "0.11"` (`Cargo.toml:94`)
- `hostname = "0.4"` (`Cargo.toml:95`)
- `hex = "0.4.3"` (`Cargo.toml:134`)

## Executive Summary
- `age` integration is functionally correct (passphrase-mode encrypt/decrypt, per-file random salt, restrictive `0600` perms on Unix), but the passphrase source is weak and predictable.
- Current passphrase derivation (`hostname + constant`) does **not** provide strong secrecy against offline attack if `secrets.age` is exfiltrated.
- A decryption failure currently degrades to empty secrets and can be overwritten on next write, causing silent secret loss.
- API keys can bypass age encryption by design when sourced from environment variables (`SCRIPT_KIT_*_API_KEY`), and UI text encourages plaintext shell/env-file storage.
- `hex` encoding itself is used correctly for SHA-256 digest output, but blob hash consumers do not validate hex format before filesystem path construction.

## Findings (Severity Ordered)

### 1) High: `age` passphrase is low-entropy and predictable (`hostname` + constant)
- Evidence:
  - Passphrase derivation is deterministic and based on hostname: `src/secrets.rs:125`.
  - Fallback is static `"unknown-host"`: `src/secrets.rs:128`.
  - Final passphrase format is `"{hostname}:com.scriptkit.secrets"`: `src/secrets.rs:131`.
- Impact:
  - Hostnames are usually guessable/observable.
  - If an attacker gets `~/.scriptkit/secrets.age`, they can run offline guesses against a very small candidate set compared to a random secret.
  - `age`+scrypt slows brute-force but cannot compensate for low passphrase entropy.
- Recommendation:
  - Replace hostname-derived passphrase with a high-entropy installation secret (random 32+ bytes) stored with strict permissions (or OS secure storage), then feed that into `age` passphrase mode.
  - If machine binding is required, combine machine signal + random install secret, not machine signal alone.

### 2) High: Decrypt/read errors return empty map, enabling silent overwrite/loss
- Evidence:
  - Multiple decrypt/read/parse failures return `HashMap::new()`: `src/secrets.rs:149`, `src/secrets.rs:164`, `src/secrets.rs:179`, `src/secrets.rs:185`, `src/secrets.rs:220`.
  - `set_secret` writes whatever is in memory cache (possibly empty after failure): `src/secrets.rs:346`-`src/secrets.rs:356`.
- Impact:
  - If hostname changes or decryption fails transiently, a later `set_secret` can rewrite the store with only the new key(s), effectively deleting prior secrets.
  - This is primarily integrity/availability risk, but it also masks crypto failures.
- Recommendation:
  - Introduce explicit error state for load/decrypt failure; block writes until successful decrypt or explicit user recovery/migration.
  - Use atomic write (`tempfile + fsync + rename`) and backup strategy to reduce corruption/loss risk.

### 3) Medium: Plaintext secret path bypasses age by design (env vars are preferred)
- Evidence:
  - Lookup order is environment first, then encrypted store: `src/ai/config.rs:187`-`src/ai/config.rs:202`.
  - User guidance suggests `~/.zshrc` or `~/.scriptkit/.env` for API keys: `src/ai/window.rs:348`.
- Impact:
  - Keys may reside in plaintext config files and process environments outside age-protected storage.
  - This is expected behavior, but it directly answers “secrets bypassing age”: **yes, when users configure env vars**.
- Recommendation:
  - Prefer secrets store guidance first in UX copy.
  - If env vars are kept as supported fallback, explicitly label them as less secure than encrypted store.

### 4) Medium: `hex` digest consumers accept unvalidated hash strings for file paths
- Evidence:
  - Hashes are generated as hex via `hex::encode`: `src/clipboard_history/blob_store.rs:30`.
  - On read/delete, any `blob:<suffix>` is accepted and used in path join without hex validation: `src/clipboard_history/blob_store.rs:59`, `src/clipboard_history/blob_store.rs:69`, `src/clipboard_history/blob_store.rs:89`, `src/clipboard_history/blob_store.rs:98`, `src/clipboard_history/blob_store.rs:118`.
- Impact:
  - If attacker-controlled `content` reaches blob APIs, path traversal strings could be interpreted as filenames (e.g., `../...`).
  - This is not a break in `hex` itself, but a missing input validation step where hex is expected.
- Recommendation:
  - Enforce strict blob hash validation (`len == 64` and ASCII hex only, or `hex::decode` round-trip) before any filesystem operation.
  - Reject invalid blob references early and log structured warnings.

### 5) Low: Some prompt submission paths log raw submitted values
- Evidence:
  - Force-submit logs full value: `src/prompt_handler.rs:1103`, `src/prompt_handler.rs:1127`.
  - Generic prompt response path logs `value` verbatim: `src/app_impl.rs:7090`.
  - Prompt listener logs full incoming prompt message debug payload: `src/execute_script.rs:160`.
- Impact:
  - Potential plaintext exposure in logs for sensitive prompt content.
  - Not specific to `age`, but relevant to secrets hygiene.
- Recommendation:
  - Redact/omit submitted values in logs by default; allow opt-in debug redaction bypass only for local dev.

## Direct Answers to Requested Questions

1. Is `age` encryption used correctly for secrets storage?
- **Partially.** API usage pattern is correct (`with_user_passphrase`, `scrypt::Identity`, file mode `0600` on Unix), but the passphrase source is weak and predictable, which materially weakens security.

2. Is the scrypt passphrase derived securely from hostname?
- **No.** Hostname-based derivation is not high entropy and includes a static fallback (`unknown-host`), making offline guessing practical relative to random passphrases.

3. Are there timing attacks?
- **No meaningful remote timing attack identified** in this audited surface. Most operations are local file/decrypt and hash-map lookups. The dominant risk is weak passphrase entropy, not timing leakage.

4. Is hex encoding used properly?
- **Encoding yes; validation no.** `hex::encode` for SHA-256 output is correct. Consumers that expect hex hashes do not validate input before path usage.

5. Any secrets bypass age and are stored in plaintext?
- **Yes, by design via environment variables.** API key discovery prioritizes `SCRIPT_KIT_*_API_KEY` env vars over encrypted store (`src/ai/config.rs:187`), and current guidance explicitly suggests plaintext shell/env-file locations (`src/ai/window.rs:348`).

## Priority Remediation Plan
1. Replace hostname-derived passphrase with a high-entropy install secret and migration path.
2. Make decrypt failure a hard error for writes (no implicit empty-map fallback), and add atomic write/backup semantics.
3. Add strict blob hash validation at all blob path entry points.
4. Adjust AI setup/user guidance to prefer encrypted secrets store over plaintext env files.
5. Redact sensitive prompt submission values in logs.

## Verification Commands Used
- `rg -n "\\bage\\b|hostname|hex::|scrypt|Passphrase|encrypt|decrypt" src Cargo.toml`
- `rg -n "get_secret\\(|set_secret\\(|SCRIPT_KIT_[A-Z_]*API_KEY|secrets.age" src Cargo.toml`
- `sed -n` / `nl -ba` on:
  - `src/secrets.rs`
  - `src/ai/config.rs`
  - `src/ai/window.rs`
  - `src/clipboard_history/blob_store.rs`
  - related callsites in `src/prompt_handler.rs`, `src/app_impl.rs`, `src/execute_script.rs`
