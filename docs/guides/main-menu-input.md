# Main Menu Input

The Script Kit launcher is a search box, but its first token can also switch into file browsing, Agent Chat, command picking, source-filtered search, or structured capture.

## Quick Reference

| Type | Mode |
| --- | --- |
| `deploy` | Normal launcher search across scripts, built-ins, apps, and passive source previews. |
| `Tab` | Open Agent Chat, apply a visible picker row, or browse into a selected root directory row. |
| `Shift+Tab` | Move up in a visible picker or root directory browse context. |
| `Cmd+Enter` | Send the focused launcher context to Agent Chat, or ask for an inline Power Syntax suggestion. |
| `Cmd+V` | Route large pasted text or images straight into Agent Chat as context. |
| `Cmd+K` | Open actions for the selected row or current input mode. |
| `Enter` | Run the selected row, accept a visible picker row, or execute a complete capture/command expression. |
| `Escape` | Close the active popup/proposal first, then clear/dismiss the launcher. |
| `~` or `~/src` | Open mini file search rooted at your home directory/path. |
| `/` | Open Agent Chat with the slash/skill picker. |
| `@` | Open Agent Chat with the context mention picker. |
| `>` | Open Quick Terminal when it is the whole input. |
| `?` | Open the launcher actions/help surface when actions are available. |
| `:` | Open the filter picker for advanced launcher search. |
| `files: report` or `f:report` | Restrict search to one source. |
| `-apps: chrome` | Exclude a source. |
| `type:script deploy` | Refine normal search with property filters. |
| `;` or `;todo Renew passport #errands p1` | Pick a capture target or capture structured local data. |
| `+` or `+todo Renew passport` | Legacy capture picker/alias. |
| `todo: Renew passport` | Keyword capture alias, only for known/registered targets. |
| `!` or `!dep` | Discover registered command heads in the command picker. |
| `>deploy env:prod #release -- --dry-run` | Invoke a command head with fields, tags, and argv. |

Syntax only takes over when the first token clearly opts in. Plain text, URLs, `localhost:3000`, `C#`, `hello!`, unknown `;target` / `+target` heads, and top-level `#tag` remain normal launcher search.

## Normal Search

Most input is fuzzy search. Type a script name, built-in name, app, note, browser tab, clipboard phrase, or general task:

```text
clipboard
current app
open pr
template
```

Normal search can still show passive sections such as files, notes, clipboard history, tabs, and recent items. Add a source head when you want that source to become the active search scope.

## File and Utility Triggers

Some one-character entries are direct launcher handoffs:

- `~` opens mini file search and normalizes to `~/` so your home folder lists immediately.
- `~/Downloads` opens mini file search at that path.
- `/` opens Agent Chat with the slash/skill picker.
- `@` opens Agent Chat with the context mention picker.
- `>` by itself opens Quick Terminal.
- `?` opens the actions/help surface when the current launcher state has actions.

These are intentionally narrow. For example, `/tmp` is treated as path/search text, `@browser` is not the same as the bare `@` context picker trigger, and `>deploy` is command syntax rather than Quick Terminal.

## Keyboard Triggers

Keyboard shortcuts can change what the same input means:

- `Enter` runs the selected launcher row in normal search. When a menu-syntax picker is visible, it accepts the selected picker row first. When capture or command syntax owns the input, it submits that expression to its handler.
- `Tab` has a priority order. It applies a visible menu-syntax picker row first, browses into a selected root file directory row next, and otherwise opens Agent Chat from the launcher.
- Plain `Tab` with non-empty launcher text forwards that text to Agent Chat as the first turn. Empty launcher input opens Agent Chat without auto-submitting.
- `Shift+Tab` moves up in a visible menu-syntax picker or root file directory browse context. It does not submit typed launcher text to Agent Chat.
- `Cmd+Enter` sends the current launcher context/focused row to Agent Chat. While composing Power Syntax (`:`, `;`, or `>head`), it asks for an inline suggestion instead of leaving the launcher.
- `Cmd+V` detects large text or image clipboard content from the main menu and attaches it to Agent Chat rather than filling the launcher input with document-sized text.
- `Cmd+K` opens the actions dialog for the selected row or current mode.
- `Escape` closes/dismisses the active picker, inline suggestion, or actions popup before falling through to the launcher's normal clear/dismiss behavior.

