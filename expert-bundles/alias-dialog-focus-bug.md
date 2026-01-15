# Alias Dialog Focus Bug - Expert Bundle

## Original Goal

> Why this dialog doesn't gain focus
>
> The "Set Alias" modal dialog appears but the user cannot type in the input field because it doesn't receive keyboard focus.

## Executive Summary

The alias input dialog in Script Kit GPUI appears visually but fails to capture keyboard focus. The root cause is that unlike the working `ShortcutRecorder` component (which is a proper GPUI entity with `Focusable` trait and its own `FocusHandle`), the alias input is implemented as plain state (`Option<AliasInputState>`) with a div-based fake input that has no focus handling infrastructure.

### Key Problems:
1. **No FocusHandle** - `AliasInputState` stores only `command_id`, `command_name`, and `alias_text` - it has no `FocusHandle` for keyboard event routing.
2. **Not a GPUI Entity** - The alias input is just state, not an `Entity<T>` that can implement `Focusable` trait.
3. **Fake Input Field** - The "input" is a `div()` displaying static text, not a component that can receive and process keyboard input.
4. **No Focus Call** - Unlike `ShortcutRecorder` which explicitly calls `window.focus(&recorder_fh, cx)`, the alias overlay never requests focus.

### Required Fixes:
1. **Create `AliasInput` Entity** - `src/components/alias_input.rs` - A proper GPUI entity with `FocusHandle` and `Focusable` implementation.
2. **Add `TextInputState` Integration** - Use the existing `TextInputState` component for text editing, selection, and clipboard support.
3. **Focus Management** - In `render_alias_input_overlay`, focus the entity's `FocusHandle` after creation (like shortcut recorder does).
4. **Wire Key Handlers** - Use `track_focus(&self.focus_handle)` and proper `on_key_down` handlers.

### Files Included:
- `src/main.rs` (lines 953-965, 1374-1375): `AliasInputState` struct and field declaration
- `src/app_impl.rs` (lines 3595-3860): `show_alias_input`, `render_alias_input_overlay` - the broken implementation
- `src/app_impl.rs` (lines 3379-3475): `render_shortcut_recorder_overlay` - the working pattern to follow
- `src/components/shortcut_recorder.rs`: Complete working example of modal entity with focus
- `src/components/text_input.rs`: `TextInputState` for text editing
- `src/aliases/persistence.rs`: Alias storage (load/save)

---

## File: src/main.rs (AliasInputState struct - lines 953-965)

```rust
/// State for the inline alias input overlay.
///
/// When this is Some, the alias input modal is displayed.
/// Used for configuring command aliases.
#[derive(Debug, Clone)]
struct AliasInputState {
    /// The unique command identifier (e.g., "builtin/clipboard-history", "app/com.apple.Safari")
    command_id: String,
    /// Human-readable name of the command being configured
    command_name: String,
    /// Current alias text being edited
    alias_text: String,
}
```

## File: src/main.rs (ScriptListApp struct fields - lines 1370-1375)

```rust
    /// Shortcut recorder state - when Some, shows the inline recorder overlay
    shortcut_recorder_state: Option<ShortcutRecorderState>,
    /// The shortcut recorder entity (persisted to maintain focus)
    shortcut_recorder_entity:
        Option<Entity<crate::components::shortcut_recorder::ShortcutRecorder>>,
    /// Alias input state - when Some, shows the alias input modal
    alias_input_state: Option<AliasInputState>,
```

**Note the difference**: `shortcut_recorder` has BOTH a `_state` AND an `_entity`. The entity holds the `FocusHandle` and implements `Focusable`. The alias input only has `_state` - no entity!

## File: src/main.rs (Focusable trait - lines 1434-1437)

```rust
impl Focusable for ScriptListApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
```

---

## File: src/app_impl.rs (show_alias_input - lines 3595-3631)

```rust
    /// Show the alias input overlay for configuring a command alias.
    ///
    /// The alias input allows users to set a text alias that can be typed
    /// in the main menu to quickly run a command.
    fn show_alias_input(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "ALIAS",
            &format!(
                "Showing alias input for '{}' (id: {})",
                command_name, command_id
            ),
        );

        // Load existing alias if any
        let existing_alias = crate::aliases::load_alias_overrides()
            .ok()
            .and_then(|overrides| overrides.get(&command_id).cloned())
            .unwrap_or_default();

        // Store state
        self.alias_input_state = Some(AliasInputState {
            command_id,
            command_name,
            alias_text: existing_alias,
        });

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        cx.notify();
    }
```

