use script_kit_gpui::inline_agent::{place_compact_overlay, InlineAgentLayoutDefaults};
use script_kit_gpui::platform::accessibility::geometry::{DisplayBounds, RectPx};

#[test]
fn compact_defaults_match_product_target() {
    let defaults = InlineAgentLayoutDefaults::default();
    assert_eq!(defaults.compact_width, 420.0);
    assert_eq!(defaults.compact_min_width, 320.0);
    assert_eq!(defaults.compact_max_width, 560.0);
    assert_eq!(defaults.compact_idle_height, 118.0);
    assert_eq!(defaults.compact_thinking_height, 144.0);
    assert_eq!(defaults.compact_completed_height, 252.0);
    assert_eq!(defaults.edge_gutter, 12.0);
    assert_eq!(defaults.anchor_gap, 8.0);
    assert_eq!(defaults.expanded_width, 680.0);
}

#[test]
fn compact_overlay_flips_above_when_anchor_is_near_bottom() {
    let rect = place_compact_overlay(
        RectPx {
            x: 100.0,
            y: 860.0,
            width: 20.0,
            height: 20.0,
        },
        DisplayBounds::default(),
        InlineAgentLayoutDefaults::default().compact_completed_height,
    );

    assert!(rect.y < 860.0);
    assert!(rect.x >= 12.0);
    assert!(rect.x + rect.width <= 1440.0 - 12.0);
}
