# 035 Settings, Theme, Config, and Preferences Oracle Prompt

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code, create downloadable artifacts, or ask the user to open files. Return text only.

## Feature Scope

Feature id: `035-settings-theme-config-preferences`

Feature name: Settings, Theme, Config, and Preferences

Cover these Script Kit GPUI capabilities:

- Settings Hub mini built-in list: operational settings rows, filtering, keyboard selection, row activation, actions popup routing, footer ownership, `getState`, and `getElements`.
- Theme Chooser: preset list, preview, customizer controls, native footer ownership, actions dialog catalog, handled-key propagation, Enter ownership, Escape/Cmd+W exit, explicit blur-dismiss policy, and transient focus churn during theme changes.
- Theme system: stock presets, user theme directory, `theme.json`, user-authored theme save/load behavior, row-state opacity guardrails, contrast audits, font/rem sizing, vibrancy/material/appearance effects, and preview/color token boundaries.
- Config system: `~/.scriptkit/config.ts`, config defaults, loader/cache behavior, schema/SDK type surface, config CLI, command shortcut config, built-in enablement settings, theme/dictation/AI/window preferences, and validation/update/reset flows.
- Runtime preference reads: selected theme preset, editor/font size or UI scale where implemented, last-selected AI model/provider preferences, dictation device preference, window-management/window-appearance preferences, and which fields are schema-only versus actually wired.
- Automation/protocol proof surfaces for settings/theme/config: state/elements receipts, source audits, smoke tests, config fingerprint tests, theme contrast tests, theme chooser propagation tests, settings visible-row tests, and config schema tests.

Explicitly distinguish:

- Settings Hub versus Theme Chooser.
- Theme preset selection in config versus color overrides in `theme.json` or user theme files.
- UI-facing preferences that are implemented versus schema-only options.
- Config CLI/script SDK type surfaces versus Rust runtime loading.
- Native footer ownership versus GPUI fallback footer behavior.
- Source-audit proof versus runtime visual proof.

## Required Output Shape

```markdown
## 035 Settings, Theme, Config, and Preferences

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps
```

## Context Notes

This repository requires `lat check` after atlas updates. The maintained atlas preserves the complete Oracle answer under `feature-map/raw-oracle/<feature-id>/answer.md` and distills a readable chapter under `feature-map/features/<feature-id>.md`.
