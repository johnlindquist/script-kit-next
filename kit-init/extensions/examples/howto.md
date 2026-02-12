---
name: How To Create Scriptlets
description: Learn how to create your own text expansions and automations
author: Script Kit
icon: graduation-cap
---

> Frontmatter fields define extension metadata: `name`, `description`, `author`, `icon`.
> Use Lucide icon names in kebab-case for `icon` (for example `graduation-cap`): https://lucide.dev/icons.
> Each scriptlet uses a `metadata` fenced block before its tool block, in either `key: value` format or JSON.
> Supported scriptlet metadata fields include `keyword` (also accepted as `expand` or `snippet`), `description`, `alias`, `shortcut`, `icon`, `schedule`, `cron`, `hidden`, and `background`.

# How to Create Your Own Scriptlets

This guide explains how to create custom text expansions for Script Kit.

---

# File Location

Create `.md` files in: `~/.scriptkit/kit/YOUR-KIT-NAME/extensions/`

Example paths:
- `~/.scriptkit/kit/main/extensions/snippets.md`
- `~/.scriptkit/kit/work/extensions/emails.md`
- `~/.scriptkit/kit/personal/extensions/signatures.md`

---

# Bundle Frontmatter (Optional)

Add YAML frontmatter at the top of your file:

```yaml
---
name: My Scriptlets
description: My personal text expansions
author: Your Name
icon: star
---
```

Available fields: `name`, `description`, `author`, `icon`

`icon` should be a Lucide icon name in kebab-case (for example `star`, `book-open`, `graduation-cap`).

---

# Scriptlet Structure

Each scriptlet follows this pattern:

```markdown
## Scriptlet Name

```metadata
keyword: !trigger
description: What it does
```

```paste
The text to expand
```
```

---

# Metadata Fields

| Field | Description | Example |
|-------|-------------|---------|
| `keyword` | Trigger text (required for expansion) | `!sig`, `:email`, `;addr` |
| `description` | Short description | `Insert email signature` |
| `shortcut` | Keyboard shortcut | `cmd shift s` |
| `alias` | Alternative trigger | `sig` |
| `icon` | Optional per-scriptlet icon | `mail`, `calendar` |
| `schedule` | Human-friendly schedule shorthand (where supported) | `daily`, `weekday 09:00` |
| `cron` | Cron expression schedule (where supported) | `0 9 * * 1-5` |
| `(boolean flags)` | Feature-specific true/false options | `passive: true` |

`metadata` can be either key/value lines or JSON:

```metadata
keyword: !trigger
description: Key/value format
shortcut: cmd shift t
```

```metadata
{
  "keyword": "!json",
  "description": "JSON format",
  "alias": "j"
}
```

---

# Template Variables

| Variable | Output | Syntax |
|----------|--------|--------|
| `clipboard` | Clipboard contents | `${clipboard}` |
| `date` | 2026-01-11 | `${date}` |
| `date_long` | January 11, 2026 | `${date_long}` |
| `date_short` | 01/11/2026 | `${date_short}` |
| `time` | 14:30:45 | `${time}` |
| `time_12h` | 2:30 PM | `${time_12h}` |
| `datetime` | 2026-01-11 14:30:45 | `${datetime}` |
| `day` | Saturday | `${day}` |
| `month` | January | `${month}` |
| `year` | 2026 | `${year}` |
| `timestamp` | Unix seconds | `${timestamp}` |

Both `${var}` and `{{var}}` syntax work.

---

# Tool Types

The code fence language determines what happens:

| Tool | Behavior |
|------|----------|
| `paste` | Pastes text directly |
| `ts` | Runs TypeScript |
| `bash` | Runs shell command |
| `template` | Expands template |

---

# Legacy Format

HTML comments also work:

```markdown
## My Scriptlet

<!--
keyword: !trigger
description: What it does
-->

```paste
Expansion text
```
```

---

# Quick Start Template

Copy this to create your first scriptlet:

```markdown
---
name: My Snippets
description: Personal text expansions
author: Me
icon: zap
---

## Email Sign-off

```metadata
keyword: !bye
description: Quick email sign-off
```

```paste
Thanks,
Your Name
```
```

Save as: `~/.scriptkit/kit/main/extensions/my-snippets.md`

---

# Tips

1. Keywords must start with `!`, `:`, `;`, or similar
2. Keep keywords short but memorable
3. Use `${clipboard}` to transform clipboard content
4. File changes auto-reload (no restart needed)
5. Test keywords in any text field
