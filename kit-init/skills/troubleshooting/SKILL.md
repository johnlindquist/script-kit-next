---
name: troubleshooting
description: Diagnose and fix common Script Kit issues. Use when the user reports bugs, crashes, missing features, or unexpected behavior in Script Kit GPUI.
---

# Troubleshooting

Diagnose and fix common Script Kit issues.

## Log Files

```
~/.scriptkit/logs/script-kit-gpui.jsonl
```

JSONL format — each line is a JSON object with `timestamp`, `level`, `message`, and optional fields.

### Reading Logs

```bash
# Last 50 lines
tail -50 ~/.scriptkit/logs/script-kit-gpui.jsonl

# Filter errors
grep '"level":"ERROR"' ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -20

# Filter by module
grep '"target":"script_kit"' ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -20
```

### Compact AI Log Mode

Set `SCRIPT_KIT_AI_LOG=1` for compact, human-readable log output (useful for debugging):

```bash
SCRIPT_KIT_AI_LOG=1 ~/.scriptkit/cache/Script\ Kit.app/Contents/MacOS/script-kit-gpui
```

## Common Issues

### Script Not Appearing in Menu

1. **Check file location**: Must be in `~/.scriptkit/kit/main/scripts/*.ts`
2. **Check metadata export**: Must have `export const metadata = { name: "..." }`
3. **Check syntax**: Run `bun check ~/.scriptkit/kit/main/scripts/your-script.ts`
4. **Check logs**: Look for parse errors in the log file

### Script Crashes on Run

1. **Check SDK import**: First line must be `import "@scriptkit/sdk";`
2. **Check Bun availability**: Run `which bun` in terminal
3. **Check for Node.js patterns**: Replace CommonJS imports, `fs.readFile`, `child_process` with Bun equivalents
4. **Check logs** for the error stack trace

### Hotkey Not Working

1. **Check config syntax**: Validate `~/.scriptkit/kit/config.ts` has correct hotkey format
2. **Check for conflicts**: Another app may have claimed the hotkey
3. **Try a different hotkey**: Some key combinations are reserved by macOS
4. **Restart Script Kit**: Close and reopen the app

### Theme Not Applying

1. **Validate JSON**: Run `cat ~/.scriptkit/kit/theme.json | python3 -m json.tool`
2. **Check color format**: Use `"#RRGGBB"`, `"rgb(r,g,b)"`, or `"rgba(r,g,b,a)"`
3. **Save the file**: Theme reloads on save, not on edit

### Extensions Not Loading

1. **Check file location**: Must be in `~/.scriptkit/kit/main/scriptlets/*.md`
2. **Check frontmatter**: Must have `---\nname: ...\n---` at the top
3. **Check fence syntax**: Use `` ```bash ``, `` ```tool:name ``, or `` ```template:name ``
4. **Check for syntax errors** in `tool:` scriptlets

## Debugging a Script

### Add Logging

```typescript
import "@scriptkit/sdk";

export const metadata = { name: "Debug Example", description: "Testing" };

console.log("Script started");
const input = await arg("Enter something");
console.log("User entered:", input);
// Check logs for output
```

### Test in Terminal

```bash
cd ~/.scriptkit/kit
bun run main/scripts/your-script.ts
```

### Check TypeScript Errors

```bash
cd ~/.scriptkit/kit
bun run typecheck
```

## File Locations Quick Reference

| What | Where |
|------|-------|
| App logs | `~/.scriptkit/logs/script-kit-gpui.jsonl` |
| Scripts | `~/.scriptkit/kit/main/scripts/` |
| Extensions | `~/.scriptkit/kit/main/scriptlets/` |
| Config | `~/.scriptkit/kit/config.ts` |
| Theme | `~/.scriptkit/kit/theme.json` |
| SDK (read-only) | `~/.scriptkit/sdk/kit-sdk.ts` |
| Databases | `~/.scriptkit/db/` |
| Cache | `~/.scriptkit/cache/` |

## Reset to Defaults

To reset configuration:
```bash
rm ~/.scriptkit/kit/config.ts
rm ~/.scriptkit/kit/theme.json
```
Script Kit will recreate them with defaults on next launch.

To reset everything (nuclear option):
```bash
rm -rf ~/.scriptkit
```
Script Kit will recreate the full workspace on next launch.
