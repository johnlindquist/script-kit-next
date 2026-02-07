# Actions/Dialog System Audit (`src/actions/**/*.rs`)

## Scope Reviewed
- Core implementation: `src/actions/dialog.rs`, `src/actions/builders.rs`, `src/actions/types.rs`
- Dialog-focused tests: all `src/actions/dialog*tests*.rs` modules referenced from `src/actions/mod.rs`

## Executive Summary
The actions/dialog subsystem is feature-rich and heavily tested, but several structural inconsistencies can produce ambiguous routing and stale UI state in edge cases. The highest-risk issues are runtime config updates that do not rebuild grouped rows, constructor inconsistency around initial selection, and ambiguous SDK action lookup by non-unique name.

## Findings

### 1) Runtime `set_config` can leave stale grouped rows and selection
- Severity: High
- Evidence:
  - `set_config` mutates config and `hide_search` only in `src/actions/dialog.rs:551`.
  - Grouping depends on `self.config.section_style` in `src/actions/dialog.rs:987` and `src/actions/dialog.rs:991`.
  - No rebuild/selection coercion occurs in `set_config`.
- Impact:
  - If section style changes (e.g. `Separators` -> `Headers`) after creation, rendered row model can remain stale until another path triggers `refilter`/`rebuild_grouped_items`.
  - `selected_index` may no longer map to an item row after style changes.
- Recommendation:
  - Make `set_config` rebuild grouped rows when relevant fields change (`section_style`, potentially `show_icons` if row shape changes).
  - Re-run selection coercion (`coerce_action_selection`) after rebuild.

### 2) Constructors inconsistently initialize selection (header-skip only in one path)
- Severity: High
- Evidence:
  - `with_config` uses `coerce_action_selection` in `src/actions/dialog.rs:598`.
  - Other constructors set `selected_index: 0` directly (`src/actions/dialog.rs:301`, `src/actions/dialog.rs:349`, `src/actions/dialog.rs:404`, `src/actions/dialog.rs:456`, `src/actions/dialog.rs:515`).
- Impact:
  - Initial selection behavior differs by constructor.
  - If grouped rows begin with a header (or future style defaults change), selection can start on non-selectable rows.
- Recommendation:
  - Centralize construction through one initializer that always computes initial selection via `coerce_action_selection`.

### 3) SDK action routing is ambiguous when action names collide
- Severity: High
- Evidence:
  - SDK conversion sets internal id to `pa.name` in `src/actions/dialog.rs:674`.
  - Selected protocol action resolves via first `name == action_id` match in `src/actions/dialog.rs:1147`.
- Impact:
  - Duplicate SDK names can map the selected UI row to the wrong original `ProtocolAction`.
  - `selected_action_should_close()` may apply incorrect close behavior (`src/actions/dialog.rs:1153`).
- Recommendation:
  - Introduce a stable protocol action identifier (e.g. `id`) or maintain index mapping from rendered action to original protocol action.
  - Add validation/logging when duplicate SDK names are supplied.

### 4) Script/agent flag overlap allows duplicate/conflicting action IDs
- Severity: Medium
- Evidence:
  - `get_script_context_actions` applies independent branches for `is_script`, `is_scriptlet`, `is_agent` (`src/actions/builders.rs:657`, `src/actions/builders.rs:715`, `src/actions/builders.rs:762`).
  - Duplicate `edit_script` entries can be produced in mixed-flag cases (documented by test in `src/actions/dialog_cross_context_tests.rs:543` to `src/actions/dialog_cross_context_tests.rs:562`).
- Impact:
  - Duplicate IDs can create ambiguous routing semantics and confusing UI labels.
- Recommendation:
  - Encode item kind as a single enum instead of independent booleans.
  - Short-term: enforce precedence (`if/else if`) or dedupe by ID at the end of builder assembly.

