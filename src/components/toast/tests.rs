use super::types::{
    TOAST_ACTIONS_GAP_PX, TOAST_ACTIONS_MARGIN_TOP_PX, TOAST_BORDER_WIDTH_PX, TOAST_CONTENT_GAP_PX,
    TOAST_CONTENT_PADDING_X_PX, TOAST_CONTENT_PADDING_Y_PX, TOAST_ICON_SIZE_PX, TOAST_MAX_WIDTH_PX,
    TOAST_MESSAGE_COLUMN_GAP_PX, TOAST_RADIUS_PX,
};
use super::{ToastColors, ToastVariant};
use crate::designs::DesignColors;
use crate::theme::Theme;

#[test]
fn test_toast_colors_from_theme_uses_selected_subtle_for_details_background() {
    let mut theme = Theme::default();
    theme.colors.accent.selected_subtle = 0x334455;

    let colors = ToastColors::from_theme(&theme, ToastVariant::Info);
    assert_eq!(colors.details_bg, 0x33445520);
}

#[test]
fn test_toast_colors_from_design_uses_selected_background_for_details_background() {
    let design = DesignColors {
        background_selected: 0x556677,
        ..Default::default()
    };

    let colors = ToastColors::from_design(&design, ToastVariant::Info);
    assert_eq!(colors.details_bg, 0x55667720);
}

#[test]
fn test_toast_layout_tokens_stay_consistent_when_spacing_is_adjusted() {
    assert_eq!(TOAST_MAX_WIDTH_PX, 400.0);
    assert_eq!(TOAST_BORDER_WIDTH_PX, 4.0);
    assert_eq!(TOAST_RADIUS_PX, 8.0);
    assert_eq!(TOAST_CONTENT_GAP_PX, 12.0);
    assert_eq!(TOAST_CONTENT_PADDING_X_PX, 16.0);
    assert_eq!(TOAST_CONTENT_PADDING_Y_PX, 12.0);
    assert_eq!(TOAST_ICON_SIZE_PX, 24.0);
    assert_eq!(TOAST_MESSAGE_COLUMN_GAP_PX, 8.0);
    assert_eq!(TOAST_ACTIONS_GAP_PX, 8.0);
    assert_eq!(TOAST_ACTIONS_MARGIN_TOP_PX, 4.0);
}
