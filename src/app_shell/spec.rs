//! Shell specification types
//!
//! These types describe what a view needs from the shell without allocating.
//! Views return a `ShellSpec` that the shell renders.

use gpui::{AnyElement, SharedString};
use smallvec::SmallVec;

use super::chrome::ChromeSpec;
use super::focus::FocusPolicy;
use super::keymap::KeymapSpec;
use crate::components::button::ButtonVariant;

/// Specification for what a view needs from the shell
///
/// This is returned by each view to describe its layout requirements.
/// The shell renders the appropriate frame, header, footer, and content.
#[derive(Default)]
pub struct ShellSpec {
    /// Optional header configuration
    pub header: Option<HeaderSpec>,
    /// Optional footer configuration
    pub footer: Option<FooterSpec>,
    /// The main content element
    pub content: Option<AnyElement>,
    /// Chrome/frame configuration
    pub chrome: ChromeSpec,
    /// Focus policy when this view becomes active
    pub focus_policy: FocusPolicy,
    /// Optional per-view keybindings
    pub keymap: KeymapSpec,
}

impl ShellSpec {
    /// Create a new empty shell spec
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the header specification
    pub fn header(mut self, header: HeaderSpec) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the footer specification
    pub fn footer(mut self, footer: FooterSpec) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set the content element
    pub fn content(mut self, content: impl gpui::IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Set the chrome specification
    pub fn chrome(mut self, chrome: ChromeSpec) -> Self {
        self.chrome = chrome;
        self
    }

    /// Set the focus policy
    pub fn focus_policy(mut self, policy: FocusPolicy) -> Self {
        self.focus_policy = policy;
        self
    }

    /// Set per-view keybindings
    pub fn keymap(mut self, keymap: KeymapSpec) -> Self {
        self.keymap = keymap;
        self
    }

    /// Check if this spec has a header
    pub fn has_header(&self) -> bool {
        self.header.is_some()
    }

    /// Check if this spec has a footer
    pub fn has_footer(&self) -> bool {
        self.footer.is_some()
    }
}

/// Header specification for the shell
///
/// Describes the header layout without owning callbacks.
/// Actions are routed through ShellAction enum.
#[derive(Clone, Default)]
pub struct HeaderSpec {
    /// Search input configuration
    pub input: Option<InputSpec>,
    /// Buttons to display in the header (max 4 typically)
    pub buttons: SmallVec<[ButtonSpec; 4]>,
    /// Whether to show the logo
    pub show_logo: bool,
    /// Optional path prefix before input
    pub path_prefix: Option<SharedString>,
    /// Optional "Ask AI" hint
    pub show_ask_ai_hint: bool,
}

impl HeaderSpec {
    /// Create a new empty header spec
    pub fn new() -> Self {
        Self {
            show_logo: true,
            ..Default::default()
        }
    }

    /// Create a header with search input
    pub fn search(placeholder: impl Into<SharedString>) -> Self {
        Self {
            input: Some(InputSpec {
                placeholder: placeholder.into(),
                text: SharedString::default(),
                cursor_visible: true,
                is_focused: true,
            }),
            show_logo: true,
            ..Default::default()
        }
    }

    /// Set the input text
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        if let Some(ref mut input) = self.input {
            input.text = text.into();
        } else {
            self.input = Some(InputSpec {
                text: text.into(),
                ..Default::default()
            });
        }
        self
    }

    /// Set cursor visibility
    pub fn cursor_visible(mut self, visible: bool) -> Self {
        if let Some(ref mut input) = self.input {
            input.cursor_visible = visible;
        }
        self
    }

    /// Set input focused state
    pub fn focused(mut self, focused: bool) -> Self {
        if let Some(ref mut input) = self.input {
            input.is_focused = focused;
        }
        self
    }

    /// Add a button to the header
    pub fn button(
        mut self,
        label: impl Into<SharedString>,
        shortcut: impl Into<SharedString>,
    ) -> Self {
        self.buttons.push(ButtonSpec::primary(label, shortcut));
        self
    }

