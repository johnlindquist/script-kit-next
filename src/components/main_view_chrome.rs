use gpui::{
    div, px, rgb, rgba, AnyElement, InteractiveElement, IntoElement, ParentElement, Styled,
};

use crate::designs::MainMenuThemeDef;

pub(crate) const MAIN_VIEW_SHELL_ID: &str = "main-view-shell";
pub(crate) const MAIN_VIEW_INPUT_SHELL_ID: &str = "main-view-input-shell";
pub(crate) const MAIN_VIEW_INPUT_STATE_ICON_ID: &str = "main-view-input-state-icon";
pub(crate) const MAIN_VIEW_HEADER_ID: &str = "main-view-header";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_HEADER_DIVIDER_ID: &str = "main-view-header-divider";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_MAIN_ID: &str = "main-view-main";

pub(crate) struct MainViewInputChrome {
    pub(crate) body: AnyElement,
    pub(crate) leading: Option<AnyElement>,
    pub(crate) trailing: Vec<AnyElement>,
}

pub(crate) struct MainViewHeaderChrome {
    pub(crate) input: AnyElement,
    pub(crate) padding_x: f32,
    pub(crate) padding_y: f32,
    pub(crate) gap: f32,
}

pub(crate) struct MainViewDividerChrome {
    pub(crate) margin_x: f32,
    pub(crate) height: f32,
    pub(crate) visible: bool,
}

