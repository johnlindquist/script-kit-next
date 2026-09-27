//! Draws the work views in place of the list, under the same input, and the strip of agents above
//! the capsules. What each view says comes from the reducer; its header and rows are the main
//! list's own, so a view reads like the list it replaced.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, SharedString};
use sk_one_window::chat::{self, Phase as ChatPhase, Who};
use sk_one_window::connect::{self, Phase as ConnectPhase};
use sk_one_window::steps::{self, Phase as StepsPhase, StepState};
use sk_one_window::strip::{self, TaskState};

use crate::list_item::{render_section_header, IconKind, ListItem, ListItemColors};
use crate::ScriptListApp;

const STRIP_ID: &str = "one-window-strip";

impl ScriptListApp {
    /// The open work view, drawn where the list goes, or `None` when the list shows.
    pub(crate) fn render_one_window_work_view(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let top = self.one_window.views().top()?.name.clone();
        if !super::work::WORK_VIEWS.contains(&top.as_str()) {
            return None;
        }
        let def = self.current_main_menu_theme.def();
        let colors = ListItemColors::from_theme(&self.theme);
        let head_h = crate::list_item::effective_first_section_header_height_for_theme(
            self.current_main_menu_theme,
        );
        let top_inset = crate::components::main_view_chrome::main_view_header_metrics(
            def,
            Some(def.search.height),
        )
        .header_height;
        let mut rows: Vec<AnyElement> = vec![div()
            .debug_selector(|| "oneWindowSectionHeader:0".to_string())
            .h(px(head_h))
            .child(render_section_header(
                &self.one_window.views().crumb(),
                None,
                colors,
                true,
            ))
            .into_any_element()];
        match top.as_str() {
            chat::VIEW => self.one_window_chat_rows(&mut rows, colors, head_h),
            steps::VIEW => self.one_window_steps_rows(&mut rows, colors),
            connect::VIEW => self.one_window_connect_rows(&mut rows, colors, head_h),
            _ => self.one_window_agent_rows(&mut rows, colors),
        }
        let _ = cx;
        Some(
            div()
                .id("one-window-work-view")
                .w_full()
                .h_full()
                .min_h(px(0.))
                .pt(px(top_inset))
                .pb(px(
                    crate::footer_popup::main_window_footer_reservation_height(),
                ))
                .flex()
                .flex_col()
                .overflow_hidden()
                .children(rows)
                .into_any_element(),
        )
    }

    fn one_window_row(
        &self,
        id: String,
        title: impl Into<SharedString>,
        description: Option<String>,
        icon: &str,
        selected: bool,
        colors: ListItemColors,
    ) -> AnyElement {
        let height =
            crate::list_item::effective_list_item_height_for_theme(self.current_main_menu_theme);
        let selector = format!("list-row:{id}");
        div()
            .debug_selector(move || selector.clone())
            .h(px(height))
            .flex_none()
            .child(
                ListItem::new(title, colors)
                    .semantic_id(id)
                    .icon_kind(IconKind::Svg(icon.to_string()))
                    .description_opt(description)
                    .selected(selected),
            )
            .into_any_element()
    }

    fn one_window_note(&self, text: impl Into<SharedString>) -> AnyElement {
        let def = self.current_main_menu_theme.def();
        div()
            .flex_none()
            .px(px(def.shell.header_padding_x + def.search.text_inset_x))
            .py(px(def.slot_line.gap_px))
            .text_size(px(def.slot_line.title_font_size))
            .text_color(rgb(self.theme.colors.text.muted))
            .child(text.into())
            .into_any_element()
    }

    fn one_window_chat_rows(
        &self,
        rows: &mut Vec<AnyElement>,
        colors: ListItemColors,
        head_h: f32,
    ) {
        let Some(c) = self.one_window.chat() else {
            return;
        };
        let transcript = self
            .one_window_host
            .work
            .chat
            .as_ref()
            .map(|m| m.transcript.clone());
        let body = match transcript {
            Some(transcript) => div()
                .flex_1()
                .min_h(px(0.))
                .child(transcript)
                .into_any_element(),
            None => div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .children(c.turns.iter().map(|t| {
                    self.one_window_note(match t.who {
                        Who::You => format!("You: {}", t.text),
                        Who::Model => t.text.clone(),
                    })
                }))
                .into_any_element(),
        };
        rows.push(body);
        if c.reply().is_some() {
            rows.push(self.one_window_note(chat::WRITTEN_BY));
        }
        match &c.phase {
            ChatPhase::Ready => {
                rows.push(
                    div()
                        .debug_selector(|| "oneWindowSectionHeader:1".to_string())
                        .h(px(head_h))
                        .child(render_section_header(chat::NEXT_HEAD, None, colors, false))
                        .into_any_element(),
                );
                for (i, row) in c.next.iter().enumerate() {
                    rows.push(self.one_window_row(
                        format!("chat-next:{}", row.key),
                        row.title.clone(),
                        None,
                        row.icon.as_deref().unwrap_or("sparkles"),
                        i == c.sel,
                        colors,
                    ));
                }
            }
            ChatPhase::Failed(message) => rows.push(self.one_window_note(message.clone())),
            _ => {}
        }
    }

    fn one_window_steps_rows(&self, rows: &mut Vec<AnyElement>, colors: ListItemColors) {
        let Some(s) = self.one_window.steps_view() else {
            return;
        };
        let conns = self.one_window.work().connections();
        for (i, step) in s.steps.iter().enumerate() {
            let (icon, status) = match (step.state, step.waits_on(conns)) {
                (StepState::Done, _) => ("check", "Done".to_string()),
                (_, Some(key)) => (
                    "plus",
                    format!("Needs {} · ↵ connects it", connect::name_of(key)),
                ),
                (StepState::Reading, _) => ("sparkles", "Reading…".to_string()),
                _ if step.without => ("copy", "Copies a message instead".to_string()),
                _ => ("sparkles", format!("Script Kit · “{}”", step.text)),
            };
            let title = format!("{}. {}", i + 1, step.title.as_deref().unwrap_or(&step.text));
            rows.push(self.one_window_row(
                format!("step:{i}"),
                title,
                Some(status),
                icon,
                s.sel == Some(i) && s.phase != StepsPhase::Running,
                colors,
            ));
        }
    }

