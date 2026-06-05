use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, rgb, rgba, AppContext, Div, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window, WindowHandle,
};
use gpui_component::{
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    Root, Sizable,
};

use crate::dev_style_tool::{
    catalog::{
        knob_by_id, StyleKnob, StyleKnobGroup, StyleKnobId, StyleKnobSection, StyleUnit,
        StyleValue, STYLE_KNOBS,
    },
    export, runtime_overrides,
};
use crate::{theme, ScriptListApp};

struct StyleControlState {
    knob_id: StyleKnobId,
    input: Entity<InputState>,
    slider: Entity<SliderState>,
}

pub(crate) struct DevStyleToolApp {
    main_window: WindowHandle<Root>,
    main_app: Entity<ScriptListApp>,
    controls: Vec<StyleControlState>,
    save_status: Option<String>,
    subscriptions: Vec<Subscription>,
}

impl DevStyleToolApp {
    pub(crate) fn new(
        main_window: WindowHandle<Root>,
        main_app: Entity<ScriptListApp>,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let mut subscriptions = Vec::new();
        let controls = STYLE_KNOBS
            .iter()
            .map(|knob| {
                let initial = current_knob_value(knob.id);
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .tab_navigation(true)
                        .default_value(format_style_value(initial, knob.unit))
                });
                let slider = cx.new(|_| {
                    SliderState::new()
                        .min(knob.min)
                        .max(knob.max)
                        .step(knob.step)
                        .default_value(initial)
                });

                let knob_id = knob.id;
                subscriptions.push(cx.subscribe_in(
                    &slider,
                    window,
                    move |this, _, event: &SliderEvent, window, cx| match event {
                        SliderEvent::Change(value) | SliderEvent::Release(value) => {
                            this.apply_knob_value(knob_id, value.end(), window, cx);
                        }
                    },
                ));
                subscriptions.push(cx.subscribe_in(
                    &input,
                    window,
                    move |this, input, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { .. } | InputEvent::Blur => {
                            let value = input.read(cx).value().to_string();
                            this.commit_knob_text(knob_id, &value, window, cx);
                        }
                        _ => {}
                    },
                ));

