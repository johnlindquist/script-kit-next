//! Source-level contract for Liquid Glass chrome token de-drift.
//!
//! Several UI constants historically duplicated the shared Liquid Glass token
//! values (28pt min hit, 10px compact radius, 18px popup radius) as bare
//! literals. This contract pins them to reference the canonical tokens in
//! `src/ui/chrome/tokens.rs` so a future token change propagates instead of
//! silently drifting. Footer action button radius is intentionally design-tool
//! tuned separately from the shared compact radius and is pinned locally.

const TOKENS: &str = include_str!("../src/ui/chrome/tokens.rs");
const BUTTON_TYPES: &str = include_str!("../src/components/button/types.rs");
const ACTIONS_CONSTANTS: &str = include_str!("../src/actions/constants.rs");
const FOOTER_CHROME: &str = include_str!("../src/components/footer_chrome.rs");

#[test]
fn chrome_tokens_define_canonical_liquid_glass_vocabulary() {
    for needle in [
        "pub const LIQUID_GLASS_POPUP_RADIUS_PX: f32 = 18.0;",
        "pub const LIQUID_GLASS_PANEL_PADDING_PX: f32 = 16.0;",
        "pub const LIQUID_GLASS_DENSE_GAP_PX: f32 = 8.0;",
        "pub const LIQUID_GLASS_MIN_HIT_PX: f32 = 28.0;",
        "pub const LIQUID_GLASS_COMPACT_RADIUS_PX: f32 = 10.0;",
    ] {
        assert!(
            TOKENS.contains(needle),
            "chrome tokens.rs must define `{needle}`"
        );
    }
}

#[test]
fn button_ghost_height_references_shared_min_hit_token() {
    assert!(
        BUTTON_TYPES.contains(
            "pub const BUTTON_GHOST_HEIGHT: f32 = crate::ui::chrome::LIQUID_GLASS_MIN_HIT_PX;"
        ),
        "BUTTON_GHOST_HEIGHT must reference the shared Liquid Glass min hit token, not a bare 28.0 literal"
    );
}

#[test]
fn actions_radii_reference_shared_liquid_glass_tokens() {
    assert!(
        ACTIONS_CONSTANTS.contains(
            "pub const ACTIONS_POPUP_RADIUS: f32 = crate::ui::chrome::LIQUID_GLASS_POPUP_RADIUS_PX;"
        ),
        "ACTIONS_POPUP_RADIUS must reference the shared Liquid Glass popup radius token"
    );
    assert!(
        ACTIONS_CONSTANTS.contains(
            "pub const ACTIONS_ROW_RADIUS: f32 = crate::ui::chrome::LIQUID_GLASS_COMPACT_RADIUS_PX;"
        ),
        "ACTIONS_ROW_RADIUS must reference the shared Liquid Glass compact radius token"
    );
}

#[test]
fn footer_action_button_radius_stays_pinned_literal() {
    assert!(
        FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_BUTTON_RADIUS_PX: f32 = 6.0;"),
        "FOOTER_ACTION_BUTTON_RADIUS_PX must remain its design-tool pinned 6.0 literal (2026-07-07: user restored the pre-Liquid-Glass-polish less-rounded button style)"
    );
}

#[test]
fn footer_action_item_gap_stays_pinned_literal() {
    // FOOTER_ACTION_ITEM_GAP_PX is a standalone, separately test-pinned gap with
    // no shared-token meaning; the pin guards against ACCIDENTAL drift. Labels
    // are no longer bordered chips, so the footer uses the compact 2pt rhythm
    // while actual keycaps keep their borders.
    assert!(
        FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_ITEM_GAP_PX: f32 = 2.0;"),
        "FOOTER_ACTION_ITEM_GAP_PX must remain its pinned 2.0 literal"
    );
}
