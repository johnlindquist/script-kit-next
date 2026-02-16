//! Theme validation and diagnostics
//!
//! Provides validation for theme configuration with:
//! - JSON-pointer style error paths (e.g., "/vibrancy/opacity/main")
//! - Severity levels (error, warning, info)
//! - Range validation for opacity, color values
//! - Unknown key detection
//! - Last-known-good fallback support

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

// ============================================================================
// Diagnostic types
// ============================================================================

/// Severity level for theme diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Error: Theme won't work correctly (e.g., invalid color format)
    Error,
    /// Warning: Theme might not look right (e.g., unknown key, out-of-range opacity)
    Warning,
    /// Info: Informational note (e.g., default value applied)
    Info,
}

/// A single diagnostic message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// JSON-pointer style path to the problematic field
    pub path: String,
    /// Severity of the issue
    pub severity: DiagnosticSeverity,
    /// Human-readable message describing the issue
    pub message: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn info(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Collection of diagnostics from theme validation
#[derive(Debug, Clone, Default)]
pub struct ThemeDiagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl ThemeDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.add(Diagnostic::error(path, message));
    }

    pub fn warning(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.add(Diagnostic::warning(path, message));
    }

    pub fn info(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.add(Diagnostic::info(path, message));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    pub fn merge(&mut self, other: ThemeDiagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn format_for_log(&self) -> String {
        if self.diagnostics.is_empty() {
            return "Theme validation passed".to_string();
        }
        let mut output = format!(
            "Theme validation: {} error(s), {} warning(s)\n",
            self.error_count(),
            self.warning_count()
        );
        for diag in &self.diagnostics {
            let severity = match diag.severity {
                DiagnosticSeverity::Error => "ERROR",
                DiagnosticSeverity::Warning => "WARN",
                DiagnosticSeverity::Info => "INFO",
            };
            output.push_str(&format!(
                "  [{}] {}: {}\n",
                severity, diag.path, diag.message
            ));
            if let Some(ref suggestion) = diag.suggestion {
                output.push_str(&format!("    Suggestion: {}\n", suggestion));
            }
        }
        output
    }
}

// Known keys for validation
const KNOWN_TOP_LEVEL_KEYS: &[&str] = &[
    "appearance",
    "colors",
    "focus_aware",
    "opacity",
    "drop_shadow",
    "vibrancy",
    "fonts",
];
const KNOWN_COLOR_KEYS: &[&str] = &["background", "text", "accent", "ui", "terminal"];
const KNOWN_BACKGROUND_KEYS: &[&str] = &["main", "title_bar", "search_box", "log_panel"];
const KNOWN_TEXT_KEYS: &[&str] = &[
    "primary",
    "secondary",
    "tertiary",
    "muted",
    "dimmed",
    "on_accent",
];
const KNOWN_ACCENT_KEYS: &[&str] = &["selected", "selected_subtle"];
const KNOWN_UI_KEYS: &[&str] = &["border", "success", "error", "warning", "info"];
const KNOWN_OPACITY_KEYS: &[&str] = &[
    "main",
    "title_bar",
    "search_box",
    "log_panel",
    "selected",
    "hover",
    "preview",
    "dialog",
    "input",
    "panel",
    "input_inactive",
    "input_active",
    "border_inactive",
    "border_active",
    "vibrancy_background",
];
const KNOWN_VIBRANCY_KEYS: &[&str] = &["enabled", "material"];
const KNOWN_DROP_SHADOW_KEYS: &[&str] = &[
    "enabled",
    "blur_radius",
    "spread_radius",
    "offset_x",
    "offset_y",
    "color",
    "opacity",
];
const KNOWN_FONT_KEYS: &[&str] = &["mono_family", "mono_size", "ui_family", "ui_size"];
const VALID_MATERIALS: &[&str] = &["hud", "popover", "menu", "sidebar", "content"];

/// Validate a theme JSON value and return diagnostics
pub fn validate_theme_json(json: &Value) -> ThemeDiagnostics {
    let mut diags = ThemeDiagnostics::new();
    if let Value::Object(map) = json {
        check_unknown_keys(&mut diags, "", map.keys(), KNOWN_TOP_LEVEL_KEYS);
        if let Some(colors) = map.get("colors") {
            validate_colors(&mut diags, "/colors", colors);
        }
        if let Some(opacity) = map.get("opacity") {
            validate_opacity(&mut diags, "/opacity", opacity);
        }
        if let Some(vibrancy) = map.get("vibrancy") {
            validate_vibrancy(&mut diags, "/vibrancy", vibrancy);
        }
        if let Some(shadow) = map.get("drop_shadow") {
            validate_drop_shadow(&mut diags, "/drop_shadow", shadow);
        }
        if let Some(fonts) = map.get("fonts") {
            validate_fonts(&mut diags, "/fonts", fonts);
        }
    } else {
        diags.error("", "Theme must be a JSON object");
    }
    diags
}

fn check_unknown_keys<'a>(
    diags: &mut ThemeDiagnostics,
    parent_path: &str,
    keys: impl Iterator<Item = &'a String>,
    known_keys: &[&str],
) {
    let known_set: HashSet<&str> = known_keys.iter().copied().collect();
    for key in keys {
        if !known_set.contains(key.as_str()) {
            let path = if parent_path.is_empty() {
                format!("/{}", key)
            } else {
                format!("{}/{}", parent_path, key)
            };
            diags.warning(&path, format!("Unknown key '{}' will be ignored", key));
        }
    }
}

