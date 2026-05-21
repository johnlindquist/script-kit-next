# macOS Tahoe / Liquid Glass Alignment Research

Status: research memo only. No app changes were made.

## Executive Summary

Script Kit GPUI is already pointed in the right direction for Tahoe: it is a fast, keyboard-first Mac launcher with vibrancy, low-opacity chrome, native-style popups, and a strong local design brief in `.impeccable.md`. The main gap is that Tahoe's Liquid Glass design is not simply "more transparency." Apple frames it as a distinct controls/navigation layer that floats above content, adapts to what is behind it, uses larger softer shapes, respects concentric window geometry, and automatically follows accessibility settings when implemented through system frameworks.

Because Script Kit's visible UI is mostly custom GPUI, it will not receive much Tahoe polish automatically. The current AppKit bridge uses `NSVisualEffectView` materials, GPUI blur swizzling, native footer subviews, and hand-set corner radii. That gives Script Kit a practical adoption path, but it also means Tahoe alignment needs explicit token work, OS-version-gated AppKit glass experiments, and runtime visual proof.

Recommended direction: preserve Script Kit's "Fast. Focused. Minimal." contract, keep Liquid Glass sparse, and migrate in phases:

1. Define Tahoe-specific visual tokens for corner radii, control heights, glass eligibility, tint, shadows, typography scale, and accessibility fallbacks.
2. Update shared custom components and surface shells to use those tokens consistently.
3. Add an AppKit glass/material bridge behind the existing vibrancy bridge for windows, footer, popups, and selected controls.
4. Verify every phase against main menu, actions popup, ACP/Agent Chat, Theme Designer, handler forms, and reduced-transparency/high-contrast settings.

## Primary Sources

- Apple Human Interface Guidelines: overview principles of hierarchy, harmony, and consistency: https://developer.apple.com/design/human-interface-guidelines
- Apple HIG Materials: Liquid Glass as a dynamic material for controls/navigation without obscuring underlying content, with macOS blending modes behind-window and within-window: https://developer.apple.com/design/human-interface-guidelines/materials
- Apple "Adopting Liquid Glass": standard SwiftUI/UIKit/AppKit controls pick up the new design, custom elements need targeted adoption, test accessibility settings, and avoid overuse: https://developer.apple.com/documentation/TechnologyOverviews/adopting-liquid-glass
- Apple "Applying Liquid Glass to custom views": `glassEffect`, `GlassEffectContainer`, `regular` and `clear` glass, tinting, interactivity, morphing, and performance guidance: https://developer.apple.com/documentation/SwiftUI/Applying-Liquid-Glass-to-custom-views
- Apple AppKit `NSGlassEffectView`: AppKit's custom Liquid Glass surface API: https://developer.apple.com/documentation/appkit/nsglasseffectview
- Apple WWDC25 "Meet Liquid Glass": Liquid Glass lives in a distinct functional layer for controls/navigation, should avoid content-layer overuse and nested glass, and supports accessibility adaptations: https://developer.apple.com/videos/play/wwdc2025/219/
- Apple WWDC25 "Platforms State of the Union": larger/bolder left-aligned typography, increased control heights, softer shapes, inset glass controls, toolbar/scroll-edge behavior, and Xcode 26 rebuild implications: https://developer.apple.com/videos/play/wwdc2025/102/
- Apple macOS developer overview: macOS Tahoe Liquid Glass refracts content below, reflects light around it, and lenses along edges: https://developer.apple.com/macos/

Secondary source not used as a design authority: public criticism of Tahoe often centers on transparency, contrast, and inconsistent corner radii. That reinforces risk areas but should not override Apple HIG guidance.

## Apple Guidance Distilled

### What Liquid Glass Is

Apple describes Liquid Glass as a dynamic material combining glass-like optics with fluidity. It refracts content below, reflects surrounding light, and has lensing along edges. On Mac and iPad, Apple positions it as a floating controls/navigation layer above content, not as a universal background texture.

Key principles:

