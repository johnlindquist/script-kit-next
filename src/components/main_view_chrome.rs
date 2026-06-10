use gpui::{
    div, px, rgba, AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled,
};

use crate::designs::MainMenuThemeDef;

pub(crate) const MAIN_VIEW_SHELL_ID: &str = "main-view-shell";
pub(crate) const MAIN_VIEW_INPUT_SHELL_ID: &str = "main-view-input-shell";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_CONTEXT_ZONE_ID: &str = "main-view-context-zone";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_CONTEXT_LOGO_ID: &str = "main-view-context-logo";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_CONTEXT_CWD_BUTTON_ID: &str = "main-view-context-cwd-button";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID: &str = "main-view-context-model-button";
pub(crate) const MAIN_VIEW_HEADER_ID: &str = "main-view-header";
pub(crate) const MAIN_VIEW_CWD_UNAVAILABLE_LABEL: &str = "No cwd";
pub(crate) const MAIN_VIEW_AGENT_MODEL_UNAVAILABLE_LABEL: &str = "Agent model unavailable";
const DEFAULT_CONTEXT_EDGE_OUTSET_X: f32 = 8.0;
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_HEADER_DIVIDER_ID: &str = "main-view-header-divider";
#[allow(dead_code)]
pub(crate) const MAIN_VIEW_MAIN_ID: &str = "main-view-main";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MainViewContextLabels {
    pub(crate) cwd_label: String,
    pub(crate) agent_model_label: String,
}

impl MainViewContextLabels {
    pub(crate) fn new(cwd_label: impl Into<String>, agent_model_label: impl Into<String>) -> Self {
        let cwd_label = non_empty_label(cwd_label.into(), MAIN_VIEW_CWD_UNAVAILABLE_LABEL);
        let agent_model_label = non_empty_label(
            agent_model_label.into(),
            MAIN_VIEW_AGENT_MODEL_UNAVAILABLE_LABEL,
        );

        Self {
            cwd_label,
            agent_model_label,
        }
    }
}

pub(crate) struct MainViewInputChrome {
    pub(crate) body: AnyElement,
    pub(crate) trailing: Vec<AnyElement>,
}

pub(crate) struct MainViewHeaderChrome {
    pub(crate) context: Option<AnyElement>,
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
    root = root.child(render_main_view_header_with_context_outset(
        chrome.header,
        def.header_info_bar.context_edge_outset_x,
    ));

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
    render_main_view_header_with_context_outset(chrome, DEFAULT_CONTEXT_EDGE_OUTSET_X)
}

pub(crate) fn render_main_view_header_with_context_outset(
    chrome: MainViewHeaderChrome,
    context_edge_outset_x: f32,
) -> AnyElement {
    let mut header = div()
        .id(MAIN_VIEW_HEADER_ID)
        .w_full()
        .px(px(chrome.padding_x))
        .py(px(chrome.padding_y))
        .min_h(px(crate::panel::HEADER_BUTTON_HEIGHT))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(chrome.gap));

    if let Some(context) = chrome.context {
        header = header.child(div().w_full().mx(px(-context_edge_outset_x)).child(context));
    }

    header.child(chrome.input).into_any_element()
}

#[allow(dead_code)] // Used by the binary target through include!-merged render code.
pub(crate) fn render_main_view_context_header(context: AnyElement, padding_x: f32) -> AnyElement {
    div()
        .id(MAIN_VIEW_HEADER_ID)
        .w_full()
        .px(px(padding_x))
        .py(px(crate::ui::chrome::HEADER_PADDING_Y))
        .min_h(px(crate::panel::HEADER_BUTTON_HEIGHT))
        .flex()
        .flex_col()
        .items_center()
        .child(context)
        .into_any_element()
}

fn non_empty_label(label: String, fallback: &'static str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        label
    }
}

#[allow(dead_code)]
pub(crate) fn render_main_view_context_zone(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    cwd_label: Option<String>,
    agent_model_label: Option<String>,
    on_cwd_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_agent_model_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    let labels = MainViewContextLabels::new(
        cwd_label.unwrap_or_else(|| MAIN_VIEW_CWD_UNAVAILABLE_LABEL.to_string()),
        agent_model_label.unwrap_or_else(|| MAIN_VIEW_AGENT_MODEL_UNAVAILABLE_LABEL.to_string()),
    );

    render_main_view_context_zone_required(theme, def, labels, on_cwd_click, on_agent_model_click)
}

