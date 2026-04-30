# Main Menu Power-User Syntax

Design and implement additional power-user syntaxes for the Script Kit main menu input, extending the existing single-character triggers without breaking them.

## User goals (verbatim)

> I'd like for the main menu to support a few more custom syntaxes:
>
> ## Current
> / - Open Agent chat with the slash command popup menu
> @ - Open Agent chat with the mention popup menu
> ~ - Open the file search in "directory mode"
>
> ## Desired
> - Some sort of "advanced searching" syntax that would help people auto-filter by type, shortcut, and other properties of the scripts/skills/scriptlets/commands/etc
> - Some sort of tagging system syntax when when people want to quickly add to todo lists, socials, etc
> - A date inference so people could quickly add to calendars, todos, etc
> - anything else that makes sense?
>
> I think scripts/scriptlets/skills might need some metadata to expose them to these features so that the main menu only shows the items these are available for
>
> But the general idea is that power-users will have access to power-user syntax (similar to Todoist and others) where you can type in a complete string in the main menu box and expect to accomplish a much more powerful, repeatable task without relying on AI.
>
> We'll of course need to ship with a handful of examples for each as well.

## Constraints

- Existing triggers in `src/app_impl/filter_input_core.rs` (`is_transient_script_list_trigger`, `special_entry_from_script_list_filter`): `~` `/` `@` `>` `?` — must keep working.
- Power-user syntax should be **composable** and **repeatable** without LLM inference for the common path.
- Scripts / scriptlets / skills should be able to opt in via metadata so the matching result only surfaces when the typed syntax applies.
- Must ship with a handful of working examples per syntax family.
- Follow `lat.md/` patterns — every new concept gets a section; tests should have `@lat:` comments.
- Do not break `make smoke-main-menu`.

## Success criteria (iteration exit condition)

When Oracle and the implementation together reach a state where:

1. At least one non-AI advanced-filter syntax is shipped end-to-end (parser → filter pass → examples → test).
2. A tagging syntax family has a target-routing mechanism that can hand off the parsed payload to a script/scriptlet via a stable contract.
3. Date inference produces a normalised `{iso, relative}` pair that at least one example script consumes.
4. Script/scriptlet metadata has a new field (or equivalent) so handlers can opt in to a syntax family, and the main menu filters accordingly.
5. `lat.md/` has a new section documenting the syntax layer; `lat check` passes.
6. At least three shipped example scripts/scriptlets demonstrate the new syntaxes.
7. Commit trail reads cleanly — one commit per concern; each commit message is an agent-reproducible prompt.

## Non-goals

- Rewriting the existing `~/@/` triggers. Extend alongside; do not replace.
- AI-driven natural-language parsing. The point is power-user muscle memory.
- Shipping a full template / snippet manager. Focus is the syntax + dispatch layer.