- Hierarchy: controls and navigation become a separate elevated layer above app content.
- Harmony: rounded controls should nest concentrically inside rounded windows and hardware/software shapes.
- Consistency: standard framework controls adopt updated metrics and materials; custom controls need explicit alignment.
- Adaptivity: material appearance changes with background content, display size, input, and accessibility settings.
- Restraint: avoid putting glass in the content layer, avoid glass-on-glass nesting, and avoid tinting every element.

### Implementation Implications

Apple's easiest path is standard SwiftUI/UIKit/AppKit controls. Script Kit is custom GPUI, so "rebuild with Xcode 26" alone is unlikely to make most UI look native. The relevant API concepts are:

- AppKit: `NSGlassEffectView` and `NSGlassEffectContainerView` for custom Liquid Glass surfaces.
- SwiftUI: `glassEffect(_:in:)`, `GlassEffectContainer`, `glassEffectID`, `glassEffectUnion`, `Glass.regular`, `Glass.clear`, `.interactive()`, tint, and performance constraints.
- Existing AppKit controls: `NSToolbar`, `NSSplitView`, title bars, sidebars, forms, search, popovers, and menus get framework help only if Script Kit uses those framework controls.

## Current Script Kit Design Baseline

### Repo Design Contract

`.impeccable.md` defines Script Kit as "Fast. Focused. Minimal." The visual target is Raycast-like Mac vibrancy, dark-mode-first, low-opacity chrome, keyboard-first operations, and a three-affordance footer. Important current principles:

- Footer should stay at most three actions.
- Discovery belongs in Actions (`Cmd+K`), not persistent chrome.
- List-only surfaces stay mini; preview-dependent surfaces use expanded split views.
- Chrome should whisper: low opacity, hairlines, spacing over boxes.
- Native macOS feel matters more than decorative personality.

Tahoe alignment should not erase this. The goal is native Mac polish without turning Script Kit into a translucent dashboard.

### Current Implementation Shape

Relevant source areas inspected:

- Theme and semantic chrome: `src/theme/types.rs`, `src/theme/opacity.rs`, `src/theme/chrome.rs`, `src/theme/gpui_integration.rs`.
- Shared UI helpers: `src/ui_foundation/mod.rs`, `src/ui/chrome/tokens.rs`, `src/components/*`.
- Window/material bridge: `src/platform/vibrancy_config.rs`, `src/platform/vibrancy_swizzle_materials.rs`, `src/platform/secondary_window_config.rs`, `src/notes/window/vibrancy.rs`, `src/ai/window/window_api.rs`.
- Main shell and surfaces: `src/main_sections/render_impl.rs`, `src/main_sections/app_view_state.rs`, `src/render_builtins/*`, `src/render_prompts/*`.
- Footer/chrome: `src/footer_popup.rs`, `src/components/prompt_footer.rs`, `src/components/hint_strip.rs`.
- Popups/dialogs: `src/actions/dialog.rs`, `src/actions/window.rs`, `src/confirm/window.rs`, `src/components/inline_dropdown/*`, `src/app_impl/menu_syntax_trigger_popup_window.rs`.
- Agent Chat: `src/ai/acp/view.rs`, `src/ai/acp/components/*`, `src/ai/window/*`.
- Theme Designer and Storybook/design surfaces: `src/render_builtins/theme_chooser.rs`, `src/render_builtins/theme_chooser_*`, `src/storybook/*`, `src/designs/traits/visual.rs`.

### Current Strengths

- Existing vibrancy bridge already walks `NSVisualEffectView` descendants, sets appearance/material/state/blending, preserves native tint behavior through `BlurredView` swizzling, and uses theme-configured materials.
- Theme opacity already models background, dialog, panel, search box, hover, selected, and text opacity tiers.
- `AppChromeColors` centralizes many semantic surface, text, hover, selection, badge, and popup colors.
- Main footer is already native AppKit subviews inside the main window and can become an early Tahoe proving ground.
- Popups are separate `WindowKind::PopUp` windows with vibrancy setup and lifecycle receipts.
- Theme Designer already exposes material, opacity, font size, and background gradient controls, which can become the user-facing lab for Tahoe tokens.
- Storybook/design explorer already has variations and visual token concepts.

