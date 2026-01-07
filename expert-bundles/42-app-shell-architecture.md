# Script Kit GPUI - Expert Bundle 42: App Shell Architecture

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Architecture:**
- **GPUI** for UI rendering (custom immediate-mode reactive UI framework from Zed)
- **Bun** as the TypeScript runtime for user scripts
- **Stdin/stdout JSON protocol** for bidirectional script <-> app communication
- **SQLite** for persistence (clipboard history, notes, chat)
- **macOS-first** with floating panel window behavior

**Key Constraints:**
- Must maintain backwards compatibility with existing Script Kit scripts
- Performance-critical: launcher must appear instantly, list scrolling at 60fps
- Multi-window: main launcher + Notes window + AI chat window (all independent)
- Theme hot-reload across all windows

---

## Goal

Create a consistent, reusable **App Shell** architecture for the main window that provides:
1. Predictable layout structure (header, content, footer regions)
2. Consistent focus management and keyboard navigation
3. Unified vibrancy/transparency handling
4. Reusable patterns for all prompt types

---

## Current State

The main window (`ScriptListApp` in `src/main.rs`) is a ~700 line struct with:
- Mixed concerns: window lifecycle, state management, rendering, event handling
- No clear separation between shell (chrome) and content (views)
- Each view (ArgPrompt, DivPrompt, EditorPrompt, etc.) independently renders headers/footers

### Current Structure
```rust
struct ScriptListApp {
    // ~45 fields mixing state, UI, and business logic
    current_view: AppView,
    scripts: Vec<Arc<Script>>,
    theme: Theme,
    focus_handle: FocusHandle,
    // ... dozens more
}

impl Render for ScriptListApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Giant match on current_view
        let main_content = match self.current_view {
            AppView::ScriptList => self.render_script_list(cx),
            AppView::ArgPrompt { .. } => self.render_arg_prompt(...),
            AppView::DivPrompt { .. } => self.render_div_prompt(...),
            // ... 12+ more variants
        };
        // Wrap with grid overlay, etc.
    }
}
```

---

## Problems

### 1. No Consistent Shell Structure
Each prompt type independently builds its own layout:
- `render_script_list()` builds header inline
- `render_arg_prompt()` duplicates header construction
- `render_div_prompt()` has yet another header implementation

### 2. Header Duplication
The header pattern (search input + buttons + logo) is rebuilt in:
- `render_script_list.rs` (~200 lines)
- `render_prompts/arg.rs` (~100 lines)
- `render_prompts/path.rs` (~100 lines)
- `render_prompts/editor.rs` (~80 lines)

### 3. Focus Management Scattered
Focus enforcement logic is duplicated across:
- `ScriptListApp::render()` (~100 lines of focus checks)
- Individual prompt render methods
- Actions dialog overlay

### 4. Footer Patterns Missing
No consistent footer region for:
- Hint text (e.g., "Press Tab to autocomplete")
- Character count (editor prompts)
- Progress indicators
- Action shortcuts summary

### 5. Vibrancy Applied Per-View
Each view independently handles:
- Background opacity calculation
- Box shadow creation
- Border radius application

---

## Proposed Architecture

### App Shell Component

```rust
/// Unified app shell providing consistent chrome for all views
pub struct AppShell {
    /// Header configuration (search input, buttons, etc.)
    header: Option<HeaderConfig>,
    /// Footer configuration (hints, shortcuts, etc.)
    footer: Option<FooterConfig>,
    /// Theme and vibrancy settings
    theme: Theme,
    /// Focus management
    focus_handle: FocusHandle,
}

impl AppShell {
    pub fn new(theme: Theme, focus_handle: FocusHandle) -> Self { ... }
    
    /// Build the shell with a content slot
    pub fn render<E: IntoElement>(
        &self,
        content: E,
        cx: &mut Context<impl Any>,
    ) -> impl IntoElement {
        let opacity = self.theme.get_opacity();
        let bg = self.hex_to_rgba_with_opacity(self.theme.colors.background.main, opacity.main);
        let shadows = self.create_box_shadows();
        
        div()
            .track_focus(&self.focus_handle)
            .bg(rgba(bg))
            .shadow(shadows)
            .rounded(px(12.))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // Header region
            .when_some(self.header.as_ref(), |d, h| d.child(h.render(cx)))
            // Divider
            .child(self.render_divider())
            // Content region (flex_1)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(content)
            )
            // Footer region
            .when_some(self.footer.as_ref(), |d, f| d.child(f.render(cx)))
    }
}
```

