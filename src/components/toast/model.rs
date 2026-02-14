use gpui::{IntoElement, SharedString};
use std::rc::Rc;

use crate::transitions::{AppearTransition, Opacity};

use super::{ToastAction, ToastColors, ToastDismissCallback, ToastVariant};

/// A reusable toast notification component
///
/// Supports:
/// - Four variants: Success, Warning, Error, Info
/// - Optional auto-dismiss with configurable duration
/// - Dismissible mode with X button
/// - Expandable details section
/// - Action buttons (e.g., "Copy Error", "View Details")
/// - Appear/dismiss transitions via `AppearTransition`
///
#[derive(IntoElement)]
pub struct Toast {
    /// The main message to display
    pub(super) message: SharedString,
    /// Pre-computed colors for this toast
    pub(super) colors: ToastColors,
    /// Visual variant (Success, Warning, Error, Info)
    pub(super) variant: ToastVariant,
    /// Auto-dismiss duration in milliseconds (None = persistent)
    pub(super) duration_ms: Option<u64>,
    /// Whether to show a dismiss (X) button
    pub(super) dismissible: bool,
    /// Optional expandable details text
    pub(super) details: Option<String>,
    /// Action buttons to display
    pub(super) actions: Vec<ToastAction>,
    /// Callback when toast is dismissed
    pub(super) on_dismiss: Option<Rc<ToastDismissCallback>>,
    /// Transition state for appear/dismiss animations
    pub(super) transition: AppearTransition,
}

impl Toast {
    /// Create a new toast with the given message and pre-computed colors
    pub fn new(message: impl Into<SharedString>, colors: ToastColors) -> Self {
        Self {
            message: message.into(),
            colors,
            variant: ToastVariant::default(),
            duration_ms: Some(5000), // Default 5 second auto-dismiss
            dismissible: true,
            details: None,
            actions: Vec::new(),
            on_dismiss: None,
            transition: AppearTransition::visible(), // Default to fully visible
        }
    }

    /// Set the toast variant (Success, Warning, Error, Info)
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the auto-dismiss duration in milliseconds
    /// Use None for persistent toasts that don't auto-dismiss
    pub fn duration_ms(mut self, duration: Option<u64>) -> Self {
        self.duration_ms = duration;
        self
    }

    /// Set whether the toast is dismissible (shows X button)
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Set optional details text (expandable section)
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Set optional details text (convenience for Option<String>)
    pub fn details_opt(mut self, details: Option<String>) -> Self {
        self.details = details;
        self
    }

    /// Add an action button to the toast
    pub fn action(mut self, action: ToastAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set the dismiss callback
    pub fn on_dismiss(mut self, callback: super::ToastDismissCallback) -> Self {
        self.on_dismiss = Some(Rc::new(callback));
        self
    }

    /// Make this a persistent toast (no auto-dismiss)
    pub fn persistent(mut self) -> Self {
        self.duration_ms = None;
        self
    }

    /// Set the transition state for appear/dismiss animations
    ///
    /// Use this to animate the toast by interpolating between states:
    /// - `AppearTransition::hidden()` - Initial state (invisible, offset down)
    /// - `AppearTransition::visible()` - Fully visible state
    /// - `AppearTransition::dismissed()` - Dismiss state (invisible, offset up)
    ///
    pub fn with_transition(mut self, transition: AppearTransition) -> Self {
        self.transition = transition;
        self
    }

    /// Set just the opacity (convenience for simple fade effects)
    ///
    /// This sets the opacity without affecting slide offset.
    pub fn with_opacity(mut self, opacity: Opacity) -> Self {
        self.transition.opacity = opacity;
        self
    }

    /// Get the current transition state
    pub fn get_transition(&self) -> &AppearTransition {
        &self.transition
    }

    /// Get the auto-dismiss duration
    pub fn get_duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    /// Get the toast message
    pub fn get_message(&self) -> &SharedString {
        &self.message
    }

    /// Get the toast variant
    pub fn get_variant(&self) -> ToastVariant {
        self.variant
    }

    /// Get the toast details
    pub fn get_details(&self) -> Option<&String> {
        self.details.as_ref()
    }
}