**BUG**: This only sets `alias_input_state`. It doesn't create an entity with a `FocusHandle`.

---

## File: src/app_impl.rs (render_alias_input_overlay - lines 3719-3858) - THE BROKEN CODE

```rust
    /// Render the alias input overlay if state is set.
    ///
    /// Returns None if no alias input is active.
    fn render_alias_input_overlay(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let state = self.alias_input_state.as_ref()?;
        let command_name = state.command_name.clone();
        let alias_text = state.alias_text.clone();

        // Get design tokens for styling
        let tokens = crate::designs::get_tokens(self.current_design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();

        // Create save handler
        let on_save = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.save_alias(cx);
        });

        // Create cancel handler
        let on_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.close_alias_input(cx);
        });

        // Create key handler for Enter/Escape
        let on_key = cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            match key {
                "enter" | "Enter" => {
                    this.save_alias(cx);
                }
                "escape" | "Escape" => {
                    this.close_alias_input(cx);
                }
                _ => {}
            }
        });

        // Build the overlay UI
        Some(
            div()
                .id("alias-input-overlay")
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui::rgba(0x00000080)) // Semi-transparent background
                .on_key_down(on_key)  // BUG: Without focus, this never fires!
                .child(
                    div()
                        .id("alias-input-modal")
                        .flex()
                        .flex_col()
                        .w(gpui::px(400.0))
                        .p(gpui::px(spacing.padding_xl))
                        .bg(gpui::rgb(colors.background))
                        .rounded(gpui::px(visual.radius_lg))
                        .border_1()
                        .border_color(gpui::rgb(colors.border))
                        .gap(gpui::px(spacing.gap_md))
                        // Header
                        .child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(colors.text_primary))
                                .font_family(typography.font_family)
                                .child(format!("Set Alias for \"{}\"", command_name)),
                        )
                        // Description
                        .child(
                            div()
                                .text_sm()
                                .text_color(gpui::rgb(colors.text_muted))
                                .font_family(typography.font_family)
                                .child(
                                    "Type the alias + space in the main menu to run this command",
                                ),
                        )
                        // Input field (simplified - just show the current value)
                        // BUG: This is NOT a real input! Just a div showing text.
                        .child(
                            div()
                                .id("alias-input-field")
                                .w_full()
                                .px(gpui::px(spacing.padding_md))
                                .py(gpui::px(spacing.padding_sm))
                                .bg(gpui::rgb(colors.background_secondary))
                                .rounded(gpui::px(visual.radius_md))
                                .border_1()
                                .border_color(gpui::rgb(colors.border))
                                .text_color(gpui::rgb(colors.text_primary))
                                .font_family(typography.font_family)
                                .child(if alias_text.is_empty() {
                                    div()
                                        .text_color(gpui::rgb(colors.text_dimmed))
                                        .child("Enter alias (e.g., 'ch' for Clipboard History)")
                                } else {
                                    div().child(alias_text.clone())
                                }),
                        )
                        // Buttons
                        .child(
                            div()
                                .flex()
                                .gap(gpui::px(spacing.gap_sm))
                                .justify_end()
                                .child(
                                    div()
                                        .id("cancel-button")
                                        .px(gpui::px(spacing.padding_md))
                                        .py(gpui::px(spacing.padding_sm))
                                        .rounded(gpui::px(visual.radius_md))
                                        .bg(gpui::rgb(colors.background_tertiary))
                                        .text_color(gpui::rgb(colors.text_primary))
                                        .cursor_pointer()
                                        .on_click(on_cancel)
                                        .child("Cancel"),
                                )
                                .child(
                                    div()
                                        .id("save-button")
                                        .px(gpui::px(spacing.padding_md))
                                        .py(gpui::px(spacing.padding_sm))
                                        .rounded(gpui::px(visual.radius_md))
                                        .bg(gpui::rgb(colors.accent))
                                        .text_color(gpui::rgb(colors.text_on_accent))
                                        .cursor_pointer()
                                        .on_click(on_save)
                                        .child("Save"),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }
```

**PROBLEMS**:
1. No `.track_focus(&focus_handle)` - keyboard events can't route here
2. The "input field" is a `div()` showing static text - not editable
3. No `window.focus(...)` call to grab focus
4. No entity creation with its own `FocusHandle`

