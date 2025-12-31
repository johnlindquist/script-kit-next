# DivPrompt Scrollable Content & Clickable Links Expert Bundle

## Executive Summary

The DivPrompt component in Script Kit GPUI needs to be enhanced to support:
1. **Scrollable content** - When HTML content exceeds the window height, native scrolling should be enabled
2. **Clickable links** - Links should be interactive, opening external URLs in the browser
3. **Submit protocol links** - Special `submit:value` href protocol that submits a value and resolves the div() promise

Currently, links are rendered as styled text but are NOT clickable, and content is clipped with `overflow_y_hidden`. The div() function returns `Promise<void>` and cannot return values from link clicks.

### Key Problems:

1. **No scroll support**: `DivPrompt::render()` uses `.overflow_y_hidden()` which clips content - should use `.overflow_y_scroll()` with a scroll handle
2. **Links are not clickable**: `render_element()` for `HtmlElement::Link` just renders styled text with no click handler
3. **No submit link protocol**: The `submit:value` href pattern (e.g., `[Option A](submit:optionA)`) is not implemented
4. **div() returns void**: SDK div() function returns `Promise<void>` instead of `Promise<string | void>` to support link-based selection
5. **External links not handled**: No mechanism to detect `http://` links and open in browser via `open::that`

### Required Fixes:

1. `src/prompts/div.rs`: 
   - Add `ScrollHandle` for native scrolling
   - Change `.overflow_y_hidden()` to `.overflow_y_scroll()` 
   - Add click handler to `HtmlElement::Link` rendering
   - Detect `submit:` protocol and call submit callback with value
   - Detect `http://` protocol and open in system browser

2. `scripts/kit-sdk.ts`:
   - Change `div()` return type from `Promise<void>` to `Promise<string | void>`
   - Handle submit response with value from link clicks

3. `src/protocol/message.rs`:
   - Submit message already supports `value: Option<String>` - no changes needed

4. `src/utils.rs`:
   - `HtmlElement::Link` already parses `href` - no changes needed

### Files Included:

- `src/prompts/div.rs`: Core DivPrompt implementation (921 lines) - needs scroll and link handlers
- `src/utils.rs`: HtmlElement enum and HTML parser - has Link { href, children } 
- `src/protocol/message.rs`: Message::Submit with optional value - already supports string values
- `scripts/kit-sdk.ts`: SDK div() function - needs return type change
- `tests/smoke/test-div-md-links-special.ts`: Existing link visual tests

---

## File: src/prompts/div.rs (Full File)

