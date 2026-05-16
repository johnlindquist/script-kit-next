# 033 Sharing and Clipboard Trust Install Bundle Map


Initial browser MCP attempts using `sharing-clipboard-trust-atlas`, `sharing-clipboard-trust-atlas-retry`, and `sharing-clipboard-trust-atlas-compact` produced empty model logs after the 120s MCP wrapper timeout. Those failed attempts are preserved in this directory under `attempt-*`.


```bash
oracle --engine browser --model "5.5 Pro" --browser-attachments never --timeout 3600 \
  -s sharing-clipboard-trust-cli \
  --write-output /Users/johnlindquist/dev/script-kit-gpui/feature-map/raw-oracle/033-sharing-clipboard-trust/answer.md \
  -f /Users/johnlindquist/.oracle/bundles/sharing-clipboard-trust-atlas-retry.txt \
     /Users/johnlindquist/dev/script-kit-gpui/feature-map/raw-oracle/033-sharing-clipboard-trust/prompt.md \
  -p "[sharing-clipboard-trust-cli] ..."
```



## Lat context

```bash
source search "sharing clipboard trust prompt share URI install plugin scripts scriptlets skills agents deeplink"
```


- `removed-docs`
- `removed-docs Trust Prompt`
- `removed-docs Skills`
- `removed-docs Portal Contract#Plugin skill target-thread contract`
- `removed-docs Chat#Entry paths#Plugin Skill Thread Affinity`

## Packx command

```bash
packx --limit 49k -l 18 \
  -s "scriptkit-share" \
  -s "copy_deeplink" \
  -s "bundle_from_search_result" \
  -s "install_share_bundle" \
  -s "spawn_clipboard_share_watcher" \
  -s "ClipboardShareImport" \
  -s "ScriptShareBundle" \
  -s "ShareKind" \
  -s "validate_share_relative_path" \
  -s "mark_recently_exported_share" \
  -s "confirm_with_parent_dialog" \
  -s "clipboard_share" \
  -s "portable Script Kit share link" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/storage-cache-security/SKILL.md \
  .agents/skills/platform-windowing-macos/SKILL.md \
  .agents/skills/actions-popups/SKILL.md \
  removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs \
  src/script_sharing.rs src/app_actions/handle_action/files.rs src/actions/builders/script_context.rs \
  src/app_impl/startup.rs src/confirm/parent_dialog.rs src/confirm/window.rs \
  src/plugins/discovery.rs src/plugins/manifest.rs src/plugins/types.rs \
  src/scripts/types.rs src/scripts/loader.rs src/scripts/scriptlet_loader/loading.rs \
  src/actions/builders/clipboard.rs src/actions/dialog.rs src/action_helpers.rs \
  tests/agent_workspace_contract.rs tests/source_audits/action_script_management.rs \
  src/actions/builders_tests.rs src/actions/dialog_behavior_tests.rs src/actions/dialog_cross_context_tests.rs \
  > ~/.oracle/bundles/sharing-clipboard-trust-atlas.txt
```