When an inline Power Syntax suggestion is visible, `Tab` or `Enter` accepts it and `Escape` dismisses it.

## Source-Filtered Search

Source heads are valueless filters. They can appear at the beginning or later in the query, with or without a space after the colon.

| Source | Heads | Example |
| --- | --- | --- |
| Files | `files:`, `f:` | `f: invoice` |
| Notes | `notes:`, `n:` | `notes: standup` |
| Clipboard history | `clipboard:`, `c:` | `c: meeting notes` |
| Browser tabs | `tabs:`, `t:` | `tabs: docs` |
| Browser history | `history:`, `h:` | `history: gpui` |
| Apps | `apps:`, `a:` | `apps: safari` |
| Scripts | `scripts:`, `s:` | `scripts: deploy` |
| Commands | `commands:`, `cmd:` | `cmd: deploy` |
| AI conversations | `conversations:`, `ai:` | `ai: refactor` |
| AI vault | `vault:`, `v:` | `vault: session` |
| Dictation history | `dictation:`, `d:` | `d: meeting notes` |
| Windows | `windows:`, `w:` | `w: preview` |

Useful forms:

```text
f:report
files: report
meeting notes n:
c: 
ai: 
-apps: chrome
files: report -notes:
```

Source-only inputs with a trailing space browse that source's default recent/current set. For example, `c: ` browses recent clipboard entries, `n: ` browses pinned/recent notes, `t: ` browses current tabs, and `d: ` browses recent dictations.

`processes:` / `p:` is not a committed root source head yet. Use the **Process Manager** built-in for process search.

Source-filter mode disables launcher input-history recall, so Up/Down stay focused on navigating the visible result list.

## Advanced Filters

Type `:` to open the filter picker, or type a complete filter directly. Filters compose with normal search words:

```text
:type:script deploy
type:script deploy
kind:script deploy
shortcut:any
shortcut:none
shortcut:cmd+k
has:shortcut
source:main inbox
plugin:main.todo
name:deploy
desc:database
alias:db
tag:work
:#work notes
meta.category:inbox
-type:app triage
```

Supported filter heads:

- `type:` / `kind:` — row kind such as scripts, scriptlets, skills, built-ins, apps, windows, files, notes, browser tabs/history, clipboard history, dictation history, AI conversations, fallbacks, and issues.
- `shortcut:` — `any`, `none`, or an exact shortcut such as `cmd+k`.
- `source:` — broad source/plugin/kit-name match.
- `plugin:` — precise plugin-pair match.
- `name:`, `desc:` / `description:`, `alias:`, `tag:`, `has:`.
- `meta.<path>:` — nested metadata value match.
- leading `-` — negates a filter or source head.

Tag rules:

- `:#work` is filter sugar for `tag:work`.
- `#work` by itself remains normal launcher search.
- Inside capture mode, `#work` becomes a saved capture tag.

Partial filter text such as `:`, `:typ`, `:type:`, `:has:sh`, and `:#` stays in picker mode until you commit a valid filter.

Picker controls:

- `Up` / `Down` moves the picker highlight.
- `Tab` applies the highlighted filter row.
- `Enter` accepts the highlighted filter row.
- `Escape` closes the picker without clearing the whole launcher first.

For open-value qualifiers such as `source:`, `plugin:`, `name:`, or `meta.category:`, applying the row keeps the picker open so you can type the value. Complete source heads such as `files:` close the picker and search that source.

## Capture Mode

Capture mode turns the launcher into a structured local-data composer. Built-in targets are:

| Target | Use |
| --- | --- |
| `todo` | Save a task. |
| `cal` | Save a calendar-style item. |
| `note` | Save a note. |
| `social` | Save social/post text. |
| `link` | Save a URL. |

