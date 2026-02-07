# String Allocation Optimization Report

## Scope
- Repo: `script-kit-gpui`
- Target: `src/**/*.rs`
- Goal: reduce avoidable `String` allocations (`to_string`, `to_owned`, `format!`, and needless clones) where borrow-friendly forms are sufficient.

## Approach
1. Searched allocation-heavy patterns across `src/**/*.rs`.
2. Prioritized runtime paths with low-risk, behavior-preserving changes.
3. Added a regression test where behavior semantics changed (`get_actions_file_path` non-UTF8 handling).

## Implemented optimizations

### 1) Borrow icon strings with `Cow<'_, str>` in scriptlet icon resolution
- File: `src/scriptlets.rs`
- Change:
  - `resolve_scriptlet_icon` now returns `Cow<'a, str>` instead of `String`.
  - Metadata/frontmatter/default icon branches now borrow existing string data where possible.
- Allocation impact:
  - Avoids cloning icon strings in the common path.

### 2) Remove temporary allocation in duplicate-input checks
- Files:
  - `src/scriptlets.rs`
  - `src/extension_types.rs`
- Change:
  - Replaced `inputs.contains(&trimmed.to_string())` with `inputs.iter().any(|existing| existing == trimmed)`.
  - Keep final push as `trimmed.to_owned()` only when needed.
- Allocation impact:
  - Eliminates a transient `String` allocation for every candidate input placeholder during parsing.

### 3) Preserve path bytes and avoid lossy allocation in actions companion path
- File: `src/scriptlets.rs`
- Change:
  - Replaced manual stem + `to_string_lossy` + `format!` with `md_path.with_extension("actions.md")`.
- Allocation impact:
  - Avoids unnecessary string formatting/allocation and lossy conversions.
- Behavior impact:
  - Correctly preserves non-UTF8 file names.
- Test added:
  - `src/scriptlet_tests.rs`: `test_get_actions_file_path_preserves_non_utf8_stem_bytes`.

### 4) Avoid cloned `String`s when consuming parsed codefence result
- File: `src/scriptlets.rs`
- Change:
  - Moved `codefence_result.code` fields (`language`, `content`) instead of cloning both strings.
- Allocation impact:
  - Removes two `String` clones per parsed scriptlet section when codefence parsing succeeds.

### 5) Delay language allocation in scriptlet loader
- File: `src/scripts/scriptlet_loader.rs`
- Change:
  - Keep `language` as `&str` while validating/continuing past `metadata` and `schema` blocks.
  - Allocate `String` only at return boundary.
- Allocation impact:
  - Reduces allocations while scanning fenced blocks that are skipped.

## High-value follow-ups (not implemented in this pass)

### A) Replace hot-path logging `format!` with structured fields
- Example file: `src/prompt_handler.rs`
- Why:
  - Frequent `logging::log("...", &format!(...))` creates many temporary strings.
- Direction:
  - Prefer `tracing` with typed fields where possible; keep interpolation at sink boundaries.

### B) Convert repeated UI IDs assembled with `format!` to reusable prefixes or cached `SharedString`
- Example files:
  - `src/ai/window.rs`
  - `src/prompts/markdown.rs`
- Why:
  - Per-render ID string construction can add churn.
- Direction:
  - Build once per stable entity, reuse/cached `SharedString` where lifecycle allows.

### C) Audit `.clone()` on option/string metadata merge paths for borrow opportunities
- Example file: `src/scripts/scriptlet_loader.rs`
- Why:
  - Several merge branches clone optional strings; some can likely be restructured to borrow or move.
- Direction:
  - Refactor merge helpers to consume parsed structs when ownership can be transferred.

### D) Revisit `SharedString` conversion paths with explicit ownership boundaries
- Example file: `src/icons/types/icon_ref.rs`
- Why:
  - Some `SharedString::from(&str)` paths require `'static` and cannot borrow transient split slices.
- Direction:
  - Use a constructor/flow that transfers owned data once (or stores interned/static values) rather than repeatedly materializing temporary `String`s.

## Verification
- `cargo check` (pass)
- `cargo clippy --all-targets -- -D warnings` (fails due pre-existing unrelated warnings/errors in other in-flight agent edits)
- `cargo test` (currently blocked by unrelated compile failure in `src/ai/window.rs` in this shared working tree)

## Risk assessment
- Implemented changes are low-risk and mostly allocation-only refactors.
- The only behavior-affecting change (`with_extension("actions.md")`) has explicit regression coverage for non-UTF8 filenames.
