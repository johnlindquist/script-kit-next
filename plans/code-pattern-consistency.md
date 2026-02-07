# Code Pattern Consistency Audit (`src/**/*.rs`)

Date: 2026-02-07  
Agent: `codex-code-patterns`

## Scope and Method

- Scope audited: all Rust sources under `src/**/*.rs`.
- Method: grep-driven scan for repeated patterns and mismatches in logging, error handling, return-type semantics, naming/organization, and comment-to-code drift.
- Goal: identify inconsistency patterns that increase maintenance cost or create behavior drift between similar modules.

## Executive Summary

- The codebase has strong local patterns inside some subsystems (for example, `notes`/`ai` storage), but cross-module consistency is uneven.
- Highest impact issues are:
1. Mixed logging APIs and logging styles in similar execution paths.
2. Fragmented error model (`anyhow`, `String`, boxed dynamic errors) for comparable operations.
3. Duplicated accessibility helper stacks with behavior drift.
4. Runtime `unwrap` usage in non-test paths.
- There are also lower-risk but high-friction issues: stale comments/constants, naming artifacts, and test-module fragmentation.

## Findings (Ordered by Impact)

### 1. Logging style is inconsistent across comparable runtime paths

Evidence:

- Legacy string-category logging API is still heavily used in prompt handling:
  - `src/prompt_handler.rs:18`
  - `src/prompt_handler.rs:28`
  - `src/prompt_handler.rs:469`
  - `src/prompt_handler.rs:511`
- Structured `tracing` fields are used in other UI/runtime modules:
  - `src/ai/window.rs:1548`
  - `src/ai/window.rs:1567`
  - `src/app_execute.rs:1505`
- Unstructured `tracing` with string interpolation appears in similar code paths:
  - `src/app_impl.rs:431`
  - `src/app_impl.rs:2436`

Why this is inconsistent:

- Similar operational events are logged through different APIs and field styles.
- Some logs are typed (`error = %e`, `chat_id = %chat_id`) while others flatten context into strings.

Impact:

- Harder queryability in JSON logs.
- More difficult correlation across modules when events are not field-structured consistently.

Recommendation:

1. Standardize on `tracing` macros with typed fields for all new/modified runtime code.
2. Keep `logging::log` only as a compatibility shim at boundaries (if needed), not as a primary API.
3. Add a style note in `AGENTS.md` for preferred field-based logging forms.

---

### 2. Error type strategy is fragmented (`anyhow`, `String`, boxed dyn errors)

Evidence:

- `anyhow::Result` + context in storage modules:
  - `src/notes/storage.rs:6`
  - `src/notes/storage.rs:42`
  - `src/ai/storage.rs:6`
  - `src/ai/storage.rs:42`
- Extensive `Result<_, String>` APIs in system/executor modules:
  - `src/system_actions.rs:23`
  - `src/system_actions.rs:77`
  - `src/executor/runner.rs:564`
  - `src/executor/runner.rs:663`
  - `src/executor/runner.rs:865`
- Boxed dynamic error return types in platform helpers:
  - `src/platform.rs:3125`

Why this is inconsistent:

- Similar boundary-level operations return materially different error types.
- Upstream callers lose structured context or must normalize manually.

Impact:

- Error propagation and telemetry are uneven.
- Harder to classify and match failures across modules.

Recommendation:

1. Define per-domain typed errors (`thiserror`) where callers branch on error class.
2. Use `anyhow::Result` at application boundaries that only need propagation + context.
3. Avoid `Result<T, String>` in new code; migrate incrementally via adapter conversions.

---

### 3. `Option` vs `Result` semantics are inconsistent for parsing flows

Evidence:

- Agent frontmatter parser suppresses parsing failures via `Option`:
  - `src/agents/parser.rs:39`
  - `src/agents/parser.rs:57`
  - `src/agents/parser.rs:284`
- Other parsers preserve parse failure details:
  - `src/metadata_parser.rs:166`
  - `src/schema_parser.rs:206`
  - `src/scriptlet_metadata.rs:39`

Why this is inconsistent:

- Invalid input in one parser can silently collapse to defaults, while others return explicit parse errors.

Impact:

- Debugging malformed frontmatter is harder.
- Silent fallback behavior can hide data-quality issues.

Recommendation:

1. Convert `parse_frontmatter` to return `Result<Option<...>, ParseError>` (or equivalent).
2. Reserve `Option` for true absence; use `Result` for malformed/invalid content.
3. Align parser contracts in docs and tests.

---

### 4. Accessibility helper logic is duplicated with behavior drift

Evidence:

- Repeated CoreFoundation/AX helper implementations in:
  - `src/window_control.rs:297`
  - `src/menu_executor.rs:268`
  - `src/menu_bar.rs:275`
  - `src/window_control_enhanced/capabilities.rs:137`
