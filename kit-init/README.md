# kit-init

Files in this directory are embedded into the Script Kit binary and copied to `~/.sk/kit/` during first-run setup.

## Files

| File | Destination | Purpose |
|------|-------------|---------|
| `GUIDE.md` | `~/.sk/kit/GUIDE.md` | Comprehensive user tutorial |
| `config-template.ts` | `~/.sk/kit/config.ts` | Default configuration |
| `theme.example.json` | `~/.sk/kit/theme.json` | Default theme |

## Behavior

- Files are only created if they don't exist (user-owned, never overwritten)
- Embedded at compile time via `include_str!()` in `src/setup.rs`
- Build system watches these files for changes (`build.rs`)
