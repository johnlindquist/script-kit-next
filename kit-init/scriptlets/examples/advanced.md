---
name: Advanced Examples
description: Advanced scriptlet patterns - JSON metadata, schemas, and alternative syntaxes
author: Script Kit
icon: beaker
---

> YAML frontmatter fields `name`, `description`, `author`, and `icon` are bundle-level metadata.
> Use Lucide icon names in kebab-case for `icon` values (for example `beaker`): https://lucide.dev/icons.
> Each `##` heading creates a scriptlet.
> Supported scriptlet metadata includes `keyword` (or `expand` / `snippet`) for triggers, `shortcut` for hotkeys, `alias` for alternates, `schedule` / `cron` for automation, `icon` for per-scriptlet icons, and boolean flags like `hidden`, `background`, and `system`.

# JSON Metadata Format

The metadata block also accepts JSON when a scriptlet needs structured values or quoted strings.

## JSON Config Example

Use JSON metadata when the YAML-style block would be less clear.

```metadata
{
  "keyword": "!json",
  "description": "Demonstrates JSON metadata format",
  "alias": "jsonex",
  "icon": "braces"
}
```

```paste
This scriptlet uses JSON metadata format.
```

---

# Alternative Variable Syntax

Scriptlet templates accept both `${var}` and `{{var}}` variable markers.

## Mustache Style Variables

Use mustache-style variables when they fit better with surrounding text.

```metadata
keyword: !mv
description: Uses {{var}} syntax instead of ${var}
```

```paste
Date: {{date}}
Time: {{time}}
Clipboard: {{clipboard}}
```
