Implemented the scoped header/footer P1 fixes and replaced brittle footer tests with behavior contracts.

**Changed Files**
- `src/components/prompt_header.rs:19`
  - Added `HeaderActionsDensity` with explicit width policy (`Compact|Normal|Expanded`) and wired it into header action slot sizing.
  - Added `HEADER_PATH_PREFIX_MAX_WIDTH_PX` and truncation/ellipsis on `path_prefix` so long prefixes don’t crowd query text.
  - Made actions-mode search input width follow density policy.
- `src/components/prompt_footer.rs:39`
  - Added footer layout width caps: `PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX` and `PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX`.
  - Added `footer_surface_rgba(...)` and switched render background to use `PromptFooterColors.background` (fixes footer color ownership inconsistency).
  - Added `primary_disabled` / `secondary_disabled` to `PromptFooterConfig` and passed disabled state into `FooterButton`.
  - Added truncation/ellipsis for helper and info labels and made left footer content shrink safely.
- `tests/prompt_footer.rs:13`
  - Replaced source-string assertions with behavior tests for:
    - footer disabled-state config defaults/builders,
    - footer surface color resolution,
    - footer max-width contracts,
    - header action-density width policy,
    - header path-prefix width contract.

**How To Test**
1. `cargo test --test prompt_footer`
2. `cargo check`
3. UI sanity (stdin protocol):  
   `timeout 8 bash -lc 'echo "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`

**Verification Run**
- `cargo test --test prompt_footer` passed (6/6).
- `cargo check` passed.
- Stdin `show` launch ran successfully; logs showed no panic/error lines during startup/show flow.

**Risks / Known Gaps**
- Full-suite `cargo test` and `cargo clippy --all-targets -- -D warnings` were not run in this scoped pass.
- New footer disabled fields are available but not yet wired by all prompt call sites (defaults preserve existing behavior).
- Width constants are conservative defaults and may need tuning with visual regression passes on narrow windows.

**Commits**
- No commits made.