                StyleControlState {
                    knob_id,
                    input,
                    slider,
                }
            })
            .collect();

        Self {
            main_window,
            main_app,
            controls,
            save_status: None,
            subscriptions,
        }
    }

    fn apply_knob_value(
        &mut self,
        knob_id: StyleKnobId,
        value: f32,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(change) = runtime_overrides::set_value(knob_id, StyleValue::Number(value)) {
            let StyleValue::Number(applied) = change.applied;
            self.sync_control_to_value(knob_id, applied, window, cx);
            self.refresh_main_window(cx);
        }
    }

    fn commit_knob_text(
        &mut self,
        knob_id: StyleKnobId,
        value: &str,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(knob) = knob_by_id(knob_id) else {
            return;
        };
        let Ok(number) = parse_style_value(value, knob.unit) else {
            self.sync_control_to_value(knob_id, current_knob_value(knob_id), window, cx);
            return;
        };
        self.apply_knob_value(knob_id, number, window, cx);
    }

    fn reset_knob(
        &mut self,
        knob_id: StyleKnobId,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let _ = runtime_overrides::reset_value(knob_id);
        self.sync_control_to_value(knob_id, current_knob_value(knob_id), window, cx);
        self.refresh_main_window(cx);
    }

    fn undo_style_change(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if runtime_overrides::undo_last().is_some() {
            self.sync_all_controls(window, cx);
            self.refresh_main_window(cx);
        }
    }

    fn redo_style_change(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if runtime_overrides::redo_last().is_some() {
            self.sync_all_controls(window, cx);
            self.refresh_main_window(cx);
        }
    }

    fn reset_all_controls(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        runtime_overrides::reset_all();
        self.sync_all_controls(window, cx);
        self.refresh_main_window(cx);
    }

    fn sync_all_controls(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let knob_ids: Vec<StyleKnobId> = self
            .controls
            .iter()
            .map(|control| control.knob_id)
            .collect();
        for knob_id in knob_ids {
            self.sync_control_to_value(knob_id, current_knob_value(knob_id), window, cx);
        }
    }

    fn sync_control_to_value(
        &mut self,
        knob_id: StyleKnobId,
        value: f32,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(knob) = knob_by_id(knob_id) else {
            return;
        };
        let Some(control) = self
            .controls
            .iter()
            .find(|control| control.knob_id == knob_id)
        else {
            return;
        };
        control.slider.update(cx, |slider, cx| {
            if (slider.value().end() - value).abs() > f32::EPSILON {
                slider.set_value(SliderValue::Single(value), window, cx);
            }
        });
        let label = format_style_value(value, knob.unit);
        control.input.update(cx, |input, cx| {
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

    fn save_current_settings(&mut self, cx: &mut gpui::Context<Self>) {
        self.save_status = Some(match export::save_current_settings_markdown() {
            Ok(path) => export::export_summary_for_path(&path),
            Err(error) => format!("Save failed: {error}"),
        });
        cx.notify();
    }

    fn render_groups(
        &self,
        groups: &[StyleKnobGroup],
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> Div {
        let mut column = div().flex().flex_col().gap(px(10.0)).flex_1();
        for group in groups {
            let controls: Vec<&StyleControlState> = self
                .controls
                .iter()
                .filter(|control| {
                    knob_by_id(control.knob_id).is_some_and(|knob| knob.group == *group)
                })
                .collect();
            if controls.is_empty() {
                continue;
            }
            column = column.child(self.render_group(*group, controls, chrome, cx));
        }
        column
    }

    fn render_group(
        &self,
        group: StyleKnobGroup,
        controls: Vec<&StyleControlState>,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(ElementId::Name(
                format!("style-section:{}", group_slug(group)).into(),
            ))
            .flex()
            .flex_col()
            .gap(px(5.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(chrome.text_secondary_hex))
                    .child(group.label()),
            )
            .children(self.group_controls_by_section(controls).into_iter().map(
                |(section, controls)| {
                    self.render_control_section(group, section, controls, chrome, cx)
                },
            ))
    }

    fn group_controls_by_section<'a>(
        &self,
        controls: Vec<&'a StyleControlState>,
    ) -> Vec<(StyleKnobSection, Vec<&'a StyleControlState>)> {
        let mut sections: Vec<(StyleKnobSection, Vec<&StyleControlState>)> = Vec::new();
        for control in controls {
            let Some(knob) = knob_by_id(control.knob_id) else {
                continue;
            };
            let section = StyleKnobSection::for_knob(knob);
            if let Some((_, section_controls)) = sections
                .iter_mut()
                .find(|(existing, _)| *existing == section)
            {
                section_controls.push(control);
            } else {
                sections.push((section, vec![control]));
            }
        }
        sections
    }

    fn render_control_section(
        &self,
        group: StyleKnobGroup,
        section: StyleKnobSection,
        controls: Vec<&StyleControlState>,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(ElementId::Name(
                format!("style-subsection:{}:{}", group_slug(group), section.slug).into(),
            ))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chrome.text_secondary_hex))
                    .child(section.label),
            )
            .children(
                controls
                    .into_iter()
                    .map(|control| self.render_control(control, chrome, cx)),
            )
    }

    fn render_control(
        &self,
        control: &StyleControlState,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let knob = knob_by_id(control.knob_id).expect("style control must reference catalog knob");
        let base = knob_base_value(knob);
        let effective = current_knob_value(knob.id);
        let knob_id = knob.id;

        div()
            .id(ElementId::Name(
                format!("control:dev-style-tool:{}", knob.id.as_str()).into(),
            ))
            .flex()
            .flex_col()
            .gap(px(3.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(div().text_xs().child(knob.label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(format!(
                                "{} | base {}",
                                format_style_value(effective, knob.unit),
                                format_style_value(base, knob.unit)
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .id(ElementId::Name(
                                format!("slider:dev-style-tool:{}", knob.id.as_str()).into(),
                            ))
                            .h(px(22.0))
                            .flex_1()
                            .flex()
                            .items_center()
                            .child(Slider::new(&control.slider).horizontal()),
                    )
                    .child(
                        div()
                            .id(ElementId::Name(
                                format!("input:dev-style-tool:{}", knob.id.as_str()).into(),
                            ))
                            .w(px(64.0))
                            .child(Input::new(&control.input).small()),
                    )
                    .child(
                        div()
                            .w(px(42.0))
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(knob.unit.label()),
                    )
                    .child(
                        div()
                            .id(ElementId::Name(
                                format!("button:dev-style-tool-reset:{}", knob.id.as_str()).into(),
                            ))
                            .px(px(6.0))
                            .py(px(3.0))
                            .rounded(px(crate::ui::chrome::LIQUID_GLASS_COMPACT_RADIUS_PX))
                            .border(px(1.0))
                            .border_color(rgba(chrome.border_rgba))
                            .text_xs()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgba(chrome.hover_rgba)))
                            .on_click(cx.listener(move |this, _event, window, cx| {
                                this.reset_knob(knob_id, window, cx);
                            }))
                            .child("Reset"),
                    ),
            )
    }

    fn render_toolbar_button(
        &self,
        semantic_id: &'static str,
        label: &'static str,
        enabled: bool,
        chrome: theme::AppChromeColors,
    ) -> gpui::Stateful<Div> {
        div()
            .id(semantic_id)
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(crate::ui::chrome::LIQUID_GLASS_COMPACT_RADIUS_PX))
            .border(px(1.0))
            .border_color(rgba(chrome.border_rgba))
            .text_xs()
            .cursor_pointer()
            .opacity(if enabled { 1.0 } else { 0.45 })
            .hover(|style| style.bg(rgba(chrome.hover_rgba)))
            .child(label)
    }
}

