---
name: Examples
description: Complete reference for scriptlet patterns - keyword triggers, text expansion, and template variables
author: Script Kit
icon: book-open
---

> YAML frontmatter fields `name`, `description`, `author`, and `icon` are bundle-level metadata.
> Use Lucide icon names in kebab-case for `icon` values (for example `book-open`): https://lucide.dev/icons.
> Each `##` heading creates a scriptlet.
> Supported scriptlet metadata includes `keyword` (or `expand` / `snippet`) for triggers, `shortcut` for hotkeys, `alias` for alternates, `schedule` / `cron` for automation, `icon` for per-scriptlet icons, and boolean flags like `hidden`, `background`, and `system`.

# Getting Started

Type a keyword anywhere (e.g., `!sig`) and Script Kit will replace it with the expansion text.

## Hello World

```metadata
keyword: !hello
description: Your first scriptlet
```

```paste
Hello, World!
```

---

# Text Expansions

## Email Signature

```metadata
keyword: !sig
description: Professional email signature
```

```paste
Best regards,
John

Script Kit - https://scriptkit.com
```

---

## Quick Thank You

```metadata
keyword: !ty
description: Quick thank you
```

```paste
Thank you! I really appreciate your help.
```

---

## Meeting Link

```metadata
keyword: !zoom
description: Meeting link
```

```paste
Join my meeting: https://zoom.us/j/your-meeting-id
```

---

## Mailing Address

```metadata
keyword: !addr
description: Mailing address
```

```paste
123 Main Street
Anytown, ST 12345
```

---

# Date & Time Variables

Use `${variable}` syntax for dynamic values.

## Current Date (ISO)

```metadata
keyword: !date
description: Today's date YYYY-MM-DD
```

```paste
${date}
```

---

## Current Date (Long)

```metadata
keyword: !datel
description: Today's date long format
```

```paste
${date_long}
```

---

## Current Time

```metadata
keyword: !time
description: Current time HH:MM:SS
```

```paste
${time}
```

---

## Time (12-hour)

```metadata
keyword: !time12
description: Current time with AM/PM
```

```paste
${time_12h}
```

---

## Full DateTime

```metadata
keyword: !dt
description: Date and time
```

```paste
${datetime}
```

---

## Day of Week

```metadata
keyword: !day
description: Day name
```

```paste
${day}
```

---

# Clipboard Variables

Use `${clipboard}` to include clipboard contents.

## Clipboard Contents

```metadata
keyword: !cb
description: Paste clipboard
```

```paste
${clipboard}
```

---

## Clipboard Quoted

```metadata
keyword: !quote
description: Clipboard in quotes
```

```paste
"${clipboard}"
```

---

## Clipboard as Code

```metadata
keyword: !code
description: Clipboard as code block
```

```paste
```
${clipboard}
```
```

---

## All Variables Demo

```metadata
keyword: !vars
description: Show all template variables
```

```paste
DATE: ${date} | ${date_long} | ${date_short}
TIME: ${time} | ${time_12h}
DATETIME: ${datetime}
COMPONENTS: ${year}-${month_num}-${day_num} ${hour}:${minute}
CLIPBOARD: ${clipboard}
```

---

# Code Snippets

## Console Log

```metadata
keyword: !log
description: console.log statement
```

```paste
console.log('DEBUG:', );
```

---

## Arrow Function

```metadata
keyword: !arrow
description: Arrow function
```

```paste
const fn = () => {

}
```

---

## Try-Catch

```metadata
keyword: !try
description: Try-catch block
```

```paste
try {

} catch (error) {
  console.error('Error:', error)
}
```

---

## Async Function

```metadata
keyword: !async
description: Async function
```

```paste
async function example() {

}
```

---

# Emoticons

## Shrug

```metadata
keyword: !shrug
description: Shrug emoticon
```

```paste
¯\_(ツ)_/¯
```

---

## Table Flip

```metadata
keyword: !flip
description: Table flip
```

```paste
(╯°□°)╯︵ ┻━┻
```

---

## Lenny

```metadata
keyword: !lenny
description: Lenny face
```

```paste
( ͡° ͜ʖ ͡°)
```

---

# Templates

## Daily Standup

```metadata
keyword: !standup
description: Standup template
```

```paste
## Standup - ${date_long}

**Yesterday:**
**Today:**
**Blockers:** None
```

---

## Meeting Notes

```metadata
keyword: !meeting
description: Meeting notes template
```

```paste
# Meeting - ${date_long}

**Attendees:**
**Action Items:**
- [ ]

**Notes:**
```

---

# Legacy Format (HTML Comments)

HTML comment metadata also works for backwards compatibility.

## Thumbs Up

<!--
keyword: !thumbsup
description: Thumbs up (legacy format)
-->

```paste
👍
```

---

## Heart

<!--
keyword: !heart
description: Heart (legacy format)
-->

```paste
❤️
```
