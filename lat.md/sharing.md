# Sharing

Script Kit GPUI shares scripts, snippets, skills, and agents as a single clipboard URI that another Script Kit app can detect and install after an explicit trust prompt.

## Clipboard Share Links

The sender path reuses the existing `copy_deeplink` action slot but produces a portable `scriptkit-share://v1/...` bundle for shareable items.

- Scripts share their file under `scripts/<filename>`.
- Scriptlets share the source markdown file under `scriptlets/<filename>`.
- Skills share the full `skills/<skill_id>/` directory so `SKILL.md` and local assets move together.
- Agents share their markdown file under `agents/<filename>`.
- Non-shareable launcher items still fall back to the older `scriptkit://run/...` deeplink behavior.

## Clipboard Trust Prompt

Startup now runs a lightweight clipboard watcher that looks for the share URI format and opens a parent confirmation dialog before any install work happens.

- The watcher polls the clipboard change count and only inspects text when the pasteboard changes.
- Recently exported share links are ignored briefly so copying your own link does not immediately trigger an install prompt locally.
- The confirm dialog shows the shared item kind, title, plugin label, and file count before the user chooses `Install` or `Ignore`.

## Install Target

Accepted share bundles install into the normal plugin container so refresh and discovery keep using the same plugin-based loading path.

- Installs write a `plugin.json` plus the shared files under `~/.scriptkit/plugins/<plugin-id>/`.
- If the requested plugin id already exists, the installer creates a unique sibling such as `<plugin-id>-shared-2`.
- Shared file paths are restricted to `scripts/`, `scriptlets/`, `skills/`, and `agents/` to prevent path traversal outside the plugin root.
- After install, the app refreshes scripts and skills and returns the main window to `ScriptList`.

## Source Files

These files define the current clipboard sharing and import contract.

- [src/script_sharing.rs](/Users/johnlindquist/dev/script-kit-gpui/src/script_sharing.rs)
- [src/app_actions/handle_action/files.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_actions/handle_action/files.rs)
- [src/actions/builders/script_context.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders/script_context.rs)
- [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs)
- [src/confirm/parent_dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/confirm/parent_dialog.rs)
- [src/plugins/discovery.rs](/Users/johnlindquist/dev/script-kit-gpui/src/plugins/discovery.rs)

## Related Pages

These pages cover adjacent loader and surface behavior the sharing flow relies on.

- [[scripting]]
- [[architecture]]
- [[surfaces]]
