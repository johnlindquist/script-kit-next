# Workspace

Script Kit GPUI uses a plugin-style `~/.scriptkit` workspace. The durable contract is the seeded workspace layout, the TypeScript config surface, and the app-managed setup or migration logic that keeps that layout alive.

## Key Facts

These facts describe the durable workspace layout and the config surfaces inside it.

- `scripts/config-schema.ts` is the authoritative reference for modifying `~/.scriptkit/config.ts`.
- The plugin container is `~/.scriptkit/plugins/`; `main/` is the default personal plugin and `scriptkit/` owns the bundled skill set.
- `~/.scriptkit/config.ts` owns hotkeys, command overrides, runtime preferences, and Claude Code or ACP-related settings.
- `config.ts.commands[commandId].shortcut` is the durable shortcut override source; script and scriptlet metadata remain inline defaults, and `shortcuts.json` is legacy-only.
- `~/.scriptkit/theme.json` is the user-owned theme override surface.
- The app seeds and repairs the workspace structure from `src/setup/mod.rs`, including bundled verb-named skills, example skills, config or theme templates, ACP agent catalog files, and legacy `~/.kenv` migration.
- MCP token and discovery metadata live at `~/.scriptkit/agent-token` and `~/.scriptkit/server.json`.

## Key Files

These files define the workspace bootstrap, loading, and config tooling paths.

- [scripts/config-schema.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-schema.ts) - Authoritative AI-facing schema and command-ID rules for `config.ts`.
- [scripts/config-cli.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-cli.ts) - Bun CLI for reading and editing `~/.scriptkit/config.ts`.
- [src/setup/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/setup/mod.rs) - Workspace bootstrap, directory seeding, legacy migration, and bundled Script Kit plugin assets.
- [src/scripts/loader.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/loader.rs) - Script loading across discovered plugins.
- [README.md](/Users/johnlindquist/dev/script-kit-gpui/README.md) - Public workspace layout and starter examples.

## Source Documents

These source files support the workspace contract summarized here.

- [scripts/config-schema.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-schema.ts)
- [scripts/config-cli.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-cli.ts)
- [src/setup/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/setup/mod.rs)
- [src/scripts/loader.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/loader.rs)
- [README.md](/Users/johnlindquist/dev/script-kit-gpui/README.md)

## Related Pages

These pages cover the scripting, built-in, and distribution contracts tied to the workspace.

- [scripting](./scripting.md)
- [builtins](./builtins.md)
- [distribution](./distribution.md)
- [shortcuts](./shortcuts.md)

## Workspace Layout

The current durable layout is centered on:

- `~/.scriptkit/config.ts`
- `~/.scriptkit/theme.json`
- `~/.scriptkit/plugins/scriptkit/skills/new-script/SKILL.md`
- `~/.scriptkit/plugins/scriptkit/skills/update-config/SKILL.md`
- `~/.scriptkit/plugins/main/scripts/`
- `~/.scriptkit/plugins/main/scriptlets/`
- `~/.scriptkit/plugins/*/skills/`
- `~/.scriptkit/acp/agents.json`
- `~/.scriptkit/sdk/`

That is the live workspace contract the app bootstraps and watches, not the older `~/.kenv`-only story.