Examples:

```text
;
;todo Renew passport #errands p1
;note "Decision: ship parser first" #project
;link https://zed.dev #rust
;cal Design review start:"friday 2pm" for:45m
todo: Renew passport #errands
+ 
+todo Renew passport
```

Capture fields:

- body text is everything that is not parsed as a structured token
- `#tag` adds tags
- `p1` through `p4` adds priority
- `http://...` / `https://...` or `url:<value>` sets the URL
- `due:`, `at:`, `start:`, and `end:` add date phrases
- `for:<duration>` adds duration
- `key=value` adds custom key/value metadata

Known targets own the input. Unknown targets fall back to normal search, so `;github` or `+github` only becomes capture syntax after a script registers `github` through metadata. `mcal` is parser-known for calendar schemas but is not shown in the default picker unless registered.

When capture data is incomplete, Enter does not run the handler. For example, `;todo` needs a body, `;link` needs a URL, and `;cal` needs both body text and a date.

Capture picker controls:

- bare `;` opens the capture target picker.
- bare `+` opens the same picker through the legacy alias.
- `Tab` or `Enter` accepts the highlighted target and inserts `;target ` / `+target `.
- `Shift+Tab`, `Up`, and `Down` navigate picker rows.
- `Escape` closes the picker first.

Capture rows can also expose actions such as creating a handler script or browsing prior captures, but those are mode-specific actions rather than normal launcher search rows.

## Command Picker and Command Invocation

The command surface is split into discovery and invocation:

- `!` / `!partial` opens the command picker for registered command heads.
- `Tab` or `Enter` on a picker row inserts the command token for that row.
- `>head ...` is the manual argv composer/invocation form.
- bare `>` still opens Quick Terminal.

Command examples:

```text
!
!dep
>deploy staging -- --dry-run
>deploy env:prod #release -- --dry-run
```

In `>head` mode:

- the first token after `>` is the command head
- `key:value` before ` -- ` becomes a command field
- `#tag` before ` -- ` becomes command tags
- tokens after ` -- ` are passed as argv to the script/scriptlet

Command heads come from script aliases/file names or scriptlet command metadata. Duplicate heads are shown as ambiguous/disabled instead of silently running the first match. Command syntax is never a shell escape; use Quick Terminal for shell commands.

Because `!` is discovery and `>head` is invocation, a command picker row may insert a `>head ` token. Continue typing argv in that `>head` composer and press `Enter` to run it.

## Agent Chat from the Main Menu

Agent Chat has keyboard and text entry paths:

- `Tab` opens Agent Chat with the current launcher surface staged as context when no higher-priority picker/root-directory browse action handles it.
- If the launcher input has text, plain `Tab` submits that text as the first Agent Chat turn.
- `Cmd+Enter` sends the focused launcher target or ambient launcher context to Agent Chat without treating the typed search as the first turn.
- `/` opens Agent Chat directly into slash/skill selection.
- `@` opens Agent Chat directly into context mention selection.
- Large `Cmd+V` pastes from the main menu become Agent Chat context attachments.

Inside Agent Chat, slash commands and mentions are composer features. From the main menu, only the bare `/` and bare `@` triggers hand off immediately; longer strings such as `@browser` remain launcher search unless they are typed inside the Agent Chat composer.

## Safe Boundaries

The parser is intentionally conservative:

- `localhost:3000` is not capture syntax.
- `/tmp` is not the slash picker.
- `@browser` is not the mention picker from the launcher.
- `#tag` is not a top-level filter unless you enter refine mode as `:#tag`.
- `hello!` is normal search text.
- unknown `;target`, `+target`, or `target:` heads stay searchable.
- `:f` is not source syntax; use `f:` / `files:` or open `:` and accept the source row.
- bare `>` is Quick Terminal, while `>deploy` is command invocation.

When in doubt, type normally first. Add `:`, a source head, `;`, `!`, or `>` only when you want to opt into that mode.
