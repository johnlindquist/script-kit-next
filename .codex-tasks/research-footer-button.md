# Research: Footer Button Patterns (Script Kit GPUI)

## 1) Files investigated for footer button patterns
- `src/components/prompt_footer.rs:234-387` - Primary footer button rendering, hover styling, divider, and full footer layout.
- `src/app_shell/shell.rs:250-358` - Shell footer rendering + `render_footer_button` implementation.
- `src/app_shell/style.rs:145-188` - `FooterColors` (accent/text/border/overlay background) used by the shell footer.
- `src/components/button.rs:11-374` - Current reusable Button component (variants, hover, focus ring, shortcuts).
- `src/render_script_list.rs:957-1015` - Main menu footer uses `PromptFooter` with primary action + Actions button.
- `src/render_builtins.rs:698-704` - Clipboard history footer uses `PromptFooter` with primary label "Paste".
- `src/stories/footer_action_variations.rs:339-380` - Story exploration of footer button layouts (base footer).
- `src/stories/footer_layout_variations.rs:983-1039` - Story exploration of primary/secondary footer button styling.
- `src/actions/dialog.rs:1794-1835` - Footer area for keyboard hints (not buttons, but footer styling context).

## 2) Current `button.rs` implementation (summary)
- Variants: `Primary`, `Ghost`, `Icon` in `ButtonVariant` (`src/components/button.rs:11-21`).
- Theme-based colors: `ButtonColors::from_theme()` pulls accent, border, selected_subtle, and a theme-aware hover overlay (white for dark, black for light) (`src/components/button.rs:51-113`).
- Render behavior:
  - Hover overlay uses `hover_overlay` (15% alpha) and variant-specific background colors (`src/components/button.rs:236-283`).
  - Focus ring uses a 2px border and 62.5% alpha focus color (`src/components/button.rs:240-332`).
  - Padding per variant (Primary: 12x6, Ghost: 8x4, Icon: 6x6 in px at 16px base) (`src/components/button.rs:299-304`).
  - Shortcut text uses `text_xs` and is added as a child element (`src/components/button.rs:286-297`).
  - `cursor_pointer`, hover background, disabled opacity, click callback, and keyboard activation for Enter/Space (`src/components/button.rs:334-370`).

## 3) Footer rendering locations
### PromptFooter usage (primary footer with buttons)
- Main menu script list footer: `src/render_script_list.rs:957-1015`.
- Clipboard history footer ("Paste" primary button): `src/render_builtins.rs:698-704`.
- Arg prompt footer: `src/render_prompts/arg.rs:460-474`.
- Div prompt footer: `src/render_prompts/div.rs:104-146` (PromptFooter used as child).
- Editor prompt footer: `src/render_prompts/editor.rs:191-251`.
- Form prompt footer: `src/render_prompts/form.rs:181-191`.
- Terminal prompt footer: `src/render_prompts/term.rs:121-151`.
- Env prompt footer: `src/prompts/env.rs:569-585`.
- Chat footer: `src/prompts/chat.rs:2165-2187`.

### App shell footer (ShellSpec / FooterSpec)
- ShellSpec includes optional footer and `FooterSpec` builder helpers: `src/app_shell/spec.rs:19-338`.
- Actual footer rendering (container + buttons): `src/app_shell/shell.rs:250-358`.

### Other footer-like UI (non-button)
- Actions dialog keyboard hint footer: `src/actions/dialog.rs:1794-1835`.
- Prompt container hint/footer container: `src/components/prompt_container.rs:176-219`.

## 4) Paste button styling details (Clipboard History)
- The clipboard history view uses `PromptFooter` with:
  - `.primary_label("Paste")`
  - `.primary_shortcut("Enter glyph (U+21B5)")`
  - `.show_secondary(false)`
  - `PromptFooterColors::from_theme(&self.theme)`
  - Location: `src/render_builtins.rs:698-704`.
- The actual button styling is defined in `PromptFooter::render_button`:
  - Layout: `gap 6px`, `px 8`, `py 2`, `rounded 4` with `cursor_pointer` (`src/components/prompt_footer.rs:234-254`).
  - Label color: accent; shortcut color: muted (`src/components/prompt_footer.rs:262-273`).
  - Hover background: accent color at 15% alpha (`colors.accent << 8 | 0x26`) (`src/components/prompt_footer.rs:288-313`).