```rust
//! DivPrompt - HTML content display
//!
//! Features:
//! - Parse and render HTML elements as native GPUI components
//! - Support for headers, paragraphs, bold, italic, code, lists, blockquotes
//! - Theme-aware styling
//! - Simple keyboard: Enter or Escape to submit

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, Render,
    Window,
};
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::theme;
use crate::utils::{parse_color, parse_html, HtmlElement, TailwindStyles};

use super::SubmitCallback;

/// Options for customizing the div container appearance
#[derive(Debug, Clone, Default)]
pub struct ContainerOptions {
    /// Background color: "transparent", "#RRGGBB", "#RRGGBBAA", or Tailwind color name
    pub background: Option<String>,
    /// Padding in pixels, or None to use default
    pub padding: Option<ContainerPadding>,
    /// Opacity (0-100), applies to entire container
    pub opacity: Option<u8>,
    /// Tailwind classes for the content container
    pub container_classes: Option<String>,
}

/// Padding options for the container
#[derive(Debug, Clone)]
pub enum ContainerPadding {
    /// No padding
    None,
    /// Custom padding in pixels
    Pixels(f32),
}

impl ContainerOptions {
    /// Parse container background to GPUI color
    pub fn parse_background(&self) -> Option<Hsla> {
        let bg = self.background.as_ref()?;
        
        // Handle "transparent"
        if bg == "transparent" {
            return Some(Hsla::transparent_black());
        }
        
        // Handle hex colors: #RGB, #RRGGBB, #RRGGBBAA
        if bg.starts_with('#') {
            return parse_hex_color(bg);
        }
        
        // Handle Tailwind color names (e.g., "blue-500", "gray-900")
        if let Some(color) = parse_color(bg) {
            return Some(rgb_to_hsla(color, self.opacity));
        }
        
        None
    }
    
    /// Get padding value
    pub fn get_padding(&self, default: f32) -> f32 {
        match &self.padding {
            Some(ContainerPadding::None) => 0.0,
            Some(ContainerPadding::Pixels(px)) => *px,
            None => default,
        }
    }
}

/// Parse hex color string to GPUI Hsla
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    
    match hex.len() {
        // #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Hsla::from(gpui::Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }))
        }
        // #RRGGBB
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Hsla::from(gpui::Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }))
        }
        // #RRGGBBAA
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Hsla::from(gpui::Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }))
        }
        _ => None,
    }
}

/// Convert RGB u32 to Hsla with optional opacity
fn rgb_to_hsla(color: u32, opacity: Option<u8>) -> Hsla {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    let a = opacity.map(|o| o as f32 / 100.0).unwrap_or(1.0);
    Hsla::from(gpui::Rgba { r, g, b, a })
}

/// DivPrompt - HTML content display
pub struct DivPrompt {
    pub id: String,
    pub html: String,
    pub tailwind: Option<String>,
    pub focus_handle: FocusHandle,
    pub on_submit: SubmitCallback,  // Arc<dyn Fn(String, Option<String>) + Send + Sync>
    pub theme: Arc<theme::Theme>,
    pub design_variant: DesignVariant,
    pub container_options: ContainerOptions,
}

impl DivPrompt {
    // ... constructors same as before ...

    /// Submit - always with None value (just acknowledgment)
    fn submit(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }
}

/// Style context for rendering HTML elements
#[derive(Clone, Copy)]
struct RenderContext {
    text_primary: u32,
    text_secondary: u32,
    text_tertiary: u32,
    accent_color: u32,
    code_bg: u32,
    quote_border: u32,
    hr_color: u32,
}

/// Render a single HtmlElement as a GPUI element
fn render_element(element: &HtmlElement, ctx: RenderContext) -> Div {
    match element {
        // ... other cases ...

        HtmlElement::Link { children, .. } => {
            // BUG: Links are styled but NOT CLICKABLE
            let text_content = collect_text(children);
            div().text_color(rgb(ctx.accent_color)).child(text_content)
        }

        // ... other cases ...
    }
}

impl Render for DivPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ... setup code ...

        // BUG: Content clips instead of scrolling
        let content_base = div()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_y_hidden()  // <-- BUG: Should be overflow_y_scroll()
            .child(styled_content);

        // ... rest of render ...
    }
}
```

---

## File: src/utils.rs (HtmlElement::Link definition - lines 87-91)

```rust
/// Represents a parsed HTML element with its type and content
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlElement {
    // ... other variants ...
    
    /// Link with href and text - HREF IS PARSED BUT NEVER USED FOR CLICKS
    Link {
        href: String,         // <-- This contains the URL/submit value
        children: Vec<HtmlElement>,
    },
    
    // ... other variants ...
}
```

The HTML parser correctly extracts href (lines 446-454):
```rust
"a" => {
    let href = attributes
        .iter()
        .find(|(k, _)| k == "href")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let children = self.parse_children("a");
    Some(HtmlElement::Link { href, children })
}
```

---

## File: src/protocol/message.rs (Submit Message - lines 63-65)

```rust
/// App responds with submission (selected value or null)
#[serde(rename = "submit")]
Submit { id: String, value: Option<String> },
```

The protocol already supports returning a string value in Submit messages.

---

## File: scripts/kit-sdk.ts (div function - lines 2517-2594)

```typescript
/**
 * Display HTML content to user.
 */
globalThis.div = async function div(
  htmlOrConfig?: string | DivConfig,
  actionsInput?: Action[]
): Promise<void> {  // <-- BUG: Should be Promise<string | void>
  const id = nextId();
  
  // ... argument parsing ...
  
  return new Promise((resolve) => {
    pending.set(id, () => {
      resolve();  // <-- BUG: Should resolve with value if provided
    });
    
    const message: DivMessage = {
      type: 'div',
      id,
      html,
      // ... other fields ...
    };
    
    send(message);
  });
};
```

---

## Reference: Original Script Kit submit: Protocol

From kit-container (the Electron version):

```typescript
// In prompt.init-utils.ts
} else if (url.protocol === 'submit:') {
  prompt.logInfo('Attempting to run submit protocol:', JSON.stringify(url));
  prompt.sendToPrompt(Channel.SET_SUBMIT_VALUE as any, url.pathname);
}
```

