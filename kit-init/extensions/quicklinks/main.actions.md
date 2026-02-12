# URL Actions
<!--
Quick Links actions notes:
- Each `###` heading defines a shared action available to all scriptlets
  in `main.md`.
- `{{content}}` is replaced with the selected parent scriptlet's
  fenced code content.
- Actions can include their own metadata comments
  (for example `description` and `shortcut`).
-->

Shared actions for all Quick Links scriptlets.
The `{{content}}` variable contains the parent scriptlet's code
(the URL for `open` tool).

<!--
How companion .actions files work:
- File name must match the parent bundle base name: `main.md` + `main.actions.md`.
- Each `###` heading defines one action command.
- Actions in this file are available from every scriptlet in the parent bundle.
- `{{content}}` is replaced with the selected parent scriptlet's
  fenced code content at runtime.
- Optional action metadata can be set with HTML comments
  (for example `description` and `shortcut`) when supported.
-->

## Shared Actions

### Copy URL
<!-- shortcut: cmd+c -->
<!-- description: Copy the URL to clipboard -->
```bash
echo -n "{{content}}" | pbcopy
```

### Open in Safari
<!-- shortcut: cmd+shift+s -->
<!-- description: Open URL in Safari -->
```bash
open -a Safari "{{content}}"
```

### Open in Firefox
<!-- description: Open URL in Firefox -->
```bash
open -a Firefox "{{content}}"
```

### Open in Chrome
<!-- description: Open URL in Chrome -->
```bash
open -a "Google Chrome" "{{content}}"
```

### Open in Private Window
<!-- shortcut: cmd+shift+p -->
<!-- description: Open URL in private/incognito window -->
```bash
open -a Safari -n --args -private "{{content}}"
```
