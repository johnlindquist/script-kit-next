Implemented the fix in `src/actions/command_bar/part_01.rs`.

**Changed files**
- `src/actions/command_bar/part_01.rs:59`
- `src/actions/command_bar/part_01.rs:77`
- `src/actions/command_bar/part_01.rs:261`

**What changed**
- Added explicit named-key handling before the `TypeChar` fallback:
  - `space` now maps to `TypeChar(' ')`.
  - Non-text named keys (`tab`, arrow left/right names, modifier/lock keys, etc.) now return `None` so they do not fall through as typed chars.
- Kept `backspace`/`delete` behavior as `Backspace`.
- Added regression test `test_command_bar_key_intent_does_not_treat_named_keys_as_typed_chars`.

**How to test**
1. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. Once workspace-level errors are resolved, run focused regression:
   - `cargo test command_bar_key_intent_does_not_treat_named_keys_as_typed_chars`

**Verification results**
- I ran `cargo check` and `cargo clippy --all-targets -- -D warnings`.
- Both currently fail due unrelated existing repo issues (outside this file), including:
  - duplicate test module file collisions (`E0761`)
  - unrelated `ClipboardEntry` field mismatches (`E0063`)
  - unrelated non-exhaustive `ContentType` matches (`E0004`)
  - unrelated broad test/import errors in other modules

**Risks / known gaps**
- Non-text named keys are handled via an explicit list; additional named keys not in that list could still require follow-up handling.