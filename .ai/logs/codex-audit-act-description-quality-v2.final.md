# Action Label/Description Audit

## Scope
- `src/actions/builders/script_context.rs`
- `src/actions/builders/notes.rs`

## Summary
- Audited all `Action::new(...)` entries in scope.
- Most command-style labels are concise and verb-first.
- No obvious spelling typos found.
- A small number of consistency/clarity issues were found.

## Findings

1. **Medium**: Missing description where a description would help disambiguation.
- File: `src/actions/builders/notes.rs:192`
- Detail: Preset actions in `get_new_chat_actions` use `None` for description.
- Why it matters: Presets can have similar names; a short description (provider/type/use case) improves scanability and selection confidence.

2. **Low**: Inconsistent capitalization for “deeplink”.
- File: `src/actions/builders/notes.rs:115`
- Detail: Description uses lowercase (`"Copy a deeplink to the note"`) while labels use `"Copy Deeplink"` and script-context descriptions also capitalize it.
- Why it matters: Inconsistent term casing weakens UI polish and can look unintentional.

3. **Low**: Description clarity can be improved for command-bar actions.
- File: `src/actions/builders/notes.rs:64`
- Detail: `"Open note browser/picker"` is understandable but informal and less explicit than other descriptions.
- Why it matters: Slash-style phrasing is less consistent with the rest of the action copy.

4. **Low**: Description tone/style differs from surrounding actions.
- File: `src/actions/builders/notes.rs:153`
- Detail: `"Window grows/shrinks with content"` is concise but reads like an implementation note rather than a user-facing action outcome.
- Why it matters: Most other descriptions are direct task-oriented statements.

5. **Low**: Label capitalization style differs in empty-state entry.
- File: `src/actions/builders/notes.rs:287`
- Detail: `"No notes yet"` is sentence case while most labels in these builders are title case.
- Why it matters: Minor style inconsistency in action list presentation.

## Checks Against Requested Criteria
- (1) Verb-first concise labels: **Mostly pass** for command actions; note switcher entries are data-item labels by design.
- (2) Informative descriptions: **Mostly pass** with clarity improvements noted above.
- (3) No `None` descriptions where helpful: **One issue found** in preset actions.
- (4) No typos: **Pass** (no obvious spelling typos found).
- (5) Consistent capitalization: **Minor issues found** (`deeplink` casing and `No notes yet` style).
