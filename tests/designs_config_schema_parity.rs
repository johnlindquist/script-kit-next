//! Parity contract between Rust `DesignsConfig` and TypeScript `DesignsConfig`.
//!
//! The Rust source of truth is `src/config/types.rs`; user-facing
//! `~/.scriptkit/config.ts` consumes the TypeScript shape declared in
//! `scripts/config-schema.ts`. Both must agree on field names and the
//! string enum values per the spec at `.goals/design-variants-overhaul.md`.

const RUST_TYPES: &str = include_str!("../src/config/types.rs");
const TS_SCHEMA: &str = include_str!("../scripts/config-schema.ts");

fn rust_has(needle: &str) -> bool {
    RUST_TYPES.contains(needle)
}

fn ts_has(needle: &str) -> bool {
    TS_SCHEMA.contains(needle)
}

#[test]
fn designs_root_field_is_declared_on_both_sides() {
    assert!(
        rust_has("pub designs: Option<DesignsConfig>"),
        "src/config/types.rs must expose `designs: Option<DesignsConfig>` on Config"
    );
    assert!(
        ts_has("designs?: DesignsConfig"),
        "scripts/config-schema.ts must expose `designs?: DesignsConfig` on the Config interface"
    );
}

#[test]
fn designs_config_top_level_fields_match() {
    // Rust uses #[serde(rename_all = "camelCase")] so the on-disk
    // form is camelCase.
    assert!(rust_has("pub struct DesignsConfig"));
    assert!(rust_has("pub active_id: Option<String>"));
    assert!(rust_has("pub cmd1_behavior: Option<Cmd1Behavior>"));
    assert!(rust_has(
        "pub overrides: Option<HashMap<String, DesignOverrides>>"
    ));

    assert!(ts_has("export interface DesignsConfig"));
    assert!(ts_has("activeId?: string"));
    assert!(ts_has("cmd1Behavior?:"));
    assert!(ts_has("overrides?: Record<string, DesignOverrides>"));
}

#[test]
fn cmd1_behavior_enum_strings_match() {
    // Rust enum variants Picker/Cycle (camelCase serde) -> "picker" | "cycle".
    assert!(rust_has("pub enum Cmd1Behavior"));
    assert!(rust_has("Picker,"));
    assert!(rust_has("Cycle,"));
    assert!(ts_has("\"picker\" | \"cycle\""));
}

#[test]
fn design_overrides_keys_match() {
    let keys: &[(&str, &str)] = &[
        ("pub accent: Option<String>", "accent?:"),
        ("pub density: Option<DesignDensityChoice>", "density?:"),
        ("pub font_family: Option<FontFamilyChoice>", "fontFamily?:"),
        ("pub font_scale: Option<i8>", "fontScale?:"),
        ("pub vibrancy: Option<VibrancyChoice>", "vibrancy?:"),
        (
            "pub chrome_opacity: Option<ChromeOpacityChoice>",
            "chromeOpacity?:",
        ),
        ("pub icon_style: Option<IconStyleChoice>", "iconStyle?:"),
        (
            "pub separator_style: Option<SeparatorStyleChoice>",
            "separatorStyle?:",
        ),
        ("pub row_height_nudge: Option<i8>", "rowHeightNudge?:"),
    ];
    for (rust, ts) in keys {
        assert!(rust_has(rust), "missing Rust field: {}", rust);
        assert!(ts_has(ts), "missing TS field: {}", ts);
    }
}

#[test]
fn density_enum_strings_match() {
    for variant in ["Compact,", "Comfortable,", "Spacious,"] {
        assert!(
            rust_has(variant),
            "Rust density variant missing: {}",
            variant
        );
    }
    assert!(ts_has("\"compact\" | \"comfortable\" | \"spacious\""));
}

#[test]
fn font_family_enum_strings_match() {
    for variant in ["FontFamilyChoice", "System,", "Monospace,", "Serif,"] {
        assert!(
            rust_has(variant),
            "Rust font-family marker missing: {}",
            variant
        );
    }
    assert!(ts_has("\"system\" | \"monospace\" | \"serif\""));
}

#[test]
fn vibrancy_enum_strings_match() {
    assert!(rust_has("pub enum VibrancyChoice"));
    for variant in ["None,", "Light,", "Medium,", "Heavy,"] {
        assert!(rust_has(variant));
    }
    assert!(ts_has("\"none\" | \"light\" | \"medium\" | \"heavy\""));
}

#[test]
fn icon_style_and_separator_enum_strings_match() {
    assert!(rust_has("pub enum IconStyleChoice"));
    assert!(ts_has("\"mono\" | \"color\" | \"hidden\""));
    assert!(rust_has("pub enum SeparatorStyleChoice"));
    assert!(ts_has("\"none\" | \"hairline\" | \"rule\" | \"grid\""));
}