    /// Add a secondary/ghost button
    pub fn secondary_button(
        mut self,
        label: impl Into<SharedString>,
        shortcut: impl Into<SharedString>,
    ) -> Self {
        self.buttons.push(ButtonSpec::secondary(label, shortcut));
        self
    }

    /// Set path prefix
    pub fn path_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.path_prefix = Some(prefix.into());
        self
    }

    /// Show/hide logo
    pub fn logo(mut self, show: bool) -> Self {
        self.show_logo = show;
        self
    }

    /// Show "Ask AI [Tab]" hint
    pub fn ask_ai_hint(mut self, show: bool) -> Self {
        self.show_ask_ai_hint = show;
        self
    }
}

/// Input field specification
#[derive(Clone, Default)]
pub struct InputSpec {
    /// Placeholder text when empty
    pub placeholder: SharedString,
    /// Current input text
    pub text: SharedString,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
    /// Whether input is focused
    pub is_focused: bool,
}

/// Button specification using action enum instead of closure
#[derive(Clone)]
pub struct ButtonSpec {
    /// Button label text
    pub label: SharedString,
    /// Keyboard shortcut hint
    pub shortcut: SharedString,
    /// Action to dispatch on click
    pub action: ButtonAction,
    /// Button style variant
    pub variant: ButtonVariant,
}

impl ButtonSpec {
    /// Create a primary button
    pub fn primary(label: impl Into<SharedString>, shortcut: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: shortcut.into(),
            action: ButtonAction::Submit,
            variant: ButtonVariant::Primary,
        }
    }

    /// Create a secondary/ghost button
    pub fn secondary(label: impl Into<SharedString>, shortcut: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: shortcut.into(),
            action: ButtonAction::OpenActions,
            variant: ButtonVariant::Ghost,
        }
    }

    /// Set the action to dispatch
    pub fn action(mut self, action: ButtonAction) -> Self {
        self.action = action;
        self
    }

    /// Set the variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

/// Button action enum - no allocations, just data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonAction {
    /// Submit/Run the current selection
    #[default]
    Submit,
    /// Open the actions dialog
    OpenActions,
    /// Toggle preview panel
    TogglePreview,
    /// Cancel/close current view
    Cancel,
    /// Custom action by ID
    Custom(u32),
}

// ButtonVariant is imported from crate::components::button

/// Footer specification for the shell
#[derive(Clone, Default)]
pub struct FooterSpec {
    /// Primary action label and shortcut
    pub primary_label: SharedString,
    pub primary_shortcut: SharedString,
    /// Secondary action label and shortcut (optional)
    pub secondary_label: Option<SharedString>,
    pub secondary_shortcut: Option<SharedString>,
    /// Whether to show the logo
    pub show_logo: bool,
    /// Helper text (e.g., "Tab 1 of 2")
    pub helper_text: Option<SharedString>,
    /// Info label (e.g., language indicator)
    pub info_label: Option<SharedString>,
}

impl FooterSpec {
    /// Create a new footer spec
    pub fn new() -> Self {
        Self {
            show_logo: true,
            ..Default::default()
        }
    }

    /// Set primary action
    pub fn primary(
        mut self,
        label: impl Into<SharedString>,
        shortcut: impl Into<SharedString>,
    ) -> Self {
        self.primary_label = label.into();
        self.primary_shortcut = shortcut.into();
        self
    }

    /// Set secondary action
    pub fn secondary(
        mut self,
        label: impl Into<SharedString>,
        shortcut: impl Into<SharedString>,
    ) -> Self {
        self.secondary_label = Some(label.into());
        self.secondary_shortcut = Some(shortcut.into());
        self
    }

    /// Show/hide logo
    pub fn logo(mut self, show: bool) -> Self {
        self.show_logo = show;
        self
    }

    /// Set helper text
    pub fn helper(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Set info label
    pub fn info(mut self, label: impl Into<SharedString>) -> Self {
        self.info_label = Some(label.into());
        self
    }
}
