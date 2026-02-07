# Audit Report: `src/actions/builders/shared.rs` + `src/actions/builders_rewire.rs`

## Scope
- `src/actions/builders/shared.rs`
- `src/actions/builders_rewire.rs`
- Cross-check usage sites for `copy_deeplink` in builders (`script_context`, `scriptlet`, `notes`) and deeplink parser/runtime wiring.

## What I verified
1. URL encoding correctness
2. Special character handling
3. Empty string handling
4. Consistent usage across builders (including rewire exports)

## Findings (ordered by severity)

### 1) `to_deeplink_name` is slugifying but not URL-encoding; non-ASCII output is emitted directly into URL paths
- Severity: **Medium**
- Evidence:
  - `src/actions/builders/shared.rs:7` and `src/actions/builders/shared.rs:10` keep all Unicode alphanumeric chars (`is_alphanumeric`) and only replace non-alnum with `-`.
  - `src/actions/builders/script_context.rs:256` and `src/actions/builders/scriptlet.rs:191` interpolate output directly into `scriptkit://run/{...}`.
  - Runtime copy path does the same in `src/app_actions/handle_action.rs:567` and `src/app_actions/handle_action.rs:568`.
  - Deeplink parsing is raw string-prefix based (`src/main_sections/deeplink.rs:21` to `src/main_sections/deeplink.rs:31`) and does not percent-decode.
- Why this matters:
  - URLs containing Unicode may be percent-encoded by external clients (`caf%C3%A9`). Parser currently forwards encoded bytes into command IDs (`script/caf%C3%A9`), which can diverge from expected slug (`script/café`).
- Current behavior appears intentional in tests (e.g. `src/actions/dialog_cross_context_tests/part_03.rs:446` expects `"café"` to remain `"café"`), so this is a compatibility/robustness gap rather than an accidental regression.

### 2) Empty/symbol-only names produce an empty deeplink segment
- Severity: **Medium**
- Evidence:
  - Empty tokens are dropped and joined, returning `""` for empty/all-special input (`src/actions/builders/shared.rs:12` to `src/actions/builders/shared.rs:15`).
  - Builders then format `scriptkit://run/{deeplink_name}` (`src/actions/builders/script_context.rs:256`, `src/actions/builders/scriptlet.rs:191`), yielding `scriptkit://run/` when slug is empty.
  - Parser maps this to `script/` (`src/main_sections/deeplink.rs:29` to `src/main_sections/deeplink.rs:31`), which is not a meaningful command target.
- Existing tests explicitly assert this empty return contract (`src/actions/dialog_cross_context_tests/part_04.rs:9`, `src/actions/dialog_cross_context_tests/part_04.rs:14`, `src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_15/tests_part_01.rs:53`).

### 3) Builder usage consistency is good; `builders_rewire` exports are aligned
- Severity: **Info**
- Evidence:
  - `src/actions/builders_rewire.rs:29` re-exports `shared::to_deeplink_name`, mirroring legacy `builders.rs` shape.
  - Active module wiring uses rewire (`src/actions/mod.rs:17` and `src/actions/mod.rs:30`).
  - Script and scriptlet builders both call shared utility before building deeplink descriptions (`src/actions/builders/script_context.rs:250`, `src/actions/builders/scriptlet.rs:185`).
  - Notes builder uses `copy_deeplink` for note deeplinks with a separate `scriptkit://notes/{id}` path (not `run/{slug}`), which is domain-appropriate (`src/actions/builders/notes.rs:113`, runtime in `src/notes/window/notes_actions.rs:86`).

## Verification run
- Attempted scoped test run:
  - `cargo test --lib to_deeplink_name`
- Result:
  - **Blocked by unrelated in-flight tree breakage** (many pre-existing compile errors outside this audit scope), including:
    - inner doc attribute placement errors in multiple `src/actions/tests/...` files
    - missing symbols/imports in unrelated test rewires
    - unrelated missing `BUTTON_RADIUS_PX` import in `src/components/prompt_header/component.rs`

## Change summary
- No production code changes were made in audited source files.
- Deliverable produced: this findings log.
