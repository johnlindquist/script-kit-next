# Power Syntax Grammar Handoff

This is the handoff note for the Script Kit main-menu Power Syntax work as of 2026-04-25. It is written for another AI agent taking over the feature while the worktree may still be dirty.

## Product Intent

Power Syntax turns the main launcher input into a keyboard-first command language without sacrificing normal fuzzy search. The experience we are aiming for is closer to Todoist quick-add than a command palette: users type one natural sentence, develop muscle memory for small symbols, and stay in flow.

The core advantage over Todoist is that Script Kit is local and programmable. The same one-line grammar can create local todos, notes, links, calendar events, social drafts, custom artifacts, and structured payloads; refine the launcher catalog; intentionally run registered commands; and feed context to an agent.

The mental model:

```text
+   capture/create local structured data
:   refine/narrow launcher search
#   label captured data, or filter by tag after :
!   run a registered Script Kit command
/   use an agent chat skill
@   attach/select current context
```

The important UX promise is safety: ordinary text stays ordinary search unless the first token clearly opts into a known grammar surface. Power users should feel fast, but existing launcher users should not feel the grammar stealing input.

## Golden Rules

1. Symbols are verbs, not decoration.
2. `+` creates data.
3. `:` narrows the launcher catalog.
4. `#` labels data inside capture and filters tags only after `:`.
5. `!` runs a registered Script Kit command, never shell text.
6. `/` and `@` stay the agent-chat skill/context language.
7. Top-level `#work`, unknown `+target`, URLs, `localhost:3000`, `C#`, `Decision: ship`, and `hello!` stay plain launcher search.
8. Parser ownership must be strict and collision-safe.
9. If grammar-owned input makes the main list irrelevant, use the list area to teach the grammar instead of showing stale rows.

## Canonical Grammar

```text
Plain search
  deploy
  git branch
  safari
  localhost:3000
  C# parser
  #work
  +github

Agent chat
  /review @current-file
  /explain @selection
  /summarize @clipboard

Capture
  ;todo Renew passport #errands p1 due:tomorrow
  ;note Decision to ship parser first #product
  ;link https://zed.dev #rust title:"GPUI notes"
  ;cal Design review start:"friday 2pm" for:45m #work
  ;social Shipping Power Syntax today #release

Refine
  :type:script deploy
  :type:script shortcut:any
  :-type:app triage
  :#work type:script
  :tag:client/acme type:issue

Run
  >deploy -- prod --dry-run
  !open-pr -- 123
  >test-menu-syntax -- --watch
  >ps-env env:dev #demo -- --dry-run
```

Recommended tagline for docs and UI:

```text
+ add
: narrow
# label
! run
/ skill
@ context
```

## Symbol Semantics

### `+` Capture/Add

`+` means create structured local data. It is not "run the todo script." It is "compose a payload and send it to the best handler."

Basic flow:

```text
+
+t
;todo
;todo Renew passport #errands p1 due:tomorrow
```

Expected behavior:

- `+` opens the capture target popup.
- `+t` narrows the popup by target slug.
- Accepting `;todo` rewrites to `;todo ` and closes the picker.
- Once body text begins, the popup closes.
- The main list is suppressed while composing a capture body.
- Enter routes through capture execution, not main-list selection.
- The handler receives a JSON payload through `KIT_MENU_SYNTAX_PAYLOAD_PATH`.

Built-in targets:

```text
;todo
;note
;link
;cal
;social
```

Dynamic targets are allowed when script metadata registers `menuSyntax: [{ family: "capture.v1", targets: [...] }]`. Unknown targets such as `+github` remain search until registered.

Capture body tokens:

- `#tag` becomes a payload tag.
- `p1` through `p4` become priority.
- `due:`, `at:`, `start:`, `end:` become explicit dates.
- `for:45m` becomes duration.
- `url:https://...` or a bare URL becomes the URL field.
- `key=value` becomes generic kv.
- Remaining text is body.
- Ordinary colon phrases like `Decision: ship parser first` remain body text unless the key is known.

### `:` Refine/Search

`:` means narrow the launcher catalog before search words run. It is search, not capture, and it should not be described as "structured query" in user-facing copy.

Preferred wording:

