# Main Menu Input

The launcher input takes whatever you type: a command name, a search, or a sentence saying what should happen. Search is local by default. If you enable [remote readings](jev-intent.md), Jev can offer up to three interpretations under a "Jev · ⌘1–3" header, while the list below keeps matching commands. The ↵ label always names exactly what Enter will run.

A small set of symbols says part of a sentence exactly, turning the input into a prompt builder, capture composer, or filtered search without leaving the main list. Each word the app understands gets a colored underline in its role's color (subject, action, target, details, owner); the text itself stays white. The parser is conservative: plain text, URLs, `localhost:3000`, `hello@world.com`, and unknown heads all stay normal search.

## Quick Reference

| Input | Mode |
| --- | --- |
| `deploy` | Normal fuzzy search across scripts, built-ins, apps, and passive sources |
| `@selection`, `@file:readme` | Attach context to the prompt you're building |
| `/rewrite` | Slash command |
| `.professional` | Writing style |
| `\|creative` | AI profile |
| `>` , `>:dev` | Working-directory (cwd) picker |
| `todo; buy milk` | Capture (canonical postfix form) |
| `;todo buy milk`, `+todo buy milk` | Capture (prefix form; `+` is a legacy alias) |
| `note: Decision made` | Capture via keyword head (known targets only) |
| `:type:script deploy` | Filtered search |
| `~` / `~/Downloads` | Exit to File Search |
| `!` | Exit to Quick Terminal |
| `?` | Exit to Help |

## Prompt-Builder Sigils

`@`, `/`, `|`, `.`, and `>` each claim the sigil plus one word; everything else stays free text, so you can compose:

```text
>:dev @file:README.md /rewrite .concise explain the setup section
```

- **`@` context mentions** — `@selection`, `@clipboard`, `@file:notes.md`. The `@type:query` form sub-searches inside one source. Available sub-search prefixes include `file`, `project`, `clipboard`, `browser-history`, `notes`, `history` (Agent Chat history), `scripts`, `scriptlets`, `skills`, `dictation`, and `calendar`.
- **`/` slash commands** — pick a command from the slash catalog.
- **`.` styles** — built-in styles are `.professional`, `.concise`, `.friendly`, `.direct`; user-defined styles extend the catalog.
- **`|` profiles** — switch which AI profile handles the prompt.
- **`>` project cwd** — `>` alone opens the cwd picker rows (recents + browse); `>:dev` filters them; `>:~/dev/` browses a directory. The resolved cwd shows as a footer chip. `>name field:value -- args` also invokes a registered command head with argv passed after `--`.

A sigil only counts at a word boundary — `hello@world.com` is plain search.

## Keyboard Behavior

