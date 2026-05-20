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

**Reminder `;reminder` capture walkthrough**

`;reminder` is now a Todo-owned compatibility alias. It resolves to the
canonical `todo` target with the `remind` operation while preserving the raw
target in the capture payload for older handlers.

    ;reminder Walk dog every day at 8am

The app-owned grammar resolves the alias as if the handler had declared this
canonical `todo` shape:

    menuSyntax:
      - family: capture.v1
        targets:
          - todo
        accepts:
          - tags
          - date
          - duration
          - recurrence
          - priority
        required:
          - body
        label: Todo reminder
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true

**Snooze `;snooze` capture walkthrough**

`;snooze` is now a Todo-owned compatibility alias. It resolves to the canonical
`todo` target with the `snooze` operation and keeps numeric issue references
such as `#432` in the body instead of treating them as tags.

    ;snooze in 30 minutes Review PR #432

The app-owned grammar resolves the alias as if the handler had declared this
canonical `todo` shape:

    menuSyntax:
      - family: capture.v1
        targets:
          - todo
        accepts:
          - tags
          - date
          - relativeDate
          - duration
        required:
          - body
          - date
        label: Todo snooze
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true

**Defer `;defer` capture walkthrough**

`;defer` is now a Todo-owned compatibility alias. It resolves to the canonical
`todo` target with the `defer` operation for fuzzy future scheduling.

    ;defer until next week Refactor settings panel

The app-owned grammar resolves the alias as if the handler had declared this
canonical `todo` shape:

    menuSyntax:
      - family: capture.v1
        targets:
          - todo
        accepts:
          - tags
          - date
          - relativeDate
          - priority
        required:
          - body
          - date
        label: Todo defer
        payloadSchema: kit://schema/menu-syntax/payload-v1
        defaultHandler: true