    fn one_window_connect_rows(
        &self,
        rows: &mut Vec<AnyElement>,
        colors: ListItemColors,
        head_h: f32,
    ) {
        let Some(c) = self.one_window.connecting() else {
            return;
        };
        let Some(info) = connect::integration(&c.integration) else {
            return;
        };
        rows.push(self.one_window_row(
            format!("connect:{}", c.integration),
            format!("This needs {}", info.name),
            Some(format!("{}. Connect once and it stays connected.", c.does)),
            "plus",
            true,
            colors,
        ));
        rows.push(self.one_window_row(
            "connect:waiting".to_string(),
            format!("“{}”", c.waiting),
            Some("Waiting to run · runs as soon as you're connected, no retyping".to_string()),
            "sparkles",
            false,
            colors,
        ));
        let status = match (c.phase, &c.error) {
            (ConnectPhase::Connecting, _) => Some(format!("Signing in to {}…", info.name)),
            (ConnectPhase::Ask, Some(error)) => Some(error.clone()),
            (ConnectPhase::Ask, None) => None,
        };
        if let Some(status) = status {
            rows.push(self.one_window_note(status));
        }
        rows.push(
            div()
                .debug_selector(|| "oneWindowSectionHeader:1".to_string())
                .h(px(head_h))
                .child(render_section_header(
                    "Also works once connected",
                    None,
                    colors,
                    false,
                ))
                .into_any_element(),
        );
        for (i, can) in info.can.iter().enumerate() {
            rows.push(self.one_window_row(
                format!("connect-can:{i}"),
                *can,
                None,
                "check",
                false,
                colors,
            ));
        }
        rows.push(self.one_window_note("⌘3 copy a message instead"));
    }

    fn one_window_agent_rows(&self, rows: &mut Vec<AnyElement>, colors: ListItemColors) {
        let Some(task) = self.one_window.open_task() else {
            return;
        };
        rows.push(self.one_window_row(
            format!("agent:{}", task.id),
            task.label.clone(),
            Some(task.status.clone()),
            "terminal",
            true,
            colors,
        ));
        if let Some(model) = self.one_window_host.work.agents.get(&task.id) {
            rows.push(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .child(model.transcript.clone())
                    .into_any_element(),
            );
        }
        rows.push(self.one_window_note("Esc keeps it working in the strip · ⌘J brings it back"));
    }

    /// The list or view with the strip of agents floating above the capsules. The strip never
    /// takes room from the list, so a task starting never moves a row.
    pub(crate) fn with_one_window_strip(&self, content: AnyElement) -> AnyElement {
        let tasks = self.one_window.work().strip().tasks();
        if tasks.is_empty() || !self.one_window_owns_list() {
            return content;
        }
        let def = self.current_main_menu_theme.def();
        let tokens = def.slot_line;
        let colors = &self.theme.colors;
        let target = self
            .one_window
            .work()
            .strip()
            .target()
            .map(|t| t.id.clone());
        let edge = def.header_info_bar.context_edge_outset_x + def.shell.header_padding_x;
        let strip = div()
            .id(STRIP_ID)
            .debug_selector(|| STRIP_ID.to_string())
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(crate::footer_popup::main_window_footer_reservation_height()))
            .flex()
            .flex_col()
            .children(tasks.iter().map(|t| {
                let needs_you = t.state == TaskState::NeedsYou;
                let accent = if needs_you {
                    colors.accent.selected
                } else {
                    colors.text.muted
                };
                div()
                    .relative()
                    .h(px(tokens.height_px))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(tokens.gap_px))
                    .pl(px(edge + def.search.text_inset_x))
                    .pr(px(edge + def.search.text_inset_x * 0.5))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .when(needs_you, |row| {
                        row.bg(rgba((colors.accent.selected << 8) | tokens.holding_tint_alpha))
                    })
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(tokens.title_font_size))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text.primary))
                            .child(t.label.clone()),
                    )
                    .child(
                        div()
                            .min_w(px(0.))
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(tokens.title_font_size))
                            .text_color(rgb(accent))
                            .child(t.status.clone()),
                    )
                    .child(div().flex_1())
                    .when(target.as_deref() == Some(t.id.as_str()), |row| {
                        row.child(
                            crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens(
                                ["⌘", "J"],
                                &self.theme,
                                (colors.text.muted << 8) | 0xFF,
                            ),
                        )
                    })
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .bottom_0()
                            .h(px(tokens.progress_height_px))
                            .w(gpui::relative(t.progress.clamp(0.0, 1.0)))
                            .bg(rgb(accent)),
                    )
            }));
        div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .child(content)
            .child(strip)
            .into_any_element()
    }
}

/// What the strip shows for a task, for automation.
pub(crate) fn strip_snapshot(app: &ScriptListApp) -> serde_json::Value {
    let strip = app.one_window.work().strip();
    serde_json::json!({
        "tasks": strip.tasks().iter().map(|t| serde_json::json!({
            "id": t.id,
            "label": t.label,
            "status": t.status,
            "state": format!("{:?}", t.state),
            "view": t.view.name == strip::AGENT_VIEW,
        })).collect::<Vec<_>>(),
        "cmdJ": strip.target().map(|t| t.id.clone()),
    })
}