pub(crate) fn render_main_view_context_zone_required(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    labels: MainViewContextLabels,
    on_cwd_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_agent_model_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    let info = def.header_info_bar;
    let text_alpha = (info.opacity.clamp(0.0, 1.0) * 255.0).round() as u32;
    let border = rgba((theme.colors.ui.border << 8) | info.pill_border_alpha);
    let hover_border = rgba((theme.colors.ui.border << 8) | info.pill_hover_border_alpha);
    let rest_bg = rgba((theme.colors.background.search_box << 8) | info.pill_bg_alpha);
    let hover_bg = rgba((theme.colors.text.primary << 8) | info.pill_hover_bg_alpha);
    let text_color = rgba((theme.colors.text.primary << 8) | text_alpha);
    let hover_text_color = rgba((theme.colors.text.primary << 8) | info.pill_hover_text_alpha);
    let show_pills = info.pill_padding_x > 0.0 || info.pill_border_alpha > 0;
    let header_keycap_font_size = (info.font_size * 0.88).max(8.0);
    let header_keycap_height = (info.font_size + 7.0).max(16.0);

    let cwd_label = labels.cwd_label;
    let agent_model_label = labels.agent_model_label;

    let cwd_key = if info.show_keys {
        div()
            .opacity(info.key_opacity.clamp(0.0, 1.0))
            .child(
                crate::components::footer_chrome::render_footer_hint_button_like(
                    crate::components::footer_chrome::FooterHintButtonSpec {
                        label: cwd_label.clone().into(),
                        key: "⇥".into(),
                        slot_width_px: None,
                        key_first: false,
                        justify: crate::components::footer_chrome::FooterHintContentJustify::Start,
                        label_font_size_px: Some(info.font_size),
                        keycap_font_size_px: Some(header_keycap_font_size),
                        keycap_height_px: Some(header_keycap_height),
                        hover_text_alpha: Some(info.pill_hover_text_alpha),
                        hover_glyph_alpha: Some(info.pill_hover_key_alpha),
                        hover_keycap_border_alpha: Some(info.pill_hover_border_alpha),
                    },
                    theme,
                ),
            )
            .into_any_element()
    } else {
        div()
            .min_w(px(0.0))
            .overflow_hidden()
            .text_ellipsis()
            .child(cwd_label.clone())
            .into_any_element()
    };

    let model_key = if info.show_keys {
        div()
            .opacity(info.key_opacity.clamp(0.0, 1.0))
            .child(
                crate::components::footer_chrome::render_footer_hint_button_like(
                    crate::components::footer_chrome::FooterHintButtonSpec {
                        label: agent_model_label.clone().into(),
                        key: "⇧⇥".into(),
                        slot_width_px: None,
                        key_first: false,
                        justify: crate::components::footer_chrome::FooterHintContentJustify::Start,
                        label_font_size_px: Some(info.font_size),
                        keycap_font_size_px: Some(header_keycap_font_size),
                        keycap_height_px: Some(header_keycap_height),
                        hover_text_alpha: Some(info.pill_hover_text_alpha),
                        hover_glyph_alpha: Some(info.pill_hover_key_alpha),
                        hover_keycap_border_alpha: Some(info.pill_hover_border_alpha),
                    },
                    theme,
                ),
            )
            .into_any_element()
    } else {
        div()
            .min_w(px(0.0))
            .overflow_hidden()
            .text_ellipsis()
            .child(agent_model_label.clone())
            .into_any_element()
    };

    let mut cwd_chip = div()
        .id(MAIN_VIEW_CONTEXT_CWD_BUTTON_ID)
        .min_w(px(0.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(info.gap_px))
        .px(px(info.pill_padding_x))
        .py(px(info.pill_padding_y))
        .rounded(px(info.pill_radius))
        .font_family(info.font_family)
        .text_size(px(info.font_size))
        .text_color(text_color)
        .cursor_pointer()
        .hover(move |s| {
            s.bg(hover_bg)
                .text_color(hover_text_color)
                .border_color(hover_border)
        })
        .on_click(on_cwd_click)
        .child(cwd_key);
    if show_pills {
        cwd_chip = cwd_chip.border_1().border_color(border).bg(rest_bg);
    }

    let mut model_chip = div()
        .id(MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID)
        .min_w(px(0.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(info.gap_px))
        .px(px(info.pill_padding_x))
        .py(px(info.pill_padding_y))
        .rounded(px(info.pill_radius))
        .font_family(info.font_family)
        .text_size(px(info.font_size))
        .text_color(text_color)
        .cursor_pointer()
        .hover(move |s| {
            s.bg(hover_bg)
                .text_color(hover_text_color)
                .border_color(hover_border)
        })
        .on_click(on_agent_model_click)
        .child(model_key);
    if show_pills {
        model_chip = model_chip.border_1().border_color(border).bg(rest_bg);
    }

    let mut left_lane = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_start()
        .gap(px(info.gap_px));
    if info.show_cwd {
        left_lane = left_lane.child(cwd_chip);
    }
    if info.show_cwd
        && info.show_agent_model
        && !matches!(info.layout, crate::designs::HeaderInfoBarLayout::Split)
    {
        left_lane = left_lane.child(
            div()
                .font_family(info.font_family)
                .text_size(px(info.font_size))
                .text_color(text_color)
                .child(info.separator),
        );
    }

    let mut right_lane = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .gap(px(info.gap_px));
    if info.show_agent_model {
        right_lane = right_lane.child(model_chip);
    }

    div()
        .id(MAIN_VIEW_CONTEXT_ZONE_ID)
        .w_full()
        .h(px(info.height_px))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(info.gap_px))
        .child(left_lane)
        .child(right_lane)
        .into_any_element()
}

#[allow(dead_code)]
pub(crate) fn render_main_view_context_zone_inert(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    cwd_label: Option<String>,
    agent_model_label: Option<String>,
) -> AnyElement {
    render_main_view_context_zone(
        theme,
        def,
        cwd_label,
        agent_model_label,
        |_event, _window, _cx| {},
        |_event, _window, _cx| {},
    )
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
        .flex()
        .flex_col()
        .child(main)
        .into_any_element()
}

pub(crate) fn main_view_input_text_inset_left(def: MainMenuThemeDef) -> f32 {
    def.search.text_inset_x
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

pub(crate) fn render_main_view_input_shell(
    theme: &crate::theme::Theme,
    def: MainMenuThemeDef,
    chrome: MainViewInputChrome,
) -> AnyElement {
    let search = def.search;
    let text_inset_left = main_view_input_text_inset_left(def);

    let mut input = div()
        .id(MAIN_VIEW_INPUT_SHELL_ID)
        .w_full()
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
