# xorloop Report — 20260213-081630

**Project:** script-kit-gpui
**Branch:** main
**Started:** Fri Feb 13 08:16:30 MST 2026

---

## Iteration 1 — security fix (08:21)

**Feature:** FEATURE: The "run-in-terminal" fallback passes user input through AppleScript string interpolation with incorrect escaping order, enabling AppleScript injection via crafted input

Here's the extracted fix list:

- [critical] src/main_sections/fallbacks.rs::execute_fallback_action — Wrong escape order enables AppleScript injection from search bar
- [major] src/file_search/mod.rs::open_in_terminal — Redundant `quoted form of` on already-escaped AppleScript string
- [major] src/scriptlets/mod.rs::substitute_args — No escaping on individual `$N`/`%N` shell argument substitution
- [minor] src/executor/scriptlet.rs::execute_type — Inline AppleScript escaping instead of shared `utils` helper
- [minor] src/file_search/mod.rs::escape_applescript — Duplicate escape function should use canonical `utils` helper


---

## Iteration 2 — security fix (08:42)

**Feature:** FEATURE: Secrets encryption using a passphrase derived solely from the machine hostname, which is predictable and publicly discoverable, making the encrypted secrets file trivially decryptable by any local user or anyone who copies the file

Here are the fixes extracted from the analysis:

- **[critical]** `src/secrets.rs::derive_passphrase` — Replace hostname+constant passphrase with Keychain-stored random key
- **[major]** `src/secrets.rs::secrets_path` — Replace `.expect()` with `Option<PathBuf>` return type
- **[major]** `src/secrets.rs::get_cached_secrets` — Recover from poisoned mutex instead of panicking via `.expect()`
- **[major]** `src/secrets.rs::update_cache` — Recover from poisoned mutex instead of panicking via `.expect()`
- **[major]** `src/secrets.rs::load_secrets_from_disk` — Add 10MB size limit before decryption to prevent OOM


---

## Summary

**Completed:** Fri Feb 13 08:43:17 MST 2026
**Iterations:** 2
**Status:** signal