### Header Configuration

```rust
pub struct HeaderConfig {
    /// Input field configuration
    pub input: Option<InputConfig>,
    /// Buttons to show (Run, Actions, etc.)
    pub buttons: Vec<ButtonConfig>,
    /// Show logo
    pub show_logo: bool,
    /// Custom right-side content
    pub trailing: Option<Box<dyn Fn(&mut Context<Any>) -> AnyElement>>,
}

pub struct InputConfig {
    pub placeholder: String,
    pub value: String,
    pub on_change: Option<Box<dyn Fn(String)>>,
    pub show_cursor: bool,
}

pub struct ButtonConfig {
    pub label: String,
    pub shortcut: Option<String>,
    pub on_click: Box<dyn Fn()>,
}
```

### Footer Configuration

```rust
pub struct FooterConfig {
    /// Hint text (left-aligned)
    pub hint: Option<String>,
    /// Character count (editor prompts)
    pub char_count: Option<(usize, Option<usize>)>, // (current, max)
    /// Shortcut summary (right-aligned)
    pub shortcuts: Vec<(String, String)>, // (key, description)
}
```

---

## Usage Example

```rust
// In ScriptListApp::render_arg_prompt()
fn render_arg_prompt(&self, ...) -> impl IntoElement {
    let shell = AppShell::new(self.theme.clone(), self.focus_handle.clone())
        .with_header(HeaderConfig {
            input: Some(InputConfig {
                placeholder: placeholder.clone(),
                value: self.arg_input.text().to_string(),
                on_change: Some(Box::new(|text| { /* update state */ })),
                show_cursor: true,
            }),
            buttons: vec![
                ButtonConfig {
                    label: "Submit".to_string(),
                    shortcut: Some("Enter".to_string()),
                    on_click: Box::new(|| { /* submit */ }),
                },
            ],
            show_logo: true,
            trailing: None,
        })
        .with_footer(FooterConfig {
            hint: Some("Type to filter, Enter to select".to_string()),
            char_count: None,
            shortcuts: vec![("Esc", "Cancel"), ("Tab", "Complete")],
        });
    
    shell.render(
        self.render_arg_content(choices, selected_index),
        cx
    )
}
```

---

## Key Benefits

1. **Single Source of Truth** - All layout chrome in one place
2. **Consistent Look** - Every prompt has same header/footer style
3. **Focus Centralized** - Shell handles focus management
4. **Vibrancy Unified** - Background, shadows handled once
5. **Easy Testing** - Shell can be tested in isolation

---

## Implementation Checklist

- [ ] Create `src/app_shell/mod.rs` module
- [ ] Implement `AppShell` struct with render method
- [ ] Create `HeaderConfig` and `FooterConfig` types
- [ ] Extract vibrancy helpers to shell (from `app_impl.rs`)
- [ ] Create `shell_builder.rs` with fluent API
- [ ] Migrate `render_script_list` to use shell
- [ ] Migrate `render_arg_prompt` to use shell
- [ ] Migrate `render_editor_prompt` to use shell
- [ ] Migrate remaining prompts (div, path, form, etc.)
- [ ] Add footer region for hints and shortcuts
- [ ] Document shell usage in `src/app_shell/README.md`
- [ ] Add visual tests for shell with different configs

---

## Key Questions

1. Should AppShell own the focus handle, or receive it as a parameter?
2. Should the shell handle keyboard events (global shortcuts like Cmd+K)?
3. How to handle prompts that need no header (e.g., HUD notifications)?
4. Should divider be optional/configurable (some designs omit it)?
5. Should shell emit layout measurements for debug grid overlay?

---

## Related Bundles

- Bundle 40: UI Layout Helpers - provides `vstack()`, `hstack()` primitives
- Bundle 43: Shared UI Components - provides Button, Input components
- Bundle 39: Window Management - handles window lifecycle separately
