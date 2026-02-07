# Action Section/Category Audit (codex-audit-act-section-grouping)

## Scope
- `src/actions/builders/chat.rs`
- `src/actions/builders/clipboard.rs`
- `src/actions/builders/file_path.rs`
- `src/actions/builders/notes.rs`
- `src/actions/builders/script_context.rs`

## Verification criteria
1. Sections group related actions logically
2. Categories are used consistently
3. Section headers display correctly
4. Order within sections makes sense
5. Most-used actions are easy to find

## Key findings

### F1 (High): Several high-traffic builders provide no sections at all
- `src/actions/builders/clipboard.rs:39` through `src/actions/builders/clipboard.rs:217` builds a long mixed list with no `.with_section(...)` calls.
- `src/actions/builders/file_path.rs:23` through `src/actions/builders/file_path.rs:186` has no section assignment in either file or path context actions.
- `src/actions/builders/chat.rs:32` through `src/actions/builders/chat.rs:77` (chat context actions) has no section assignment.
- Impact:
  - Logical grouping is weaker in these contexts.
  - Header rendering cannot help because headers are only injected when `action.section` is set.
  - Discoverability suffers as lists grow.

### F2 (Medium): AI command bar splits the same section into two non-contiguous blocks
- In `src/actions/builders/chat.rs`, `branch_from_last` is tagged as `"Actions"` at `src/actions/builders/chat.rs:174`, but appears after `"Export"` at `src/actions/builders/chat.rs:166`.
- Grouping logic adds a header every time the section changes (`src/actions/dialog/part_01.rs:172`-`src/actions/dialog/part_01.rs:177`), so users will see `Actions` twice.
- This is valid technically, but visually fragmented.

### F3 (Medium): Note switcher section ordering depends entirely on incoming note order
- `src/actions/builders/notes.rs:232`-`src/actions/builders/notes.rs:281` iterates notes as given and assigns section per note (`"Pinned"` or `"Recent"`) at `src/actions/builders/notes.rs:269`.
- If upstream note order is mixed (Pinned/Recent/Pinned), headers repeat and grouping appears unstable.
- There is no local normalization in this builder to ensure a single contiguous `Pinned` block followed by `Recent`.

### F4 (Medium): Most-used chat actions can be pushed down by model entries
- `get_chat_context_actions` appends one action per model first (`src/actions/builders/chat.rs:25`-`src/actions/builders/chat.rs:43`), then appends `continue_in_chat` and other likely frequent actions (`src/actions/builders/chat.rs:45`-`src/actions/builders/chat.rs:77`).
- With many models, high-frequency actions become less accessible.

## What is consistent / working well
- Category consistency is strong: all audited actions use `ActionCategory::ScriptContext` across the five scoped builders.
- Notes command bar grouping is coherent and feature-gated:
  - `Notes`, `Edit`, `Copy`, `Export`, `Settings` sections are explicit in `src/actions/builders/notes.rs:43`, `src/actions/builders/notes.rs:82`, `src/actions/builders/notes.rs:108`, `src/actions/builders/notes.rs:144`, `src/actions/builders/notes.rs:158`.
- Script context grouping is generally coherent:
  - `Actions` first (`src/actions/builders/script_context.rs:30`), then `Edit`, `Share`, and `Destructive` sections, with destructive actions intentionally appended near the end (`src/actions/builders/script_context.rs:278`).
- Section header rendering behavior is correct and tested:
  - headers appear on section transitions (`src/actions/dialog/part_01.rs:172`-`src/actions/dialog/part_01.rs:177`),
  - no headers when section is absent (`src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_14/tests_part_03.rs:52`-`src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_14/tests_part_03.rs:60`).

## Criterion-by-criterion verdict
1. **Logical section grouping**: Mixed. Strong in `notes.rs` and `script_context.rs`; absent in `clipboard.rs`, `file_path.rs`, and chat context in `chat.rs`.
2. **Category consistency**: Pass (all scoped builders use `ScriptContext`).
3. **Section header display correctness**: Pass in renderer/tests; effectiveness depends on builders actually setting sections.
4. **Order within sections**: Mostly good where sections exist; one notable split section in AI command bar (`Actions` appears in two places).
5. **Most-used actions easy to find**: Mixed. Good in file/clipboard primaries (open/paste first), weaker in chat context due model-first ordering and no sections.

## Suggested follow-up changes (not implemented in this audit)
1. Add explicit sections to clipboard/file/path/chat-context builders.
2. Move `branch_from_last` so all `Actions` entries in AI command bar are contiguous.
3. Normalize note-switcher ordering to contiguous `Pinned` then `Recent` before action creation.
4. Promote `continue_in_chat` (and optionally copy/clear) ahead of model list or into a dedicated top section.