### Current Gaps

- Radius values are scattered. Defaults are 4/8/12/16 in `DesignVisual`; Agent Chat uses 8/10/12/16; buttons use 6; many local surfaces hardcode 4, 6, 8, 10, 12, 999.
- Some explicit contracts still prefer sharp surfaces. Actions dialog has a source-level expectation for `rounded(px(0.0))`, and minimal prompt shells are often called with zero radius.
- Current vibrancy uses legacy `NSVisualEffectView` materials, not Tahoe `NSGlassEffectView`.
- Root window and custom controls are GPUI divs, so standard AppKit control metrics, updated search fields, grouped forms, native buttons, and toolbar behavior do not arrive automatically.
- Footer uses `NSVisualEffectView` with `setCornerRadius: 0.0`, `setBlendingMode: 1`, and flat native text/buttons. It is native, but not yet glass-shaped or inset.
- Confirm popup deliberately removes rounded corners because it sits flush at the bottom of its parent. Tahoe may instead prefer an inset floating confirmation treatment.
- Form and handler prompts use whisper surfaces, but controls remain custom fields with small radii and hand-managed focus/hover states.
- Agent Chat has its own dense token system, sidebar, message bubbles, composer, toolbar, status dots, and popups. It can drift from the launcher unless it consumes shared Tahoe tokens.
- Accessibility behavior is mostly theme-driven today. There is no clear central mapping from macOS Reduce Transparency, Increase Contrast, or Reduce Motion to Script Kit's custom glass/animation decisions.

## Surface-by-Surface Findings

### Main Menu

Current alignment:

- Matches the launcher/product direction: low chrome, fast list, semantic opacity tiers, dark vibrancy, mini view.
- Selection and hover already use semantic opacity from theme tokens.

Tahoe deltas:

- The main window needs concentric outer geometry and inner control geometry. If Tahoe increases third-party window radii, sharp or near-sharp inner shells will feel dated.
- The search/input/header area should be treated as the primary control/navigation layer, not just another content div.
- Use larger/softer search metrics only if they do not violate launcher density.

Likely code areas:

- `src/main_sections/render_impl.rs`
- `src/render_builtins/common.rs`
- `src/components/text_input/render.rs`
- `src/components/unified_list_item/*`
- `src/theme/chrome.rs`
- `src/window_resize/*`

Recommendation:

- Keep the list itself content-first and avoid glass per row.
- Introduce Tahoe shell/input/list tokens and migrate row hover/selection through them.
- Keep gold accent for selected rows, but evaluate whether the left bar should become a subtler glass/tint affordance under Tahoe.

### Popups and Actions Dialog

Current alignment:

- Actions popup is a separate vibrancy window and already feels native compared with an in-window modal.
- It preserves keyboard routing and compact density.

Tahoe deltas:

- Apple frames popovers, menus, and toolbars as places where glass can morph from controls into presentations.
- Current `.impeccable.md` says the Actions dialog container should have no rounded corners and match the main window's sharp edge treatment. That conflicts with Tahoe's softer, concentric glass direction.

Likely code areas:

- `src/actions/dialog.rs`
- `src/actions/window.rs`
- `src/platform/secondary_window_config.rs`
- `src/components/inline_dropdown/*`
- `src/app_impl/menu_syntax_trigger_popup_window.rs`
- `src/ai/acp/picker_popup.rs`

Recommendation:

- Treat Actions as a top-priority Tahoe experiment because it is controls/navigation, not content.
- Add a Tahoe mode where popup windows are inset, softly rounded, shadowed, and optionally backed by `NSGlassEffectView`.
- Do not glass every action row; glass the popup shell or grouped control region.
- Update the sharp-container design contract only after proving the result does not reduce scan speed.

### Agent Chat / ACP

