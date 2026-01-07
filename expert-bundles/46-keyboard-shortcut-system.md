# Script Kit GPUI - Expert Bundle 46: Keyboard Shortcut System

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner.

**Keyboard Shortcuts Are Critical:**
- Global hotkey to invoke launcher (default: Cmd+;)
- Script-specific shortcuts (e.g., `// Shortcut: cmd+shift+g`)
- In-app shortcuts (Cmd+K for actions, Cmd+L for logs)
- Context-sensitive shortcuts (different in list vs. editor)

---

## Goal

Create a **unified keyboard shortcut system** that:
1. Centralizes all shortcut definitions
2. Handles conflicts gracefully
3. Supports user customization
4. Provides consistent display format
5. Enables discovery (shortcut hints, cheat sheet)

---

## Current State

### Shortcut Sources

| Source | Location | Format | Count |
|--------|----------|--------|-------|
| Global hotkey | `config.ts` | `{ modifiers: ["meta"], key: "Semicolon" }` | 3 |
| Script shortcuts | Script metadata | `// Shortcut: cmd+shift+g` | N/A |
| App shortcuts | `render_script_list.rs` | Inline match statements | ~15 |
| Actions shortcuts | `protocol.rs` | `ProtocolAction.shortcut` | ~10 |

### Problems

1. **No Central Registry** - Shortcuts defined inline across 10+ files
2. **Inconsistent Parsing** - Multiple parsers for shortcut strings
3. **No Conflict Detection** - Same shortcut can be bound twice
4. **Platform Differences** - Cmd vs Ctrl not handled uniformly
5. **No User Override** - Can't customize built-in shortcuts
6. **Discovery Gap** - No way to see all available shortcuts

### Current Key Handling

```rust
// In render_script_list.rs - inline pattern matching
let handle_key = cx.listener(|this, event, window, cx| {
    let key_str = event.keystroke.key.to_lowercase();
    let has_cmd = event.keystroke.modifiers.platform;
    let has_shift = event.keystroke.modifiers.shift;
    
    if has_cmd {
        match key_str.as_str() {
            "l" => this.toggle_logs(cx),
            "k" => this.toggle_actions(cx, window),
            "1" => this.cycle_design(cx),
            "e" => this.handle_action("edit_script", cx),
            "f" if has_shift => this.handle_action("reveal_in_finder", cx),
            // ... 10+ more cases
        }
    }
    
    match key_str.as_str() {
        "up" | "arrowup" => this.move_selection_up(cx),
        "down" | "arrowdown" => this.move_selection_down(cx),
        // ...
    }
});
```

---

## Proposed Architecture

### 1. Shortcut Definition

```rust
/// A keyboard shortcut binding
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// Key code (lowercase, e.g., "k", "enter", "f1")
    pub key: String,
    /// Modifier keys
    pub modifiers: Modifiers,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,    // Cmd on macOS, Ctrl on Windows/Linux
    pub ctrl: bool,   // Control key
    pub alt: bool,    // Option on macOS, Alt on Windows/Linux
    pub shift: bool,
}

impl Shortcut {
    /// Parse from string format (e.g., "cmd+shift+k")
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.to_lowercase().split('+').collect();
        let key = parts.last()?.to_string();
        let mut modifiers = Modifiers::default();
        
        for part in &parts[..parts.len()-1] {
            match *part {
                "cmd" | "meta" | "command" | "super" => modifiers.cmd = true,
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" | "opt" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                _ => {}
            }
        }
        
        Some(Self { key, modifiers })
    }
    
    /// Display format for UI (e.g., "⌘⇧K")
    pub fn display(&self) -> String {
        let mut s = String::new();
        if self.modifiers.ctrl { s.push_str("⌃"); }
        if self.modifiers.alt { s.push_str("⌥"); }
        if self.modifiers.shift { s.push_str("⇧"); }
        if self.modifiers.cmd { s.push_str("⌘"); }
        s.push_str(&self.key_display());
        s
    }
    
    /// Display format for key alone (uppercase for letters, symbols for special)
    fn key_display(&self) -> String {
        match self.key.as_str() {
            "enter" | "return" => "↵".to_string(),
            "escape" | "esc" => "⎋".to_string(),
            "tab" => "⇥".to_string(),
            "space" => "␣".to_string(),
            "backspace" | "delete" => "⌫".to_string(),
            "up" | "arrowup" => "↑".to_string(),
            "down" | "arrowdown" => "↓".to_string(),
            "left" | "arrowleft" => "←".to_string(),
            "right" | "arrowright" => "→".to_string(),
            k => k.to_uppercase(),
        }
    }
    
    /// Check if keystroke matches this shortcut
    pub fn matches(&self, keystroke: &gpui::Keystroke) -> bool {
        let key_matches = keystroke.key.to_lowercase() == self.key;
        let mods_match = 
            keystroke.modifiers.platform == self.modifiers.cmd &&
            keystroke.modifiers.control == self.modifiers.ctrl &&
            keystroke.modifiers.alt == self.modifiers.alt &&
            keystroke.modifiers.shift == self.modifiers.shift;
        key_matches && mods_match
    }
}
```