- Drift in action error handling:
  - `src/window_control.rs:337` handles fewer AX error variants.
  - `src/menu_executor.rs:275` includes `kAXErrorActionUnsupported` and `kAXErrorCannotComplete`.
- Repeated `CString::new(...).unwrap()` patterns:
  - `src/window_control.rs:242`
  - `src/menu_executor.rs:165`
  - `src/menu_bar.rs:277`
  - `src/window_control_enhanced/capabilities.rs:139`

Why this is inconsistent:

- Same conceptual helper behavior differs by module copy, not policy.

Impact:

- Fixes in one path do not propagate.
- User-visible error quality and diagnosability vary by call site.

Recommendation:

1. Extract AX helper primitives into one shared module.
2. Define a single `AxError` mapping table and reuse it.
3. Replace `unwrap` in helper construction with fallible conversion paths.

---

### 5. Constant/comment drift causes contradictory local documentation

Evidence:

- Actual list item height constant:
  - `src/list_item.rs:32` (`LIST_ITEM_HEIGHT: 40.0`)
- Stale comments referencing 48px:
  - `src/list_item.rs:1507`
  - `src/render_script_list.rs:196`
  - `src/render_script_list.rs:276`
  - `src/render_script_list.rs:411`
- Action height is 36px, but test comment says 44px:
  - `src/actions/constants.rs:13`
  - `src/actions/constants.rs:65`

Why this is inconsistent:

- Explanatory text no longer matches executable constants.

Impact:

- Incorrect mental models for future changes.
- Increased risk of introducing regressions when tuning layout or scroll behavior.

Recommendation:

1. Fix stale comments to match current constants.
2. Prefer comments that derive from named constants, not hardcoded pixel numbers.

---

### 6. Naming/organization patterns show residue and fragmentation

Evidence:

- Backup artifact in source tree:
  - `src/term_prompt.rs.orig`
- Test export naming workaround:
  - `src/prompts/chat.rs:3384` (`next_reveal_boundary_pub`)
  - `src/prompts/markdown.rs:1340`
- Highly fragmented generated-style test modules:
  - `src/actions/mod.rs:97`
  - `src/actions/mod.rs:101`
  - `src/actions/mod.rs:105`
  - Total files matching `src/actions/dialog_builtin_action_validation_tests*.rs`: 46

Why this is inconsistent:

- Source tree includes non-source artifact (`.orig`).
- Test access strategy uses ad-hoc naming rather than a stable shared test helper location.
- Many numbered test files reduce discoverability.

Impact:

- Grepability and onboarding cost increase.
- Harder to reason about ownership and logical grouping.

Recommendation:

1. Remove `.orig` residue from `src/`.
2. Move shared test-only utilities into a dedicated `#[cfg(test)]` helper module.
3. Consolidate numbered test files into scenario-grouped modules where feasible.

---

### 7. Runtime `unwrap`/`expect` usage is inconsistent with project guidance

Evidence:

- Runtime-path `unwrap` in `scriptlets` parsing/templating:
  - `src/scriptlets.rs:513`
  - `src/scriptlets.rs:632`
  - `src/scriptlets.rs:1339`
  - `src/scriptlets.rs:1461`
- Runtime helper `unwrap` in platform/accessibility code:
  - `src/platform.rs:999`
  - `src/window_control.rs:242`
  - `src/window_control_enhanced/capabilities.rs:139`

Why this is inconsistent:

- Repository guidance explicitly discourages `unwrap`/`expect` in production paths, but several remain.

Impact:

- Panic risk on malformed or unexpected runtime input.

Recommendation:

1. Replace runtime `unwrap`/`expect` with `Result` propagation + context.
2. Keep `unwrap` only in tests/benchmarks where failure is intentional.

## Suggested Remediation Order

1. Logging unification (`tracing` field-based standard + compatibility shim boundaries).
2. Shared AX helper extraction and error mapping normalization.
3. Parser API contract alignment (`Option` vs `Result`).
4. Runtime `unwrap` elimination in high-surface modules (`scriptlets`, AX helpers).
5. Comment/constant cleanup and naming hygiene pass.

## Verification Hooks to Add

- Add lint checks or CI grep guardrails for:
  - new `Result<_, String>` in `src/` (allowlist legacy files during migration),
  - new runtime `unwrap`/`expect` in non-test modules,
  - stale numeric literals in comments for key UI constants.
- Add parser contract tests that distinguish:
  - missing data (None),
  - malformed data (Err),
  - valid data (Ok).

## Known Gaps in This Audit

- This report focuses on Rust source consistency only (`src/**/*.rs`), not TypeScript SDK/test files.
- It identifies pattern mismatches and migration strategy; it does not include code refactors in this task.