- Use "Filters", not "Predicates".
- Use "Search words", not "free text".
- Use "`:` narrows the launcher catalog before search words run."
- For empty results, say "No matches after these filters."

Supported filters:

```text
type:/kind:
shortcut:
source:
plugin:
name:
desc:/description:
alias:
tag:
has:
meta.<path>:
-<filter>
```

Examples:

```text
:type:script deploy
:type:script shortcut:any
:shortcut:none
:source:main capture
:plugin:core clipboard
:name:deploy
:desc:database
:has:menuSyntax
:meta.domain.kind:fixture
:-type:app triage
```

Behavior:

- Bare `:` opens filter rows.
- Partial `:typ` narrows rows.
- Typo candidates such as `:typ:script` can show "Did you mean type:script?"
- Concrete filters such as `type:script` close the popup when accepted.
- Open-value filters such as `source:`, `plugin:`, `name:`, `desc:`, `alias:`, `#`, and `tag:` keep the popup open for value entry.
- Complete refine inputs keep showing filtered launcher results.
- Bare/partial refine inputs can replace the main list with a guide card because the user is still learning the grammar.

### `#` Tag

`#` does not create a top-level mode in v1. Its purpose depends on context:

```text
#work              plain launcher search
:#work             filter launcher rows tagged #work
;todo ... #work    label the captured item as #work
```

This is intentionally strict. Top-level `#work` staying search avoids stealing common technical or personal text. The UI should teach this boundary anywhere it can.

Current tag UX:

- Bare `:#` shows a main-list guide titled "Filter by tag".
- The guide explicitly contrasts `#work`, `:#work`, and `+... #work`.
- The `:` popup includes both `#` and `tag:` rows.
- Both rows keep the popup open.
- Empty `:#work ...` results say "No launcher items tagged #work" and show rows labeled "Filters" and "Search words".
- Plain top-level `#work` has no `menuSyntaxMainHint`; it uses the normal empty state with a nudge toward `:#work` or `;todo ... #work`.

### `!` Run

`!` means explicit registered Script Kit command invocation. It must never be a shell escape.

Examples:

```text
!
!dep
>deploy -- prod --dry-run
!open-pr -- 123
>test-menu-syntax -- --watch
>deploy env:prod #release -- --dry-run
```

Behavior:

- `!` opens a command popup backed by registered scripts and scriptlets.
- Command heads are normalized from script aliases/file names and scriptlet command metadata.
- Accepting a row inserts `!<slug> `.
- Unknown heads such as `!important` show not-found safety copy and do not run shell text.
- Duplicate heads are disabled/ambiguous instead of running the first match.
- Fields before ` -- ` become command metadata.
- Args after ` -- ` pass through to the script/scriptlet.

### `/` And `@`

These stay with the agent chat model:

```text
/review @current-file
/explain @selection
/summarize @clipboard
```

Do not teach `/deploy` or `/todo` as launcher commands. Launcher discovery for skills can use `:type:skill review`, but skill execution belongs in chat.

## Main User Stories

### Plain Search Stays Boring

Users can type ordinary technical text and get normal launcher search:

```text
deploy
localhost:3000
https://localhost:3000
C# tutorial
email@example.com
Decision: ship parser first
hello!
#random
+github
+react component
not-a-target: stuff
```

The grammar must not make everyday text dangerous.

### Todoist-Style Capture

Users type a sentence like:

```text
;todo Renew passport #errands p1 due:tomorrow
```

They should see themselves composing a payload, not searching for "todo Renew passport". Stale main-list rows and "No results for..." copy are wrong during capture composition.

### Refine The Catalog

Users type:

```text
:type:script deploy
```

They are finding the script they want to run. The typed filters should disappear from the search words and apply as post-filters. Search should still feel like the launcher.

### Tags As Labels And Filters

Users type:

```text
;todo Send proposal #client/acme
:#client/acme type:issue
```

The first labels saved data. The second filters launcher/catalog rows. This distinction is central and should be repeated in UI copy.

### Intentional Command Execution

Users type:

```text
>test-menu-syntax -- --watch
```

This runs a registered Script Kit command. It is different from `:type:script test`, which finds scripts, and from `;todo Test menu syntax`, which captures a task.

## Main Menu Hint Experience

When grammar-owned input makes the normal result list misleading, the list area should become a read-only teaching surface.

