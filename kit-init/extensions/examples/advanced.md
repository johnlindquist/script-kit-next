---
name: Advanced Examples
description: Advanced scriptlet patterns - JSON metadata, schemas, and alternative syntaxes
author: Script Kit
icon: beaker
---

## Inline Reference

- Frontmatter fields define the extension bundle:
- `name`: display name in Script Kit.
- `description`: summary text for the bundle.
- `author`: bundle author/owner.
- `icon`: Lucide icon name in kebab-case (example: `beaker`) from https://lucide.dev/icons.
- Scriptlet settings go in a ```metadata``` fenced block before each tool block.
- `metadata` supports both formats shown in this file:
- key/value lines (for example `keyword: !short`).
- JSON objects (for example `{ "keyword": "!json" }`).
- Common metadata fields include `keyword` (aliases: `expand`, `snippet`), `description`, `alias`, `shortcut`, `icon`, `schedule`, `cron`, plus `hidden` and `background` booleans (`true`/`false`).

# JSON Metadata Format

The metadata block also accepts JSON syntax for complex configurations.

## JSON Config Example

```metadata
{
  "keyword": "!json",
  "description": "Demonstrates JSON metadata format",
  "alias": "jsonex"
}
```

```paste
This scriptlet uses JSON metadata format!
```

---

# Alternative Variable Syntax

Both `${var}` and `{{var}}` syntaxes work identically.

## Mustache Style Variables

```metadata
keyword: !mustache
description: Uses {{var}} syntax instead of ${var}
```

```paste
Date: {{date}}
Time: {{time}}
Clipboard: {{clipboard}}
```

---

## Mixed Syntax

```metadata
keyword: !mixed
description: Both syntaxes in same template
```

```paste
Dollar syntax: ${date}
Mustache syntax: {{time}}
Both work the same way!
```

---

# Shortcut and Alias

Scriptlets can have keyboard shortcuts and aliases for quick access.

## With Shortcut

```metadata
keyword: !short
description: Has a keyboard shortcut
shortcut: cmd shift e
```

```paste
This scriptlet has a keyboard shortcut: Cmd+Shift+E
```

---

## With Alias

```metadata
keyword: !aliased
description: Can be triggered by alias too
alias: al
```

```paste
Triggered by !aliased or by typing "al" in the menu
```

---

# Extended Date/Time Variables

All available date and time template variables.

## ISO 8601 Date

```metadata
keyword: !dateiso
description: ISO 8601 formatted date
```

```paste
${date_iso}
```

---

## Short Time

```metadata
keyword: !timeshort
description: Time without seconds
```

```paste
${time_short}
```

---

## Month Number

```metadata
keyword: !monthnum
description: Numeric month
```

```paste
${month_num}
```

---

## Day Number

```metadata
keyword: !daynum
description: Day of month
```

```paste
${day_num}
```

---

## Weekday

```metadata
keyword: !weekday
description: Short weekday name
```

```paste
${weekday}
```

---

## Hour/Minute/Second

```metadata
keyword: !hms
description: Individual time components
```

```paste
Hour: ${hour}
Minute: ${minute}
Second: ${second}
```

---

## Unix Timestamp

```metadata
keyword: !unix
description: Unix timestamp in seconds
```

```paste
${timestamp}
```

---

# Clipboard Transformations

More examples of clipboard manipulation.

## Uppercase Clipboard

Uses clipboard content with surrounding text.

```metadata
keyword: !upper
description: Clipboard with UPPER label
```

```paste
CONTENT: ${clipboard}
```

---

## Markdown Link from Clipboard

Assumes clipboard contains a URL.

```metadata
keyword: !mdlink
description: Create markdown link from clipboard URL
```

```paste
[Link Text](${clipboard})
```

---

## HTML Link from Clipboard

```metadata
keyword: !htmllink
description: Create HTML anchor from clipboard URL
```

```paste
<a href="${clipboard}">Link Text</a>
```

---

