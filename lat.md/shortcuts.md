# Shortcuts

Launcher shortcuts have one durable user-owned source: `~/.scriptkit/config.ts`.

## Key Facts

These facts define where shortcut state may live and how conflicts are resolved.

- Top-level app hotkeys are read from `config.ts` hotkey fields.
- Command-specific launcher shortcuts are read from `config.ts.commands[commandId].shortcut`.
- Script and scriptlet metadata shortcuts remain defaults authored beside the script itself.
- `config.ts.commands` wins over script and scriptlet metadata for the same launcher command ID.
- `shortcuts.json` is legacy data only and must not be an active startup, display, recorder, or removal source.

## Registration Priority

Startup registers shortcuts in deterministic source order so user config wins.

The hotkey listener registers top-level app hotkeys first, then config-backed command shortcuts, then inline script or scriptlet metadata shortcuts that are not overridden by config. This keeps `config.ts` as the override layer without mutating script files.

## Command IDs

Launcher command IDs are shared by read, write, and hotkey paths.

Scripts use `script/{owner}:{name}` and scriptlets use `scriptlet/{owner}:{name}`. The owner is the plugin ID when present, otherwise the script kit name for scripts or scriptlet group for scriptlets, falling back to `main`.

## Recorder Writes

The shortcut recorder writes config-backed command shortcuts.

Recorder saves call `scripts/update-config-shortcut.ts`, a compatibility wrapper around `scripts/config-cli.ts set-command-shortcut`. The live hotkey table is updated after the config write succeeds so the shortcut works before restart.

## Removal Writes

Shortcut removal edits only the shortcut field in config.

Removal calls `scripts/remove-config-shortcut.ts`, which wraps `scripts/config-cli.ts remove-command-shortcut`. Removing a shortcut preserves sibling command fields such as `hidden` and `confirmationRequired`; empty command entries are deleted.

## Source Documents

These files define and protect the shortcut source-of-truth contract.

- [src/hotkeys/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/hotkeys/mod.rs)
- [src/scripts/types.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/types.rs)
- [src/app_impl/shortcut_recorder.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/shortcut_recorder.rs)
- [src/app_actions/handle_action/shortcuts.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_actions/handle_action/shortcuts.rs)
- [scripts/config-cli.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-cli.ts)
- [scripts/update-config-shortcut.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/update-config-shortcut.ts)
- [scripts/remove-config-shortcut.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/remove-config-shortcut.ts)
- [tests/source_audits/shortcut_config_source.rs](/Users/johnlindquist/dev/script-kit-gpui/tests/source_audits/shortcut_config_source.rs)
- [scripts/config-cli.test.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-cli.test.ts)

## Related Pages

These pages cover adjacent config and scripting contracts.

- [workspace](./workspace.md)
- [scripting](./scripting.md)
- [builtins](./builtins.md)