Usage example from API.md:
```javascript
let name = await div(md(`# Pick a Name
* [John](submit:John)
* [Mindy](submit:Mindy)
* [Joy](submit:Joy)
`))
// name will be "John", "Mindy", or "Joy"

await div(md(`# You selected ${name}`))
```

---

## Implementation Guide

### Step 1: Add ScrollHandle and Link Callback (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: Add imports at the top (replace existing imports)

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Div, FocusHandle, Focusable, FontWeight, 
    Hsla, Render, Window, ScrollHandle,  // ADD ScrollHandle
};
use std::sync::Arc;

// Location: Add to DivPrompt struct (after container_options field)
pub struct DivPrompt {
    pub id: String,
    pub html: String,
    pub tailwind: Option<String>,
    pub focus_handle: FocusHandle,
    pub on_submit: SubmitCallback,
    pub theme: Arc<theme::Theme>,
    pub design_variant: DesignVariant,
    pub container_options: ContainerOptions,
    // NEW: Add scroll handle for native scrolling
    pub scroll_handle: ScrollHandle,
}
```

### Step 2: Update DivPrompt Constructor (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: Replace the with_options function body

#[allow(clippy::too_many_arguments)]
pub fn with_options(
    id: String,
    html: String,
    tailwind: Option<String>,
    focus_handle: FocusHandle,
    on_submit: SubmitCallback,
    theme: Arc<theme::Theme>,
    design_variant: DesignVariant,
    container_options: ContainerOptions,
) -> Self {
    logging::log(
        "PROMPTS",
        &format!(
            "DivPrompt::new with theme colors: bg={:#x}, text={:#x}, design: {:?}, container_opts: {:?}",
            theme.colors.background.main, theme.colors.text.primary, design_variant, container_options
        ),
    );
    DivPrompt {
        id,
        html,
        tailwind,
        focus_handle,
        on_submit,
        theme,
        design_variant,
        container_options,
        scroll_handle: ScrollHandle::new(),  // NEW: Initialize scroll handle
    }
}
```

### Step 3: Add submit_with_value and handle_link_click Methods (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: After the submit() method in impl DivPrompt

impl DivPrompt {
    // ... existing new(), with_design(), with_options() ...

    /// Submit - always with None value (just acknowledgment)
    fn submit(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }
    
    /// Submit with a value (from submit: link protocol)
    fn submit_with_value(&mut self, value: String) {
        logging::log("UI", &format!("DivPrompt submit with value: {}", value));
        (self.on_submit)(self.id.clone(), Some(value));
    }
    
    /// Handle link click - determine action based on href protocol
    fn handle_link_click(&mut self, href: &str) {
        logging::log("UI", &format!("DivPrompt link clicked: {}", href));
        
        if let Some(value) = href.strip_prefix("submit:") {
            // submit:value protocol - submit the value
            self.submit_with_value(value.to_string());
        } else if href.starts_with("http://") || href.starts_with("https://") {
            // External URL - open in system browser
            if let Err(e) = open::that(href) {
                logging::log("ERROR", &format!("Failed to open URL {}: {}", href, e));
            }
        } else if href.starts_with("file://") {
            // Local file - try to open
            let path = href.strip_prefix("file://").unwrap_or(href);
            if let Err(e) = open::that(path) {
                logging::log("ERROR", &format!("Failed to open file {}: {}", path, e));
            }
        } else {
            // Unknown protocol - log and ignore
            logging::log("UI", &format!("Unknown link protocol: {}", href));
        }
    }
}
```

### Step 4: Update RenderContext to Include Link Callback (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: Replace the RenderContext struct definition

/// Callback type for link clicks
type LinkClickCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// Style context for rendering HTML elements
#[derive(Clone)]
struct RenderContext {
    /// Primary text color
    text_primary: u32,
    /// Secondary text color (for muted content)
    text_secondary: u32,
    /// Tertiary text color
    text_tertiary: u32,
    /// Accent/link color
    accent_color: u32,
    /// Code background color
    code_bg: u32,
    /// Blockquote border color
    quote_border: u32,
    /// HR color
    hr_color: u32,
    /// Callback for link clicks (NEW)
    on_link_click: Option<LinkClickCallback>,
}

impl RenderContext {
    fn from_theme(colors: &theme::ColorScheme) -> Self {
        Self {
            text_primary: colors.text.primary,
            text_secondary: colors.text.secondary,
            text_tertiary: colors.text.tertiary,
            accent_color: colors.accent.selected,
            code_bg: colors.background.search_box,
            quote_border: colors.ui.border,
            hr_color: colors.ui.border,
            on_link_click: None,  // Will be set in render()
        }
    }
    
