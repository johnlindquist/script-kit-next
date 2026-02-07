Implemented the requested audit report at `crates-audits/accessibility.md`.

**Changed files**
- `crates-audits/accessibility.md`

**What I verified**
1. Ran selected-text smoke flow via stdin JSON protocol:
   - `tests/smoke/test-get-selected-text.ts`
   - Observed `GetSelectedText request` and `GetSelectedText success: 0 chars`
2. Ran permission + selected-text smoke flow:
   - `tests/smoke/test-accessibility-check.ts`
   - Observed `CheckAccessibility: granted=true`, `GetSelectedText success: 0 chars`, and HUD `No text selected`
3. Ran `cargo check`
   - Fails in current shared tree due unrelated in-progress changes in `src/hotkeys.rs` (`Option<HotkeyConfig>` type mismatches), not from this audit file.

**Key risks/known gaps captured in the report**
- `get-selected-text` hybrid approach is only partially reliable due per-app cached method behavior (cached AX/clipboard path does not retry the other strategy on later failures).
- No-focused-app case is surfaced as an error (`No active window found`) rather than a graceful empty-selection outcome.
- Local docs in `src/selected_text.rs` mention AX range fallback that current crate code does not implement.

**Commits**
- None.