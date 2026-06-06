use std::path::PathBuf;

use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, rgb, rgba, AppContext, ClipboardItem, Div, ElementId, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window,
    WindowHandle,
};
use gpui_component::{
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    tab::{Tab, TabBar},
    Root, Sizable,
};

use crate::dev_style_tool::{
    actions_popup_catalog::{
        actions_popup_knob_by_id, ActionsPopupKnob, ActionsPopupKnobGroup, ActionsPopupKnobId,
        ACTIONS_POPUP_KNOBS,
    },
    catalog::{
        knob_by_id, StyleKnob, StyleKnobGroup, StyleKnobId, StyleKnobSection, StyleUnit,
        StyleValue, STYLE_KNOBS,
    },
    copy_catalog::{copy_control_by_id, CopyControlId, COPY_CONTROLS},
    export, runtime_overrides,
};
use crate::{theme, ScriptListApp};

struct StyleControlState {
    knob_id: StyleKnobId,
    input: Entity<InputState>,
    slider: Entity<SliderState>,
}

struct CopyControlState {
    control_id: CopyControlId,
    input: Entity<InputState>,
}

struct ActionsPopupControlState {
    knob_id: ActionsPopupKnobId,
    input: Entity<InputState>,
    slider: Entity<SliderState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevStyleToolTab {
    MainWindowStyling,
    TextCopy,
    ActionsPopupStyling,
}

impl DevStyleToolTab {
    const ALL: &'static [Self] = &[
        Self::MainWindowStyling,
        Self::TextCopy,
        Self::ActionsPopupStyling,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::MainWindowStyling => "Main Window Styling",
            Self::TextCopy => "Text",
            Self::ActionsPopupStyling => "Actions Popup Styling",
        }
    }

    const fn semantic_id(self) -> &'static str {
        match self {
            Self::MainWindowStyling => "tab:dev-style-tool:main-window-styling",
            Self::TextCopy => "tab:dev-style-tool:text-copy",
            Self::ActionsPopupStyling => "tab:dev-style-tool:actions-popup-styling",
        }
    }
}

