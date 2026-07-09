// Flow UX exploration surfaces (Flash / Dispatch / Lens).
//
// One renderer, parameterized by `crate::flows::model::FlowUxVariant`, over
// the shared flow substrate (`crate::flows`): the mdflow CLI owns discovery
// and execution (docs/ai/flow-ux-protocol.md); these surfaces are thin,
// swappable views so John can compare interaction grammars by feel.
//
// Shared grammar (protocol §5): Enter = variant's primary lifecycle,
// Shift+Enter = background, Cmd+Enter = launch + focus Flow Manager,
// Esc backgrounds Flash's engaged run (never cancels), ⌥←/⌥→ cycles
// variants in place.

impl ScriptListApp {
    /// Effective cwd for flow discovery: the spine cwd chip when set,
    /// otherwise $HOME. mdflow resolves project vs global flows from here.
    pub(crate) fn flow_ux_cwd(&self) -> String {
        // Test seam: probes point flow discovery at a fixture project
        // without touching spine state (same pattern as
        // SCRIPT_KIT_TEST_BRAIN_DB_PATH).
        if let Ok(dir) = std::env::var("SCRIPT_KIT_FLOW_UX_CWD") {
            if !dir.is_empty() {
                return dir;
            }
        }
        if let Some(cwd) = &self.spine_cwd {
            return cwd.to_string_lossy().to_string();
        }
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    }