Current alignment:

- ACP has its own token scale and already uses larger radii than many launcher surfaces.
- It contains navigation/sidebar/toolbars/composer areas that map naturally to Liquid Glass layers.

Tahoe deltas:

- Sidebar and toolbar/composer are likely the best glass candidates.
- Message content should remain content-layer, not glass-layer.
- Permission/setup cards need careful restraint; glass cards inside glass windows can become nested-glass clutter.

Likely code areas:

- `src/ai/acp/view.rs`
- `src/ai/acp/components/toolbar.rs`
- `src/ai/acp/components/composer.rs`
- `src/ai/acp/components/transcript.rs`
- `src/ai/window/types.rs`
- `src/ai/window/render_*`
- `src/ai/window/window_api.rs`

Recommendation:

- Centralize ACP radii/control metrics into shared app chrome or a Tahoe token adapter.
- Make the composer and toolbar glass candidates; leave transcript/message bubbles mostly opaque or whisper-tinted content.
- Verify Faraday's active Agent Chat footer/status work before touching footer-adjacent ACP styles.

### Theme Designer

Current alignment:

- Already owns user theme persistence, surface opacity, font size, material, gradients, and preview behavior.
- It is the right place to preview and tune Tahoe tokens before applying them globally.

Tahoe deltas:

- Theme Designer should separate "theme colors/material" from "Tahoe appearance mode" or "native glass eligibility." Liquid Glass is not just a theme preset.
- It needs explicit controls or preview states for reduced transparency/high contrast.

Likely code areas:

- `src/render_builtins/theme_chooser.rs`
- `src/render_builtins/theme_chooser_customize_controls.rs`
- `src/render_builtins/theme_chooser_preview_panel.rs`
- `src/theme/types.rs`
- `src/theme/user_themes.rs`
- `src/storybook/*`

Recommendation:

- Add Tahoe preview variants in Storybook first, then expose only stable user-facing controls.
- Avoid making "glassmorphism" the Tahoe implementation. The existing `GlassmorphismDesignTokens` are decorative web-style tokens; Tahoe needs native material semantics.

### Handler Forms and Prompt Surfaces

Current alignment:

- Prompt fields have whisper surfaces, semantic text palettes, and shared prompt layout helpers.
- Forms can stay keyboard-first and dense.

Tahoe deltas:

- Apple says grouped form layouts update automatically in SwiftUI; Script Kit's custom forms will need manual grouped-form rhythm.
- Fields and buttons likely need larger/softer metrics, but not so much that script prompts lose density.

Likely code areas:

- `src/render_prompts/form/render.rs`
- `src/components/form_fields/*`
- `src/components/prompt_layout_shell.rs`
- `src/components/prompt_container.rs`
- `src/components/button/*`
- `src/components/prompt_header/*`

Recommendation:

- Introduce Tahoe form metrics: field radius, field height, vertical spacing, label weight, and focus ring.
- Keep fields content-layer with glass only for form shell/header/footer if needed.

### Footer and Chrome

Current alignment:

- Footer is already a native AppKit host and intentionally limited to compact commands.
- It has semantic status dot/model label support and can render independent of GPUI.

Tahoe deltas:

- Tahoe favors inset glass controls/toolbars rather than flat bottom strips.
- Current native footer is flush, `cornerRadius = 0`, and uses fixed 4px key-label capsules.
- Apple's guidance about toolbar items sitting within glass and scroll-edge separation is relevant to the footer/status strip.

Likely code areas:

- `src/footer_popup.rs`
- `src/components/prompt_footer.rs`
- `src/components/hint_strip.rs`
- `src/app_impl/ui_window.rs`
- `src/main_sections/app_view_state.rs`

Recommendation:

- Prototype footer as an inset rounded glass toolbar on Tahoe only, with a compatibility fallback to the current flat strip.
- Keep the three-affordance rule.
- Consider merging native footer button layout with `HintStrip` token logic so GPUI and AppKit footers cannot diverge.