### 2. Shortcut Context

```rust
/// Context in which a shortcut is active
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    /// Always active (global to the app)
    Global,
    /// Active when main menu is shown
    MainMenu,
    /// Active when script list is focused
    ScriptList,
    /// Active when arg prompt is shown
    ArgPrompt,
    /// Active when editor is focused
    Editor,
    /// Active when terminal is focused
    Terminal,
    /// Active when actions dialog is open
    ActionsDialog,
    /// Active in any prompt
    AnyPrompt,
}

impl ShortcutContext {
    /// Check if this context is active for the current app state
    pub fn is_active(&self, current_view: &AppView, has_actions_popup: bool) -> bool {
        match self {
            Self::Global => true,
            Self::MainMenu => true, // Always in main menu context
            Self::ScriptList => matches!(current_view, AppView::ScriptList),
            Self::ArgPrompt => matches!(current_view, AppView::ArgPrompt { .. }),
            Self::Editor => matches!(current_view, AppView::EditorPrompt { .. }),
            Self::Terminal => matches!(current_view, AppView::TermPrompt { .. }),
            Self::ActionsDialog => has_actions_popup,
            Self::AnyPrompt => !matches!(current_view, AppView::ScriptList),
        }
    }
}
```

### 3. Shortcut Registry

```rust
/// Central registry of all keyboard shortcuts
pub struct ShortcutRegistry {
    /// Built-in shortcuts (cannot be removed, can be overridden)
    builtins: HashMap<String, ShortcutBinding>,
    /// Script-defined shortcuts
    scripts: HashMap<String, ShortcutBinding>,
    /// User overrides
    user_overrides: HashMap<String, Shortcut>,
    /// Disabled shortcuts
    disabled: HashSet<String>,
}

pub struct ShortcutBinding {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Default shortcut
    pub shortcut: Shortcut,
    /// Context where shortcut is active
    pub context: ShortcutContext,
    /// Category for organization
    pub category: ShortcutCategory,
    /// Whether user can override
    pub customizable: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum ShortcutCategory {
    Navigation,
    Actions,
    Edit,
    View,
    Scripts,
    System,
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtins();
        registry
    }
    
    fn register_builtins(&mut self) {
        // Navigation
        self.register(ShortcutBinding {
            id: "move_up".into(),
            name: "Move Up".into(),
            shortcut: Shortcut::parse("up").unwrap(),
            context: ShortcutContext::Global,
            category: ShortcutCategory::Navigation,
            customizable: true,
        });
        
        // Actions
        self.register(ShortcutBinding {
            id: "toggle_actions".into(),
            name: "Toggle Actions".into(),
            shortcut: Shortcut::parse("cmd+k").unwrap(),
            context: ShortcutContext::Global,
            category: ShortcutCategory::Actions,
            customizable: true,
        });
        
        self.register(ShortcutBinding {
            id: "submit".into(),
            name: "Submit / Run".into(),
            shortcut: Shortcut::parse("enter").unwrap(),
            context: ShortcutContext::Global,
            category: ShortcutCategory::Actions,
            customizable: false, // Enter is fundamental
        });
        
        // Edit
        self.register(ShortcutBinding {
            id: "edit_script".into(),
            name: "Edit Script".into(),
            shortcut: Shortcut::parse("cmd+e").unwrap(),
            context: ShortcutContext::ScriptList,
            category: ShortcutCategory::Edit,
            customizable: true,
        });
        
        // ... etc
    }
    
    /// Get effective shortcut (considering user overrides)
    pub fn get_shortcut(&self, id: &str) -> Option<Shortcut> {
        if self.disabled.contains(id) {
            return None;
        }
        
        self.user_overrides.get(id).cloned()
            .or_else(|| self.builtins.get(id).map(|b| b.shortcut.clone()))
            .or_else(|| self.scripts.get(id).map(|b| b.shortcut.clone()))
    }
    
    /// Find binding that matches keystroke in context
    pub fn find_match(
        &self,
        keystroke: &gpui::Keystroke,
        context: impl Fn(ShortcutContext) -> bool,
    ) -> Option<&str> {
        // Check scripts first (can override builtins)
        for (id, binding) in &self.scripts {
            if !self.disabled.contains(id) && context(binding.context) {
                let shortcut = self.user_overrides.get(id).unwrap_or(&binding.shortcut);
                if shortcut.matches(keystroke) {
                    return Some(id);
                }
            }
        }
        
        // Then builtins
        for (id, binding) in &self.builtins {
            if !self.disabled.contains(id) && context(binding.context) {
                let shortcut = self.user_overrides.get(id).unwrap_or(&binding.shortcut);
                if shortcut.matches(keystroke) {
                    return Some(id);
                }
            }
        }
        
        None
    }
    
    /// Register script shortcuts
    pub fn register_script(&mut self, script_path: &str, shortcut_str: &str) {
        if let Some(shortcut) = Shortcut::parse(shortcut_str) {
            let id = format!("script:{}", script_path);
            self.scripts.insert(id.clone(), ShortcutBinding {
                id,
                name: script_path.to_string(),
                shortcut,
                context: ShortcutContext::Global,
                category: ShortcutCategory::Scripts,
                customizable: false,
            });
        }
    }
    
    /// Detect conflicts
    pub fn find_conflicts(&self) -> Vec<ShortcutConflict> {
        // Group by (shortcut, context) and find duplicates
        let mut by_shortcut: HashMap<(Shortcut, ShortcutContext), Vec<String>> = HashMap::new();
        
        for (id, binding) in self.builtins.iter().chain(self.scripts.iter()) {
            if !self.disabled.contains(id) {
                let shortcut = self.user_overrides.get(id).unwrap_or(&binding.shortcut);
                by_shortcut
                    .entry((shortcut.clone(), binding.context))
                    .or_default()
                    .push(id.clone());
            }
        }
        
        by_shortcut.into_iter()
            .filter(|(_, ids)| ids.len() > 1)
            .map(|((shortcut, context), ids)| ShortcutConflict {
                shortcut,
                context,
                binding_ids: ids,
            })
            .collect()
    }
}

pub struct ShortcutConflict {
    pub shortcut: Shortcut,
    pub context: ShortcutContext,
    pub binding_ids: Vec<String>,
}
```

