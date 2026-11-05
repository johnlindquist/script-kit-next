//! Draws the work views in place of the list, under the same input, and the strip of agents above
//! the capsules. What each view says comes from the reducer; its header and rows are the main
//! list's own, so a view reads like the list it replaced.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, SharedString};
use sk_one_window::chat::{self, Phase as ChatPhase, Who};
use sk_one_window::connect::{self, Phase as ConnectPhase};
use sk_one_window::past_chats;
use sk_one_window::steps::{self, Phase as StepsPhase, StepState};
use sk_one_window::strip::{self, TaskState};

use crate::list_item::{render_section_header, IconKind, ListItem, ListItemColors};
use crate::{FocusedInput, ScriptListApp};

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
        let crumb = self.one_window.views().crumb();
        let mut rows: Vec<AnyElement> = vec![self.one_window_header(0, &crumb, colors, head_h)];
        match top.as_str() {
            chat::VIEW => self.one_window_chat_rows(&mut rows, colors, head_h, cx),
            steps::VIEW => self.one_window_steps_rows(&mut rows, colors),
            connect::VIEW => self.one_window_connect_rows(&mut rows, colors, head_h),
            past_chats::VIEW => self.one_window_past_chat_rows(&mut rows, colors, head_h),
            _ => self.one_window_agent_rows(&mut rows, colors),
        }
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

    fn one_window_header(
        &self,
        ix: usize,
        label: &str,
        colors: ListItemColors,
        head_h: f32,
    ) -> AnyElement {
        div()
            .debug_selector(move || format!("oneWindowSectionHeader:{ix}"))
            .flex_none()
            .h(px(head_h))
            .child(render_section_header(label, None, colors, ix == 0))
            .into_any_element()
    }

    /// One turn of the chat: yours on the user-message surface with what went with it, the
    /// model's tagged as its own, and a tool it ran as a row with how it went.
    fn one_window_turn(
        &self,
        ix: usize,
        who: Who,
        text: &str,
        sub: Option<&str>,
        markdown: Option<AnyElement>,
        complete: bool,
    ) -> AnyElement {
        let def = self.current_main_menu_theme.def();
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let colors = &self.theme.colors;
        let edge = def.shell.header_padding_x + def.search.text_inset_x;
        let turn = div()
            .flex_none()
            .mx(px(edge))
            .my(px(def.slot_line.gap_px))
            .flex()
            .flex_col()
            .gap(px(def.slot_line.gap_px))
            .text_size(px(def.search.font_size * 0.8))
            .text_color(rgb(colors.text.primary))
            .min_w(px(0.));
        match who {
            Who::Tool => turn
                .debug_selector(move || format!("chat:tool:{ix}"))
                .flex_row()
                .items_center()
                .text_size(px(def.slot_line.title_font_size))
                .when_some(sub, |d, sub| {
                    d.child(
                        div()
                            .flex_none()
                            .text_color(rgba(chrome.text_muted_rgba))
                            .child(sub.split(" · ").next().unwrap_or(sub).to_owned()),
                    )
                })
                .child(div().truncate().child(text.to_owned()))
                .into_any_element(),
            Who::You => turn
                .child(text.to_owned())
                .px(px(def.slot_line.gap_px * 2.0))
                .py(px(def.slot_line.gap_px))
                .rounded(px(def.slot_line.gap_px * 2.0))
                .bg(rgba(chrome.chat_user_message_surface_rgba))
                .when_some(sub, |turn, sub| {
                    turn.child(
                        div()
                            .text_size(px(def.slot_line.label_font_size))
                            .text_color(rgb(colors.text.muted))
                            .child(sub.to_string()),
                    )
                })
                .into_any_element(),
            Who::Model => turn
                .debug_selector(move || format!("chat-turn:{ix}"))
                .when_some(markdown, |d, md| d.child(md))
                .when(complete, |d| {
                    d.child(
                        div()
                            .text_size(px(def.slot_line.label_font_size))
                            .text_color(rgb(colors.text.muted))
                            .child(chat::WRITTEN_BY.to_uppercase()),
                    )
                })
                .into_any_element(),
        }
    }

    fn one_window_chat_rows(
        &mut self,
        rows: &mut Vec<AnyElement>,
        colors: ListItemColors,
        head_h: f32,
        cx: &mut Context<Self>,
    ) {
        let Some(c) = self.one_window.chat() else {
            return;
        };
        if c.acting.is_some() {
            let (_, offered) = c.rows();
            let mut heads = 0;
            for (i, row) in offered.iter().enumerate() {
                if let Some(head) = row.head.as_deref() {
                    heads += 1;
                    rows.push(self.one_window_header(heads, head, colors, head_h));
                }
                rows.push(self.one_window_row(
                    format!("chat-next:{}", row.key),
                    row.title.clone(),
                    row.sub.clone(),
                    row.icon.as_deref().unwrap_or("sparkles"),
                    i == c.sel,
                    colors,
                ));
            }
            if offered.is_empty() {
                rows.push(self.one_window_note(c.enter(&c.filter).label));
            }
            return;
        }
        let owner = self
            .one_window_host
            .work
            .chat
            .as_ref()
            .map(|m| m.thread.entity_id());
        let (head, next) = c.rows();
        let cache = &mut self.one_window_host.work.presentation;
        let states = cache.reconcile(owner, c, cx);
        cache.reveal_choice(head, c.sel);
        let transcript_scroll = cache.transcript.clone();
        let choice_scroll = cache.choices.clone();
        let mut answers = states.iter();
        let prompt = crate::theme::PromptColors::from_theme(&self.theme);
        let style = crate::components::conversation_style::production_conversation_style();
        let pending = c.asks.is_some() || c.sign_in.is_some();
        let transcript = div()
            .id("chat-history")
            .debug_selector(|| "chat-history".into())
            .min_h(px(0.))
            .when(c.activity_label().is_none(), |d| d.flex_1())
            .overflow_y_scroll()
            .track_scroll(&transcript_scroll)
            .children(c.turns.iter().enumerate().map(|(ix, turn)| {
                let markdown = if turn.who == Who::Model {
                    answers.next().map(|state| {
                        crate::components::conversation_text::conversation_markdown_view(
                            state,
                            &self.theme,
                            &prompt,
                            rgb(prompt.text_primary),
                            &style,
                            format!("one-window/chat/{ix}").into(),
                            gpui_component::text::MarkdownLinkLabelPolicy::CompactLongBareHttp,
                        )
                        .into_any_element()
                    })
                } else {
                    None
                };
                self.one_window_turn(
                    ix,
                    turn.who,
                    &turn.text,
                    turn.sub.as_deref(),
                    markdown,
                    ix + 1 != c.turns.len() || c.phase != ChatPhase::Writing,
                )
            }))
            .into_any_element();
        let mut history = Some(transcript);
        if !pending && !c.turns.is_empty() {
            rows.extend(history.take());
        }
        if let Some(asks) = c.asks.as_ref() {
            rows.push(
                self.one_window_note(format!("The model asks to {}", lower_first(&asks.title))),
            );
            if let Some(facts) = &asks.launcher_review {
                rows.push(
                    div()
                        .id("chat-launcher-review")
                        .debug_selector(|| "chat-launcher-review".into())
                        .child(self.one_window_note(facts.clone()))
                        .into_any_element(),
                );
            }
        } else if let Some(provider) = c.sign_in.as_ref() {
            rows.push(self.one_window_note(format!(
                "The model needs you to sign in to {provider} before it can answer"
            )));
        }
        if let Some(head) = head {
            rows.push(self.one_window_header(1, head, colors, head_h));
            let height = crate::list_item::effective_list_item_height_for_theme(
                self.current_main_menu_theme,
            );
            // Reserve two ordinary action rows; a decision gets three so Allow and Reject
            // are visible together. More choices scroll without covering the answer or footer.
            let visible = next.len().min(if pending { 3 } else { 2 });
            rows.push(
                div()
                    .id("chat-choices")
                    .debug_selector(|| "chat-choices".into())
                    .h(px(height * visible as f32))
                    .flex_none()
                    .overflow_y_scroll()
                    .track_scroll(&choice_scroll)
                    .children(next.iter().enumerate().map(|(i, row)| {
                        self.one_window_row(
                            format!("chat-next:{}", row.key),
                            row.title.clone(),
                            row.sub.clone(),
                            row.icon.as_deref().unwrap_or("sparkles"),
                            i == c.sel,
                            colors,
                        )
                    }))
                    .into_any_element(),
            );
        } else if let Some(status) = c.activity_label() {
            use gpui_component::Sizable as _;
            let def = self.current_main_menu_theme.def();
            rows.push(
                div()
                    .debug_selector(|| "chat-status".into())
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(def.slot_line.gap_px))
                    .px(px(def.shell.header_padding_x + def.search.text_inset_x))
                    .h(px(head_h))
                    .text_size(px(def.slot_line.title_font_size))
                    .text_color(rgba(
                        crate::theme::AppChromeColors::from_theme(&self.theme).text_muted_rgba,
                    ))
                    .child(gpui_component::spinner::Spinner::new().small())
                    .child(status)
                    .into_any_element(),
            );
        } else {
            match &c.phase {
                ChatPhase::Idle if !c.with.is_empty() => {
                    rows.push(self.one_window_note("Say what to ask about it"))
                }
                ChatPhase::Idle => rows.push(self.one_window_note("Say what to ask the model")),
                ChatPhase::EndedEmpty { stopped } => rows.push(self.one_window_note(if *stopped {
                    "Reply stopped."
                } else {
                    "The model returned no text."
                })),
                ChatPhase::Failed(message) => rows.push(self.one_window_note(message.clone())),
                _ => {}
            }
        }
        if pending {
            rows.extend(history);
        }
    }

    fn one_window_past_chat_rows(
        &self,
        rows: &mut Vec<AnyElement>,
        colors: ListItemColors,
        head_h: f32,
    ) {
        let Some(p) = self.one_window.past_chats() else {
            return;
        };
        if p.staged.is_some() {
            rows.push(self.one_window_header(1, p.head(), colors, head_h));
        }
        let listed = p.rows();
        if listed.is_empty() {
            rows.push(self.one_window_note(p.enter().label));
        }
        for (i, row) in listed.iter().enumerate() {
            rows.push(self.one_window_row(
                row.key.clone(),
                row.title.clone(),
                row.sub.clone(),
                row.icon.as_deref().unwrap_or("message"),
                i == p.sel,
                colors,
            ));
        }
    }

    fn one_window_steps_rows(&self, rows: &mut Vec<AnyElement>, colors: ListItemColors) {
        let Some(s) = self.one_window.steps_view() else {
            return;
        };
        let conns = self.one_window.work().connections();
        let focus = s.sel.or_else(|| s.missing(conns).map(|(i, _)| i));
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
                focus == Some(i) && s.phase != StepsPhase::Running,
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
            format!("Waiting to run · “{}”", c.waiting),
            Some("Runs as soon as you're connected, no retyping".to_string()),
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
        rows.push(self.one_window_header(1, "Also works once connected", colors, head_h));
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
        if let Some(reply) = self.one_window_agent_reply(&task.id) {
            let parsed = crate::prompts::markdown::render_markdown(
                reply,
                &crate::theme::PromptColors::from_theme(&self.theme),
            );
            rows.push(self.one_window_turn(
                0,
                Who::Model,
                reply,
                None,
                Some(parsed.into_any_element()),
                true,
            ));
        }
        rows.push(self.one_window_note("Esc keeps it working in the strip · ⌘J brings it back"));
    }

    /// Keep the agent rows above the capsules and clip the scrolling list at their top edge.
    pub(crate) fn with_one_window_strip(&self, content: AnyElement) -> AnyElement {
        let tasks = self.one_window.work().strip().tasks();
        if tasks.is_empty() || !self.one_window_owns_list() {
            return content;
        }
        let def = self.current_main_menu_theme.def();
        let tokens = def.slot_line;
        let colors = &self.theme.colors;
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
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
            .relative()
            .flex_none()
            .bg(rgba(chrome.popup_surface_rgba))
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
        let footer = px(crate::footer_popup::main_window_footer_reservation_height());
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .pb(footer)
            .child(
                div()
                    .debug_selector(|| "one-window-strip-list-clip".into())
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    // The list and hosted views already reserve a footer inside their
                    // viewport. Keep that inset below the clip so the last row can
                    // scroll all the way to the strip and still receive pointer input.
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom(-footer)
                            .child(content),
                    ),
            )
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

