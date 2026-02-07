use gpui::SharedString;

use crate::error::ErrorSeverity;

use super::{Toast, ToastColors, ToastVariant};

impl Toast {
    /// Create a success toast
    pub fn success(message: impl Into<SharedString>, theme: &crate::theme::Theme) -> Self {
        let colors = ToastColors::from_theme(theme, ToastVariant::Success);
        Self::new(message, colors).variant(ToastVariant::Success)
    }

    /// Create a warning toast
    pub fn warning(message: impl Into<SharedString>, theme: &crate::theme::Theme) -> Self {
        let colors = ToastColors::from_theme(theme, ToastVariant::Warning);
        Self::new(message, colors).variant(ToastVariant::Warning)
    }

    /// Create an error toast
    pub fn error(message: impl Into<SharedString>, theme: &crate::theme::Theme) -> Self {
        let colors = ToastColors::from_theme(theme, ToastVariant::Error);
        Self::new(message, colors).variant(ToastVariant::Error)
    }

    /// Create an info toast
    pub fn info(message: impl Into<SharedString>, theme: &crate::theme::Theme) -> Self {
        let colors = ToastColors::from_theme(theme, ToastVariant::Info);
        Self::new(message, colors).variant(ToastVariant::Info)
    }

    /// Create a toast from an ErrorSeverity
    pub fn from_severity(
        message: impl Into<SharedString>,
        severity: ErrorSeverity,
        theme: &crate::theme::Theme,
    ) -> Self {
        let variant = ToastVariant::from_severity(severity);
        let colors = ToastColors::from_theme(theme, variant);
        Self::new(message, colors).variant(variant)
    }
}