### 4. Shortcut Handler

```rust
/// Simplified key event handling using registry
impl ScriptListApp {
    fn handle_key_event(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let registry = SHORTCUT_REGISTRY.lock().unwrap();
        
        // Build context checker
        let current_view = &self.current_view;
        let has_actions = self.show_actions_popup;
        let context_active = |ctx: ShortcutContext| ctx.is_active(current_view, has_actions);
        
        // Find matching shortcut
        if let Some(action_id) = registry.find_match(&event.keystroke, context_active) {
            self.execute_shortcut_action(action_id, window, cx);
            return;
        }
        
        // Handle text input if no shortcut matched
        if let Some(char) = event.keystroke.key_char {
            self.handle_text_input(char, cx);
        }
    }
    
    fn execute_shortcut_action(&mut self, action_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        match action_id {
            "move_up" => self.move_selection_up(cx),
            "move_down" => self.move_selection_down(cx),
            "submit" => self.execute_selected(cx),
            "toggle_actions" => self.toggle_actions(cx, window),
            "edit_script" => self.handle_action("edit_script", cx),
            id if id.starts_with("script:") => {
                let path = &id[7..];
                self.run_script_by_path(path, cx);
            }
            _ => {}
        }
    }
}
```

### 5. Shortcut Cheat Sheet

```rust
/// Built-in command to show all shortcuts
pub fn render_shortcut_cheat_sheet(registry: &ShortcutRegistry) -> impl IntoElement {
    let categories = [
        ShortcutCategory::Navigation,
        ShortcutCategory::Actions,
        ShortcutCategory::Edit,
        ShortcutCategory::View,
        ShortcutCategory::Scripts,
        ShortcutCategory::System,
    ];
    
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .children(categories.iter().map(|cat| {
            let bindings: Vec<_> = registry.builtins.values()
                .filter(|b| b.category == *cat && !registry.disabled.contains(&b.id))
                .collect();
            
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x888888))
                        .child(format!("{:?}", cat))
                )
                .children(bindings.iter().map(|b| {
                    let shortcut = registry.get_shortcut(&b.id).unwrap();
                    div()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .child(div().text_sm().child(&b.name))
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .bg(rgb(0x333333))
                                .rounded(px(4.))
                                .text_xs()
                                .child(shortcut.display())
                        )
                }))
        }))
}
```

---

## Implementation Checklist

- [ ] Create `src/shortcuts/mod.rs` module
- [ ] Implement `Shortcut` with parsing and display
- [ ] Implement `ShortcutContext` enum
- [ ] Create `ShortcutRegistry` with builtin registration
- [ ] Add conflict detection
- [ ] Migrate inline key handlers to use registry
- [ ] Add user override support in config
- [ ] Create shortcut cheat sheet built-in
- [ ] Add shortcut hints to list items
- [ ] Document shortcut format in SDK docs

---

## Key Questions

1. Should scripts be able to override built-in shortcuts?
2. How to handle shortcuts that conflict with macOS system shortcuts?
3. Should there be a "shortcut recording" mode for customization?
4. How to persist user-customized shortcuts?
5. Should shortcuts work when window is hidden (global hotkeys)?

---

## Related Bundles

- Bundle 10: Hotkey System - global hotkey registration
- Bundle 42: App Shell - integrates shortcut handling
- Bundle 51: Actions/Context Menu - shows shortcut hints
