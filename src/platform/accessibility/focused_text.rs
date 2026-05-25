use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use super::app_identity::ActiveAppIdentity;
use super::geometry::FocusedFieldGeometry;
use super::metrics::TextMetrics;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FocusedTextSessionId(pub String);

impl FocusedTextSessionId {
    pub fn new_for_tests(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for FocusedTextSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureFocusedTextOptions {
    pub allow_secure_fields: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedTextTargetDescriptor {
    pub role: Option<String>,
    pub subrole: Option<String>,
    pub title: Option<String>,
    pub content_kind: FocusedTextContentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedTextContentKind {
    PlainText,
    RichText,
    Secure,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRangeUtf16 {
    pub location: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusedTextCapabilities {
    pub can_replace: bool,
    pub can_append: bool,
    pub can_copy: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FocusedTextSnapshot {
    pub session_id: FocusedTextSessionId,
    pub captured_at_ms: u128,
    pub app: ActiveAppIdentity,
    pub target: FocusedTextTargetDescriptor,
    pub text: String,
    pub selected_range_utf16: Option<TextRangeUtf16>,
    pub caret_range_utf16: Option<TextRangeUtf16>,
    pub metrics: TextMetrics,
    pub geometry: FocusedFieldGeometry,
    pub capabilities: FocusedTextCapabilities,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FocusedTextError {
    #[error("accessibility permission is required")]
    AccessibilityPermissionRequired,
    #[error("focused field is secure")]
    SecureField,
    #[error("focused field is unsupported")]
    UnsupportedTarget,
    #[error("focused text session is stale")]
    StaleSession,
    #[error("{0}")]
    Platform(String),
}

pub fn capture_focused_text_field(
    options: CaptureFocusedTextOptions,
) -> Result<FocusedTextSnapshot, FocusedTextError> {
    if !super::permissions::has_accessibility_permission() {
        return Err(FocusedTextError::AccessibilityPermissionRequired);
    }

    capture_focused_text_field_platform(options)
}

#[cfg(target_os = "macos")]
fn capture_focused_text_field_platform(
    options: CaptureFocusedTextOptions,
) -> Result<FocusedTextSnapshot, FocusedTextError> {
    let app = super::app_identity::current_frontmost_app_identity();
    let element = super::ax::focused_ui_element_for_app(app.process_id)
        .map_err(|err| FocusedTextError::Platform(err.to_string()))?;
    let role = super::ax::role(element.as_ptr());
    let subrole = super::ax::subrole(element.as_ptr());

    let content_kind = classify_content_kind(role.as_deref(), subrole.as_deref());
    if content_kind == FocusedTextContentKind::Secure && !options.allow_secure_fields {
        return Err(FocusedTextError::SecureField);
    }

    let text = super::ax::whole_text(element.as_ptr()).map_err(|err| {
        if content_kind == FocusedTextContentKind::Unsupported {
            FocusedTextError::UnsupportedTarget
        } else {
            FocusedTextError::Platform(err.to_string())
        }
    })?;
    let selected_range_utf16 = super::ax::selected_text_range(element.as_ptr());
    let caret_range_utf16 = selected_range_utf16.map(|range| TextRangeUtf16 {
        location: range.location + range.length,
        length: 0,
    });
    let geometry = super::ax::focused_geometry(element.as_ptr(), selected_range_utf16);
    let can_edit = super::ax::is_enabled(element.as_ptr()).unwrap_or(true)
        && content_kind != FocusedTextContentKind::Secure;
    let captured_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    let session_id = FocusedTextSessionId(format!("focused-text-{captured_at_ms}"));
    super::ax::register_focused_text_session(
        &session_id,
        element.as_ptr(),
        captured_at_ms,
        text.clone(),
        app.process_id,
    )
    .map_err(|err| FocusedTextError::Platform(err.to_string()))?;

    Ok(FocusedTextSnapshot {
        session_id,
        captured_at_ms,
        app,
        target: FocusedTextTargetDescriptor {
            role,
            subrole,
            title: None,
            content_kind,
        },
        metrics: TextMetrics::from_text(&text),
        geometry,
        capabilities: FocusedTextCapabilities {
            can_replace: can_edit,
            can_append: can_edit,
            can_copy: true,
        },
        text,
        selected_range_utf16,
        caret_range_utf16,
    })
}

#[cfg(not(target_os = "macos"))]
fn capture_focused_text_field_platform(
    _options: CaptureFocusedTextOptions,
) -> Result<FocusedTextSnapshot, FocusedTextError> {
    Err(FocusedTextError::UnsupportedTarget)
}

pub fn focused_text_snapshot_for_tests(text: impl Into<String>) -> FocusedTextSnapshot {
    let text = text.into();
    FocusedTextSnapshot {
        session_id: FocusedTextSessionId::new_for_tests("focused-text-test-session"),
        captured_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default(),
        app: ActiveAppIdentity::unknown(),
        target: FocusedTextTargetDescriptor {
            role: Some("AXTextArea".to_string()),
            subrole: None,
            title: None,
            content_kind: FocusedTextContentKind::PlainText,
        },
        metrics: TextMetrics::from_text(&text),
        geometry: FocusedFieldGeometry::default(),
        capabilities: FocusedTextCapabilities {
            can_replace: true,
            can_append: true,
            can_copy: true,
        },
        text,
        selected_range_utf16: None,
        caret_range_utf16: None,
    }
}

pub fn classify_content_kind(role: Option<&str>, subrole: Option<&str>) -> FocusedTextContentKind {
    let role = role.unwrap_or_default();
    let subrole = subrole.unwrap_or_default();
    let combined = format!("{role} {subrole}");

    if combined.contains("Secure") || combined.contains("Password") {
        return FocusedTextContentKind::Secure;
    }
    if combined.contains("TextArea") || combined.contains("TextField") {
        return FocusedTextContentKind::PlainText;
    }
    if combined.contains("Text") {
        return FocusedTextContentKind::RichText;
    }
    FocusedTextContentKind::Unsupported
}