    fn with_link_callback(mut self, callback: LinkClickCallback) -> Self {
        self.on_link_click = Some(callback);
        self
    }
}
```

### Step 5: Update render_element for Link with Click Handler (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: Replace the HtmlElement::Link case in render_element function

HtmlElement::Link { href, children } => {
    let text_content = collect_text(children);
    let href_clone = href.clone();
    
    let mut link_div = div()
        .id(gpui::ElementId::Name(format!("link:{}", href_clone).into()))
        .text_color(rgb(ctx.accent_color))
        .cursor_pointer()  // Show pointer cursor on hover
        .child(text_content);
    
    // Add click handler if callback is provided
    if let Some(ref callback) = ctx.on_link_click {
        let cb = callback.clone();
        let href_for_click = href_clone;
        link_div = link_div.on_mouse_down(
            gpui::MouseButton::Left,
            move |_event, _cx| {
                cb(&href_for_click);
            },
        );
    }
    
    link_div
}
```

Also update the render_inline function's Link case similarly:

```rust
// File: src/prompts/div.rs
// Location: Replace HtmlElement::Link case in render_inline function

HtmlElement::Link { href, children } => {
    let href_clone = href.clone();
    
    let mut link_div = div()
        .id(gpui::ElementId::Name(format!("link-inline:{}", href_clone).into()))
        .flex()
        .flex_row()
        .items_baseline()
        .text_color(rgb(ctx.accent_color))
        .cursor_pointer()
        .children(children.iter().map(|c| render_inline(c, ctx.clone())));
    
    if let Some(ref callback) = ctx.on_link_click {
        let cb = callback.clone();
        let href_for_click = href_clone;
        link_div = link_div.on_mouse_down(
            gpui::MouseButton::Left,
            move |_event, _cx| {
                cb(&href_for_click);
            },
        );
    }
    
    link_div
}
```

### Step 6: Update Render Method with Scrolling and Link Callback (src/prompts/div.rs)

```rust
// File: src/prompts/div.rs
// Location: Replace the entire Render impl

impl Render for DivPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        // Key handler for Enter/Escape
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  _cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                match key_str.as_str() {
                    "enter" | "escape" => this.submit(),
                    _ => {}
                }
            },
        );

        // Create link click callback that captures entity handle
        let entity_handle = cx.entity().clone();
        let on_link_click: LinkClickCallback = Arc::new(move |href: &str| {
            let href_owned = href.to_string();
            entity_handle.update(|this, _cx| {
                this.handle_link_click(&href_owned);
            }).ok();
        });

        // Parse HTML into elements
        let elements = parse_html(&self.html);

        // Create render context with link callback
        let render_ctx = if self.design_variant == DesignVariant::Default {
            RenderContext::from_theme(&self.theme.colors)
                .with_link_callback(on_link_click)
        } else {
            RenderContext {
                text_primary: colors.text_primary,
                text_secondary: colors.text_secondary,
                text_tertiary: colors.text_muted,
                accent_color: colors.accent,
                code_bg: colors.background_tertiary,
                quote_border: colors.border,
                hr_color: colors.border,
                on_link_click: Some(on_link_click),
            }
        };

        // Container background (same logic as before)
        let container_bg = if let Some(custom_bg) = self.container_options.parse_background() {
            custom_bg
        } else if self.design_variant == DesignVariant::Default {
            let base_color = self.theme.colors.background.main;
            if let Some(opacity) = self.container_options.opacity {
                rgb_to_hsla(base_color, Some(opacity))
            } else {
                Hsla::from(rgb(base_color))
            }
        } else if let Some(opacity) = self.container_options.opacity {
            rgb_to_hsla(colors.background, Some(opacity))
        } else {
            Hsla::from(rgb(colors.background))
        };

        let container_padding = self.container_options.get_padding(spacing.padding_lg);
        let panel_semantic_id = format!("panel:content-{}", self.id);

        // Render elements
        let content = render_elements(&elements, render_ctx);
        let styled_content = if let Some(tw) = &self.tailwind {
            apply_tailwind_styles(content, tw)
        } else {
            content
        };

        // Build SCROLLABLE content container (CHANGED from overflow_y_hidden)
        let content_base = div()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_y_scroll()  // CHANGED: Enable native scrolling
            .track_scroll(&self.scroll_handle)  // Track scroll position
            .child(styled_content);

        let content_styled = if let Some(ref classes) = self.container_options.container_classes {
            apply_tailwind_styles(content_base, classes)
        } else {
            content_base
        };

        let content_container = content_styled.id(gpui::ElementId::Name(panel_semantic_id.into()));

        // Main container
        div()
            .id(gpui::ElementId::Name("window:div".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .bg(container_bg)
            .p(px(container_padding))
            .key_context("div_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(content_container)
    }
}
```