---

## File: src/app_impl.rs (render_shortcut_recorder_overlay - lines 3379-3450) - THE WORKING PATTERN

```rust
    /// Render the shortcut recorder overlay if state is set.
    ///
    /// The recorder is created once and persisted to maintain keyboard focus.
    /// Callbacks use cx.entity() to communicate back to the parent app.
    fn render_shortcut_recorder_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        use crate::components::shortcut_recorder::ShortcutRecorder;

        // Check if we have state but no entity yet - need to create the recorder
        let state = self.shortcut_recorder_state.as_ref()?;

        // Create entity if needed (only once per show)
        if self.shortcut_recorder_entity.is_none() {
            let command_id = state.command_id.clone();
            let command_name = state.command_name.clone();
            let theme = std::sync::Arc::clone(&self.theme);

            // Get a weak reference to the app for callbacks
            let app_entity = cx.entity().downgrade();
            let app_entity_for_cancel = app_entity.clone();

            let recorder = cx.new(move |cx| {
                // Create the recorder with its own focus handle from its own context
                // This is CRITICAL for keyboard events to work
                let mut r = ShortcutRecorder::new(cx, theme);
                r.set_command_name(Some(command_name.clone()));
                r.set_command_description(Some(format!("ID: {}", command_id)));

                // Set save callback
                let app_for_save = app_entity.clone();
                r.on_save = Some(Box::new(move |recorded| {
                    logging::log("SHORTCUT", &format!(
                        "Recorder on_save triggered: {}",
                        recorded.to_config_string()
                    ));
                    if app_for_save.upgrade().is_some() {
                        logging::log("SHORTCUT", "Save callback - app entity available");
                    }
                }));

                // Set cancel callback
                let app_for_cancel = app_entity_for_cancel.clone();
                r.on_cancel = Some(Box::new(move || {
                    logging::log("SHORTCUT", "Recorder on_cancel triggered");
                    if let Some(_app) = app_for_cancel.upgrade() {
                        logging::log("SHORTCUT", "Cancel callback - app entity available");
                    }
                }));

                r
            });

            self.shortcut_recorder_entity = Some(recorder);
            logging::log("SHORTCUT", "Created new shortcut recorder entity");
        }

        // Get the existing entity
        let recorder = self.shortcut_recorder_entity.as_ref()?;

        // ALWAYS focus the recorder to ensure it captures keyboard input
        // This is critical for modal behavior - the recorder must have focus
        let recorder_fh = recorder.read(cx).focus_handle.clone();
        let was_focused = recorder_fh.is_focused(window);
        window.focus(&recorder_fh, cx);  // <-- THIS IS THE KEY LINE
        if !was_focused {
            logging::log("SHORTCUT", "Focused shortcut recorder (was not focused)");
        }

        // Check for pending actions from the recorder (Save or Cancel)
        let pending_action = recorder.update(cx, |r, _cx| r.take_pending_action());
        // ... handle pending action ...

        Some(recorder.clone().into_any_element())
    }
```

**KEY DIFFERENCES FROM ALIAS INPUT**:
1. Creates an `Entity<ShortcutRecorder>` via `cx.new(...)` 
2. The entity creates its own `FocusHandle` via `cx.focus_handle()` in `ShortcutRecorder::new()`
3. Explicitly focuses: `window.focus(&recorder_fh, cx)`
4. Entity is persisted in `self.shortcut_recorder_entity` for re-render stability

---

## File: src/components/shortcut_recorder.rs - Complete Working Example

