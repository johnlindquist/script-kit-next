# CleanShot Actions

Shared actions for CleanShot X screenshot tools.

> Maintainer notes:
>
> - Each `###` heading defines one shared action available to scriptlets in `main.md`.
> - `{{content}}` is substituted with the selected scriptlet's code fence content before the action runs.
> - Optional per-action metadata can be set as HTML comments above the action code fence, for example: `<!-- description: Copy URL -->`, `<!-- shortcut: cmd+c -->`, `<!-- keyword: copy -->`.
> - The code fence language selects which tool executes the action (`bash`, `open`, `ts`, `applescript`, etc.).

### Copy URL Scheme

<!-- shortcut: cmd+c -->
<!-- description: Copy the CleanShot URL scheme to clipboard -->

```bash
echo -n "{{content}}" | pbcopy
```

### Open CleanShot Settings

<!-- description: Open CleanShot X preferences -->

```open
cleanshot://preferences
```

### Show in Menu Bar

<!-- description: Click CleanShot menu bar icon -->

```applescript
tell application "System Events"
    tell process "CleanShot X"
        click menu bar item 1 of menu bar 2
    end tell
end tell
```