- `Enter` — run what the ↵ label names: the current eligible selected row, or the highlighted picker row when a picker is listed. A stale query or missing row is refused; Enter never silently substitutes a neighboring result.
- `⌘1`–`⌘3` — run one of Jev's readings.
- `Tab` — with typed text: hand the words to an agent (Tab again takes them back). With empty input and subject tabs showing: select the next subject. Type `>` for the working-directory picker. Directory-browse queries keep `Tab` for path completion.
- `Shift+Tab` — select the previous subject when the input is empty and the subject tabs are showing.
- `⌘K` — stage the selected row in the line above the input; the list becomes its actions, in place.
- `⌘↵` — with typed words, **Do It**: run Jev's first reading (same as `⌘1`). With empty input, **Agent**: open the model's chat.
- `↑` on an empty input — select the last run; press again for an earlier one. ⌘V selects the copy. Space stages the selected subject; selecting it alone does not run anything.
- `Escape` — let go of what's staged first, then go back one view, then clear the input, and only then hide the window.
- Lists opened from the launcher (Apps, File Search, Clipboard History, Help, a script's choice prompt) follow their own order: `Escape` first clears what you typed, then goes back, or cancels the script's prompt. Three exceptions leave at once: an action list (`⌘K`) lets go of what's staged, and choosing a working folder after `>` or picking a file or copy to attach returns without clearing.
- The Notes list shows 60 notes at a time; a last "Show N More" row says how many matched in all, and ↵ on it lists the next ones.

### Selection while results arrive

- Before you deliberately choose a row, selection follows the **first eligible result**. An asynchronous result that ranks above the old first row becomes selected automatically.
- Arrow navigation, clicking a row, or an agent selecting its semantic identity anchors that row. New results can change its position without changing your choice. Deliberately choosing the already-first row also anchors it.
- Changing the query restores automatic selection. If an anchored row disappears, selection falls back to the first eligible result; an empty or status-only list has no selected result.
- Empty `@source:` sub-searches are initially unarmed: the first Down arrow selects the first eligible row, not the second. Headers and layout-only slots are never choices.
- Unavailable explanatory choices may remain selectable for inspection, but cannot run.
- Result selection does not take keyboard focus away from the input. Refreshes do not undo deliberate scrolling. The preview, the capsule, ⌘K, and Enter refer to the same current subject.
- While a sentence is typed forward, the ↵ label changes only when what Enter runs truly changes, and it never goes back to a label it already showed.
- `~` File Search keeps its own automatic-first/user-locked-path policy; it is not the launcher's query state.

## Capture

Capture turns the input into a structured local-data composer. The canonical spelling is postfix — type the target, then `;`:

```text
todo; Renew passport #errands p1
note; "Decision: ship parser first" #project
link; https://zed.dev #rust
cal; Design review start:"friday 2pm" for:45m
snippet; update @snippet:fetch-json -- const value = 1
```

Prefix (`;todo ...`), legacy `+todo ...`, and keyword (`todo: ...`) forms parse the same way. A bare `;` (or `+`) lists **capture targets** in the main list (the trigger picker), suggesting targets and offering to scaffold a handler script for a new one.

Built-in targets: `todo`, `note`, `link`, `snippet`, `cal`, `social`. Todo aliases: `reminder`, `snooze`, `defer`. `mcal` (macOS Calendar) is parser-known but hidden from the picker. Unknown heads like `github;` stay plain search until a script registers that target through metadata.

Capture body fields:

| Token | Meaning |
| --- | --- |
| plain words | Body text |
| `#tag` | Tags |
| `p1`–`p4` | Priority |
| `https://...` or `url:<value>` | URL |
| `due:`, `at:`, `start:`, `end:` | Date phrases (quoted phrases allowed: `due:"tomorrow 3pm"`) |
| `for:45m` | Duration |
| `key=value` | Custom metadata |
| `@name` / `@todo:id` | Reference an existing object (e.g. update a snippet) |

Incomplete captures don't run: `todo;` needs a body, `link;` needs a URL, `cal;` needs a body and a date. The hint row tells you what's missing.

## Filtered Search

Type `:` to open the filter picker, or type a complete filter directly. Filters compose with search words:

```text
:type:script deploy
:tag:work notes
:shortcut:any
:has:shortcut
:source:main inbox
:plugin:main.todo
:name:deploy  :desc:database  :alias:db
:meta.domain.kind:calendar
:#work            (sugar for tag:work)
:-type:app triage (leading - negates)
```

Qualifier heads: `type:`/`kind:`, `tag:`, `shortcut:` (`any`, `none`, or a literal like `cmd+k`), `source:`, `plugin:`, `name:`, `desc:`/`description:`, `alias:`, `has:`, and `meta.<path>:`. `type:` values cover scripts, scriptlets, skills, builtins, apps, windows, files, notes, todos, clipboard, tabs, history, conversations, vault, dictation, fallbacks, and issues.

A top-level `#work` (without `:`) remains normal search; inside a capture body it becomes a tag.

> **Planned, not shipped:** unified source heads like `files:`/`f:`, `notes:`/`n:`, `clipboard:`/`c:` are defined in the grammar but marked *planned* in the code. Don't rely on them yet — use `:type:` filters or the dedicated built-ins (File Search, Notes, Clipboard History) instead.

## Mode Exits

Three sigils leave the main-list grammar entirely:

- `~` or `~/path` — open **File Search** rooted at home/that path.
- `!` — open **Quick Terminal** for a shell command.
- `?` — open **Help**: what each key and symbol does. Actions for the highlighted row are on ⌘K.

## Safe Boundaries

- `localhost:3000`, `C#`, `hello!` — normal search.
- `hello world; not a capture` — a `;` after multiple words is not a capture head.
- `https://example.com; nope` — URLs disqualify the capture head.
- Unknown `;target` / `+target` / `target:` heads stay searchable.

When in doubt, type normally first; opt into a mode with a sigil only when you want it.

## Related

- [Feature Tour](./feature-tour.md) — where these modes fit in the app
- [SDK Scripting](./sdk-scripting.md) — registering capture handlers and command heads from scripts
