use gpui::{
    div, px, rgb, rgba, AppContext, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window, WindowHandle,
};
use gpui_component::{
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    Root, Sizable,
};

use crate::dev_style_tool::{
    catalog::{knob_by_id, StyleValue},
    runtime_overrides, SEARCH_HEIGHT_KNOB_ID,
};
use crate::{theme, ScriptListApp};

pub(crate) struct DevStyleToolApp {
    main_window: WindowHandle<Root>,
    main_app: Entity<ScriptListApp>,
    search_height_input: Entity<InputState>,
    search_height_slider: Entity<SliderState>,
    subscriptions: Vec<Subscription>,
}

impl DevStyleToolApp {
    pub(crate) fn new(
        main_window: WindowHandle<Root>,
        main_app: Entity<ScriptListApp>,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let initial = current_search_height_value();
        let search_height_input = cx.new(|cx| {
            InputState::new(window, cx)
                .tab_navigation(true)
                .default_value(format_style_value(initial))
        });
        let search_height_slider = cx.new(|_| {
            SliderState::new()
                .min(search_height_knob().min)
                .max(search_height_knob().max)
                .step(search_height_knob().step)
                .default_value(initial)
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe_in(
            &search_height_slider,
            window,
            move |this, _, event: &SliderEvent, window, cx| match event {
                SliderEvent::Change(value) | SliderEvent::Release(value) => {
                    this.apply_search_height(*value, window, cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &search_height_input,
            window,
            move |this, input, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } | InputEvent::Blur => {
                    let value = input.read(cx).value().to_string();
                    this.commit_search_height_text(&value, window, cx);
                }
                _ => {}
            },
        ));

        Self {
            main_window,
            main_app,
            search_height_input,
            search_height_slider,
            subscriptions,
        }
    }

    fn apply_search_height(
        &mut self,
        value: SliderValue,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let value = value.end();
        if let Some(change) =
            runtime_overrides::set_value(SEARCH_HEIGHT_KNOB_ID, StyleValue::Number(value))
        {
            let StyleValue::Number(applied) = change.applied;
            self.sync_controls_to_value(applied, window, cx);
            self.refresh_main_window(cx);
        }
    }

    fn commit_search_height_text(
        &mut self,
        value: &str,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Ok(number) = value.trim().parse::<f32>() else {
            self.sync_controls_to_value(current_search_height_value(), window, cx);
            return;
        };
        if let Some(change) =
            runtime_overrides::set_value(SEARCH_HEIGHT_KNOB_ID, StyleValue::Number(number))
        {
            let StyleValue::Number(applied) = change.applied;
            self.sync_controls_to_value(applied, window, cx);
            self.refresh_main_window(cx);
        }
    }

    fn reset_search_height(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let _ = runtime_overrides::reset_value(SEARCH_HEIGHT_KNOB_ID);
        self.sync_controls_to_value(current_search_height_value(), window, cx);
        self.refresh_main_window(cx);
    }

    fn sync_controls_to_value(
        &mut self,
        value: f32,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.search_height_slider.update(cx, |slider, cx| {
            if (slider.value().end() - value).abs() > f32::EPSILON {
                slider.set_value(SliderValue::Single(value), window, cx);
            }
        });
        let label = format_style_value(value);
        self.search_height_input.update(cx, |input, cx| {
            if input.value().as_ref() != label {
                input.set_value(label, window, cx);
            }
        });
        cx.notify();
    }

    fn refresh_main_window(&self, cx: &mut gpui::Context<Self>) {
        self.main_app.update(cx, |view, cx| {
            view.update_theme(cx);
            cx.notify();
        });
        let _ = self.main_window.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

impl Render for DevStyleToolApp {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let chrome = theme::AppChromeColors::from_theme(&theme);
        let knob = search_height_knob();
        let effective = current_search_height_value();
        let base = match (knob.get)(&crate::designs::current_main_menu_theme().base_def()) {
            StyleValue::Number(value) => value,
        };
        let generation = runtime_overrides::generation();

        div()
            .id("dev-style-tool")
            .size_full()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(16.0))
            .bg(rgba(chrome.window_surface_rgba))
            .text_color(rgb(chrome.text_primary_hex))
            .font_family(crate::list_item::FONT_SYSTEM_UI)
            .child(
                div()
                    .id("panel:dev-style-tool")
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(super::window::DEV_STYLE_TOOL_TITLE),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(format!("Runtime generation {generation}")),
                    ),
            )
            .child(
                div()
                    .id("style-section:search")
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_sm().child(knob.label))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child(format!("base {}", format_style_value(base))),
                            ),
                    )
                    .child(
                        div()
                            .id("slider:dev-style-tool:search-height")
                            .h(px(26.0))
                            .flex()
                            .items_center()
                            .child(Slider::new(&self.search_height_slider).horizontal()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("input:dev-style-tool:search-height")
                                    .w(px(92.0))
                                    .child(Input::new(&self.search_height_input).small()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child("px"),
                            )
                            .child(
                                div()
                                    .id("button:dev-style-tool-reset-search-height")
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .rounded(px(crate::ui::chrome::LIQUID_GLASS_COMPACT_RADIUS_PX))
                                    .border(px(1.0))
                                    .border_color(rgba(chrome.border_rgba))
                                    .text_xs()
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgba(chrome.hover_rgba)))
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.reset_search_height(window, cx);
                                    }))
                                    .child("Reset"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child(format!("current {}", format_style_value(effective))),
                            ),
                    ),
            )
    }
}

fn search_height_knob() -> &'static crate::dev_style_tool::catalog::StyleKnob {
    knob_by_id(SEARCH_HEIGHT_KNOB_ID).expect("search.height style knob must exist")
}

fn current_search_height_value() -> f32 {
    match runtime_overrides::current_value(SEARCH_HEIGHT_KNOB_ID).unwrap_or_else(|| {
        (search_height_knob().get)(&crate::designs::current_main_menu_theme().base_def())
    }) {
        StyleValue::Number(value) => value,
    }
}

fn format_style_value(value: f32) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}
