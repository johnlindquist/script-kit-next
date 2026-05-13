# Designs

Design catalog docs describe stable design ids, token signatures, and the migration boundary from legacy enum variants to catalog-backed rendering. The catalog also owns the Design Picker entry that replaces the blind Cmd+1 cycle.

## Catalog invariants

The design catalog in [[src/designs/core/registry.rs#CATALOG]] is the source of truth for stable ids, token signatures, and renderer modes.

Every catalog entry keeps a stable kebab-case id, declares its palette, typography, density, chrome, and vibrancy choices, and resolves through [[src/designs/core/registry.rs#lookup]] or [[src/designs/core/registry.rs#fallback]] rather than ad hoc enum matching. No two entries may share the same [[src/designs/core/registry.rs#DesignSignature]] — a hash-based test enforces this.

Phase 1 ships ten curated entries; Phase 2 expands to exactly twenty-five. Renderer sprawl is not allowed — every entry maps to one of the shared [[src/designs/core/registry.rs#RendererMode]] values with palette/typography/density driving the visible difference.

## Legacy migration

Legacy design variants are bridged through [[src/designs/legacy_migration.rs#map_legacy_variant_to_id]] so old configs keep working while new code consumes catalog ids.

The migration table maps every `DesignVariant` enum value to a stable kebab-case id; [[src/designs/legacy_migration.rs#resolve_possibly_legacy_id]] also handles string forms of the legacy enum so JSON configs upgrade cleanly. Unknown ids fall back to `script-kit-classic` rather than panic, and the loader rewrites the migrated id on the next save.

[[src/designs/legacy_migration.rs#legacy_variant_def]] keeps Phase-2-only ids safe by returning the canonical default when a legacy variant maps to an id that has not yet been seeded into the catalog.

## Design Picker key handling

The Design Picker (Phase 1 follow-up) owns its handled key events so selection, undo, and preview shortcuts cannot fall through into the focused main filter. The pattern mirrors [[theme#Theme chooser key handling]].

`InputEvent::PressEnter` dispatches to the Design Picker while `AppView::DesignPickerView` is active, just like the theme chooser. Every handled key path and every click handler calls `cx.stop_propagation()` before mutating state so a preview keystroke cannot leak into the launcher filter underneath.

Escape clears the filter first; on an empty filter it restores the captured snapshot and closes the picker. Cmd+W also restores and closes explicitly. The Picker surface contract declares ignore-window-blur dismiss so transient native focus shifts during material or vibrancy preview cannot dismiss the surface.

Cmd+1 routing is decided at the built-in dispatch boundary in [[src/app_execute/builtin_execution.rs#ScriptListApp#execute_builtin_inner]] before any picker view opens. [[src/designs/overrides.rs#effective_cmd1_behavior]] reads [[src/config/types.rs#ScriptKitUserPreferences]]`.cmd1Behavior` and resolves it to `Picker` (default) or `Cycle`. `Picker` opens [[src/app_execute/builtin_execution.rs#ScriptListApp#execute_builtin_inner]]; `Cycle` calls [[src/app_impl/theme_focus.rs#ScriptListApp#cycle_design]] once and commits the post-cycle id through [[src/config/loader.rs#save_user_preferences]] without opening the picker. The picker's own key handlers stay unchanged.

## Phase plan

Design overhaul ships in six phases — see `.goals/design-variants-overhaul.md` for the authoritative spec.

Phase 1 lands the registry, legacy migration, and picker MVP on a ten-design subset. Phase 2 completes the catalog to twenty-five and tightens the uniqueness test. Phase 3 adds the customizer and `ActionsDialogHost::DesignPicker` Cmd+K catalog. Phase 4 wires persistence through `kit_config` and the `~/.scriptkit/config.ts` schema — partially shipped via [[designs#Persistence]] (Design Picker commit + startup hydration). Phase 5 adds the Full split-preview surface and the agentic visual matrix. Phase 6 retires the `DesignVariant` enum behind `#[deprecated]` and enforces no production references via source-audit test.

## Persistence

Design selection persists the committed catalog id while previews remain in-memory only.

[[src/render_builtins/design_picker.rs#ScriptListApp#preview_design_picker_id]] and [[src/render_builtins/design_picker.rs#ScriptListApp#preview_design_picker_filtered_index]] are preview-only and must not write config. Explicit selection — Enter via [[src/render_builtins/design_picker.rs#ScriptListApp#submit_design_picker_from_input_enter]] and row click via [[src/render_builtins/design_picker.rs#ScriptListApp#render_design_picker]] — commits through [[src/render_builtins/design_picker.rs#ScriptListApp#persist_design_picker_selection]], which writes the canonical catalog id into [[src/config/types.rs#ScriptKitUserPreferences]] via [[src/config/loader.rs#save_user_preferences]]. Startup hydrates `current_design` on [[src/main_sections/app_state.rs#ScriptListApp]] through [[src/render_builtins/design_picker.rs#ScriptListApp#current_design_id]] and [[src/designs/legacy_migration.rs#resolve_possibly_legacy_id]]; unknown ids fall back to `script-kit-classic` via [[src/designs/core/registry.rs#FALLBACK_ID]].

Escape and Cmd+W restoration use [[src/render_builtins/design_picker.rs#ScriptListApp#restore_design_picker_original]] which is preview-only — it rolls back the in-memory variant and does not call `save_user_preferences`. Config writes serialize through `CONFIG_PREFERENCE_WRITE_LOCK` so concurrent theme/dictation/AI writes stay atomic, and `save_user_preferences` mutates only `designs.activeId`, preserving `cmd1Behavior` and `overrides`.

Cmd+1 in `Cycle` mode reuses [[src/app_impl/theme_focus.rs#ScriptListApp#cycle_design]] as an in-memory primitive and commits the resulting [[src/render_builtins/design_picker.rs#ScriptListApp#current_design_id]] through [[src/config/loader.rs#save_user_preferences]] inside the dispatch arm; `cycle_design` itself stays preview-only. Per-design customizer changes flow through the new [[src/config/loader.rs#save_user_preferences]] helper, which mirrors `save_user_preferences`'s lock + `write_preference_group` pattern but mutates only `designs.overrides[id]`, preserving `activeId` and `cmd1Behavior`. The Design Picker `design_picker_toggle_density` Cmd+K action commits through this helper from [[src/app_impl/actions_dialog.rs#ScriptListApp#execute_action_for_actions_host]].

Runtime state exposes the `design` envelope on `getState`/`kit/state` via [[src/render_builtins/design_picker.rs#ScriptListApp#design_state_receipt]]: `activeId` (always non-null, falls back to [[src/designs/core/registry.rs#FALLBACK_ID]]), `persistedActiveId` (canonical [[src/config/types.rs#ScriptKitUserPreferences]]`.active_id` after [[src/designs/legacy_migration.rs#resolve_possibly_legacy_id]]; `null` when missing or unresolved), `fallbackApplied` (`true` only when the raw persisted id was missing or unresolved), and `currentVariant` (debug name of the legacy `DesignVariant`).