# Multi-line Templates

Complex multi-line expansions with variables.

## PR Description

```metadata
keyword: !prdesc
description: Pull request description template
```

```paste
## Summary
Brief description of changes.

## Changes
-

## Testing
- [ ] Unit tests pass
- [ ] Manual testing completed

## Date
${date_long}
```

---

## Changelog Entry

```metadata
keyword: !changelog
description: Changelog entry template
```

```paste
## [Unreleased] - ${date}

### Added
-

### Changed
-

### Fixed
-
```

---

## Git Commit

```metadata
keyword: !commit
description: Conventional commit template
```

```paste
feat:

Refs: #
```

---

# Edge Cases

## Empty Expansion

```metadata
keyword: !empty
description: Expands to nothing (cursor placeholder)
```

```paste

```

---

## Special Characters

```metadata
keyword: !special
description: Contains special characters
```

```paste
Quotes: "double" and 'single'
Backslash: \path\to\file
Angle brackets: <tag>
Ampersand: A & B
Dollar sign (literal): $100
```

---

## Unicode Content

```metadata
keyword: !unicode
description: Unicode characters
```

```paste
Japanese: こんにちは
Emoji: 🚀 🎉 ✨
Symbols: © ® ™ § ¶
Math: ∑ ∏ ∫ √ ∞
Arrows: → ← ↑ ↓ ⇒
```

---

# Keyword Prefix Styles

Different trigger prefix conventions.

## Colon Prefix

```metadata
keyword: :colon
description: Uses colon prefix style
```

```paste
Triggered with :colon
```

---

## Semicolon Prefix

```metadata
keyword: ;semi
description: Uses semicolon prefix style
```

```paste
Triggered with ;semi
```

---

## Double Bang

```metadata
keyword: !!double
description: Uses double bang prefix
```

```paste
Triggered with !!double
```

---

## Slash Prefix

```metadata
keyword: /slash
description: Uses slash prefix style
```

```paste
Triggered with /slash
```

---

# Keyword Aliases

The `keyword` field has aliases: `expand` and `snippet`.

## Using "expand" alias

```metadata
expand: !expand
description: Uses expand instead of keyword
```

```paste
The "expand" alias works the same as "keyword"
```

---

## Using "snippet" alias

```metadata
snippet: !snippet
description: Uses snippet instead of keyword
```

```paste
The "snippet" alias also works the same as "keyword"
```

---

# Whitespace Handling

## Trailing Newline

```metadata
keyword: !trail
description: Content with trailing newline
```

```paste
Line 1
Line 2
Line 3
```

---

## Indented Content

```metadata
keyword: !indent
description: Preserves indentation
```

```paste
function example() {
    if (true) {
        console.log("indented");
    }
}
```

---

## Tabs

```metadata
keyword: !tabs
description: Content with tabs
```

```paste
Col1	Col2	Col3
A	B	C
```

---

# Tool Types

Scriptlets support different tool types via the code fence language.

## Open Tool (URLs)

The `open` tool opens URLs in the default browser.

```metadata
description: Open Script Kit website
```

```open
https://scriptkit.com
```

---

## Open Tool (URL Schemes)

URL schemes trigger native apps (like cleanshot://, raycast://, etc.).

```metadata
description: Open Finder Downloads
```

```open
file:///Users/Shared
```

---

## Open Tool with Keyword

Combine `open` with a keyword trigger to quickly open URLs.

```metadata
keyword: !docs
description: Open Script Kit docs
```

```open
https://scriptkit.com/docs
```

---

## Open Tool - GitHub

```metadata
keyword: !gh
description: Open GitHub
```

```open
https://github.com
```

---

# Schema Blocks (Input Validation)

Schema blocks define input validation (parsed but not executed).

## With Input Schema

```metadata
keyword: !withschema
description: Has input schema defined
```

```schema
{
  "input": {
    "name": "string",
    "count": "number"
  }
}
```

```paste
Schema validated input would go here
```