Current hint behavior:

- Capture picker states explain selected `+target`.
- Capture composer states preview body, tags, priority, dates, URLs, and fields.
- Command picker/composer states explain command resolution, argv, fields, tags, ambiguity, and unknown-head safety.
- Bare/partial refine states show `AdvancedQueryGuide`.
- `:#` shows a dedicated "Filter by tag" guide.
- Advanced-query empty states explain filters and search words.
- Top-level `#tag` stays normal search and uses the normal empty state nudge.

The hint is exposed to automation as `stateResult.menuSyntaxMainHint`, so agentic tests should prefer that receipt before screenshot scraping.

Current visual goal:

- The hint card fills the available main-list area.
- Examples stay visible/open, especially in mini launcher height.
- The card is read-only, not selectable launcher data.
- It should feel like contextual help, not a marketing panel.

## Popup Experience

Menu syntax uses a detached popup NSPanel anchored under the main input, matching ACP `/` and `@` pickers.

Popup principles:

- `+`, partial `+t`, `:`, partial `:typ`, `!`, and partial `!dep` open/update the popup.
- Legacy `~ / @ > ?` never open this popup.
- `+target body` closes the capture target popup.
- Command body after `>head ` closes the command picker.
- Advanced-query `:` popup does not suppress real results for complete queries.
- Selection is preserved by row id across rebuilds.
- Arrow Up/Down, Tab, Shift+Tab, Enter, and Escape route through the popup while it is open.
- Escape order is popup close, then clear filter, then hide window.

Recent important fix:

- Typing `;todo` and then more body text should keep the capture composer/hint active rather than collapsing into main search.
- Tab-apply re-runs the state machine so accepted rows do not leave stale popup state.

## Local-First Data Model

Capture execution writes a versioned payload tempfile and passes its path to handlers:

```text
KIT_MENU_SYNTAX=1
KIT_MENU_SYNTAX_VERSION=menu-syntax.payload.v1
KIT_MENU_SYNTAX_PAYLOAD_PATH=/absolute/path/to/capture_v1-<id>.json
KIT_MENU_SYNTAX_FAMILY=capture.v1
KIT_MENU_SYNTAX_TARGET=todo
KIT_MENU_SYNTAX_HANDLER_KIND=script|scriptlet
KIT_MENU_SYNTAX_HANDLER_COMMAND_ID=<id>
```

Payload shape:

```json
{
  "version": "menu-syntax.payload.v1",
  "family": "capture.v1",
  "target": "todo",
  "raw": ";todo Renew passport #errands p1 due:tomorrow",
  "body": "Renew passport",
  "tags": ["errands"],
  "priority": 1,
  "url": null,
  "duration": null,
  "kv": {},
  "dates": [],
  "handler": {
    "kind": "script",
    "commandId": "script/main:Capture Todo Inbox",
    "name": "Capture Todo Inbox",
    "pluginId": "main"
  }
}
```

Shipped handlers write local artifacts under `$SK_PATH`:

- `;todo` -> `menu-syntax/todos.jsonl`
- `;cal` -> `menu-syntax/calendar/*.ics`
- `;note` -> `notes/YYYY-MM-DD.md`
- `;social` -> `menu-syntax/social-drafts/*.md`
- `;link` -> `menu-syntax/bookmarks.jsonl`

The demo pack adds dynamic targets and command examples:

- Dynamic captures: `+github`, `;expense`, `+snippet`, `+fixture`
- Commands: `>ps-env`, `!ps-payload`, `!ps-stamp`, intentional duplicate `!ps-dupe`

Do not add wildcard demo handlers unless the goal is explicitly to test wildcard behavior. They weaken the collision proof by making too many `+target` heads claim input.

## Source Map

Core parser and payload modules:

- `src/menu_syntax/parse.rs` - top-level classifier and parser boundary.
- `src/menu_syntax/query.rs` - `:` advanced/refine query parsing.
- `src/menu_syntax/capture.rs` - capture body parser.
- `src/menu_syntax/payload.rs` - payload structs and known targets.
- `src/menu_syntax/date.rs` - date resolution.
- `src/menu_syntax/execute.rs` - payload/env construction.
- `src/menu_syntax/command.rs` - command head normalization.
- `src/menu_syntax/mode.rs` - raw-guarded parser state, search text, input highlight spans.

