# URL Actions

Shared actions for all Quick Links scriptlets.
The `{{content}}` variable contains the parent scriptlet's code (the URL for `open` tool).

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
