# 037 Storybook, Design Explorer, and Visual Verification Bundle Map



## Lat Context

```bash
source search "storybook design explorer visual gallery component showcase image library verification screenshots stories design surface"
```


- `removed-docs`
- `removed-docs Quality`
- `removed-docs State Matrix`
- `removed-docs Contract`
- `removed-docs`
- `removed-docs Navigator`
- `removed-docs pixel audit`
- `removed-docs Explorer`

## Skills

- `.agents/skills/storybook-design/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Packx Command

```bash
packx --limit 49k -l 6 \
  -s "Storybook" \
  -s "StoryBrowser" \
  -s "catalog" \
  -s "canonicalState" \
  -s "adoptableVariation" \
  -s "Design Gallery" \
  -s "surface-navigate" \
  -s "verify-shot" \
  -s "image-library" \
  -s "screenshot" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md \
  .agents/skills/storybook-design/SKILL.md \
  .agents/skills/agentic-testing/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  .agents/skills/testing-quality-gates/SKILL.md \
  removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs \
  src/bin/storybook.rs src/storybook/mod.rs src/storybook/story.rs src/storybook/registry.rs src/storybook/browser.rs src/storybook/adoption.rs src/storybook/audit_report.rs src/storybook/diagnostics.rs src/storybook/layout.rs \
  src/render_builtins/design_gallery.rs src/stories/mod.rs src/stories/main_menu_variations.rs src/stories/dictation_states.rs src/stories/built_in_browser_states.rs src/stories/about_surface.rs src/stories/component_primitives_states.rs \
  scripts/agentic/surface-navigator.ts scripts/agentic/verify-shot.ts scripts/agentic/surface-navigator-inventory-audit.ts scripts/agentic/verify-shot-blank-rejection-matrix.ts scripts/agentic/design-picker-visual-matrix.ts \
  tests/storybook_adoption_contract.rs tests/storybook_compare_contract.rs tests/storybook_lifecycle_contract.rs tests/storybook_main_menu_render_path_contract.rs tests/collect_elements_design_gallery_arm_contract.rs tests/design_gallery_triggerbuiltin_contract.rs tests/design_picker_visual_matrix_script_contract.rs tests/agentic_surface_navigator_contract.rs tests/verify_shot_strict_window_contract.rs \
  > ~/.oracle/bundles/storybook-design-visual-atlas.txt
```
