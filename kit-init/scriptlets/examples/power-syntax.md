---
name: Power Syntax Examples
description: Local-first scriptlets for the Power Syntax > command demos
author: Script Kit
icon: terminal
---

# Power Syntax

These scriptlets are intentionally local and inspectable. They pair with the
`scripts/examples/menu-syntax/` demo scripts and write only under
`$SK_PATH/menu-syntax/`.

## PS Stamp

```metadata
description: Append a local stamp from !ps-stamp command metadata
alias: power-stamp
```

```bash
set -eu

ROOT="${SK_PATH:-${HOME:-.}/.scriptkit}/menu-syntax/scriptlets"
mkdir -p "$ROOT"

printf '{"createdAt":"%s","head":"%s","fields":%s,"tags":%s}\n' \
  "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  "${KIT_MENU_SYNTAX_COMMAND_HEAD:-}" \
  "${KIT_MENU_SYNTAX_COMMAND_FIELDS:-[]}" \
  "${KIT_MENU_SYNTAX_COMMAND_TAGS:-[]}" \
  >> "$ROOT/ps-stamp.jsonl"
```

---

## PS Dupe

```metadata
description: Intentionally collides with the ps-dupe script command
alias: power-dupe
```

```bash
set -eu

ROOT="${SK_PATH:-${HOME:-.}/.scriptkit}/menu-syntax/scriptlets"
mkdir -p "$ROOT"

printf '{"createdAt":"%s","warning":"duplicate scriptlet unexpectedly ran"}\n' \
  "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  >> "$ROOT/ps-dupe-scriptlet-ran.jsonl"
```

---

# `menuSyntax` author reference

Scripts and scriptlets opt into Power Syntax handling by declaring
`menuSyntax` in their metadata. The shapes below are the canonical templates
script and scriptlet authors copy-paste — each mirrors a variant of the
Rust `MenuSyntaxHandlerSpec` (src/menu_syntax/payload.rs) so the launcher
parses them without surprise.

The reference uses bold-text headings (not `##` H2s) so the scriptlet loader
does not mistake these template sections for executable scriptlets — only
the `## PS Stamp` / `## PS Dupe` H2 sections above ship as runnable scriptlets.

**Capture handler template (`;target body…`)**

A capture handler claims one or more `;target` slugs and tells the launcher
which payload tokens it understands. The `accepts` list is a hint — the
launcher's own field-schema (src/menu_syntax/capture_schema.rs) decides
which fields are *required* before Enter ships the payload.

    # Paste under metadata.menuSyntax in a TypeScript script,
    # or under a scriptlet's `metadata` codefence once nested YAML
    # in scriptlet metadata is supported (see caveat below).
    - family: capture.v1            # only "capture.v1" today
      targets:
        - link                      # the ;link / link: head
        # - "*"                     # opt into every ;target in this family
      accepts:
        - tags                      # #tag tokens
        - url                       # url:"…" tokens
        - kv                        # key=value pairs
      label: Save tagged link       # shown in the handler picker row
      payloadSchema: kit://schema/menu-syntax/payload-v1
      defaultHandler: true          # win the rank-tie among handlers for "link"
      kvEnums:                       # Run 14: per-key enum overrides
        env: [prod, staging, dev]    #   typing `;link foo env:` ranks
        priority: [P0, P1, P2]       #   declared values FIRST in the popup;
                                     #   history-only values (e.g. "custom")
                                     #   appear after, dimmed with a
                                     #   "previously used" badge. Empty or
                                     #   absent kvEnums → pure-history popup.

**Command handler template (`>head -- argv…`)**

A command handler binds a `>head` invocation to your script. The optional
`args` / `flags` / `usage` block lets the launcher render hint rows in the
main-menu hint card (src/menu_syntax/main_hint.rs) so authors can see
expected arguments before running.

    - family: command.v1
      head: deploy                  # the >deploy bare slug, no leading >
      label: Deploy a service
      description: Run a guarded production deploy
      args:
        - name: env
          required: true
          values: [prod, staging, dev]
      flags:
        - name: --dry-run
          alias: -n
          description: Print the plan without applying
      usage: ">deploy -- <env> [--dry-run]"

**Skill handler template (`/slug` AI route)**

A skill handler registers a `/slug` AI route that surfaces in `:type:skill`
filters but does NOT auto-bind a `>command` head — keep them separate so
authors can choose to expose only one surface. `contextRequirements` is
how the launcher decides whether a skill is relevant to the current
selection / frontmost app.

    - family: skill.v1
      slug: review                  # /review (no leading slash)
      label: Review current file
      description: AI review of the active editor file
      contextRequirements:
        - selection.file
        - frontmost.app
      # acceptsCaptureTarget: note  # optionally chain a capture target as input