impl Render for DevStyleToolApp {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let chrome = theme::AppChromeColors::from_theme(&theme);
        let generation = runtime_overrides::generation();
        let history = runtime_overrides::history_state();
        let left_groups = [
            StyleKnobGroup::Shell,
            StyleKnobGroup::Search,
            StyleKnobGroup::List,
            StyleKnobGroup::Row,
        ];
        let right_groups = [
            StyleKnobGroup::Icon,
            StyleKnobGroup::Metadata,
            StyleKnobGroup::Typography,
            StyleKnobGroup::Footer,
            StyleKnobGroup::HeaderInfoBar,
        ];

        div()
            .id("dev-style-tool")
            .size_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .p(px(14.0))
            .bg(rgba(chrome.window_surface_rgba))
            .text_color(rgb(chrome.text_primary_hex))
            .font_family(crate::list_item::FONT_SYSTEM_UI)
            .overflow_hidden()
            .child(
                div()
                    .id("panel:dev-style-tool")
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(super::window::DEV_STYLE_TOOL_TITLE),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child(format!(
                                        "{} controls | runtime generation {generation}",
                                        STYLE_KNOBS.len()
                                    )),
                            )
                            .child(
                                self.render_toolbar_button(
                                    "button:dev-style-tool-undo",
                                    "Undo",
                                    history.can_undo,
                                    chrome,
                                )
                                .on_click(cx.listener(
                                    |this, _event, window, cx| {
                                        this.undo_style_change(window, cx);
                                    },
                                )),
                            )
                            .child(
                                self.render_toolbar_button(
                                    "button:dev-style-tool-redo",
                                    "Redo",
                                    history.can_redo,
                                    chrome,
                                )
                                .on_click(cx.listener(
                                    |this, _event, window, cx| {
                                        this.redo_style_change(window, cx);
                                    },
                                )),
                            )
                            .child(
                                self.render_toolbar_button(
                                    "button:dev-style-tool-reset-all",
                                    "Reset All",
                                    history.override_count > 0,
                                    chrome,
                                )
                                .on_click(cx.listener(
                                    |this, _event, window, cx| {
                                        this.reset_all_controls(window, cx);
                                    },
                                )),
                            )
                            .child(
                                self.render_toolbar_button(
                                    "button:dev-style-tool-save",
                                    "Save",
                                    true,
                                    chrome,
                                )
                                .on_click(cx.listener(
                                    |this, _event, _window, cx| {
                                        this.save_current_settings(cx);
                                    },
                                )),
                            ),
                    ),
            )
            .when_some(self.save_status.as_ref(), |view, status| {
                view.child(
                    div()
                        .id("status:dev-style-tool-save")
                        .text_xs()
                        .text_color(rgb(chrome.text_secondary_hex))
                        .child(status.clone()),
                )
            })
            .child(
                div()
                    .id("body:dev-style-tool-scroll")
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .gap(px(14.0))
                    .pr(px(4.0))
                    .overflow_y_scroll()
                    .child(self.render_groups(&left_groups, chrome, cx))
                    .child(self.render_groups(&right_groups, chrome, cx)),
            )
    }
}

fn current_knob_value(knob_id: StyleKnobId) -> f32 {
    match runtime_overrides::current_value(knob_id).unwrap_or_else(|| {
        let knob = knob_by_id(knob_id).expect("style knob must exist");
        (knob.get)(&crate::designs::current_main_menu_theme().base_def())
    }) {
        StyleValue::Number(value) => value,
    }
}

fn knob_base_value(knob: &StyleKnob) -> f32 {
    match (knob.get)(&crate::designs::current_main_menu_theme().base_def()) {
        StyleValue::Number(value) => value,
    }
}

fn parse_style_value(value: &str, unit: StyleUnit) -> Result<f32, std::num::ParseFloatError> {
    let trimmed = value.trim();
    let trimmed = match unit {
        StyleUnit::Px => trimmed.trim_end_matches("px").trim(),
        StyleUnit::Alpha => trimmed.trim_end_matches("alpha").trim(),
        StyleUnit::Opacity => trimmed.trim_end_matches('%').trim(),
    };
    let parsed = trimmed.parse::<f32>()?;
    Ok(
        if matches!(unit, StyleUnit::Opacity) && value.trim().ends_with('%') {
            parsed / 100.0
        } else {
            parsed
        },
    )
}

fn format_style_value(value: f32, unit: StyleUnit) -> String {
    match unit {
        StyleUnit::Opacity => format!("{value:.2}"),
        StyleUnit::Alpha => format!("{value:.0}"),
        StyleUnit::Px if (value.fract()).abs() < f32::EPSILON => format!("{value:.0}"),
        StyleUnit::Px => format!("{value:.1}"),
    }
}

fn group_slug(group: StyleKnobGroup) -> &'static str {
    match group {
        StyleKnobGroup::Shell => "shell",
        StyleKnobGroup::Search => "search",
        StyleKnobGroup::List => "list",
        StyleKnobGroup::Row => "row",
        StyleKnobGroup::Icon => "icon",
        StyleKnobGroup::Metadata => "metadata",
        StyleKnobGroup::Typography => "typography",
        StyleKnobGroup::Footer => "footer",
        StyleKnobGroup::HeaderInfoBar => "header-info-bar",
    }
}
