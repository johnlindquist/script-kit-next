//! Keyboard routing for the shell
//!
//! The shell handles global shortcuts and routes them to typed actions.
//! Views can override bindings via KeymapSpec.

use smallvec::SmallVec;

/// Shell action - typed enum instead of closures
///
/// The shell converts keystrokes to actions, then the app handles them.
/// This keeps the shell presentational and the business logic in the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellAction {
    /// Escape pressed - cancel/close/back depending on context
    Cancel,
    /// Enter pressed - run/submit/select
    Run,
    /// Cmd+K pressed - open actions dialog
    OpenActions,
    /// Focus the search input
    FocusSearch,
    /// Navigate to next item
    Next,
    /// Navigate to previous item
    Prev,
    /// Tab pressed - next field or AI hint
    Tab,
    /// Shift+Tab pressed - previous field
    ShiftTab,
    /// Cmd+W pressed - close window
    Close,
    /// Cmd+, pressed - open settings
    Settings,
    /// No action (key not handled by shell)
    None,
}

impl ShellAction {
    /// Check if this is a "no action" result
    pub fn is_none(&self) -> bool {
        matches!(self, ShellAction::None)
    }

    /// Check if this action should prevent further key processing
    pub fn is_handled(&self) -> bool {
        !self.is_none()
    }
}

/// Per-view keybinding overrides
///
/// Views can specify additional bindings or override defaults.
/// Uses SmallVec to avoid allocation for common case (0-4 overrides).
#[derive(Clone, Default)]
pub struct KeymapSpec {
    /// Custom key bindings for this view
    pub bindings: SmallVec<[KeyBinding; 4]>,
    /// Whether to block global shortcuts (e.g., for modal dialogs)
    pub modal: bool,
}

impl KeymapSpec {
    /// Create a new empty keymap spec
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a modal keymap that blocks global shortcuts
    pub fn modal() -> Self {
        Self {
            bindings: SmallVec::new(),
            modal: true,
        }
    }

    /// Add a key binding
    pub fn bind(mut self, key: impl Into<String>, action: ShellAction) -> Self {
        self.bindings.push(KeyBinding {
            key: key.into(),
            modifiers: Modifiers::default(),
            action,
        });
        self
    }

    /// Add a key binding with modifiers
    pub fn bind_with_modifiers(
        mut self,
        key: impl Into<String>,
        modifiers: Modifiers,
        action: ShellAction,
    ) -> Self {
        self.bindings.push(KeyBinding {
            key: key.into(),
            modifiers,
            action,
        });
        self
    }

    /// Set whether this is a modal keymap
    pub fn set_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Look up an action for a key
    pub fn lookup(&self, key: &str, modifiers: &Modifiers) -> Option<ShellAction> {
        self.bindings
            .iter()
            .find(|b| b.key == key && b.modifiers == *modifiers)
            .map(|b| b.action)
    }
}

/// A single key binding
#[derive(Clone, Debug)]
pub struct KeyBinding {
    /// The key (e.g., "k", "escape", "enter")
    pub key: String,
    /// Required modifiers
    pub modifiers: Modifiers,
    /// Action to dispatch
    pub action: ShellAction,
}

/// Keyboard modifiers
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Command/Meta key (Cmd on macOS, Ctrl on Windows/Linux)
    pub command: bool,
    /// Shift key
    pub shift: bool,
    /// Alt/Option key
    pub alt: bool,
    /// Control key (separate from Command on macOS)
    pub control: bool,
}

impl Modifiers {
    /// Create modifiers with only Command
    pub fn command() -> Self {
        Self {
            command: true,
            ..Default::default()
        }
    }

    /// Create modifiers with only Shift
    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }

    /// Create modifiers with Command+Shift
    pub fn command_shift() -> Self {
        Self {
            command: true,
            shift: true,
            ..Default::default()
        }
    }

    /// Check if no modifiers are pressed
    pub fn is_empty(&self) -> bool {
        !self.command && !self.shift && !self.alt && !self.control
    }
}

/// Default key bindings for the shell
///
/// These are the global shortcuts that the shell handles before
/// passing to views. Views can override via KeymapSpec.
pub fn default_bindings() -> SmallVec<[KeyBinding; 8]> {
    let mut bindings = SmallVec::new();

    // Escape - cancel/close
    bindings.push(KeyBinding {
        key: "escape".to_string(),
        modifiers: Modifiers::default(),
        action: ShellAction::Cancel,
    });

    // Enter - run/submit
    bindings.push(KeyBinding {
        key: "enter".to_string(),
        modifiers: Modifiers::default(),
        action: ShellAction::Run,
    });

    // Cmd+K - open actions
    bindings.push(KeyBinding {
        key: "k".to_string(),
        modifiers: Modifiers::command(),
        action: ShellAction::OpenActions,
    });

    // Cmd+W - close window
    bindings.push(KeyBinding {
        key: "w".to_string(),
        modifiers: Modifiers::command(),
        action: ShellAction::Close,
    });

    // Cmd+, - settings
    bindings.push(KeyBinding {
        key: ",".to_string(),
        modifiers: Modifiers::command(),
        action: ShellAction::Settings,
    });

    // Tab - next field
    bindings.push(KeyBinding {
        key: "tab".to_string(),
        modifiers: Modifiers::default(),
        action: ShellAction::Tab,
    });

    // Shift+Tab - previous field
    bindings.push(KeyBinding {
        key: "tab".to_string(),
        modifiers: Modifiers::shift(),
        action: ShellAction::ShiftTab,
    });

    // Arrow up - previous
    bindings.push(KeyBinding {
        key: "up".to_string(),
        modifiers: Modifiers::default(),
        action: ShellAction::Prev,
    });

    // Arrow down - next
    bindings.push(KeyBinding {
        key: "down".to_string(),
        modifiers: Modifiers::default(),
        action: ShellAction::Next,
    });

    bindings
}

/// Route a key event to an action
///
/// Priority order:
/// 1. View-specific overrides (KeymapSpec)
/// 2. Default shell bindings
///
/// Returns ShellAction::None if no binding matches.
pub fn route_key(key: &str, modifiers: &Modifiers, view_keymap: &KeymapSpec) -> ShellAction {
    // Check view-specific bindings first
    if let Some(action) = view_keymap.lookup(key, modifiers) {
        return action;
    }

    // If modal, don't check default bindings
    if view_keymap.modal {
        return ShellAction::None;
    }

    // Check default bindings
    let defaults = default_bindings();
    for binding in defaults.iter() {
        if binding.key == key && binding.modifiers == *modifiers {
            return binding.action;
        }
    }

    ShellAction::None
}
