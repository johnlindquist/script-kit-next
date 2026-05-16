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

Scripts use `script/{owner}:{name}` and scriptlets use `scriptlet/{owner}:{name}`. Built-ins use `builtin/{id}` and apps use `app/{bundleId}` when available. Config-backed rows copy `scriptkit://commands/{commandId}` links so deeplinks and shortcut writes share the same ID.

## Recorder Writes

The shortcut recorder writes config-backed command shortcuts.

Recorder saves call `scripts/update-config-shortcut.ts`, a compatibility wrapper around `scripts/config-cli.ts set-command-shortcut`. The live hotkey table is updated after the config write succeeds so the shortcut works without restart when registration succeeds.

Recorder conflict checks read the live hotkey route table before save. They block conflicts with already-registered config, script, scriptlet, and app hotkeys while allowing the selected command to keep its current shortcut.

## Transient SDK hotkey capture

SDK `hotkey()` captures one shortcut for a running script without becoming shortcut assignment.

The host maps `type:"hotkey"` messages to a `HotkeyPrompt` surface backed by the shortcut capture component, but its submit path returns only the serialized `HotkeyInfo` value to the script. It does not call the persistent recorder entry point, write `config.ts`, or update live global shortcut registrations. Pinned by [[tests/hotkey_prompt_contract.rs#sdk_hotkey_routes_to_real_host_prompt]].

### Does not mutate shortcut config

Transient capture must stay separate from command shortcut persistence.

The persistent shortcut recorder remains the only path that calls config shortcut writes and live hotkey registration updates. The SDK HotkeyPrompt reuses capture rendering and key parsing only, then returns JSON through prompt submission. Pinned by [[tests/hotkey_prompt_contract.rs#hotkey_prompt_is_transient_and_does_not_use_persistent_shortcut_save]].

### Automation receipts

HotkeyPrompt proof uses prompt receipts instead of shortcut assignment side effects.

`getState.promptType` reports `hotkey`, `getElements` exposes `panel:hotkey-capture` and `input:hotkey-shortcut`, and stdin `simulateKey` records modifier-plus-key chords into the same `HotkeyInfo` JSON that SDK `hotkey()` resolves. Escape submits `null` and cancels script execution, matching the SDK cancellation contract. Pinned by [[tests/hotkey_prompt_contract.rs#hotkey_prompt_has_state_first_capture_and_cancel_receipts]].

## Removal Writes

Shortcut removal edits only the shortcut field in config.

Removal calls `scripts/remove-config-shortcut.ts`, which wraps `scripts/config-cli.ts remove-command-shortcut`. Removing a shortcut preserves sibling command fields such as `hidden` and `confirmationRequired`; empty command entries are deleted.

Removal also unregisters the live dynamic shortcut route when one exists. Missing live routes are treated as no-ops; app routes are removed before best-effort OS unregister so failures cannot dispatch removed commands.

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