```rust
//! ShortcutRecorder - Modal component for recording keyboard shortcuts
//!
//! Usage:
//! let recorder = ShortcutRecorder::new(focus_handle, theme)
//!     .with_command_name("My Command")
//!     .on_save(|shortcut| { /* handle save */ })
//!     .on_cancel(|| { /* handle cancel */ });

use gpui::{
    div, px, rgb, rgba, App, Context, Element, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
};
// ... imports ...

/// Shortcut Recorder Modal Component
///
/// A modal dialog for recording keyboard shortcuts with visual feedback.
pub struct ShortcutRecorder {
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Pre-computed colors
    pub colors: ShortcutRecorderColors,
    /// Name of the command being configured
    pub command_name: Option<String>,
    /// Description of the command
    pub command_description: Option<String>,
    /// Currently recorded shortcut (final result with key)
    pub shortcut: RecordedShortcut,
    /// Currently held modifiers (for live display before final key)
    pub current_modifiers: gpui::Modifiers,
    /// Current conflict if any
    pub conflict: Option<ShortcutConflict>,
    /// Callback when save is pressed
    pub on_save: Option<OnSaveCallback>,
    /// Callback when cancel is pressed
    pub on_cancel: Option<OnCancelCallback>,
    /// Function to check for conflicts
    pub conflict_checker: Option<ConflictChecker>,
    /// Whether recording is active (listening for keys)
    pub is_recording: bool,
    /// Pending action for the parent to handle (polled after render)
    pub pending_action: Option<RecorderAction>,
}

impl ShortcutRecorder {
    /// Create a new shortcut recorder
    /// The focus_handle MUST be created from the entity's own context (cx.focus_handle())
    /// for keyboard events to work properly.
    pub fn new(cx: &mut Context<Self>, theme: Arc<Theme>) -> Self {
        let colors = ShortcutRecorderColors::from_theme(&theme);
        // Create focus handle from THIS entity's context - critical for keyboard events
        let focus_handle = cx.focus_handle();  // <-- THIS IS KEY
        logging::log("SHORTCUT", "Created ShortcutRecorder with new focus handle");
        Self {
            focus_handle,
            theme,
            colors,
            command_name: None,
            // ... rest of initialization ...
        }
    }
    // ... other methods ...
}

// THIS TRAIT IMPLEMENTATION IS CRITICAL FOR FOCUS
impl Focusable for ShortcutRecorder {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutRecorder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ... build UI ...

        // Key down event handler - captures modifiers and keys
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;

            match key.to_lowercase().as_str() {
                "escape" => this.handle_escape(cx),
                "enter" if this.shortcut.is_complete() && this.conflict.is_none() => {
                    this.save();
                    cx.notify();
                }
                _ => this.handle_key_down(key, mods, cx),
            }
        });

        // Full-screen overlay with backdrop and centered modal
        // The overlay captures ALL keyboard and modifier events while open
        div()
            .id("shortcut-recorder-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)  // <-- THIS ENABLES KEYBOARD ROUTING
            .on_key_down(handle_key_down)
            .on_modifiers_changed(handle_modifiers_changed)
            .child(/* backdrop */)
            .child(/* modal content */)
    }
}
```

---

## File: src/components/text_input.rs - For Text Editing

```rust
//! TextInput - Single-line text input with selection and clipboard support
//!
//! A reusable component for text input fields that supports:
//! - Text selection (shift+arrows, cmd+a, mouse drag)
//! - Clipboard operations (cmd+c, cmd+v, cmd+x)
//! - Word navigation (alt+arrows)
//! - Standard cursor movement (arrows, home/end)

/// State for a single-line text input with selection support
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// The text content
    text: String,
    /// Selection state (anchor and cursor positions)
    selection: TextSelection,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selection: TextSelection::caret(0),
        }
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let len = text.chars().count();
        Self {
            text,
            selection: TextSelection::caret(len), // Cursor at end
        }
    }

    // === Getters ===
    pub fn text(&self) -> &str { &self.text }
    pub fn cursor(&self) -> usize { self.selection.cursor }
    pub fn selection(&self) -> TextSelection { self.selection }
    pub fn has_selection(&self) -> bool { !self.selection.is_empty() }
    pub fn is_empty(&self) -> bool { self.text.is_empty() }

    // === Text Manipulation ===
    
    /// Insert a character at cursor, replacing selection if any
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.selection.cursor);
        self.text.insert(byte_pos, ch);
        self.selection = TextSelection::caret(self.selection.cursor + 1);
    }

    /// Insert a string at cursor, replacing selection if any
    pub fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.selection.cursor);
        self.text.insert_str(byte_pos, s);
        let new_pos = self.selection.cursor + s.chars().count();
        self.selection = TextSelection::caret(new_pos);
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.has_selection() {
            self.delete_selection();
        } else if self.selection.cursor > 0 {
            let pos = self.selection.cursor - 1;
            let byte_pos = self.char_to_byte(pos);
            let next_byte = self.char_to_byte(pos + 1);
            self.text.drain(byte_pos..next_byte);
            self.selection = TextSelection::caret(pos);
        }
    }

    /// Handle key input events (call from on_key_down handler)
    pub fn handle_key(
        &mut self,
        key: &str,
        modifiers: gpui::Modifiers,
        cx: &mut impl gpui::ClipboardContext,
    ) -> bool {
        match key {
            "backspace" | "Backspace" => { self.backspace(); true }
            "delete" | "Delete" => { self.delete(); true }
            "left" | "arrowleft" | "Left" | "ArrowLeft" => {
                if modifiers.alt { self.word_left(modifiers.shift); }
                else if modifiers.platform { self.home(modifiers.shift); }
                else { self.left(modifiers.shift); }
                true
            }
            "right" | "arrowright" | "Right" | "ArrowRight" => {
                if modifiers.alt { self.word_right(modifiers.shift); }
                else if modifiers.platform { self.end(modifiers.shift); }
                else { self.right(modifiers.shift); }
                true
            }
            "a" | "A" if modifiers.platform => { self.select_all(); true }
            "c" | "C" if modifiers.platform => { self.copy(cx); true }
            "x" | "X" if modifiers.platform => { self.cut(cx); true }
            "v" | "V" if modifiers.platform => { self.paste(cx); true }
            _ if key.chars().count() == 1 && !modifiers.platform && !modifiers.control => {
                self.insert_char(key.chars().next().unwrap());
                true
            }
            _ => false
        }
    }
    // ... more methods ...
}
```

