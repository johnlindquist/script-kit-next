//! Focus management for the shell
//!
//! Centralized focus handling - focus handles are created once and owned by
//! the window root. The shell receives them by reference and applies focus
//! transitions once per view change (not every render).

use gpui::{App, Context, FocusHandle, Window};

/// Focus policy for a view
///
/// Determines where focus should land when a view becomes active.
/// Applied once per transition, not every render.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusPolicy {
    /// Don't change focus when view becomes active
    /// Used for: HUD notifications, overlays that shouldn't steal focus
    Preserve,

    /// Focus the header input (search box)
    /// Used for: ScriptList, ArgPrompt, most prompts with search
    #[default]
    HeaderInput,

    /// Focus the main content area
    /// Used for: EditorPrompt (focus the editor), TermPrompt
    Content,
}

/// Stable focus handles for the shell
///
/// Created once per window and stored in the root state.
/// The shell receives these by reference - it never creates focus handles.
pub struct ShellFocus {
    /// Root focus handle for track_focus
    pub shell: FocusHandle,
    /// Focus handle for the header input (search box)
    pub header_input: FocusHandle,
    /// Focus handle for the main content area
    pub content: FocusHandle,
}

impl ShellFocus {
    /// Create a new ShellFocus with fresh handles from the context
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Self {
        Self {
            shell: cx.focus_handle(),
            header_input: cx.focus_handle(),
            content: cx.focus_handle(),
        }
    }

    /// Apply a focus policy
    ///
    /// This should be called once per view transition, not every render.
    /// The caller tracks whether the view has changed and calls this accordingly.
    pub fn apply_policy(&self, policy: FocusPolicy, window: &mut Window, cx: &mut App) {
        match policy {
            FocusPolicy::Preserve => {
                // Do nothing - let existing focus remain
            }
            FocusPolicy::HeaderInput => {
                self.header_input.focus(window, cx);
            }
            FocusPolicy::Content => {
                self.content.focus(window, cx);
            }
        }
    }

    /// Check if the header input is focused
    pub fn is_header_focused(&self, window: &Window) -> bool {
        self.header_input.is_focused(window)
    }

    /// Check if the content area is focused
    pub fn is_content_focused(&self, window: &Window) -> bool {
        self.content.is_focused(window)
    }

    /// Check if any shell focus handle is focused
    pub fn is_any_focused(&self, window: &Window) -> bool {
        self.shell.is_focused(window)
            || self.header_input.is_focused(window)
            || self.content.is_focused(window)
    }

    /// Focus the header input
    pub fn focus_header(&self, window: &mut Window, cx: &mut App) {
        self.header_input.focus(window, cx);
    }

    /// Focus the content area
    pub fn focus_content(&self, window: &mut Window, cx: &mut App) {
        self.content.focus(window, cx);
    }

    /// Get the shell root focus handle for track_focus
    pub fn root_handle(&self) -> &FocusHandle {
        &self.shell
    }
}