Discovery and picker modules:

- `src/menu_syntax/trigger_picker.rs` - pure popup row snapshots for `+`, `:`, and `!`.
- `src/menu_syntax/trigger_picker_keys.rs` - intent reducer for keyboard dispatch.
- `src/app_impl/menu_syntax_trigger_popup.rs` - pure state-machine adapter and row adaptation.
- `src/app_impl/menu_syntax_trigger_popup_window.rs` - GPUI/NSPanel popup window.
- `src/components/inline_popup_window.rs` - shared NSPanel helpers.
- `src/components/inline_picker.rs` - shared row shape.

Hint surface:

- `src/menu_syntax/main_hint.rs` - pure `MenuSyntaxMainHintSnapshot` builder.
- `src/app_impl/menu_syntax_main_hint.rs` - ScriptList adapter.
- `src/render_script_list/mod.rs` - renders hint card, empty-state nudges, and input highlight ranges.
- `src/protocol/message/variants/query_ops.rs` - `stateResult.menuSyntaxMainHint`.
- `src/prompt_handler/mod.rs` - getState computes the hint for automation.

Filtering and execution:

- `src/app_impl/filtering_cache.rs` - hides stale results for capture/command composer ownership and applies advanced query filters.
- `src/app_impl/filter_input_change.rs` - runs popup state machine on real input.
- `src/app_impl/filter_input_updates.rs` - runs popup state machine on automation `setFilter`.
- `src/app_impl/selection_fallback.rs` - routes Enter to capture/command before main selection.
- `src/app_execute/menu_syntax_execution.rs` - capture and command execution entry points.

Examples and setup:

- `scripts/examples/menu-syntax/*.ts`
- `kit-init/scriptlets/examples/power-syntax.md`
- `src/setup/mod.rs`
- `src/scripts/scriptlet_loader/tests.rs`

Docs and test contracts:

- `lat.md/menu-syntax.md`
- `.notes/power-user-stories.md`
- `tests/menu_syntax_text_entry_contract.rs`
- `tests/sdk_automation_runtime/mod.rs`

## Current Progress

The old inline SectionHeader takeover was replaced by a detached popup window and main-list hint surface.

Landed pivot commits before later local work:

```text
e5dca4779  A  chip removal
e76af941a  B  inline popup window extract
cb83f0b0c  C  neutral InlinePickerRow
0a6b2a6f7  D1 pure state machine
e7c7a2b7a  D2a live filter wiring
99bd1bea2  D2b popup window + takeover removal
28e34ef1c  D2c keyboard dispatch
da9820295  bug fix: partial trigger retention + Tab re-sync
```

After those commits, additional work in the dirty tree extends the grammar:

- `!` command invocation is wired to registered scripts/scriptlets.
- `#` tags are first-class in capture and refine.
- Dynamic capture targets come from metadata.
- Demo scripts/scriptlets exercise capture, refine, command, duplicates, and local artifacts.
- The main menu hint surface replaces stale list/no-results UI when grammar-owned input needs teaching.
- Bare `:` and `:#` now get user-facing guides.
- `#` and `tag:` rows exist in the `:` popup and keep the popup open.
- The launcher input accents `+text`, `:text`, and `!text` like ACP `/text` / `@text`.

Because the worktree contains unrelated/ongoing changes, the next agent must not use destructive git commands or revert files it did not intentionally edit.

## Verification Status

Recent passing checks from the current handoff work:

```text
cargo test --lib menu_syntax -- --nocapture
  247 passed

cargo test --test menu_syntax_text_entry_contract -- --nocapture
  15 passed

cargo test --test sdk_automation_runtime state_result -- --nocapture
  2 passed

cargo build
  passed with existing warnings

lat check
  passed
```

Agentic state receipts verified:

- `:` -> `menuSyntaxMainHint.kind == "AdvancedQueryGuide"`, title "Refine launcher search".
- `:#` -> title "Filter by tag", rows for `#work`, `:#work`, and `+... #work`.
- `:#work __definitely_no_menu_hint_match_zzzz` -> `AdvancedQueryEmpty`, title "No launcher items tagged #work", rows "Filters" and "Search words".
- `#work` -> no `menuSyntaxMainHint`, normal search/empty-state path.
- Visual screenshot showed the `:#` guide card replacing launcher rows and keeping all examples visible:
  `.test-screenshots/menu-syntax-hash-hint-final.png`.

