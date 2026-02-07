# Form Validation & UX Analysis

## Scope Reviewed
- `src/components/form_fields.rs`
- `src/components/form_fields_tests.rs`
- `src/render_prompts/form.rs`

## Executive Summary
The current form prompt path is usable for basic text entry, but there are important correctness and UX gaps:
- Submission is not validation-gated.
- Textarea behavior conflicts with form-level Enter handling.
- Long forms and long textarea content can be clipped with no scroll path.
- Field-type semantics (`email`, `number`, `password`, checkbox values) are mostly cosmetic.
- Test coverage does not exercise runtime form behavior.

## Findings (Ordered by Severity)

### 1) `Enter` always submits, preventing multiline textarea input
- Severity: High
- Evidence:
  - `src/render_prompts/form.rs:108` submits on Enter for all focused fields.
  - `src/components/form_fields.rs:1126` expects Enter to insert newline in `FormTextArea`.
- Impact:
  - Users cannot reliably enter multiline values in textarea fields.
  - Behavior is internally inconsistent between renderer and field component contracts.
- Recommendation:
  - Gate form-level Enter: if focused field is textarea, forward Enter to field handler instead of submitting.
  - Add explicit submit shortcut (e.g. `Cmd+Enter`) for multiline fields.

### 2) Footer claims required-field enforcement but no validation exists
- Severity: High
- Evidence:
  - `src/render_prompts/form.rs:193` shows status text: `complete required fields and press Enter`.
  - `src/render_prompts/form.rs:111` collects and submits values directly with no validation checks.
- Impact:
  - UX promise is misleading.
  - Invalid or incomplete forms can be submitted silently.
- Recommendation:
  - Implement pre-submit validation stage and field-level error state.
  - If validation is not implemented yet, change footer copy to avoid claiming required checks.

### 3) Field types are not behaviorally validated (`email`, `number`, etc.)
- Severity: Medium
- Evidence:
  - `FormTextField` accepts any printable input regardless of field type in `src/components/form_fields.rs:553`.
  - No per-type constraints, format checks, or normalization in reviewed files.
- Impact:
  - `type="number"` and `type="email"` currently behave as plain text.
  - Increased downstream validation failures.
- Recommendation:
  - Add lightweight client-side rules by `field_type` (number chars/sign/decimal, email format hint+check).
  - Keep server-side validation authoritative, but provide immediate client feedback.

### 4) Overflow strategy can hide content and fields with no recovery path
- Severity: Medium
- Evidence:
  - Form container is capped at `700px` in `src/render_prompts/form.rs:155-156`.
  - Inner content uses `.overflow_y_hidden()` in `src/render_prompts/form.rs:181`.
  - Textarea input also uses `.overflow_hidden()` in `src/components/form_fields.rs:1271`.
- Impact:
  - Large forms or long textarea values can be clipped.
  - Keyboard-only users may lose visibility into focused content.
- Recommendation:
  - Enable vertical scrolling for form body.
  - Add textarea internal scroll once content exceeds visible rows.

### 5) Legacy byte-index handlers still exist beside char-safe handlers
- Severity: Medium
- Evidence:
  - Legacy methods use byte-index behavior in `src/components/form_fields.rs:260-306` (`insert_str`/`remove` with `cursor_position`).
  - New char-safe handlers are used elsewhere (`src/components/form_fields.rs:490+`).
- Impact:
  - Increases maintenance risk and reintroduction of UTF-8 cursor bugs.
  - Confusing code path ownership for future contributors.
- Recommendation:
  - Remove or hard-deprecate legacy handlers; keep one canonical key/input path per component.

### 6) Checkbox value semantics are reduced to `"true"/"false"`
- Severity: Medium
- Evidence:
  - Checkbox state is stored and emitted as `"true"`/`"false"` in `src/components/form_fields.rs:1302-1307` and `src/components/form_fields.rs:1331-1335`.
- Impact:
  - Loses HTML-style value semantics where checked value may be custom (e.g. `"yes"`, ids).
  - Limits interoperability for scripts expecting non-boolean payloads.
- Recommendation:
  - Preserve both checked boolean and original field value contract.
  - Consider payload shape `{ checked: bool, value: string }` or map checked->configured value.

### 7) Sensitive values are logged in debug/info paths
- Severity: Medium
- Evidence:
  - Text field render logs raw values in `src/components/form_fields.rs:588-590` (including password field backing value).
  - Submitted full form JSON is logged in `src/render_prompts/form.rs:112`.
- Impact:
  - Potential leakage of passwords/secrets into logs.
- Recommendation:
  - Redact/mask sensitive fields by type/name (`password`, tokens, keys).
  - Keep structured logging, but avoid raw value dumps.

### 8) Placeholder UX and focus affordance are inconsistent
- Severity: Low
- Evidence:
  - Empty focused text field hides placeholder and shows only cursor in `src/components/form_fields.rs:676-679`.
- Impact:
  - Users can lose context in forms with many similarly styled inputs.
- Recommendation:
  - Keep placeholder visible while focused (faded) until first character, or add stronger label/description affordance.

### 9) Test coverage is helper-centric, not behavior-centric
- Severity: Medium
- Evidence:
  - Current tests primarily cover string helper functions in `src/components/form_fields_tests.rs:39-132`.
  - Remaining tests are source-string assertions in `src/components/form_fields_tests.rs:134-170`.
- Impact:
  - Key regressions (Enter behavior, validation gates, overflow, checkbox contract) can ship undetected.
- Recommendation:
  - Add behavior-oriented tests for field input handling and form submission rules.

## Suggested Test Additions
- `test_form_prompt_does_not_submit_when_textarea_receives_enter`
- `test_form_prompt_submits_on_cmd_enter_for_textarea`
- `test_form_prompt_blocks_submit_when_required_field_empty`
- `test_number_field_rejects_non_numeric_input_characters`
- `test_checkbox_preserves_configured_checked_value_contract`
- `test_form_body_scrolls_when_field_count_exceeds_max_height`

## Prioritized UX/Validation Roadmap
1. Fix Enter-vs-textarea conflict and update submission shortcut behavior.
2. Add minimal validation pipeline (required + type-aware checks) with inline field errors.
3. Make form and textarea overflow scrollable.
4. Consolidate key/input handlers to char-safe path only.
5. Expand tests from helper coverage to interaction/behavior coverage.

## Related Cross-Scope Note
- `select` fields are parsed elsewhere but may degrade to text behavior depending on field-entity mapping (`src/form_prompt.rs`). Worth auditing in the same validation pass.
