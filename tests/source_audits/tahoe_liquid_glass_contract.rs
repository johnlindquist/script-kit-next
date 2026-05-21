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
    let native_bridge = read_source("src/platform/native_material_bridge.rs");
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
        platform.contains("include!(\"native_material_bridge.rs\");"),
        "platform module must include the native material bridge"
    );
    for needle in [
        "NativeMaterialKind::TahoeGlassRegular",
        "objc::runtime::Class::get(\"NSGlassEffectView\")",
        "native_material_selection",
        "NativeMaterialKind::OpaqueFallback",
        "configure_native_material_view",
    ] {
        assert!(
            native_bridge.contains(needle),
            "native material bridge missing `{needle}`"
        );
    }
    assert!(
        chrome.contains("system_appearance_accessibility()")
            && chrome.contains("native_material_policy(")
            && chrome.contains("native_material_selection(")
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

#[test]
fn tahoe_popup_and_footer_geometry_use_shared_radii() {
    let secondary_windows = read_source("src/platform/secondary_window_config.rs");
    let footer = read_source("src/footer_popup.rs");

    assert!(
        secondary_windows.contains("TAHOE_CHROME_METRICS.popup_shell_radius")
            && secondary_windows.contains("setHasShadow: true"),
        "confirm/prompt popup windows must use Tahoe popup radius and shadow instead of a flush zero-radius shell"
    );
    assert!(
        !secondary_windows.contains("setCornerRadius: 0.0_f64"),
        "popup geometry should not reintroduce local zero-radius drift"
    );
    assert!(
        footer.contains("native_material_selection(")
            && footer.contains("configure_native_material_view(")
            && footer.contains("TAHOE_CHROME_METRICS.panel_radius"),
        "native footer must resolve Tahoe material and panel radius through shared tokens"
    );
}

#[test]
fn theme_designer_exposes_tahoe_material_preview_affordance() {
    let theme_chooser = read_source("src/render_builtins/theme_chooser.rs");

    for needle in [
        "\"TAHOE MATERIAL\"",
        "native_material_selection",
        "TAHOE_CHROME_METRICS",
        "\"Native Material\"",
        "\"Accessibility Fallbacks\"",
    ] {
        assert!(
            theme_chooser.contains(needle),
            "Theme Designer Tahoe preview missing `{needle}`"
        );
    }
}

#[test]
fn storybook_has_required_tahoe_surface_matrix() {
    let story = read_source("src/storybook/tahoe_design_system_states.rs");
    let stories_mod = read_source("src/stories/mod.rs");
    let storybook_mod = read_source("src/storybook/mod.rs");

    for needle in [
        "tahoe-main-menu",
        "tahoe-actions-popup",
        "tahoe-footer",
        "tahoe-acp-chat",
        "tahoe-theme-designer",
        "tahoe-form-prompt",
        "native_material_selection",
        "TAHOE_CHROME_METRICS",
    ] {
        assert!(
            story.contains(needle),
            "Tahoe Storybook matrix missing `{needle}`"
        );
    }
    assert!(
        stories_mod.contains("TahoeDesignSystemStatesStory")
            && storybook_mod.contains("tahoe_design_system_story_variants"),
        "Tahoe design-system story must be registered and exported"
    );
}
