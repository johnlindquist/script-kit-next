---
name: CleanShot X
description: Screenshot and screen recording tools for macOS
author: Script Kit
icon: camera
---

> Maintainer notes:
>
> - YAML frontmatter fields: `name` (display name), `description` (picker summary), `author` (attribution), `icon` (Lucide icon name such as `camera` or `monitor`).
> - `icon` should use the Lucide icon identifier only (no `lucide-` prefix).
> - Scriptlet convention: each `##` heading defines one scriptlet command and is followed by its code fence.
> - The code fence language selects the tool runner (`open`, `bash`, `ts`, `applescript`, etc.).

# CleanShot X

Powerful screenshot and screen recording commands for CleanShot X.

---

## All in One

<!--
description: Open CleanShot X all-in-one capture mode
-->

```open
cleanshot://all-in-one
```

---

## Capture Area

<!--
description: Capture a selected area of the screen
-->

```open
cleanshot://capture-area
```

---

## Capture Window

<!--
description: Capture a specific window
-->

```open
cleanshot://capture-window
```

---

## Capture Fullscreen

<!--
description: Capture the entire screen
-->

```open
cleanshot://capture-fullscreen
```

---

## Capture Previous Area

<!--
description: Capture the same area as your last capture
-->

```open
cleanshot://capture-previous-area
```

---

## Scrolling Capture

<!--
description: Capture scrolling content (long pages, chats, etc.)
-->

```open
cleanshot://scrolling-capture
```

---

## Record Screen

<!--
description: Start screen recording
-->

```open
cleanshot://record-screen
```

---

## Self Timer

<!--
description: Capture with a countdown timer
-->

```open
cleanshot://self-timer
```

---

## Pin Screenshot

<!--
description: Pin a screenshot as an overlay on screen
-->

```open
cleanshot://pin
```

---

## Annotate

<!--
description: Open the annotation editor
-->

```open
cleanshot://open-annotate
```

---

## Capture Text (OCR)

<!--
description: Capture and recognize text from screen
-->

```open
cleanshot://capture-text
```

---

## Open History

<!--
description: View your screenshot history
-->

```open
cleanshot://open-history
```

---

## Restore Recently Closed

<!--
description: Restore the most recently closed capture
-->

```open
cleanshot://restore-recently-closed
```

---

## Add Quick Access Overlay

<!--
description: Show the quick access overlay
-->

```open
cleanshot://add-quick-access-overlay
```

---

## Show Desktop Icons

<!--
description: Toggle desktop icons visibility
-->

```open
cleanshot://toggle-desktop-icons
```

---

## Capture Area for AI

<!--
description: Capture an area (use with Raycast AI or paste into AI chat)
-->

```open
cleanshot://capture-area
```
