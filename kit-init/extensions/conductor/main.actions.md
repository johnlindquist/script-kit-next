# Conductor Actions

Shared actions for Conductor integration tools.

### Copy URL
<!-- shortcut: cmd+c -->
<!-- description: Copy the URL or path to clipboard -->
```bash
echo -n "{{content}}" | pbcopy
```

### Open in Browser
<!-- shortcut: cmd+o -->
<!-- description: Open URL in default browser -->
```bash
open "{{content}}"
```

### Open Conductor App
<!-- description: Launch Conductor application -->
```open
conductor://
```

### Open Conductor Docs
<!-- description: Open Conductor documentation -->
```open
https://docs.conductor.build
```