---

## File: src/aliases/persistence.rs - Storage

```rust
//! Alias persistence for commands.
//!
//! Handles loading and saving user alias overrides to/from `~/.scriptkit/aliases.json`.
//! Format: HashMap<command_id, alias_string>

/// Get the default path for alias overrides (~/.scriptkit/aliases.json)
pub fn default_aliases_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".scriptkit")
        .join("aliases.json")
}

/// Load all alias overrides from ~/.scriptkit/aliases.json
pub fn load_alias_overrides() -> Result<HashMap<String, String>> { /* ... */ }

/// Save an alias override for a specific command
pub fn save_alias_override(command_id: &str, alias: &str) -> Result<()> { /* ... */ }

/// Remove an alias override for a specific command
pub fn remove_alias_override(command_id: &str) -> Result<()> { /* ... */ }
```

---

## Implementation Guide

### Step 1: Create AliasInput Entity Component

Create `src/components/alias_input.rs`:

```rust
//! AliasInput - Modal component for setting command aliases
//!
//! A modal dialog for entering a text alias with proper focus handling.

use gpui::{
    div, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
};
use crate::components::TextInputState;
use crate::theme::Theme;
use std::sync::Arc;

pub enum AliasInputAction {
    Save(String),  // The alias text
    Cancel,
}

pub struct AliasInput {
    /// Focus handle for keyboard input - CRITICAL for events to work
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Command being configured
    pub command_name: String,
    pub command_id: String,
    /// Text input state for editing
    pub input: TextInputState,
    /// Pending action for parent to poll
    pub pending_action: Option<AliasInputAction>,
}

impl AliasInput {
    pub fn new(
        cx: &mut Context<Self>,
        theme: Arc<Theme>,
        command_id: String,
        command_name: String,
        existing_alias: String,
    ) -> Self {
        // Create focus handle from THIS entity's context - CRITICAL
        let focus_handle = cx.focus_handle();
        
        Self {
            focus_handle,
            theme,
            command_name,
            command_id,
            input: TextInputState::with_text(existing_alias),
            pending_action: None,
        }
    }

    pub fn save(&mut self) {
        let alias = self.input.text().trim().to_string();
        self.pending_action = Some(AliasInputAction::Save(alias));
    }

    pub fn cancel(&mut self) {
        self.pending_action = Some(AliasInputAction::Cancel);
    }

    pub fn take_pending_action(&mut self) -> Option<AliasInputAction> {
        self.pending_action.take()
    }
}

// THIS IS CRITICAL - without Focusable, keyboard events won't route
impl Focusable for AliasInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AliasInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = /* get from theme */;
        
        // Key handler for text input + Enter/Escape
        let handle_key = cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;
            
            match key.to_lowercase().as_str() {
                "escape" => this.cancel(),
                "enter" => this.save(),
                _ => {
                    // Delegate to TextInputState for text editing
                    if this.input.handle_key(key, mods, window) {
                        cx.notify();
                    }
                }
            }
        });

        div()
            .id("alias-input-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)  // <-- ENABLES KEYBOARD ROUTING
            .on_key_down(handle_key)
            .child(/* backdrop */)
            .child(/* modal with input rendering */)
    }
}
```

