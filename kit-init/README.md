# kit-init

Files in this directory are embedded into the Script Kit binary and copied to `~/.scriptkit/` during first-run setup.

## Files

| File | Destination | Purpose |
|------|-------------|---------|
| `GUIDE.md` | `~/.scriptkit/GUIDE.md` | Comprehensive user tutorial |
| `config-template.ts` | `~/.scriptkit/kit/config.ts` | Default configuration |
| `theme.example.json` | `~/.scriptkit/kit/theme.json` | Default theme |
| `skills/**` | `~/.scriptkit/kit/authoring/skills/**` | Plugin-owned skills loaded from the authoring plugin |

## Behavior

- User-owned files (config, theme) are only created if they don't exist (never overwritten)
- App-managed files (SDK, skills, extensions) are refreshed if content has changed
- Embedded at compile time via `include_str!()` in `src/setup/mod.rs`
- Build system watches these files for changes (`build.rs`)