fn validate_colors(diags: &mut ThemeDiagnostics, path: &str, colors: &Value) {
    if let Value::Object(map) = colors {
        check_unknown_keys(diags, path, map.keys(), KNOWN_COLOR_KEYS);
        if let Some(bg) = map.get("background") {
            validate_color_object(
                diags,
                &format!("{}/background", path),
                bg,
                KNOWN_BACKGROUND_KEYS,
            );
        }
        if let Some(text) = map.get("text") {
            validate_color_object(diags, &format!("{}/text", path), text, KNOWN_TEXT_KEYS);
        }
        if let Some(accent) = map.get("accent") {
            validate_color_object(
                diags,
                &format!("{}/accent", path),
                accent,
                KNOWN_ACCENT_KEYS,
            );
        }
        if let Some(ui) = map.get("ui") {
            validate_color_object(diags, &format!("{}/ui", path), ui, KNOWN_UI_KEYS);
        }
    } else {
        diags.error(path, "colors must be an object");
    }
}

fn validate_color_object(
    diags: &mut ThemeDiagnostics,
    path: &str,
    obj: &Value,
    known_keys: &[&str],
) {
    if let Value::Object(map) = obj {
        check_unknown_keys(diags, path, map.keys(), known_keys);
        for (key, value) in map {
            validate_color_value(diags, &format!("{}/{}", path, key), value);
        }
    } else {
        diags.error(path, "Expected an object");
    }
}

fn validate_color_value(diags: &mut ThemeDiagnostics, path: &str, value: &Value) {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.as_u64() {
                if v > 0xFFFFFF {
                    diags.warning(path, "Color value exceeds 0xFFFFFF (16777215)");
                }
            } else if let Some(v) = n.as_i64() {
                if v < 0 {
                    diags.error(path, "Color value cannot be negative");
                }
            }
        }
        Value::String(s) => {
            if !is_valid_color_string(s) {
                diags.error(path, format!("Invalid color format: '{}'", s));
            }
        }
        _ => diags.error(path, "Color must be a number or string"),
    }
}

fn is_valid_color_string(s: &str) -> bool {
    crate::theme::hex_color::hex_color_serde::parse_color_string(s).is_ok()
}

fn validate_opacity(diags: &mut ThemeDiagnostics, path: &str, opacity: &Value) {
    if let Value::Object(map) = opacity {
        check_unknown_keys(diags, path, map.keys(), KNOWN_OPACITY_KEYS);
        for (key, value) in map {
            validate_opacity_value(diags, &format!("{}/{}", path, key), value);
        }
    } else {
        diags.error(path, "opacity must be an object");
    }
}

fn validate_opacity_value(diags: &mut ThemeDiagnostics, path: &str, value: &Value) {
    if let Value::Number(n) = value {
        if let Some(f) = n.as_f64() {
            if f < 0.0 {
                diags.warning(path, "Opacity cannot be negative, will be clamped to 0.0");
            } else if f > 1.0 {
                diags.warning(path, "Opacity exceeds 1.0, will be clamped to 1.0");
            }
        }
    } else {
        diags.error(path, "Opacity must be a number");
    }
}

fn validate_vibrancy(diags: &mut ThemeDiagnostics, path: &str, vibrancy: &Value) {
    if let Value::Object(map) = vibrancy {
        check_unknown_keys(diags, path, map.keys(), KNOWN_VIBRANCY_KEYS);
        if let Some(material) = map.get("material") {
            let material_path = format!("{}/material", path);
            if let Value::String(m) = material {
                if !VALID_MATERIALS.contains(&m.as_str()) {
                    diags.error(material_path.clone(), format!("Invalid material '{}'", m));
                }
            } else {
                diags.error(material_path, "material must be a string");
            }
        }
        if let Some(enabled) = map.get("enabled") {
            if !enabled.is_boolean() {
                diags.error(format!("{}/enabled", path), "enabled must be a boolean");
            }
        }
    } else {
        diags.error(path, "vibrancy must be an object");
    }
}

fn validate_drop_shadow(diags: &mut ThemeDiagnostics, path: &str, shadow: &Value) {
    if let Value::Object(map) = shadow {
        check_unknown_keys(diags, path, map.keys(), KNOWN_DROP_SHADOW_KEYS);
        for key in ["blur_radius", "spread_radius", "offset_x", "offset_y"] {
            if let Some(value) = map.get(key) {
                if !value.is_number() {
                    diags.error(
                        format!("{}/{}", path, key),
                        format!("{} must be a number", key),
                    );
                }
            }
        }
        if let Some(opacity) = map.get("opacity") {
            validate_opacity_value(diags, &format!("{}/opacity", path), opacity);
        }
        if let Some(color) = map.get("color") {
            validate_color_value(diags, &format!("{}/color", path), color);
        }
        if let Some(enabled) = map.get("enabled") {
            if !enabled.is_boolean() {
                diags.error(format!("{}/enabled", path), "enabled must be a boolean");
            }
        }
    } else {
        diags.error(path, "drop_shadow must be an object");
    }
}

fn validate_fonts(diags: &mut ThemeDiagnostics, path: &str, fonts: &Value) {
    if let Value::Object(map) = fonts {
        check_unknown_keys(diags, path, map.keys(), KNOWN_FONT_KEYS);
        for key in ["mono_family", "ui_family"] {
            if let Some(value) = map.get(key) {
                if !value.is_string() {
                    diags.error(
                        format!("{}/{}", path, key),
                        format!("{} must be a string", key),
                    );
                }
            }
        }
        for key in ["mono_size", "ui_size"] {
            if let Some(value) = map.get(key) {
                let field_path = format!("{}/{}", path, key);
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        if f <= 0.0 {
                            diags.warning(field_path, "Font size must be positive");
                        } else if f > 100.0 {
                            diags.warning(field_path, "Font size unusually large");
                        }
                    }
                } else {
                    diags.error(field_path, format!("{} must be a number", key));
                }
            }
        }
    } else {
        diags.error(path, "fonts must be an object");
    }
}