### 5) Shortcut formatting logic is duplicated and diverges between builders and dialog
- Severity: Medium
- Evidence:
  - Builder-local formatter: `src/actions/builders.rs:276` (simple replacement).
  - Dialog formatter: `src/actions/dialog.rs:709` (supports aliases/special keys/arrows).
- Impact:
  - Same logical shortcut may render differently depending on action source.
  - Builder path misses aliases like `command/meta/super`, arrow names, etc.
- Recommendation:
  - Move formatting to one shared function/module used by both builder and dialog conversion paths.

### 6) Grouping logic and comments diverge; category tracking is dead code
- Severity: Medium
- Evidence:
  - `build_grouped_items_static` tracks `prev_category` for separator mode (`src/actions/dialog.rs:153` to `src/actions/dialog.rs:157`), then discards it (`src/actions/dialog.rs:164`).
  - Separators are actually determined by `section` changes (`src/actions/dialog.rs:173` to `src/actions/dialog.rs:190`).
- Impact:
  - Code intent is unclear; future contributors can incorrectly assume category-based grouping exists.
- Recommendation:
  - Remove unused category tracking or implement category-based separator behavior consistently.
  - Align comments with actual grouping behavior.

### 7) Missing/underused action type model for global ops
- Severity: Medium
- Evidence:
  - `ActionCategory::ScriptOps`/`GlobalOps` are marked reserved in `src/actions/types.rs:433` to `src/actions/types.rs:452`.
  - `get_global_actions()` returns empty in `src/actions/builders.rs:852`.
  - `build_actions` still extends with global actions in `src/actions/dialog.rs:832`.
- Impact:
  - Framework implies cross-context/global routing but currently has no concrete built-in actions.
  - Adds conceptual overhead without behavior.
- Recommendation:
  - Either implement a minimal global action set or remove/feature-flag the unused category path until needed.

### 8) Validation test architecture is oversized and repetitive
- Severity: Medium
- Evidence:
  - Dialog test modules: 53 files total, 46 sharded `dialog_builtin_action_validation_tests*` files, ~80,561 LOC.
  - Sharded modules are all wired individually in `src/actions/mod.rs:97` through `src/actions/mod.rs:278`.
  - Multiple tests duplicate internal logic instead of asserting public behavior:
    - Reimplemented section counter in `src/actions/dialog_validation_tests.rs:46`.
    - Reimplemented clipboard-title truncation in `src/actions/dialog_validation_tests.rs:1760`.
- Impact:
  - High maintenance cost and slower iteration for behavior changes.
  - Copying implementation logic into tests weakens regression detection.
- Recommendation:
  - Consolidate repeated invariants into table-driven tests and helper macros.
  - Prefer public API tests (e.g. `ActionsDialog::with_clipboard_entry`) over reimplementing production logic in test code.

## Builder Pattern Edge Cases (Specific)
- `get_scriptlet_defined_actions` hardcodes section to `"Actions"` (`src/actions/builders.rs:325`), limiting future action taxonomy for scriptlet-defined actions.
- Shortcut/alias dynamic branches are duplicated across script and scriptlet builders (`src/actions/builders.rs:578` to `src/actions/builders.rs:654`, `src/actions/builders.rs:387` to `src/actions/builders.rs:455`), increasing drift risk.

## Simplification Plan (Low-Risk Order)
1. Unify constructor initialization through a single `from_actions(...)` helper (selection coercion + grouped build in one place).
2. Introduce one shared shortcut formatter module and replace both current implementations.
3. Normalize script kind modeling (enum over booleans) or enforce deterministic precedence.
4. Add stable protocol action identity for SDK actions and guard/log duplicates.
5. Collapse sharded validation files into fewer table-driven suites focused on public behavior.

## Suggested Missing Tests
- `set_config` section-style transition rebuilds grouped rows and keeps valid selection.
- SDK duplicate names produce deterministic mapping (or explicit validation error/log).
- Constructor parity test: all constructors initialize selection identically when first grouped row is a header.
- Mixed script flags should be either rejected or deterministically resolved without duplicate IDs.