    /// Spawn the repaint tick that keeps Flow UX surfaces live while runs
    /// stream. Single instance; exits when no Flow UX view is active and no
    /// run is still executing.
    pub(crate) fn start_flow_ux_tick(&mut self, cx: &mut Context<Self>) {
        if self.flow_ux_tick_running {
            return;
        }
        self.flow_ux_tick_running = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(120))
                    .await;
                let keep_going = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        let registry = crate::flows::run_registry::flow_run_registry();
                        let generation = registry.generation();
                        if generation != app.flow_ux_seen_generation {
                            app.flow_ux_seen_generation = generation;
                            cx.notify();
                        }
                        let view_active =
                            matches!(app.current_view, AppView::FlowUxView { .. });
                        let runs_active = registry.active_count() > 0;
                        let keep = view_active || runs_active;
                        if !keep {
                            app.flow_ux_tick_running = false;
                        }
                        keep
                    })
                });
                match keep_going {
                    Ok(true) => continue,
                    _ => break,
                }
            }
        })
        .detach();
    }

    /// Launch a flow through the shared runner and record the ack. Returns
    /// the registry-local run id.
    fn flow_ux_launch(
        &mut self,
        flow: &crate::flows::model::FlowDescriptor,
        variant: crate::flows::model::FlowUxVariant,
        engagement: crate::flows::model::EngagementMode,
        cx: &mut Context<Self>,
    ) -> u64 {
        let cwd = self.flow_ux_cwd();
        crate::flows::manager_window::remember_flow_cwd(&cwd);
        let run_id = crate::flows::runner::launch_flow(
            &flow.id,
            &flow.name,
            &flow.path,
            &cwd,
            variant,
            engagement,
            Vec::new(),
            std::time::Instant::now(),
        );
        self.start_flow_ux_tick(cx);
        cx.notify();
        run_id
    }

    fn flow_ux_selected_flow(
        filter: &str,
        selected_index: usize,
        roster: &crate::flows::catalog::RosterEntry,
    ) -> Option<crate::flows::model::FlowDescriptor> {
        crate::flows::catalog::filter_flows(&roster.flows, filter)
            .get(selected_index)
            .map(|flow| (*flow).clone())
    }

    /// Cycle ⌥←/⌥→ between the main-window variants (Mission Control lives
    /// in the detached manager, so the ring here is Flash→Dispatch→Lens).
    fn flow_ux_cycle_variant(
        current: crate::flows::model::FlowUxVariant,
        forward: bool,
    ) -> crate::flows::model::FlowUxVariant {
        use crate::flows::model::FlowUxVariant;
        let mut next = if forward { current.next() } else { current.prev() };
        if next == FlowUxVariant::MissionControl {
            next = if forward { next.next() } else { next.prev() };
        }
        next
    }

    fn render_flow_ux(
        &mut self,
        variant: crate::flows::model::FlowUxVariant,
        filter: String,
        selected_index: usize,
        inline_run: Option<u64>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        use crate::flows::model::{EngagementMode, FlowUxVariant};

        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let list_colors = crate::list_item::ListItemColors::from_theme(&self.theme);
        let cwd = self.flow_ux_cwd();
        let roster = crate::flows::catalog::flow_catalog().roster_for(&cwd);
        let filtered: Vec<crate::flows::model::FlowDescriptor> =
            crate::flows::catalog::filter_flows(&roster.flows, &filter)
                .into_iter()
                .cloned()
                .collect();
        let filtered_len = filtered.len();
        let registry = crate::flows::run_registry::flow_run_registry();

        // ------------------------------------------------------------------
        // Key handler (shared grammar, protocol §5)
        // ------------------------------------------------------------------
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);
                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;
                let has_alt = event.keystroke.modifiers.alt;

                let view_state = if let AppView::FlowUxView {
                    variant,
                    filter,
                    selected_index,
                    inline_run,
                } = &this.current_view
                {
                    Some((*variant, filter.clone(), *selected_index, *inline_run))
                } else {
                    None
                };
                let Some((variant, current_filter, current_selected, inline_run)) = view_state
                else {
                    return;
                };

                // Esc backgrounds Flash's engaged run; never cancels it.
                if is_key_escape(key) && !this.show_actions_popup {
                    if let Some(run_id) = inline_run {
                        crate::flows::run_registry::flow_run_registry()
                            .set_engagement(run_id, EngagementMode::Background);
                        if let AppView::FlowUxView { inline_run, .. } = &mut this.current_view {
                            *inline_run = None;
                        }
                        cx.notify();
                        cx.stop_propagation();
                        return;
                    }
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                // ⌥←/⌥→ cycles Flash → Dispatch → Lens in place.
                if has_alt && (key == "left" || key == "right") {
                    let next = Self::flow_ux_cycle_variant(variant, key == "right");
                    if let AppView::FlowUxView { variant, .. } = &mut this.current_view {
                        *variant = next;
                    }
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }

                let cwd = this.flow_ux_cwd();
                let roster = crate::flows::catalog::flow_catalog().roster_for(&cwd);
                let current_len =
                    crate::flows::catalog::filter_flows(&roster.flows, &current_filter).len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::FlowUxView { selected_index, .. } = &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.flow_ux_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }
                if is_key_down(key) {
                    if current_selected < current_len.saturating_sub(1) {
                        if let AppView::FlowUxView { selected_index, .. } = &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.flow_ux_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                if is_key_enter(key) {
                    let Some(flow) =
                        Self::flow_ux_selected_flow(&current_filter, current_selected, &roster)
                    else {
                        cx.stop_propagation();
                        return;
                    };

                    // Cmd+Enter: launch and focus the Flow Manager.
                    if has_cmd {
                        let run_id = this.flow_ux_launch(
                            &flow,
                            variant,
                            EngagementMode::ManagerFocused,
                            cx,
                        );
                        crate::flows::run_registry::flow_run_registry().select(run_id);
                        cx.defer(move |cx| {
                            let _ = crate::flows::manager_window::open_flow_manager_window(cx);
                        });
                        cx.stop_propagation();
                        return;
                    }

                    // Shift+Enter: background launch, stay put.
                    if has_shift {
                        this.flow_ux_launch(&flow, variant, EngagementMode::Background, cx);
                        this.toast_manager.push(
                            crate::components::toast::Toast::success(
                                format!("{} running in background", flow.name),
                                &this.theme,
                            )
                            .duration_ms(Some(1800)),
                        );
                        cx.stop_propagation();
                        return;
                    }

                    match variant {
                        FlowUxVariant::Flash => {
                            let run_id = this.flow_ux_launch(
                                &flow,
                                variant,
                                EngagementMode::Inline,
                                cx,
                            );
                            if let AppView::FlowUxView { inline_run, .. } =
                                &mut this.current_view
                            {
                                *inline_run = Some(run_id);
                            }
                        }
                        FlowUxVariant::Dispatch => {
                            this.flow_ux_launch(&flow, variant, EngagementMode::Background, cx);
                            this.toast_manager.push(
                                crate::components::toast::Toast::success(
                                    format!("{} dispatched — ⌘↵ opens the manager", flow.name),
                                    &this.theme,
                                )
                                .duration_ms(Some(2200)),
                            );
                        }
                        FlowUxVariant::Lens => {
                            let run_id = this.flow_ux_launch(
                                &flow,
                                variant,
                                EngagementMode::ManagerFocused,
                                cx,
                            );
                            crate::flows::run_registry::flow_run_registry().select(run_id);
                            cx.defer(move |cx| {
                                let _ =
                                    crate::flows::manager_window::open_flow_manager_window(cx);
                            });
                        }
                        FlowUxVariant::MissionControl => {}
                    }
                    cx.notify();
                    cx.stop_propagation();
                }
            },
        );

        // ------------------------------------------------------------------
        // List element
        // ------------------------------------------------------------------
        let empty_message = match roster.status {
            crate::flows::catalog::RosterStatus::Loading => "Loading flows…",
            crate::flows::catalog::RosterStatus::Legacy => {
                "mdflow is pre-protocol — upgrade with: npm i -g mdflow"
            }
            crate::flows::catalog::RosterStatus::Error => "Flow roster unavailable",
            crate::flows::catalog::RosterStatus::Ready => {
                if filter.is_empty() {
                    "No flows in this project — create one with: md create"
                } else {
                    "No flows match your filter"
                }
            }
        };

        let list_element: gpui::AnyElement = if filtered_len == 0 {
            let color_resolver =
                crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
            let typography_resolver = crate::theme::TypographyResolver::new_theme_first(
                &self.theme,
                self.current_design,
            );
            let empty_font_family = typography_resolver.primary_font().to_string();
            crate::list_item::EmptyState::new(
                empty_message,
                color_resolver.empty_text_color(),
                &empty_font_family,
            )
            .icon(crate::designs::icon_variations::IconName::Terminal)
            .into_element()
        } else {
            let rows = filtered.clone();
            let hovered = self.hovered_index;
            uniform_list("flow-ux-list", filtered_len, move |visible_range, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let flow = &rows[ix];
                        let is_selected = ix == selected_index;
                        let is_hovered = hovered == Some(ix);
                        let source_label = flow.source.label();
                        let description = match &flow.description {
                            Some(description) => {
                                format!("{description} · {} · {source_label}", flow.engine)
                            }
                            None => format!("{} · {source_label}", flow.engine),
                        };
                        let icon = if flow.is_workflow { "🧩" } else { "⚡" };
                        div().id(ix).cursor_pointer().child(
                            ListItem::new(flow.name.clone(), list_colors)
                                .description_opt(Some(description))
                                .icon(icon)
                                .selected(is_selected)
                                .hovered(is_hovered)
                                .with_accent_bar(true),
                        )
                    })
                    .collect()
            })
            .h_full()
            .track_scroll(&self.flow_ux_scroll_handle)
            .into_any_element()
        };

        let list_scrollbar =
            self.builtin_uniform_list_scrollbar(&self.flow_ux_scroll_handle, filtered_len, 8);
        let list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .child(list_element)
            .child(list_scrollbar)
            .into_any_element();

        // ------------------------------------------------------------------
        // Variant-specific main content
        // ------------------------------------------------------------------
        let main: gpui::AnyElement = match variant {
            FlowUxVariant::Flash => {
                if let Some(run) = inline_run.and_then(|id| registry.get(id)) {
                    self.render_flow_ux_inline_run(&run, &chrome)
                } else {
                    list_pane
                }
            }
            FlowUxVariant::Dispatch => list_pane,
            FlowUxVariant::Lens => {
                let preview = self.render_flow_ux_lens_preview(
                    Self::flow_ux_selected_flow(&filter, selected_index, &roster),
                    &cwd,
                    &chrome,
                );
                self.render_builtin_split_main_content(list_pane, preview)
            }
            FlowUxVariant::MissionControl => list_pane,
        };

        // ------------------------------------------------------------------
        // Footer + shell
        // ------------------------------------------------------------------
        let active_runs = registry.active_count();
        let hints: Vec<gpui::SharedString> = match variant {
            FlowUxVariant::Flash if inline_run.is_some() => vec![
                gpui::SharedString::from("Esc Background"),
                gpui::SharedString::from("⌘↵ Manager"),
            ],
            FlowUxVariant::Flash => vec![
                gpui::SharedString::from("↵ Run"),
                gpui::SharedString::from("⇧↵ Background"),
                gpui::SharedString::from("⌘↵ Manager"),
                gpui::SharedString::from("Esc Back"),
            ],
            FlowUxVariant::Dispatch => vec![
                gpui::SharedString::from("↵ Dispatch"),
                gpui::SharedString::from("⌘↵ Manager"),
                gpui::SharedString::from("Esc Back"),
            ],
            _ => vec![
                gpui::SharedString::from("↵ Run + Manager"),
                gpui::SharedString::from("⇧↵ Background"),
                gpui::SharedString::from("Esc Back"),
            ],
        };
        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            hints, None,
        ));

        let count_label = if active_runs > 0 {
            format!("{filtered_len} flows · {active_runs} running")
        } else {
            format!("{filtered_len} flows")
        };
        let variant_chip = div()
            .flex_none()
            .whitespace_nowrap()
            .text_sm()
            .text_color(rgb(chrome.accent_hex))
            .child(format!("{} · ⌥←→", variant.display_name()))
            .into_any_element();
        let trailing = vec![
            variant_chip,
            self.render_builtin_main_input_count_label(count_label),
        ];

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;
        crate::components::main_view_chrome::render_main_view_chrome_footer_flush(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(rgb(chrome.text_primary_hex))
                .font_family(self.theme_font_family())
                .key_context("flow_ux")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(trailing, cx),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main,
                footer,
                overlays: Vec::new(),
            },
        )
    }

    /// Flash's engaged run pane: live output where the list was.
    fn render_flow_ux_inline_run(
        &self,
        run: &crate::flows::run_registry::FlowRun,
        chrome: &crate::theme::AppChromeColors,
    ) -> gpui::AnyElement {
        let mut lines: Vec<String> = run.stdout_tail.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines = run.stderr_tail.lines().map(str::to_string).collect();
        }
        let shown: Vec<String> = lines.iter().rev().take(18).rev().cloned().collect();
        let status_line = format!(
            "{} — {} · {}ms",
            run.flow_name,
            run.display_status(),
            run.elapsed_ms()
        );
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_hidden()
            .p_3()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(chrome.text_primary_hex))
                    .child(status_line),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .p_2()
                    .rounded_md()
                    .bg(rgba(chrome.preview_surface_rgba))
                    .overflow_hidden()
                    .children(shown.into_iter().map(|line| {
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(line)
                    })),
            )
            .into_any_element()
    }

    /// Lens preview: FREE resolved-command view via `md explain --json`.
    fn render_flow_ux_lens_preview(
        &self,
        flow: Option<crate::flows::model::FlowDescriptor>,
        cwd: &str,
        chrome: &crate::theme::AppChromeColors,
    ) -> gpui::AnyElement {
        let container = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .h_full()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(rgba(chrome.preview_surface_rgba));

        let Some(flow) = flow else {
            return container
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(chrome.text_muted_hex))
                        .child("Select a flow to preview it."),
                )
                .into_any_element();
        };

        match crate::flows::explain_cache::explain_cache().state_for(&flow, cwd) {
            crate::flows::explain_cache::ExplainState::Loading => container
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(chrome.text_muted_hex))
                        .child(format!("Resolving {}…", flow.name)),
                )
                .into_any_element(),
            crate::flows::explain_cache::ExplainState::Failed(message) => container
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(chrome.text_muted_hex))
                        .child(format!("Preview unavailable: {message}")),
                )
                .into_any_element(),
            crate::flows::explain_cache::ExplainState::Ready(info) => {
                let command_line = format!("{} {}", info.command, info.args.join(" "));
                let prompt_lines: Vec<String> = info
                    .prompt
                    .lines()
                    .take(14)
                    .map(str::to_string)
                    .collect();
                let inputs_line = if info.inputs.is_empty() {
                    None
                } else {
                    Some(format!(
                        "Inputs: {}",
                        info.inputs
                            .iter()
                            .map(|input| {
                                // Password inputs surface by name only —
                                // values never render anywhere (protocol §6).
                                format!("{} ({:?})", input.name, input.input_type)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                };
                container
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(chrome.text_primary_hex))
                            .child(format!(
                                "{} · {} · ~{} tokens",
                                info.engine, flow.name, info.prompt_tokens_estimate
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.accent_hex))
                            .child(command_line),
                    )
                    .children(
                        info.warnings
                            .iter()
                            .map(|warning| {
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_muted_hex))
                                    .child(format!("⚠ {warning}"))
                            })
                            .collect::<Vec<_>>(),
                    )
                    .children(inputs_line.map(|line| {
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_secondary_hex))
                            .child(line)
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h(px(0.))
                            .mt_1()
                            .p_2()
                            .rounded_md()
                            .bg(rgba(chrome.panel_surface_rgba))
                            .overflow_hidden()
                            .children(prompt_lines.into_iter().map(|line| {
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_secondary_hex))
                                    .child(line)
                            })),
                    )
                    .into_any_element()
            }
        }
    }

    /// `flowUx` automation snapshot for getState (protocol §6).
    pub(crate) fn flow_ux_automation_snapshot(&self) -> serde_json::Value {
        let (active_variant, selected_flow_id, preview) = match &self.current_view {
            AppView::FlowUxView {
                variant,
                filter,
                selected_index,
                ..
            } => {
                let cwd = self.flow_ux_cwd();
                let roster = crate::flows::catalog::flow_catalog().roster_for(&cwd);
                let selected = Self::flow_ux_selected_flow(filter, *selected_index, &roster);
                (
                    Some(*variant),
                    selected.as_ref().map(|flow| flow.id.clone()),
                    selected.map(|flow| (flow.id.clone(), *variant)),
                )
            }
            _ => (None, None, None),
        };
        let cwd = self.flow_ux_cwd();
        let roster_entry = crate::flows::catalog::flow_catalog().roster_for(&cwd);
        let (manager_visible, manager_focused) =
            crate::flows::manager_window::manager_automation_state();
        crate::flows::automation::flow_ux_state(crate::flows::automation::FlowUxSnapshotInputs {
            active_variant,
            selected_flow_id: selected_flow_id.as_deref(),
            roster: Some((&roster_entry, cwd.as_str())),
            preview: preview.as_ref().map(|(flow_id, _)| {
                crate::flows::automation::PreviewSnapshot {
                    flow_id,
                    fingerprint: None,
                    valid: true,
                }
            }),
            manager_visible,
            manager_focused_run_id: manager_focused,
        })
    }
}