### Step 7: Add open crate dependency (Cargo.toml)

```toml
# File: Cargo.toml
# Location: [dependencies] section - add this line

open = "5"
```

### Step 8: Update SDK div() Function (scripts/kit-sdk.ts)

```typescript
// File: scripts/kit-sdk.ts
// Location: Replace the div function (around line 2517)

/**
 * Display HTML content to user.
 * 
 * Matches original Script Kit API: div(htmlOrConfig?, actions?)
 * Supports submit: protocol links that return the clicked value.
 * 
 * @param htmlOrConfig - HTML string or DivConfig object
 * @param actions - Optional actions for the actions panel (Cmd+K)
 * @returns The value from a submit: link click, or undefined if dismissed
 * 
 * @example
 * // Basic usage - returns undefined on dismiss
 * await div("<h1>Hello World</h1>");
 * 
 * @example
 * // With submit links - returns clicked value
 * const choice = await div(md(`
 * # Pick an option
 * - [Option A](submit:optionA)
 * - [Option B](submit:optionB)
 * `));
 * console.log(choice); // "optionA" or "optionB"
 */
globalThis.div = async function div(
  htmlOrConfig?: string | DivConfig,
  actionsInput?: Action[]
): Promise<string | void> {  // CHANGED: Return type now includes string
  const id = nextId();
  
  // Parse arguments - support both string and config object
  let html: string;
  let config: DivConfig | undefined;
  
  if (typeof htmlOrConfig === 'string') {
    html = htmlOrConfig;
  } else if (typeof htmlOrConfig === 'object' && htmlOrConfig !== null) {
    config = htmlOrConfig;
    html = config.html || '';
  } else {
    html = '';
  }
  
  // Process actions (same as before - no changes needed)
  let serializedActions: SerializableAction[] | undefined;
  if (actionsInput && actionsInput.length > 0) {
    const actionsMap = (globalThis as any).__kitActionsMap as Map<string, Action>;
    actionsMap.clear();

    const seen = new Set<string>();
    const normalized: Action[] = [];

    for (const action of actionsInput) {
      if (action.visible === false) continue;
      const name = action.name?.trim();
      if (!name) continue;
      if (seen.has(name)) continue;
      seen.add(name);

      const hasHandler = typeof action.onAction === 'function';
      const hasValue = action.value !== undefined;
      if (!hasHandler && !hasValue) continue;

      actionsMap.set(name, action);
      normalized.push(action);
    }

    if (normalized.length > 0) {
      serializedActions = normalized.map(action => ({
        name: action.name,
        description: action.description,
        shortcut: action.shortcut,
        value: action.value,
        hasAction: typeof action.onAction === 'function',
        visible: action.visible,
        close: action.close,
      }));
    }
  }
  
  return new Promise((resolve) => {
    // CHANGED: Handler now receives optional value from submit links
    pending.set(id, (value?: string) => {
      resolve(value);  // Returns string if submit:value clicked, undefined otherwise
    });
    
    const message: DivMessage = {
      type: 'div',
      id,
      html,
      containerClasses: config?.containerClasses,
      actions: serializedActions,
      placeholder: config?.placeholder,
      hint: config?.hint,
      footer: config?.footer,
      containerBg: config?.containerBg,
      containerPadding: config?.containerPadding,
      opacity: config?.opacity,
    };
    
    send(message);
  });
};
```

### Step 9: Update Submit Handler in SDK (scripts/kit-sdk.ts)

```typescript
// File: scripts/kit-sdk.ts
// Location: In the processLine() function, find the submit handler and update

// Find this section (around line 1200-1210):
if (msg.type === 'submit') {
  const callback = pending.get(msg.id);
  if (callback) {
    pending.delete(msg.id);
    callback(msg.value);  // CHANGED: Pass value to callback (was just callback())
  }
}
```

---

## Testing

### Test 1: Scrollable Content

Create `tests/smoke/test-div-scroll.ts`:

```typescript
import '../../scripts/kit-sdk';

// Generate lots of content to test scrolling
const lotsOfContent = Array.from({ length: 50 }, (_, i) => 
  `## Item ${i + 1}\n\nThis is paragraph ${i + 1} with some content to make it longer.`
).join('\n\n');

await div(md(lotsOfContent));
console.error('[TEST] Scrollable div dismissed');
process.exit(0);
```

Run: `cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-div-scroll.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

Expected: Content should scroll smoothly with native scroll behavior.

### Test 2: Submit Links

Create `tests/smoke/test-div-submit-links.ts`:

```typescript
import '../../scripts/kit-sdk';

const choice = await div(md(`
# Pick a Color

Click a color to select it:

- [Red](submit:red)
- [Green](submit:green)  
- [Blue](submit:blue)
`));

console.error(`[TEST] Selected: ${choice}`);

if (choice === 'red' || choice === 'green' || choice === 'blue') {
  console.error('[TEST] PASS - submit link returned correct value');
} else {
  console.error('[TEST] FAIL - unexpected value or undefined');
}

process.exit(0);
```

Run and click a link. Expected output: `[TEST] Selected: red` (or green/blue)

### Test 3: External Links

Create `tests/smoke/test-div-external-links.ts`:

```typescript
import '../../scripts/kit-sdk';

await div(md(`
# External Links Test

Click these links - they should open in your browser:

- [Google](https://google.com)
- [GitHub](https://github.com)
- [Script Kit](https://scriptkit.com)

Press Escape when done.
`));

console.error('[TEST] External links test complete');
process.exit(0);
```

Expected: Clicking links opens them in the default browser.

### Test 4: Mixed Links

Create `tests/smoke/test-div-mixed-links.ts`:

```typescript
import '../../scripts/kit-sdk';

const result = await div(md(`
# Mixed Links Demo

**External links** (open in browser):
- [Documentation](https://docs.scriptkit.com)

**Submit links** (return value and close):
- [Accept](submit:accept)
- [Decline](submit:decline)

---

Click a submit link to complete, or Escape to cancel.
`));

console.error(`[TEST] Result: ${result ?? 'dismissed'}`);
process.exit(0);
```

---

## Instructions For The Next AI Agent

You are reading the "DivPrompt Scrollable Content & Clickable Links Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:

* Design and describe the minimal, safe changes needed to fully resolve the issues described in the Executive Summary and Key Problems.
* Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `src/prompts/div.rs`) and, when possible, line numbers or a clear description of the location (e.g. "replace the existing `render_element` function").
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change, or show both "before" and "after" versions.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.
6. **Follow TDD**: Write failing tests FIRST, then implement the minimum code to pass, then refactor.

### TDD Approach

1. **Red Phase**: Write the test scripts in `tests/smoke/` that will fail initially
2. **Green Phase**: Implement the changes to make tests pass
3. **Refactor Phase**: Clean up code while keeping tests green

### Key Implementation Notes

1. **GPUI Scrolling**: Use `ScrollHandle` with `.overflow_y_scroll()` and `.track_scroll()` - this provides native scrolling behavior.

2. **Click Handlers**: GPUI uses `.on_mouse_down(MouseButton::Left, callback)` for click handling. The callback receives the event and context.

3. **Opening URLs**: Use the `open` crate (`open::that(url)`) which opens URLs/files in the system default application.

4. **Protocol Detection**: Check href prefix:
   - `submit:` -> extract value, call submit callback with value
   - `http://` or `https://` -> open in browser
   - `file://` -> open local file
   - Other -> log and ignore

5. **SDK Type Change**: The div() return type change from `Promise<void>` to `Promise<string | void>` is backward compatible - existing code that ignores the return value will continue to work.

6. **Entity Handle Pattern**: To call methods on `self` from within a callback, capture `cx.entity().clone()` and use `entity_handle.update(|this, cx| { ... })`.

When you answer, work directly with the code and instructions contained here and return a clear, step-by-step plan plus exact code edits.

---

## Appendix: Current File Summaries

| File | Lines | Purpose |
|------|-------|---------|
| `src/prompts/div.rs` | 921 | DivPrompt component, HTML rendering |
| `src/utils.rs` | 1705 | HtmlElement enum, HTML parser, Tailwind parser |
| `src/protocol/message.rs` | 1783 | Protocol messages including Submit |
| `scripts/kit-sdk.ts` | ~3300 | SDK functions including div() |
| `tests/smoke/test-div-*.ts` | Various | Existing div visual tests |