### Vibrancy / Material Usage

Current alignment:

- `src/platform/vibrancy_config.rs` sets `NSVisualEffectView` appearance/material/state/blending.
- `src/platform/vibrancy_swizzle_materials.rs` preserves GPUI's native tint layer.
- `Theme::VibrancyMaterial` exposes HUD, Popover, Menu, Sidebar, and Content.

Tahoe deltas:

- Liquid Glass should use `NSGlassEffectView` where available, not just `NSVisualEffectView`.
- Apple distinguishes regular and clear glass; clear is constrained and needs dimming for legibility.
- Apple warns that too many glass effects can hurt performance and legibility.

Likely code areas:

- `src/platform/vibrancy_config.rs`
- `src/platform/vibrancy_swizzle_materials.rs`
- `src/platform/secondary_window_config.rs`
- `src/ai/window/window_api.rs`
- `src/notes/window/vibrancy.rs`
- `src/theme/types.rs`
- `src/theme/gpui_integration.rs`

Recommendation:

- Add a `NativeMaterialKind` abstraction that can resolve to legacy `NSVisualEffectView` or Tahoe `NSGlassEffectView` based on OS availability.
- Keep the existing vibrancy path as fallback for macOS versions before Tahoe and for Reduce Transparency.
- Start with shell-level glass only: main control/header, footer, popup shell, ACP sidebar/composer.

### Rounded Corners and Window Shapes

Current alignment:

- Some surfaces already use 8/10/12/16 radii, especially Agent Chat.
- Dictation overlay has a custom pill radius and clips contentView cleanly.

Tahoe deltas:

- Apple emphasizes concentricity: controls should nest into window corners.
- Current root docs and tests still encode sharp dialog/shell decisions in places.

Likely code areas:

- `src/designs/traits/visual.rs`
- `src/components/button/types.rs`
- `src/components/minimal_prompt_shell.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/prompt_container.rs`
- `src/footer_popup.rs`
- `src/platform/secondary_window_config.rs`
- `src/actions/dialog.rs`
- `src/ai/window/types.rs`

Recommendation:

- Define a single Tahoe radius scale, not many local constants. Suggested starting point: `control_sm = 8`, `control_md = 12`, `control_lg = 16`, `panel = 18-22`, `pill = full`, subject to visual proof on Tahoe.
- Replace hardcoded local radii gradually through shared tokens.
- Revisit source-level sharp-corner tests only when the design decision changes.

### Typography

Current alignment:

- Theme has configurable UI font size.
- Agent Chat has consistent tokenized spacing and sizes.
- Main list prioritizes density.

Tahoe deltas:

- Apple mentions larger, bolder, left-aligned typography and increased control heights while retaining Mac density.
- Script Kit should not blindly upscale all text because launcher speed depends on scan density.

Likely code areas:

- `src/theme/types.rs`
- `src/main_sections/fonts.rs`
- `src/components/text_input/*`
- `src/components/unified_list_item/*`
- `src/ai/window/types.rs`
- `src/render_builtins/theme_chooser.rs`

Recommendation:

- Add Tahoe typography deltas for controls/header/composer, not body/list content first.
- Keep list row density stable until visual proof shows the larger type still scans well.

### Selection States

Current alignment:

- Selection and hover use semantic theme opacity and the gold accent.
- `.impeccable.md` clearly distinguishes idle, hover, focused, and active states.

Tahoe deltas:

- Apple says tint should be selective for key actions, selection, or status.
- Glass controls may flip symbols/glyphs against background and adapt for contrast.

Likely code areas:

- `src/theme/chrome.rs`
- `src/theme/types.rs`
- `src/components/unified_list_item/*`
- `src/actions/dialog.rs`
- `src/components/hint_strip.rs`
- `src/ai/acp/view.rs`

Recommendation:

- Keep gold as Script Kit's brand/selection signal, but run contrast checks over glass/vibrancy backgrounds.
- Avoid turning every selected row into a glass capsule. Selection can remain subtle content-layer tint while shell controls get glass.