**Scriptlet flat-metadata caveat**

The scriptlet `metadata` codefence currently parses only `key: value`
flat strings (src/scriptlet_metadata/mod.rs::parse_simple_metadata). To
attach a `menuSyntax` spec to a scriptlet today, declare a sibling script
(in `scripts/examples/menu-syntax/*.ts`) that exports `metadata.menuSyntax`
and lets the scriptlet handle the side effect — see the existing
`save-tagged-link.ts` + `ps-stamp` pairing above for the pattern.
A future story (`sdk-command-schema`) will lift this restriction by
extending the scriptlet metadata parser to accept nested YAML.

**macOS Calendar `;mcal` walkthrough**

The `;mcal` example demonstrates natural-language dates, ranges, durations,
recurrence, and calendar-specific key/value enums without adding another
scriptlet section to this markdown file.

    ;mcal Lunch with Ryan tomorrow at 12pm til 1pm
    ;mcal Lunch with Ryan tom 12pm for 30mins
    ;mcal Lunch w/ Ryan every mon from 1 til 2

The corresponding TypeScript script declares this full `menuSyntax` block:

    menuSyntax:
      - family: capture.v1
        targets:
          - mcal
        accepts:
          - tags
          - date
          - dateRange
          - duration
          - recurrence
          - kv
        required:
          - body
          - date
        label: Add event to macOS Calendar
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true
        kvEnums:
          calendar: [Home, Work, Personal, Family]
          alarm: ["0", "5", "15", "30", "60"]

When choosing an event end time, handlers should prefer `dates[].endIso`
from a parsed range, then `durationResolved` from phrases like `for 30mins`,
then the legacy raw `duration` string, and finally their own default.

**Todo `;todo` capture walkthrough**

The `;todo` example demonstrates anchor-only due dates, priorities, tags, and
weekday recurrence phrases for inbox-style todo capture.

    ;todo Renew passport tomorrow
    ;todo Submit form by friday p1
    ;todo Daily standup every weekday at 9am

The corresponding TypeScript script declares this `menuSyntax` block:

    menuSyntax:
      - family: capture.v1
        targets:
          - todo
        accepts:
          - tags
          - date
          - priority
        label: Add todo
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true

**Todo scheduling compatibility aliases**

The canonical public target for todo capture is `;todo`. Older inputs using
`;reminder`, `;snooze`, and `;defer` are still accepted for compatibility, but
they resolve to the built-in Todo system with specialized operations instead of
being advertised as separate product targets. Existing handlers can inspect the
payload `source.rawTarget`, `source.canonicalTarget`, and `source.operation`
fields when they need to distinguish a compatibility input from canonical todo
capture.

**Object refs in app-owned capture**

App-owned capture targets can carry inline object references in the control
part of the command. Typed refs resolve immediately into `objectRefs[]` and
`primaryObjectRef`; bare refs stay unresolved so a future picker/search UI can
take over without changing the payload shape.

    ;note @Project due:tomorrow
    ;snippet update @snippet:fetch-json lang:ts -- const data = await fetch(url)

For `;note @Project due:tomorrow`, the note capture body is empty because
`@Project` is selector context and `due:tomorrow` is parsed date context. For
`;snippet update @snippet:fetch-json -- ...`, the app-owned snippet writer uses
the resolved snippet ref as the target trigger when `trigger:` is omitted.
Snippet body text after `--` is not scanned for object refs.

**Link `;link` capture walkthrough**

`;link` is app-owned bookmark capture. It upserts links into
`$SK_PATH/menu-syntax/bookmarks.jsonl` and preserves parsed object refs for
future relation-aware browsing.

    ;link https://example.com title:"Example"
    ;link save https://example.com #docs
    ;link delete https://example.com

Handlers that want to mirror the app-owned shape can declare:

    menuSyntax:
      - family: capture.v1
        targets:
          - link
        accepts:
          - tags
          - kv
        required:
          - body
        label: Save link
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true

**Snippet `;snippet` capture walkthrough**

`;snippet` is app-owned quick snippet capture. It writes JSONL records under
`$SK_PATH/menu-syntax/snippets.jsonl`; `add`, `update`, and `remove` map to
create/update/delete operations.

    ;snippet add trigger:fj lang:ts -- const res = await fetch(url)
    ;snippet update @snippet:fj lang:ts -- const data = await fetch(url)
    ;snippet remove @snippet:fj

Handlers that want the same payload contract can declare:

    menuSyntax:
      - family: capture.v1
        targets:
          - snippet
        accepts:
          - tags
          - kv
        required:
          - body
        label: Save snippet
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true