### Step 2: Add Entity Field to ScriptListApp

In `src/main.rs`:

```rust
/// Alias input state - when Some, shows the alias input modal
alias_input_state: Option<AliasInputState>,
/// The alias input entity (persisted to maintain focus)
alias_input_entity: Option<Entity<crate::components::alias_input::AliasInput>>,
```

### Step 3: Update render_alias_input_overlay

In `src/app_impl.rs`:

```rust
fn render_alias_input_overlay(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
) -> Option<gpui::AnyElement> {
    use crate::components::alias_input::AliasInput;

    let state = self.alias_input_state.as_ref()?;

    // Create entity if needed (only once per show)
    if self.alias_input_entity.is_none() {
        let command_id = state.command_id.clone();
        let command_name = state.command_name.clone();
        let existing_alias = state.alias_text.clone();
        let theme = Arc::clone(&self.theme);

        let input_entity = cx.new(move |cx| {
            AliasInput::new(cx, theme, command_id, command_name, existing_alias)
        });

        self.alias_input_entity = Some(input_entity);
    }

    let input_entity = self.alias_input_entity.as_ref()?;

    // ALWAYS focus the input to ensure it captures keyboard input
    let input_fh = input_entity.read(cx).focus_handle.clone();
    window.focus(&input_fh, cx);  // <-- THIS IS THE KEY FIX

    // Check for pending actions (Save or Cancel)
    let pending_action = input_entity.update(cx, |input, _cx| input.take_pending_action());

    if let Some(action) = pending_action {
        match action {
            AliasInputAction::Save(alias) => {
                self.handle_alias_save(&alias, cx);
            }
            AliasInputAction::Cancel => {
                self.close_alias_input(cx);
            }
        }
    }

    Some(input_entity.clone().into_any_element())
}
```

### Step 4: Update close_alias_input

```rust
pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
    if self.alias_input_state.is_some() || self.alias_input_entity.is_some() {
        logging::log("ALIAS", "Closing alias input");
        self.alias_input_state = None;
        self.alias_input_entity = None;  // <-- Also clear entity
        cx.notify();
    }
}
```

---

## Instructions for the Next AI Agent

### Context
You are fixing a focus bug in the alias input modal dialog. The modal appears but doesn't receive keyboard focus, so users cannot type in the input field.

### Root Cause
The alias input is implemented as plain state (`AliasInputState`) without a `FocusHandle` or proper GPUI entity. Compare with `ShortcutRecorder` which works correctly by:
1. Being a proper `Entity<T>` with `Focusable` trait
2. Creating its own `FocusHandle` via `cx.focus_handle()`
3. Using `.track_focus(&self.focus_handle)` in render
4. Parent explicitly calling `window.focus(&recorder_fh, cx)`

### Implementation Checklist
- [ ] Create `src/components/alias_input.rs` with `AliasInput` struct
- [ ] Implement `Focusable` trait for `AliasInput`
- [ ] Use `TextInputState` for text editing
- [ ] Add `alias_input_entity: Option<Entity<AliasInput>>` to `ScriptListApp`
- [ ] Update `show_alias_input` to initialize state properly
- [ ] Update `render_alias_input_overlay` to create entity and focus it
- [ ] Update `close_alias_input` to clear both state and entity
- [ ] Add `pub mod alias_input;` to `src/components/mod.rs`
- [ ] Run verification gate: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

### Key GPUI Focus Pattern
```rust
// 1. Entity creates its own FocusHandle
let focus_handle = cx.focus_handle();

// 2. Entity implements Focusable
impl Focusable for MyComponent {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// 3. Render uses track_focus
div().track_focus(&self.focus_handle).on_key_down(handler)

// 4. Parent explicitly focuses
window.focus(&entity_focus_handle, cx);
```

### Testing
After implementing, test by:
1. Build: `cargo build`
2. Run: `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Select any script/command and press `Cmd+K` to open actions
4. Select "Set Alias" action
5. Verify: The input field should be focused and accept keyboard input

OUTPUT_FILE_PATH: expert-bundles/alias-dialog-focus-bug.md