### Transparency, Blur, and Accessibility

Current alignment:

- The app already has a theme vibrancy toggle and opacity controls.
- Opaque mode falls back to solid backgrounds in several helpers.

Tahoe deltas:

- Apple explicitly says Reduced Transparency, Increased Contrast, and Reduced Motion modify Liquid Glass effects automatically for standard controls. Script Kit custom controls must do this manually.
- Liquid Glass readability problems are most likely in transparent popups, sidebars, and footers.

Likely code areas:

- `src/theme/types.rs`
- `src/theme/gpui_integration.rs`
- `src/platform/*`
- `src/components/*`
- `src/storybook/*`

Recommendation:

- Add a central `SystemAppearanceAccessibility` read path for reduce-transparency, increase-contrast, reduce-motion, and possibly Liquid Glass appearance/tint settings if AppKit exposes them.
- Map reduced transparency to solid/opaque surfaces, not merely higher alpha in random components.
- Map increased contrast to stronger text/selection/divider tokens.
- Map reduced motion to disable glass morphing/pulse/stretch effects while preserving state changes.

## Likely Code Architecture

### New or Expanded Concepts

- `TahoeAppearanceMode`: off, compatible, tahoe, forced for storybook/debug.
- `NativeMaterialKind`: legacy vibrancy, Tahoe regular glass, Tahoe clear glass, opaque fallback.
- `GlassEligibility`: none, shell, toolbar, popup, control, status.
- `RadiusScale`: window/panel/control/small/pill tokens, with current and Tahoe variants.
- `ControlMetrics`: heights, padding, gap, font weight, keycap radius.
- `AccessibilityMaterialPolicy`: reduced transparency, increased contrast, reduced motion fallback decisions.

### Existing Owners to Extend

- `src/theme/types.rs`: persist appearance/material options if user-facing.
- `src/theme/chrome.rs`: resolve Tahoe-aware colors, surfaces, hover/selection, keycaps, and contrast-safe tints.
- `src/theme/gpui_integration.rs`: sync theme plus native material policy.
- `src/ui_foundation/mod.rs`: shared shell/background helpers.
- `src/designs/traits/visual.rs`: route design variants through new radius/control tokens or de-emphasize variants for core native mode.
- `src/platform/vibrancy_config.rs`: bridge to `NSGlassEffectView` where available.
- `src/platform/secondary_window_config.rs`: popup/window glass policy.
- `src/footer_popup.rs`: native footer toolbar experiment.
- `src/storybook/*`: preview matrix and visual audit.

## Migration Phases

### Phase 0: Research Spike / Baseline Audit

- Capture current screenshots or state receipts for main menu, Actions, confirm, inline dropdown, ACP, Theme Designer, form prompt, and footer.
- Record current radii/control heights/token usage.
- Add a source audit that enumerates hardcoded radii and material hooks.
- Validate on Tahoe and one pre-Tahoe macOS version.

No behavior changes required.

### Phase 1: Token Unification

- Add Tahoe radius/control/font/material tokens.
- Make button, hint strip, prompt fields, list item, actions rows, footer buttons, ACP composer, and popup shell consume shared tokens.
- Keep default values visually close to current design.
- Add source tests for token consumption.

This is low-risk and gives later AppKit work a stable target.

### Phase 2: Accessibility Policy

- Read system accessibility settings on macOS.
- Thread reduced-transparency/increased-contrast/reduced-motion through theme/chrome resolution.
- Add opaque fallback paths for every shell/popover/footer material.
- Add tests for policy mapping and storybook variants.

This should happen before heavy glass adoption.

### Phase 3: Native Glass Bridge

- Add OS-version-gated AppKit glass bridge using `NSGlassEffectView`/container APIs where available.
- Keep existing `NSVisualEffectView` as fallback.
- Prototype first in Actions popup and native footer, because both are controls/navigation layer.
- Confirm performance and interaction behavior before main window adoption.

### Phase 4: Surface Rollout