pub(crate) struct MainViewChrome {
    pub(crate) header: MainViewHeaderChrome,
    pub(crate) divider: MainViewDividerChrome,
    pub(crate) main: AnyElement,
    pub(crate) footer: Option<AnyElement>,
    pub(crate) overlays: Vec<AnyElement>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MainViewColumnMetrics {
    pub(crate) shell_left_x: f32,
    pub(crate) row_leading_x: f32,
    pub(crate) text_column_x: f32,
    pub(crate) input_text_inset_left: f32,
    pub(crate) content_right_inset_x: f32,
    pub(crate) top_inset_y: f32,
}

pub(crate) fn render_main_view_shell() -> gpui::Stateful<gpui::Div> {
    div()
        .id(MAIN_VIEW_SHELL_ID)
        .w_full()
        .h_full()
        .relative()
        .flex()
        .flex_col()
}

pub(crate) fn render_main_view_chrome(
    mut root: gpui::Stateful<gpui::Div>,
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    chrome: MainViewChrome,
) -> AnyElement {
    root = root.child(render_main_view_header(chrome.header));

    if chrome.divider.visible {
        root = root.child(render_main_view_header_divider(
            theme,
            def,
            chrome.divider.margin_x,
            chrome.divider.height,
        ));
    }

    root = root.child(render_main_view_main_slot(def, chrome.main));

    if let Some(footer) = chrome.footer {
        root = root.child(footer);
    }

    for element in chrome.overlays {
        root = root.child(element);
    }

    root.into_any_element()
}

pub(crate) fn render_main_view_header(chrome: MainViewHeaderChrome) -> AnyElement {
    div()
        .id(MAIN_VIEW_HEADER_ID)
        .w_full()
        .px(px(chrome.padding_x))
        .py(px(chrome.padding_y))
        .min_h(px(crate::panel::HEADER_BUTTON_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(chrome.gap))
        .child(chrome.input)
        .into_any_element()
}

#[allow(dead_code)]
pub(crate) fn render_main_view_header_divider(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    margin_x: f32,
    height: f32,
) -> AnyElement {
    div()
        .id(MAIN_VIEW_HEADER_DIVIDER_ID)
        .mx(px(margin_x))
        .h(px(height))
        .bg(rgba(
            (theme.colors.text.primary << 8) | def.shell.divider_alpha,
        ))
        .into_any_element()
}

#[allow(dead_code)]
pub(crate) fn render_main_view_main_slot(def: MainMenuThemeDef, main: AnyElement) -> AnyElement {
    div()
        .id(MAIN_VIEW_MAIN_ID)
        .flex_1()
        .min_h(px(0.))
        .w_full()
        .pb(px(def.shell.content_inset_bottom))
        .overflow_hidden()
        .child(main)
        .into_any_element()
}

pub(crate) fn main_view_input_text_inset_left(def: MainMenuThemeDef) -> f32 {
    main_view_content_columns(def).input_text_inset_left
}

pub(crate) fn main_view_row_leading_x(def: MainMenuThemeDef) -> f32 {
    def.row.outer_padding_x + def.row.inner_padding_x
}

pub(crate) fn main_view_text_column_x(def: MainMenuThemeDef) -> f32 {
    main_view_row_leading_x(def) + def.icon.container_size + def.row.icon_text_gap
}

pub(crate) fn main_view_content_columns(def: MainMenuThemeDef) -> MainViewColumnMetrics {
    let text_column_x = main_view_text_column_x(def);
    MainViewColumnMetrics {
        shell_left_x: def.shell.header_padding_x,
        row_leading_x: main_view_row_leading_x(def),
        text_column_x,
        input_text_inset_left: (text_column_x - def.shell.header_padding_x)
            .max(def.search.text_inset_x),
        content_right_inset_x: def.shell.header_padding_x,
        top_inset_y: def.list.first_section_header_height,
    }
}

pub(crate) fn main_view_state_icon_left(def: MainMenuThemeDef) -> f32 {
    (main_view_row_leading_x(def) - def.shell.header_padding_x).max(0.0)
}

fn main_view_state_icon_path(icon_name: &str) -> &'static str {
    if main_view_state_icon_uses_script_kit_logo(icon_name) {
        return concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg");
    }

    crate::designs::icon_variations::icon_name_from_str(icon_name)
        .unwrap_or(crate::designs::icon_variations::IconName::MagnifyingGlass)
        .external_path()
}

fn main_view_state_icon_uses_script_kit_logo(icon_name: &str) -> bool {
    matches!(
        icon_name
            .to_lowercase()
            .replace(['-', '_', ' '], "")
            .as_str(),
        "search" | "find" | "magnifyingglass"
    )
}

pub(crate) fn render_main_view_state_icon(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    icon_name: &str,
) -> AnyElement {
    let container_size = def.icon.container_size.min(def.search.height).max(16.0);
    let uses_script_kit_logo = main_view_state_icon_uses_script_kit_logo(icon_name);
    let svg_size = if uses_script_kit_logo {
        (container_size - 2.0).max(18.0)
    } else {
        def.icon.svg_size.min(container_size - 4.0).max(12.0)
    };
    let icon_color = if uses_script_kit_logo {
        theme.colors.accent.selected
    } else {
        theme.colors.text.muted
    };
    let left = main_view_state_icon_left(def);
    let top = ((def.search.height - container_size) * 0.5).max(0.0);

    div()
        .id(MAIN_VIEW_INPUT_STATE_ICON_ID)
        .absolute()
        .left(px(left))
        .top(px(top))
        .size(px(container_size))
        .flex()
        .items_center()
        .justify_center()
        .child(
            gpui::svg()
                .external_path(main_view_state_icon_path(icon_name))
                .size(px(svg_size))
                .text_color(rgb(icon_color)),
        )
        .into_any_element()
}

pub(crate) fn render_main_view_input_shell(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    chrome: MainViewInputChrome,
) -> AnyElement {
    let search = def.search;
    let text_inset_left = main_view_input_text_inset_left(def);

    let mut input = div()
        .id(MAIN_VIEW_INPUT_SHELL_ID)
        .flex_1()
        .h(px(search.height))
        .rounded(px(search.radius))
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | search.border_alpha))
        .bg(rgba(
            (theme.colors.background.search_box << 8) | search.surface_alpha,
        ))
        .relative()
        .flex()
        .items_center();

    if let Some(leading) = chrome.leading {
        input = input.child(leading);
    }

    input = input.child(
        div()
            .flex_1()
            .pl(px(text_inset_left))
            .pr(px(search.text_inset_x * 0.5))
            .flex()
            .flex_row()
            .items_center()
            .child(chrome.body),
    );

    for trailing in chrome.trailing {
        input = input.child(trailing);
    }

    input.into_any_element()
}