The prior bug-fix verification focused on:

- Popup stays open while typing partial `;todo`.
- Popup closes once capture body composition starts.
- Tab/Enter route through `InlinePickerKeyIntent`.
- Escape order remains popup, filter, window.

## Acceptance Matrix

| Input | Meaning |
| --- | --- |
| `deploy` | Plain launcher search |
| `/review` | Skill in agent chat |
| `@current-file` | Context/object picker |
| `+` | Capture target popup |
| `+t` | Partial capture target popup |
| `;todo` | Known target selected, missing body |
| `;todo Buy milk` | Capture todo payload |
| `todo: Buy milk` | Legacy capture alias |
| `:` | Refine/filter guide and popup |
| `:typ` | Partial refine popup |
| `:typ:script` | Typo-fix candidate |
| `:type:script git` | Search `git`, filter to scripts |
| `:#work type:script` | Search/filter tagged work scripts |
| `:tag:client/acme type:issue` | Canonical tag filter |
| `!` | Command popup |
| `!dep` | Partial command popup |
| `>deploy -- prod` | Run registered Script Kit command with argv |
| `!important` | Not shell; only runs if a registered command exists |
| `localhost:3000` | Plain launcher search |
| `+github` | Plain search unless `github` target exists |
| `#work` | Plain launcher search in v1 |

## UX Copy Guidance

Use:

- "Capture" for `+`
- "Refine" for `:`
- "Filter" for individual predicates/qualifiers
- "Search words" for remaining fuzzy text
- "Tag" / "Label" for `#`
- "Run" for `!`

Avoid:

- "Predicate" in visible UI
- "Structured query" in visible UI
- "Free text" in visible UI
- "Mode owns input"
- Any phrasing that implies top-level `#work` is special
- Any phrasing that implies `!` is shell

Good empty/refine copy:

```text
Refine launcher search
Use `:` to add filters, then type the words you want to match.

Filter by tag
After `:`, `#tag` narrows the launcher catalog to tagged items.

No matches after these filters
Remove a filter or change the search words.
```

## Known Cautions For The Next Agent

1. Always run `lat search` and `lat expand` before coding in this repo.
2. Update `lat.md/menu-syntax.md` when changing grammar behavior, architecture, tests, or UX.
3. Run `lat check` before final response.
4. Use `rg` first for search.
5. Use `apply_patch` for edits.
6. Do not revert unrelated dirty work.
7. Prefer pure parser/state-machine tests before UI.
8. For UI verification, use `$agentic-testing`: state receipts first, screenshot only when needed, and stop any session you start.
9. Do not test `!` by executing arbitrary shell text. It must resolve registered commands only.
10. Do not add broad dynamic capture targets without considering collision safety.
11. Keep `/` and `@` out of launcher grammar docs unless explaining agent chat.
12. Keep card/list layout compact enough for mini launcher height.

## Likely Next Work

Potential next tasks, depending on product direction:

- Continue refining the hint card layout for mini and full launcher sizes.
- Add richer tag completion from known captured tags once tag storage/indexing is explicit.
- Add a proper help surface behind "Open Menu Syntax help".
- Finish or expose the captures inverse browser.
- Wire retention after successful payload writes if not already complete in the active branch.
- Improve command schema-backed hints for `!command`.
- Decide whether body-first capture (`+ Renew passport to:todo`) is worth implementing.
- Decide whether `handler:` / `via:` should be first-class handler-selection fields.
- Add broader agentic user-story tests for the full flows:
  `;todo ...`, `:#tag ...`, `!command ...`, Escape order, Tab/Enter row apply, and plain-search collision cases.

## Demo Script For Humans

Run through these manually in the launcher:

```text
;todo Renew passport #errands p1 due:tomorrow
;link https://zed.dev #rust #gpui title:"Zed GPUI"
:type:script has:menuSyntax capture
:#script-kit type:script parser
>test-menu-syntax -- --watch
/review @src/menu_syntax/parse.rs
```

This six-line demo proves the model:

```text
+ capture
: refine
# tag
! run
/ skill
@ context
```

