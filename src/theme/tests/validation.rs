//! Tests for theme validation

use super::super::validation::*;
use serde_json::json;

#[test]
fn test_diagnostic_creation() {
    let d = Diagnostic::error("/foo/bar", "Something is wrong");
    assert_eq!(d.path, "/foo/bar");
    assert_eq!(d.severity, DiagnosticSeverity::Error);
    assert_eq!(d.message, "Something is wrong");
    assert!(d.suggestion.is_none());
}

#[test]
fn test_diagnostic_with_suggestion() {
    let d = Diagnostic::warning("/opacity", "Out of range").with_suggestion("Use 0.0-1.0");
    assert_eq!(d.severity, DiagnosticSeverity::Warning);
    assert_eq!(d.suggestion, Some("Use 0.0-1.0".to_string()));
}

#[test]
fn test_diagnostics_has_errors() {
    let mut diags = ThemeDiagnostics::new();
    assert!(!diags.has_errors());
    assert!(diags.is_ok());

    diags.warning("/foo", "Just a warning");
    assert!(!diags.has_errors());
    assert!(diags.is_ok());

    diags.error("/bar", "An error");
    assert!(diags.has_errors());
    assert!(!diags.is_ok());
}

#[test]
fn test_has_warnings_excludes_errors() {
    let mut diags = ThemeDiagnostics::new();
    diags.error("/bar", "An error");
    assert!(!diags.has_warnings());

    diags.warning("/foo", "A warning");
    assert!(diags.has_warnings());
}

#[test]
fn test_diagnostics_counts() {
    let mut diags = ThemeDiagnostics::new();
    diags.error("/a", "Error 1");
    diags.error("/b", "Error 2");
    diags.warning("/c", "Warning 1");
    diags.info("/d", "Info 1");

    assert_eq!(diags.error_count(), 2);
    assert_eq!(diags.warning_count(), 1);
}

#[test]
fn test_diagnostics_merge() {
    let mut diags1 = ThemeDiagnostics::new();
    diags1.error("/a", "Error 1");

    let mut diags2 = ThemeDiagnostics::new();
    diags2.warning("/b", "Warning 1");

    diags1.merge(diags2);
    assert_eq!(diags1.diagnostics.len(), 2);
}

#[test]
fn test_validate_empty_object() {
    let json = json!({});
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_unknown_top_level_key() {
    let json = json!({
        "foo_bar_unknown": true
    });
    let diags = validate_theme_json(&json);
    assert_eq!(diags.warning_count(), 1);
    assert!(diags.diagnostics[0].path.contains("foo_bar_unknown"));
}

#[test]
fn test_validate_allows_appearance_key_without_unknown_warning() {
    let json = json!({
        "appearance": "dark"
    });
    let diags = validate_theme_json(&json);

    assert!(!diags
        .diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown key 'appearance'")));
}

#[test]
fn test_validate_valid_color_number() {
    let json = json!({
        "colors": {
            "background": {
                "main": 1973790
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_allows_on_accent_text_key_without_unknown_warning() {
    let json = json!({
        "colors": {
            "text": {
                "on_accent": "#FFFFFF"
            }
        }
    });
    let diags = validate_theme_json(&json);

    assert!(!diags
        .diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown key 'on_accent'")));
}

#[test]
fn test_validate_valid_color_hex_string() {
    let json = json!({
        "colors": {
            "background": {
                "main": "#1E1E1E"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_valid_color_shorthand() {
    let json = json!({
        "colors": {
            "background": {
                "main": "#FFF"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_valid_color_rgba_hex() {
    let json = json!({
        "colors": {
            "background": {
                "main": "#1E1E1EFF"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_valid_color_shorthand_rgba() {
    let json = json!({
        "colors": {
            "background": {
                "main": "#FFFA"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_invalid_color_string() {
    let json = json!({
        "colors": {
            "background": {
                "main": "not-a-color"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_errors());
    assert!(diags.diagnostics[0]
        .message
        .contains("Invalid color format"));
}

#[test]
fn test_validate_color_out_of_range() {
    let json = json!({
        "colors": {
            "background": {
                "main": 0x1FFFFFF
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_warnings());
}

#[test]
fn test_validate_opacity_out_of_range() {
    let json = json!({
        "opacity": {
            "main": 1.5
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_warnings());
    assert!(diags.diagnostics[0].message.contains("exceeds 1.0"));
}

#[test]
fn test_validate_opacity_negative() {
    let json = json!({
        "opacity": {
            "main": -0.5
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_warnings());
    assert!(diags.diagnostics[0].message.contains("negative"));
}

#[test]
fn test_validate_invalid_material() {
    let json = json!({
        "vibrancy": {
            "material": "super_blur"
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_errors());
    assert!(diags.diagnostics[0].message.contains("Invalid material"));
}

#[test]
fn test_validate_valid_material() {
    let json = json!({
        "vibrancy": {
            "enabled": true,
            "material": "popover"
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.is_ok());
}

#[test]
fn test_validate_font_size_too_large() {
    let json = json!({
        "fonts": {
            "mono_size": 200.0
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_warnings());
    assert!(diags.diagnostics[0].message.contains("large"));
}

#[test]
fn test_format_for_log() {
    let mut diags = ThemeDiagnostics::new();
    diags.error("/colors/background/main", "Invalid color");
    diags.warning("/opacity/main", "Out of range");

    let output = diags.format_for_log();
    assert!(output.contains("1 error(s)"));
    assert!(output.contains("1 warning(s)"));
    assert!(output.contains("[ERROR]"));
    assert!(output.contains("[WARN]"));
}

#[test]
fn test_validate_unknown_color_key() {
    let json = json!({
        "colors": {
            "background": {
                "main": "#1E1E1E",
                "some_unknown_field": "#000000"
            }
        }
    });
    let diags = validate_theme_json(&json);
    assert!(diags.has_warnings());
    assert!(diags
        .diagnostics
        .iter()
        .any(|d| d.message.contains("some_unknown_field")));
}

#[test]
fn test_severity_ordering() {
    // With derive(Ord), ordering is by variant declaration order
    // Error is declared first, so it has the lowest value
    // This is useful for sorting (errors first)
    assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
    assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Info);
}