impl ScriptListApp {
    /// What automation sees while a work view covers the list: the input, the view's header, and
    /// its rows and turns from the same state the view draws. Words the user wrote are redacted.
    pub(crate) fn collect_one_window_work_elements(
        &self,
        limit: usize,
    ) -> (Vec<crate::protocol::ElementInfo>, usize) {
        use crate::protocol::{ElementContentKind, ElementInfo};
        let header = super::views::header_element;
        let product = |id: String, ix: usize, title: &str, selected: bool| ElementInfo {
            semantic_id: id,
            ..ElementInfo::product_static_choice(ix, title, title, selected)
        };
        let yours = |id: String, ix: usize, text: &str, selected: bool| ElementInfo {
            semantic_id: id,
            ..ElementInfo::redacted_choice(
                ix,
                text,
                text,
                selected,
                ElementContentKind::UserContent,
            )
        };
        let mut out = vec![
            ElementInfo::input(
                "filter",
                Some(self.filter_text.as_str()),
                self.focused_input != FocusedInput::None,
            ),
            header(0, &self.one_window.views().crumb()),
        ];
        if let Some(p) = self.one_window.past_chats() {
            if p.staged.is_some() {
                out.push(header(1, p.head()));
            }
            for (i, row) in p.rows().iter().enumerate() {
                out.push(ElementInfo {
                    role: Some("row".into()),
                    ..product(row.key.clone(), i, &row.title, i == p.sel)
                });
            }
        } else if let Some(c) = self.one_window.chat() {
            let acting = c.acting.is_some();
            if let Some(status) = c.activity_label() {
                out.push(crate::protocol::ElementInfo {
                    role: Some("status".into()),
                    selectable: Some(false),
                    selected: None,
                    ..product("chat-status".into(), 0, status, false)
                });
            }
            if !acting {
                for (i, t) in c.turns.iter().enumerate() {
                    let id = match t.who {
                        Who::Tool => format!("chat:tool:{i}"),
                        _ => format!("chat-turn:{i}"),
                    };
                    out.push(yours(id, i, &t.text, false));
                }
            }
            if !acting {
                if let Some(facts) = c.asks.as_ref().and_then(|a| a.launcher_review.as_deref()) {
                    out.push(ElementInfo {
                        role: Some("note".into()),
                        selectable: Some(false),
                        ..yours("chat-launcher-review".into(), 0, facts, false)
                    });
                }
            }
            let (head, next) = c.rows();
            if head.is_some() || acting {
                let mut heads = 0;
                if let Some(head) = head {
                    heads += 1;
                    out.push(header(heads, head));
                }
                for (i, row) in next.iter().enumerate() {
                    if let Some(head) = row.head.as_deref().filter(|_| acting) {
                        heads += 1;
                        out.push(header(heads, head));
                    }
                    out.push(product(
                        format!("chat-next:{}", row.key),
                        i,
                        &row.title,
                        i == c.sel,
                    ));
                }
            }
        } else if let Some(s) = self.one_window.steps_view() {
            let conns = self.one_window.work().connections();
            let focus = s.sel.or_else(|| s.missing(conns).map(|(i, _)| i));
            for (i, step) in s.steps.iter().enumerate() {
                out.push(yours(format!("step:{i}"), i, &step.text, focus == Some(i)));
            }
        } else if let Some(c) = self.one_window.connecting() {
            let name = connect::name_of(&c.integration);
            out.push(product(
                format!("connect:{}", c.integration),
                0,
                &format!("This needs {name}"),
                true,
            ));
            out.push(yours("connect:waiting".into(), 1, &c.waiting, false));
            if let Some(info) = connect::integration(&c.integration) {
                out.push(header(1, "Also works once connected"));
                for (i, can) in info.can.iter().enumerate() {
                    out.push(product(format!("connect-can:{i}"), i + 2, can, false));
                }
            }
        } else if let Some(task) = self.one_window.open_task() {
            out.push(yours(format!("agent:{}", task.id), 0, &task.label, true));
        }
        let total = out.len();
        out.truncate(limit);
        (out, total)
    }
}

/// "Run rm -rf build" → "run rm -rf build", to follow "The model asks to".
fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}
