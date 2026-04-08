use super::{IntegratedOverlayAnchor, IntegratedOverlayPlacement, IntegratedSurfaceShellConfig};

#[test]
fn integrated_surface_shell_defaults_are_scene_sized() {
    let config = IntegratedSurfaceShellConfig::default();
    assert!(config.width >= 420.0);
    assert!(config.height >= 220.0);
    assert!(config.footer_height > 0.0);
}

#[test]
fn integrated_overlay_placement_preserves_fields() {
    let placement =
        IntegratedOverlayPlacement::new(IntegratedOverlayAnchor::Footer, 24.0, 180.0, 320.0);
    assert_eq!(placement.anchor, IntegratedOverlayAnchor::Footer);
    assert_eq!(placement.left, 24.0);
    assert_eq!(placement.top, 180.0);
    assert_eq!(placement.width, 320.0);
}

#[test]
fn integrated_overlay_anchor_composer_variant_exists() {
    let placement =
        IntegratedOverlayPlacement::new(IntegratedOverlayAnchor::Composer, 0.0, 0.0, 100.0);
    assert_eq!(placement.anchor, IntegratedOverlayAnchor::Composer);
}