pub(crate) struct DevStyleToolApp {
    main_window: WindowHandle<Root>,
    main_app: Entity<ScriptListApp>,
    controls: Vec<StyleControlState>,
    copy_controls: Vec<CopyControlState>,
    actions_popup_controls: Vec<ActionsPopupControlState>,
    save_status: Option<String>,
    save_path: Option<PathBuf>,
    saved_markdown: Entity<InputState>,
    active_tab: DevStyleToolTab,
    active_group: StyleKnobGroup,
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
        let copy_controls = COPY_CONTROLS
            .iter()
            .map(|control| {
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .tab_navigation(true)
                        .default_value(runtime_overrides::effective_copy_value(control.id))
                });
                let control_id = control.id;
                subscriptions.push(cx.subscribe_in(
                    &input,
                    window,
                    move |this, input, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let value = input.read(cx).value().to_string();
                            this.apply_copy_value(control_id, value, window, cx);
                        }
                    },
                ));
                CopyControlState { control_id, input }
            })
            .collect();
        let actions_popup_controls = ACTIONS_POPUP_KNOBS
            .iter()
            .map(|knob| {
                let initial = current_actions_popup_knob_value(knob.id);
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
                            this.apply_actions_popup_knob_value(knob_id, value.end(), window, cx);
                        }
                    },
                ));
                subscriptions.push(cx.subscribe_in(
                    &input,
                    window,
                    move |this, input, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { .. } | InputEvent::Blur => {
                            let value = input.read(cx).value().to_string();
                            this.commit_actions_popup_knob_text(knob_id, &value, window, cx);
                        }
                        _ => {}
                    },
                ));

                ActionsPopupControlState {
                    knob_id,
                    input,
                    slider,
                }
            })
            .collect();
        let saved_markdown = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .tab_navigation(true)
                .default_value("")
        });

        Self {
            main_window,
            main_app,
            controls,
            copy_controls,
            actions_popup_controls,
            save_status: None,
            save_path: None,
            saved_markdown,
            active_tab: DevStyleToolTab::MainWindowStyling,
            active_group: StyleKnobGroup::Search,
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

    fn apply_copy_value(
        &mut self,
        control_id: CopyControlId,
        value: String,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if runtime_overrides::set_copy_value(control_id, value).is_some() {
            self.refresh_main_window(cx);
        }
    }

    fn reset_copy_control(
        &mut self,
        control_id: CopyControlId,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let _ = runtime_overrides::reset_copy_value(control_id);
        self.sync_copy_control_to_value(control_id, window, cx);
        self.refresh_main_window(cx);
    }

    fn apply_actions_popup_knob_value(
        &mut self,
        knob_id: ActionsPopupKnobId,
        value: f32,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(change) =
            runtime_overrides::set_actions_popup_value(knob_id, StyleValue::Number(value))
        {
            let StyleValue::Number(applied) = change.applied;
            self.sync_actions_popup_control_to_value(knob_id, applied, window, cx);
            self.refresh_actions_popup(cx);
        }
    }

    fn commit_actions_popup_knob_text(
        &mut self,
        knob_id: ActionsPopupKnobId,
        value: &str,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(knob) = actions_popup_knob_by_id(knob_id) else {
            return;
        };
        let Ok(number) = parse_style_value(value, knob.unit) else {
            self.sync_actions_popup_control_to_value(
                knob_id,
                current_actions_popup_knob_value(knob_id),
                window,
                cx,
            );
            return;
        };
        self.apply_actions_popup_knob_value(knob_id, number, window, cx);
    }

    fn reset_actions_popup_knob(
        &mut self,
        knob_id: ActionsPopupKnobId,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let _ = runtime_overrides::reset_actions_popup_value(knob_id);
        self.sync_actions_popup_control_to_value(
            knob_id,
            current_actions_popup_knob_value(knob_id),
            window,
            cx,
        );
        self.refresh_actions_popup(cx);
    }

    fn undo_style_change(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if runtime_overrides::undo_last().is_some() {
            self.sync_all_controls(window, cx);
            self.refresh_main_window(cx);
            self.refresh_actions_popup(cx);
        }
    }

    fn redo_style_change(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        if runtime_overrides::redo_last().is_some() {
            self.sync_all_controls(window, cx);
            self.refresh_main_window(cx);
            self.refresh_actions_popup(cx);
        }
    }

    fn reset_all_controls(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        runtime_overrides::reset_all();
        self.sync_all_controls(window, cx);
        self.refresh_main_window(cx);
        self.refresh_actions_popup(cx);
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
        let copy_ids: Vec<CopyControlId> = self
            .copy_controls
            .iter()
            .map(|control| control.control_id)
            .collect();
        for control_id in copy_ids {
            self.sync_copy_control_to_value(control_id, window, cx);
        }
        let actions_knob_ids: Vec<ActionsPopupKnobId> = self
            .actions_popup_controls
            .iter()
            .map(|control| control.knob_id)
            .collect();
        for knob_id in actions_knob_ids {
            self.sync_actions_popup_control_to_value(
                knob_id,
                current_actions_popup_knob_value(knob_id),
                window,
                cx,
            );
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

    fn sync_copy_control_to_value(
        &mut self,
        control_id: CopyControlId,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(control) = self
            .copy_controls
            .iter()
            .find(|control| control.control_id == control_id)
        else {
            return;
        };
        let value = runtime_overrides::effective_copy_value(control_id);
        control.input.update(cx, |input, cx| {
            if input.value().as_ref() != value {
                input.set_value(value, window, cx);
            }
        });
        cx.notify();
    }

    fn sync_actions_popup_control_to_value(
        &mut self,
        knob_id: ActionsPopupKnobId,
        value: f32,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(knob) = actions_popup_knob_by_id(knob_id) else {
            return;
        };
        let Some(control) = self
            .actions_popup_controls
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
        let _ = self.main_window.update(cx, |_root, window, cx| {
            crate::footer_popup::refresh_main_footer_popup_for_runtime_style(window, cx);
            self.main_app.update(cx, |view, cx| {
                view.refresh_runtime_copy_controls(window, cx);
            });
            cx.notify();
        });
    }

    fn refresh_actions_popup(&self, cx: &mut gpui::Context<Self>) {
        if let Some(dialog) = crate::actions::get_actions_dialog_entity(cx) {
            dialog.update(cx, |_dialog, cx| cx.notify());
            crate::actions::resize_actions_window(cx, &dialog);
            crate::actions::notify_actions_window(cx);
        }
    }

    fn save_current_settings(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        self.save_status = Some(
            match export::save_current_settings_markdown_with_contents() {
                Ok((path, contents)) => {
                    self.save_path = Some(path.clone());
                    self.saved_markdown.update(cx, |input, cx| {
                        input.set_value(contents, window, cx);
                    });
                    export::export_summary_for_path(&path)
                }
                Err(error) => format!("Save failed: {error}"),
            },
        );
        cx.notify();
    }

    fn copy_saved_markdown(&self, cx: &mut gpui::Context<Self>) {
        let markdown = self.saved_markdown.read(cx).value().to_string();
        if !markdown.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(markdown));
        }
    }

    fn render_group_tabs(
        &self,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let selected_index = STYLE_KNOB_GROUPS
            .iter()
            .position(|group| *group == self.active_group)
            .unwrap_or(0);
        div()
            .id("tabs:dev-style-tool-groups")
            .child(
                TabBar::new("tabbar:dev-style-tool-groups")
                    .segmented()
                    .small()
                    .selected_index(selected_index)
                    .children(STYLE_KNOB_GROUPS.iter().map(|group| {
                        let group = *group;
                        Tab::new().label(group.label()).on_click(cx.listener(
                            move |this, _event, _window, cx| {
                                this.active_group = group;
                                cx.notify();
                            },
                        ))
                    })),
            )
            .text_color(rgb(chrome.text_primary_hex))
    }

    fn render_primary_tabs(
        &self,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let selected_index = DevStyleToolTab::ALL
            .iter()
            .position(|tab| *tab == self.active_tab)
            .unwrap_or(0);
        div()
            .id("tabs:dev-style-tool-primary")
            .child(
                TabBar::new("tabbar:dev-style-tool-primary")
                    .segmented()
                    .small()
                    .selected_index(selected_index)
                    .children(DevStyleToolTab::ALL.iter().map(|tab| {
                        let tab = *tab;
                        Tab::new().label(tab.label()).on_click(cx.listener(
                            move |this, _event, _window, cx| {
                                this.active_tab = tab;
                                cx.notify();
                            },
                        ))
                    })),
            )
            .child(
                div()
                    .id(self.active_tab.semantic_id())
                    .h(px(0.0))
                    .overflow_hidden(),
            )
            .text_color(rgb(chrome.text_primary_hex))
    }

    fn render_saved_markdown(
        &self,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let title = self
            .save_path
            .as_ref()
            .map(|path| format!("Saved export: {}", path.display()))
            .unwrap_or_else(|| "Saved export".to_string());
        div()
            .id("panel:dev-style-tool-export")
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(title),
                    )
                    .child(
                        self.render_toolbar_button(
                            "button:dev-style-tool-copy-markdown",
                            "Copy Markdown",
                            true,
                            chrome,
                        )
                        .on_click(cx.listener(
                            |this, _event, _window, cx| {
                                this.copy_saved_markdown(cx);
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .id("input:dev-style-tool-saved-markdown")
                    .h(px(160.0))
                    .child(Input::new(&self.saved_markdown).small().h_full()),
            )
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

    fn render_copy_controls(
        &self,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> Div {
        let mut column = div().flex().flex_col().gap(px(10.0)).flex_1();
        for control in &self.copy_controls {
            let Some(copy_control) = copy_control_by_id(control.control_id) else {
                continue;
            };
            let effective = runtime_overrides::effective_copy_value(copy_control.id);
            let base = (copy_control.base)();
            let control_id = copy_control.id;
            column = column.child(
                div()
                    .id(ElementId::Name(
                        format!("control:dev-style-tool-copy:{}", copy_control.id.as_str()).into(),
                    ))
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(div().text_xs().child(copy_control.label))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(chrome.text_secondary_hex))
                                            .child(copy_control.section),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child(format!("base {base}")),
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
                                        format!(
                                            "input:dev-style-tool-copy:{}",
                                            copy_control.id.as_str()
                                        )
                                        .into(),
                                    ))
                                    .flex_1()
                                    .child(Input::new(&control.input).small()),
                            )
                            .child(
                                div()
                                    .id(ElementId::Name(
                                        format!(
                                            "button:dev-style-tool-copy-reset:{}",
                                            copy_control.id.as_str()
                                        )
                                        .into(),
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
                                        this.reset_copy_control(control_id, window, cx);
                                    }))
                                    .child("Reset"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(format!("effective {effective}")),
                    ),
            );
        }
        column
    }

    fn render_actions_popup_controls(
        &self,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> Div {
        let mut column = div().flex().flex_col().gap(px(10.0)).flex_1();
        for group in ACTIONS_POPUP_KNOB_GROUPS {
            let controls: Vec<&ActionsPopupControlState> = self
                .actions_popup_controls
                .iter()
                .filter(|control| {
                    actions_popup_knob_by_id(control.knob_id)
                        .is_some_and(|knob| knob.group == *group)
                })
                .collect();
            if controls.is_empty() {
                continue;
            }
            column = column.child(self.render_actions_popup_group(*group, controls, chrome, cx));
        }
        column
    }

    fn render_actions_popup_group(
        &self,
        group: ActionsPopupKnobGroup,
        controls: Vec<&ActionsPopupControlState>,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(ElementId::Name(
                format!("actions-style-section:{}", actions_popup_group_slug(group)).into(),
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
            .children(
                controls
                    .into_iter()
                    .map(|control| self.render_actions_popup_control(control, chrome, cx)),
            )
    }

    fn render_actions_popup_control(
        &self,
        control: &ActionsPopupControlState,
        chrome: theme::AppChromeColors,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let knob =
            actions_popup_knob_by_id(control.knob_id).expect("actions control must reference knob");
        let base = actions_popup_knob_base_value(knob);
        let effective = current_actions_popup_knob_value(knob.id);
        let knob_id = knob.id;

        div()
            .id(ElementId::Name(
                format!("control:dev-style-tool-actions:{}", knob.id.as_str()).into(),
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
                                format!("slider:dev-style-tool-actions:{}", knob.id.as_str())
                                    .into(),
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
                                format!("input:dev-style-tool-actions:{}", knob.id.as_str()).into(),
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
                                format!("button:dev-style-tool-actions-reset:{}", knob.id.as_str())
                                    .into(),
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
                                this.reset_actions_popup_knob(knob_id, window, cx);
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
                                        STYLE_KNOBS
                                            .len()
                                            .saturating_add(COPY_CONTROLS.len())
                                            .saturating_add(ACTIONS_POPUP_KNOBS.len())
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
                                    |this, _event, window, cx| {
                                        this.save_current_settings(window, cx);
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
            .when_some(self.save_path.as_ref(), |view, _| {
                view.child(self.render_saved_markdown(chrome, cx))
            })
            .child(self.render_primary_tabs(chrome, cx))
            .when(
                self.active_tab == DevStyleToolTab::MainWindowStyling,
                |view| view.child(self.render_group_tabs(chrome, cx)),
            )
            .child(
                div()
                    .id("body:dev-style-tool-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .gap(px(10.0))
                    .pr(px(4.0))
                    .overflow_y_scroll()
                    .child(match self.active_tab {
                        DevStyleToolTab::MainWindowStyling => {
                            self.render_groups(&[self.active_group], chrome, cx)
                        }
                        DevStyleToolTab::TextCopy => self.render_copy_controls(chrome, cx),
                        DevStyleToolTab::ActionsPopupStyling => {
                            self.render_actions_popup_controls(chrome, cx)
                        }
                    }),
            )
    }
}

const STYLE_KNOB_GROUPS: &[StyleKnobGroup] = &[
    StyleKnobGroup::Shell,
    StyleKnobGroup::Search,
    StyleKnobGroup::List,
    StyleKnobGroup::Row,
    StyleKnobGroup::Icon,
    StyleKnobGroup::Metadata,
    StyleKnobGroup::Typography,
    StyleKnobGroup::Footer,
    StyleKnobGroup::HeaderInfoBar,
];

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

fn current_actions_popup_knob_value(knob_id: ActionsPopupKnobId) -> f32 {
    match runtime_overrides::current_actions_popup_value(knob_id).unwrap_or_else(|| {
        let knob = actions_popup_knob_by_id(knob_id).expect("actions popup style knob must exist");
        (knob.get)(&crate::designs::base_actions_popup_theme())
    }) {
        StyleValue::Number(value) => value,
    }
}

fn actions_popup_knob_base_value(knob: &ActionsPopupKnob) -> f32 {
    match (knob.get)(&crate::designs::base_actions_popup_theme()) {
        StyleValue::Number(value) => value,
    }
}

fn parse_style_value(value: &str, unit: StyleUnit) -> Result<f32, std::num::ParseFloatError> {
    let trimmed = value.trim();
    let trimmed = match unit {
        StyleUnit::Px => trimmed.trim_end_matches("px").trim(),
        StyleUnit::Alpha => trimmed.trim_end_matches("alpha").trim(),
        StyleUnit::Opacity => trimmed.trim_end_matches('%').trim(),
        StyleUnit::Weight => trimmed.trim_end_matches("weight").trim(),
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
        StyleUnit::Weight => format!("{value:.0}"),
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

const ACTIONS_POPUP_KNOB_GROUPS: &[ActionsPopupKnobGroup] = &[
    ActionsPopupKnobGroup::Shell,
    ActionsPopupKnobGroup::Search,
    ActionsPopupKnobGroup::List,
    ActionsPopupKnobGroup::Row,
    ActionsPopupKnobGroup::Section,
    ActionsPopupKnobGroup::ContextHeader,
    ActionsPopupKnobGroup::Shortcut,
];

fn actions_popup_group_slug(group: ActionsPopupKnobGroup) -> &'static str {
    match group {
        ActionsPopupKnobGroup::Shell => "shell",
        ActionsPopupKnobGroup::Search => "search",
        ActionsPopupKnobGroup::List => "list",
        ActionsPopupKnobGroup::Row => "row",
        ActionsPopupKnobGroup::Section => "section",
        ActionsPopupKnobGroup::ContextHeader => "context-header",
        ActionsPopupKnobGroup::Shortcut => "shortcut",
    }
}