- Main menu: glass shell/input/footer only; list rows stay content-layer.
- Actions/pickers: glass popup shell and grouped key controls.
- ACP: glass sidebar/toolbar/composer, content transcript stays non-glass.
- Theme Designer: preview and tune Tahoe tokens.
- Handler forms: grouped, softer fields with content-layer backgrounds.
- Confirm/dialogs: move from flush sharp panels to inset glass only if interaction and focus behavior remain correct.

### Phase 5: Visual QA and Regression Gates

- Add storybook Tahoe comparison matrix.
- Add runtime proof scripts for each surface and accessibility setting.
- Capture pixel/contrast checks where possible.
- Keep pre-Tahoe fallback screenshots.

## Prioritized Implementation Checklist

1. Add a Tahoe design token map for radius, control height, typography, glass eligibility, shadow, keycap radius, and material fallback.
2. Add a source audit for hardcoded radii/materials and fail it only on new unowned hardcodes.
3. Centralize macOS accessibility material policy: reduced transparency, increased contrast, reduced motion.
4. Convert `Button`, `HintStrip`, `PromptFooter`, `PromptHeader`, `FormFieldColors`, `UnifiedListItem`, and ACP composer/toolbar to shared control metrics.
5. Build Storybook Tahoe variants for main menu, Actions, footer, ACP, Theme Designer, and form prompt.
6. Prototype `NSGlassEffectView` behind a platform abstraction in an isolated popup window.
7. Apply native glass to Actions popup shell with pre-Tahoe fallback.
8. Apply native glass/inset treatment to main native footer, preserving the three-affordance rule.
9. Add main menu shell/input Tahoe metrics without glassing list rows.
10. Add ACP sidebar/toolbar/composer Tahoe treatment, leaving transcript content plain.
11. Update Theme Designer to preview Tahoe tokens and accessibility policies.
12. Revisit confirm popup geometry and decide whether flush-bottom sharp confirmation still fits Tahoe.
13. Run runtime proof on Tahoe and pre-Tahoe macOS with normal, Reduce Transparency, Increase Contrast, and Reduce Motion settings.
14. Update `.impeccable.md` only after decisions are proven, especially if sharp Actions/prompt shell contracts change.

## Risks

- Legibility: too much transparent material can reduce contrast in dense lists and terminal/editor surfaces.
- Performance: Apple warns that too many glass effects or containers can degrade rendering. Script Kit already optimizes for instant launcher response.
- Native bridge churn: `NSGlassEffectView` is new relative to the existing `NSVisualEffectView` bridge and may require availability guards, Objective-C runtime checks, and AppKit-thread discipline.
- Contract conflicts: existing tests/docs intentionally enforce sharp Actions/prompt shells. Tahoe alignment may require deliberate contract changes, not incidental style edits.
- Cross-version support: the app must preserve current behavior on pre-Tahoe macOS.
- Dirty-worktree coordination: active work on Agent Chat footer status overlaps the exact footer/ACP areas that Tahoe work will eventually touch.

## Open Questions

- Which macOS versions must remain first-class after Tahoe support lands?
- Should Tahoe alignment be automatic on Tahoe, theme-selectable, or user-toggleable?
- Is the current sharp Actions dialog contract still desired, or should Tahoe become the reason to revise it?
- Should Script Kit adopt native AppKit controls in select places, or keep GPUI rendering and only bridge glass materials?
- Can `NSGlassEffectView` be embedded or layered cleanly with GPUI's current `BlurredView` and `CAChameleonLayer` swizzle?
- How should Theme Designer expose native material controls without turning them into decorative theme knobs?
- What is the acceptable row-height increase, if any, for Tahoe controls while preserving launcher density?

## Bottom Line

Tahoe alignment is feasible, but the app should not chase a generic glassmorphism look. The right path is a restrained native-material layer for shell controls, popups, footer, and ACP navigation/composer, backed by shared tokens and explicit accessibility fallbacks. Most content surfaces should stay dense, readable, and minimally tinted.
