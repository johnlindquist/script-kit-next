# 046 Shortcut Assignment Config Refresh Bundle Map

Oracle session for the shortcut assignment and config-refresh atlas.

## Session


## Token And Size Receipt


## Failed Attempt


- `output-failed-thinking-chip.log`
- `session-failed-thinking-chip.json`

## Bundle Contents

The bundle was narrowed to a compact shortcut/config ownership pass around command shortcut assignment, removal, `config.ts` mutation, recorder UI, command IDs, hotkey compatibility, source-audit contracts, and config fingerprint receipts.


- `AGENTS.md`
- `CLAUDE.md`
- `.goals/feature_map.md`
- `.agents/skills/theme-config-preferences/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- `removed-docs`
- `scripts/config-cli.ts`
- `scripts/update-config-shortcut.ts`
- `scripts/remove-config-shortcut.ts`
- `scripts/config-cli.test.ts`
- `src/app_impl/shortcut_recorder.rs`
- `src/app_actions/handle_action/shortcuts.rs`
- `src/components/shortcut_recorder/*`
- `src/shortcuts/*`
- `src/config/command_ids.rs`
- shortcut source-audit tests, popup-window contract tests, error-message tests, and config-fingerprint tests

## Prompt Intent


- Assign/update/remove shortcut user stories from launcher/actions surfaces.
- Shortcut recorder UI states, key handling, save/cancel/clear behavior, and conflict gaps.
- `config.ts.commands[commandId].shortcut` write/removal semantics.
- Command ID formats and source-priority rules.
- Live hotkey registration, restart-required behavior, refresh gaps, and config fingerprint proof.
- Safe claims, unsafe claims, verification recipes, and open implementation/proof gaps.
