# Focus Badge

A tiny Badgeware focus dashboard backed by a markdown file.

## Install

Copy the `focus_badge` folder into the badge's `/apps` folder while the badge is in disk mode:

```text
/apps/focus_badge/
  icon.png
  __init__.py
  config.py
  tasks.md
  /assets
```

Badgeware will show the app as "Focus Badge" in the launcher.

## Markdown Database

Edit `tasks.md` locally on the badge, or set `GIST_RAW_URL` in `config.py` to a raw Gist markdown URL and press `C` in the app to sync.

Supported metadata:

- `@25m` sets the focus timer duration.
- `due:09:30` or `at:09:30` makes the task show as scheduled.
- `#tag` is shown as task context.
- `!` marks a high-priority task.
- `- [x]` is ignored by the badge's active list.

Example:

```md
- [ ] Ship the badge app @25m due:09:30 #coding !
- [ ] Walk and think @15m due:10:15 #break
```

## Buttons

- `Up` / `Down`: choose the current task.
- `A`: start or pause the timer.
- `B`: mark the current task done locally.
- `C`: sync `tasks.md` from `GIST_RAW_URL`.

The app writes local session progress to `state.json`. It does not write back to the Gist, so you can keep the Gist token-free.
