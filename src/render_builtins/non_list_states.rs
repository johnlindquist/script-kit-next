struct NonListShowcaseState {
    key: &'static str,
    title: &'static str,
    body: &'static str,
    glyph: &'static str,
    hints: &'static [(&'static str, &'static str)],
}

const NON_LIST_SHOWCASE_STATES: &[NonListShowcaseState] = &[
    NonListShowcaseState {
        key: "empty",
        title: "Nothing here yet",
        body: "Use the primary action to create the first item, or start typing to search the rest of Script Kit.",
        glyph: "+",
        hints: &[("Return", "Create first item"), ("Type", "Search commands"), ("Esc", "Back to menu")],
    },
    NonListShowcaseState {
        key: "help",
        title: "Ask anything",
        body: "A help state should show the user what this surface understands before they have to guess.",
        glyph: "?",
        hints: &[("/", "Browse skills"), ("Shift Return", "New line"), ("Cmd K", "Actions")],
    },
    NonListShowcaseState {
        key: "form",
        title: "Capture a task",
        body: "Forms need visible field intent, predictable movement, and a clear submit path.",
        glyph: ";",
        hints: &[("Tab", "Next field"), ("Shift Tab", "Previous field"), ("Cmd Return", "Submit")],
    },
    NonListShowcaseState {
        key: "setup",
        title: "Finish setup",
        body: "Setup screens should make readiness legible: what is done, what is blocked, and the next repair action.",
        glyph: "!",
        hints: &[("Model", "Ready"), ("Auth", "Needs sign in"), ("Return", "Continue")],
    },
    NonListShowcaseState {
        key: "permission",
        title: "Allow access",
        body: "Permission states need plain scope, low drama, and obvious accept or decline choices.",
        glyph: "*",
        hints: &[("Scope", "Selected app only"), ("Return", "Allow"), ("Esc", "Not now")],
    },
    NonListShowcaseState {
        key: "recovery",
        title: "Something failed",
        body: "Recovery states should name the failure and put the likely repair before logs or diagnostics.",
        glyph: "!",
        hints: &[("Return", "Try again"), ("Cmd L", "Open logs"), ("Esc", "Dismiss")],
    },
    NonListShowcaseState {
        key: "about",
        title: "Script Kit",
        body: "Product identity states can be quieter, but still need useful next actions and version context.",
        glyph: "SK",
        hints: &[("Version", "0.1.8"), ("Cmd C", "Copy info"), ("Return", "Open docs")],
    },
    NonListShowcaseState {
        key: "density",
        title: "Review the rhythm",
        body: "The same guidance grammar should survive compact launcher states and larger explanatory surfaces.",
        glyph: "<>",
        hints: &[("Left Right", "Switch example"), ("Compact", "Inline help"), ("Comfortable", "Full help")],
    },
];

impl ScriptListApp {
    fn render_non_list_states_showcase(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let palette = crate::components::non_list_palette(&self.theme);
        let metrics =
            crate::components::non_list_metrics(crate::components::NonListDensity::Comfortable);
        let selected_index = self.non_list_showcase_selected_index();
        let state = &NON_LIST_SHOWCASE_STATES[selected_index];

        let content = div()
            .id("non-list-states-main-window-content")
            .size_full()
            .px(px(40.0))
            .py(px(34.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(20.0))
            .child(self.render_non_list_showcase_progress(selected_index, palette))
            .child(
                div()
                    .w_full()
                    .max_w(px(580.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(18.0))
                    .child(
                        div()
                            .id("non-list-states-glyph")
                            .size(px(68.0))
                            .rounded(px(16.0))
                            .border_1()
                            .border_color(palette.border)
                            .bg(palette.input)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(24.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(palette.title)
                            .child(state.glyph),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(7.0))
                            .child(
                                div()
                                    .text_size(px(28.0))
                                    .line_height(px(34.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(palette.title)
                                    .child(state.title),
                            )
                            .child(
                                div()
                                    .max_w(px(500.0))
                                    .text_center()
                                    .text_size(px(metrics.body_size))
                                    .line_height(px(metrics.body_line))
                                    .text_color(palette.body)
                                    .child(state.body),
                            ),
                    )
                    .child(
                        div()
                            .id("non-list-states-hints")
                            .w_full()
                            .max_w(px(420.0))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .children(state.hints.iter().map(|(keys, label)| {
                                self.render_non_list_help_hint(*keys, *label, palette)
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(palette.hint)
                    .child("Left and Right switch examples. Escape returns to the main menu."),
            );

        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            vec!["←/→ Switch".into(), "Esc Back".into()],
            None,
        ));

        div()
            .id("non-list-states-main-window")
            .size_full()
            .bg(palette.surface)
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_hidden()
                    .child(content),
            )
            .when_some(footer, |surface, footer| surface.child(footer))
            .capture_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if crate::ui_foundation::is_key_escape(key) {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                if crate::ui_foundation::is_key_left(key) {
                    this.move_non_list_showcase_selection(-1, cx);
                    cx.stop_propagation();
                    return;
                }

                if crate::ui_foundation::is_key_right(key) {
                    this.move_non_list_showcase_selection(1, cx);
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                cx.propagate();
            }))
            .into_any_element()
    }

    fn render_non_list_showcase_progress(
        &self,
        selected_index: usize,
        palette: crate::components::NonListPalette,
    ) -> gpui::Stateful<Div> {
        div()
            .id("non-list-states-progress")
            .flex()
            .items_center()
            .gap(px(6.0))
            .children(
                NON_LIST_SHOWCASE_STATES
                    .iter()
                    .enumerate()
                    .map(|(index, _state)| {
                        let is_selected = index == selected_index;
                        div()
                            .w(px(if is_selected { 28.0 } else { 7.0 }))
                            .h(px(7.0))
                            .rounded(px(999.0))
                            .bg(if is_selected {
                                palette.selected
                            } else {
                                palette.border
                            })
                    }),
            )
    }

    fn render_non_list_help_hint(
        &self,
        keys: &'static str,
        label: &'static str,
        palette: crate::components::NonListPalette,
    ) -> Div {
        div()
            .w_full()
            .min_h(px(38.0))
            .px(px(11.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(palette.border)
            .bg(palette.panel)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(14.0))
            .child(
                div().flex().items_center().gap(px(5.0)).children(
                    keys.split(' ')
                        .map(|key| self.render_non_list_keycap(key, palette)),
                ),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(palette.body)
                    .child(label),
            )
    }

    fn render_non_list_keycap(
        &self,
        key: &'static str,
        palette: crate::components::NonListPalette,
    ) -> Div {
        div()
            .min_w(px(24.0))
            .h(px(24.0))
            .px(px(7.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(palette.border)
            .bg(palette.input)
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(palette.title)
            .child(key)
    }

    fn non_list_showcase_selected_index(&self) -> usize {
        let selected_index = match &self.current_view {
            AppView::NonListStatesView { selected_index } => *selected_index,
            _ => 0,
        };
        selected_index.min(NON_LIST_SHOWCASE_STATES.len().saturating_sub(1))
    }

    fn move_non_list_showcase_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let len = NON_LIST_SHOWCASE_STATES.len();
        if len == 0 {
            return;
        }
        let current = self.non_list_showcase_selected_index();
        let next = if delta < 0 {
            current.checked_sub(1).unwrap_or(len - 1)
        } else {
            (current + 1) % len
        };
        self.current_view = AppView::NonListStatesView {
            selected_index: next,
        };
        cx.notify();
    }
}
