# CleanShot Actions

Shared actions for CleanShot X screenshot tools.

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
