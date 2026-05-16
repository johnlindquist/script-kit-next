# 037 Storybook, Design Explorer, and Visual Verification Bundle Map

Oracle slug: `storybook-design-visual-atlas`

Bundle path: `/Users/johnlindquist/.oracle/bundles/storybook-design-visual-atlas.txt`

## Lat Context

```bash
lat expand "037 Storybook Design Explorer Visual Verification: storybook design gallery component showcase screenshots visual states image library"
lat search "storybook design explorer visual gallery component showcase image library verification screenshots stories design surface"
```

Top sections used:

- `lat.md/storybook#Storybook`
- `lat.md/storybook#Storybook#Representation Quality`
- `lat.md/storybook#Storybook#Initial State Matrix`
- `lat.md/storybook#Storybook#Verification Contract`
- `lat.md/design#Design`
- `lat.md/automation#Automation#Surface Navigator`
- `lat.md/automation#Automation#Screenshot pixel audit`
- `lat.md/feature-explorer#Feature Explorer`

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
  lat.md/storybook.md lat.md/design.md lat.md/automation.md lat.md/surfaces.md lat.md/verification.md lat.md/feature-explorer.md \
  src/bin/storybook.rs src/storybook/mod.rs src/storybook/story.rs src/storybook/registry.rs src/storybook/browser.rs src/storybook/adoption.rs src/storybook/audit_report.rs src/storybook/diagnostics.rs src/storybook/layout.rs \
  src/render_builtins/design_gallery.rs src/stories/mod.rs src/stories/main_menu_variations.rs src/stories/dictation_states.rs src/stories/built_in_browser_states.rs src/stories/about_surface.rs src/stories/component_primitives_states.rs \
  scripts/agentic/surface-navigator.ts scripts/agentic/verify-shot.ts scripts/agentic/surface-navigator-inventory-audit.ts scripts/agentic/verify-shot-blank-rejection-matrix.ts scripts/agentic/design-picker-visual-matrix.ts \
  tests/storybook_adoption_contract.rs tests/storybook_compare_contract.rs tests/storybook_lifecycle_contract.rs tests/storybook_main_menu_render_path_contract.rs tests/collect_elements_design_gallery_arm_contract.rs tests/design_gallery_triggerbuiltin_contract.rs tests/design_picker_visual_matrix_script_contract.rs tests/agentic_surface_navigator_contract.rs tests/verify_shot_strict_window_contract.rs \
  > ~/.oracle/bundles/storybook-design-visual-atlas.txt
```

Final bundle summary: 42 files, ripgrep search mode, 6 context lines, 737 matches, 162 context windows, 56,042 exact tokens, 218,489 bytes.
