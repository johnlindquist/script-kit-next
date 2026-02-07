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
    let mut design = DesignColors::default();
    design.background_selected = 0x556677;

    let colors = ToastColors::from_design(&design, ToastVariant::Info);
    assert_eq!(colors.details_bg, 0x55667720);
}
