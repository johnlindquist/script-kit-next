#![allow(dead_code)]

use gpui::{div, prelude::*, Div, IntoElement, Stateful};

use crate::theme::{modal_overlay_bg, Theme};
use crate::ui::chrome::{alpha_from_opacity, MAIN_WINDOW_MODAL_DIM_OPACITY};

pub(crate) const MAIN_WINDOW_MODAL_DIM_LAYER_ID: &str = "main-window-modal-dim-layer";

pub(crate) fn main_window_modal_dim_layer(active: bool, theme: &Theme) -> Option<Stateful<Div>> {
    active.then(|| {
        div()
            .id(MAIN_WINDOW_MODAL_DIM_LAYER_ID)
            .absolute()
            .inset_0()
            .bg(modal_overlay_bg(
                theme,
                alpha_from_opacity(MAIN_WINDOW_MODAL_DIM_OPACITY) as u8,
            ))
    })
}

pub(crate) fn render_main_window_modal_dim_layer(
    active: bool,
    theme: &Theme,
) -> Option<impl IntoElement> {
    main_window_modal_dim_layer(active, theme).map(|layer| layer.into_any_element())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_window_modal_dim_layer_only_renders_when_modal_is_active() {
        let theme = Theme::default();

        assert!(main_window_modal_dim_layer(false, &theme).is_none());
        assert!(main_window_modal_dim_layer(true, &theme).is_some());
    }

    #[test]
    fn main_window_modal_dim_opacity_is_a_chrome_token() {
        assert_eq!(
            alpha_from_opacity(MAIN_WINDOW_MODAL_DIM_OPACITY),
            alpha_from_opacity(crate::theme::opacity::OPACITY_MODAL_DIM)
        );
    }
}