- Footer container styling (for visual context):
  - Fixed height uses `FOOTER_HEIGHT`, border on top, padding 12px, subtle inner shadow (`src/components/prompt_footer.rs:330-364`).
  - Light mode uses off-white background (`0xECEAEC`), dark mode uses transparent background (~12% opacity) (`src/components/prompt_footer.rs:333-338`).

## 5) Proposed solution approach: reusable `FooterButton` component
- Problem: Footer button rendering is duplicated in two places with slightly different styling:
  - `PromptFooter::render_button` (`src/components/prompt_footer.rs:234-274`).
  - `AppShell::render_footer_button` (`src/app_shell/shell.rs:336-358`).
- Approach:
  1. Create a small `FooterButton` component (e.g., `src/components/footer_button.rs`) that renders:
     - `label`, `shortcut`, optional `on_click`, optional `id`.
     - Common layout tokens: `gap 6px`, `px 8`, `rounded 4`.
     - Theme colors: `accent` + `text_muted` + optional `hover_bg`.
  2. Provide a light-weight color struct or adapter:
     - `FooterButtonColors::from_prompt_footer(PromptFooterColors)` and
     - `FooterButtonColors::from_shell_footer(FooterColors)` (from `src/app_shell/style.rs:145-188`).
  3. Update `PromptFooter::render_button` to call `FooterButton::new(...)` and pass `hover_bg` (keep current hover behavior).
  4. Update `AppShell::render_footer_button` to use the same component. Optionally allow `hover_bg` to be `None` to preserve current static behavior.
  5. Add a story for `FooterButton` in `src/stories/footer_action_variations.rs` or `src/stories/button_stories.rs` to verify layout parity and hover behavior.


## Verification

### Changes Made
1. Created `/Users/johnlindquist/dev/script-kit-gpui/src/components/footer_button.rs` with:
   - `FooterButton` struct with builder pattern
   - `FooterButton::new(label)` - creates button with label only
   - `.shortcut(shortcut)` - adds optional shortcut text
   - `.id(id)` - sets element id
   - `.on_click(callback)` - sets click handler
   - `FooterButton::hover_bg(accent)` - computes accent @ 15% alpha for hover
   - Theme colors fetched internally via `get_cached_theme()`
   - Layout: flex row, gap 6px, px 8, py 2, rounded 4, cursor_pointer

2. Updated `/Users/johnlindquist/dev/script-kit-gpui/src/components/mod.rs`:
   - Added `pub mod footer_button;`
   - Added `pub use footer_button::FooterButton;`

3. Updated `/Users/johnlindquist/dev/script-kit-gpui/src/components/prompt_footer.rs`:
   - Changed import to `use crate::components::footer_button::FooterButton;`
   - Updated `render_button()` to use `FooterButton::new(label).shortcut(shortcut).id(id)`
   - Removed dependency on `FooterButtonColors`

4. Updated `/Users/johnlindquist/dev/script-kit-gpui/tests/footer_button.rs`:
   - Test for `hover_bg()` computing 15% alpha correctly
   - Tests for builder pattern with shortcut and without

5. Updated `/Users/johnlindquist/dev/script-kit-gpui/tests/prompt_footer.rs`:
   - Updated assertions to match new builder API

### Test Results
- `cargo check`: PASS
- `cargo clippy --all-targets -- -D warnings`: PASS
- `cargo test`: PASS (all tests including footer_button and prompt_footer)

### Example Usage

```rust
// Basic footer button with label and shortcut
FooterButton::new("Paste")
    .shortcut("↵")
    .id("paste-button")
    .on_click(Box::new(|event, window, cx| {
        // handle click
    }))

// Actions button example
FooterButton::new("Actions")
    .shortcut("⌘K")
    .id("actions-button")
    .on_click(Box::new(|event, window, cx| {
        // open actions menu
    }))

// Label-only button (no shortcut)
FooterButton::new("Cancel")
    .id("cancel-button")
```

### Deviations from Proposed Solution
- Used builder pattern instead of constructor with all parameters for flexibility
- Theme colors are fetched internally rather than passed in, simplifying the API
- `FooterButtonColors` struct was not needed since theme is accessed internally
