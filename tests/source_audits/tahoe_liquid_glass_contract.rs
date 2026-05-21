use super::read_source;

#[test]
fn tahoe_metrics_are_defined_as_shared_chrome_tokens() {
    let tokens = read_source("src/ui/chrome/tokens.rs");

    for needle in [
        "pub struct TahoeChromeMetrics",
        "pub const TAHOE_CHROME_METRICS",
        "control_sm_radius: 8.0",
        "control_md_radius: 12.0",
        "popup_shell_radius: 18.0",
        "prompt_surface_radius: 12.0",
        "keycap_radius: 6.0",
        "footer_inset_x: 8.0",
        "acp_composer_min_height: 44.0",
    ] {
        assert!(tokens.contains(needle), "missing Tahoe token `{needle}`");
    }
}

#[test]
fn shared_components_consume_tahoe_metrics_instead_of_local_radius_literals() {
    let consumers = [
        "src/components/button/types.rs",
        "src/components/hint_strip.rs",
        "src/components/prompt_layout_shell.rs",
        "src/components/prompt_header/component.rs",
        "src/components/prompt_footer.rs",
        "src/actions/dialog.rs",
        "src/footer_popup.rs",
        "src/ai/acp/components/toolbar.rs",
        "src/ai/acp/components/composer.rs",
    ];

    for path in consumers {
        let source = read_source(path);
        assert!(
            source.contains("TAHOE_CHROME_METRICS"),
            "{path} must consume shared Tahoe metrics"
        );
    }
}

#[test]
fn accessibility_material_policy_has_opaque_fallback_before_glass_rollout() {
    let policy = read_source("src/platform/accessibility_material_policy.rs");
    let platform = read_source("src/platform/mod.rs");
    let chrome = read_source("src/theme/chrome.rs");

    for needle in [
        "pub(crate) struct SystemAppearanceAccessibility",
        "pub(crate) enum NativeMaterialPolicy",
        "reduce_transparency",
        "increase_contrast",
        "reduce_motion",
        "NativeMaterialPolicy::OpaqueFallback",
    ] {
        assert!(
            policy.contains(needle),
            "missing material policy `{needle}`"
        );
    }

    assert!(
        platform.contains("include!(\"accessibility_material_policy.rs\");"),
        "platform module must include the accessibility material policy"
    );
    assert!(
        chrome.contains("system_appearance_accessibility()")
            && chrome.contains("native_material_policy(")
            && chrome.contains("OpaqueFallback"),
        "AppChromeColors must thread accessibility material policy into chrome resolution"
    );
}

#[test]
fn tahoe_first_phase_keeps_glass_to_shells_and_controls() {
    let actions = read_source("src/actions/dialog.rs");
    let acp_composer = read_source("src/ai/acp/components/composer.rs");
    let footer = read_source("src/footer_popup.rs");

    assert!(
        actions.contains("TAHOE_CHROME_METRICS.popup_shell_radius"),
        "Actions shell should use the Tahoe popup shell radius"
    );
    assert!(
        acp_composer.contains("chrome.input_surface_rgba")
            && acp_composer.contains("TAHOE_CHROME_METRICS.acp_composer_min_height"),
        "ACP composer should use shared input surface and Tahoe composer metrics"
    );
    assert!(
        footer.contains("fn footer_effect_frame")
            && footer.contains("footer_inset_x")
            && footer
                .contains("setCornerRadius: crate::ui::chrome::TAHOE_CHROME_METRICS.panel_radius"),
        "native footer should be inset and softly rounded through Tahoe metrics"
    );
}
