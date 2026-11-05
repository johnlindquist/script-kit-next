//! Draws the open view, or what the staged subject offers, in place of the list: a header, then
//! rows in the shared list-row anatomy. A form's fields and an answer's facts are rows too, so the
//! window keeps one row language; the caret field is the selected row.

use gpui::{div, prelude::*, px, AnyElement};
use sk_one_window::answer::Shape;
use sk_one_window::form::{FieldState, SlotName};

use super::HostedView;
use crate::list_item::{IconKind, ListItem, ListItemColors};
use crate::protocol::{ElementContentKind, ElementInfo};
use crate::{FocusedInput, ScriptListApp};

/// One drawn row: its id, words, icon, and whether it's the one Enter or typing acts on.
struct ViewRow {
    id: String,
    title: String,
    description: Option<String>,
    icon: &'static str,
    selected: bool,
    /// A word of the reading under →: its chip and where it came from.
    word: Option<super::super::reading::WordRow>,
}

fn form_value(view: &HostedView, index: usize) -> &str {
    match view {
        HostedView::Form(form) => form
            .fields
            .get(index)
            .map_or("", |field| field.value.as_str()),
        _ => "",
    }
}

fn field_icon(name: SlotName) -> &'static str {
    match name {
        SlotName::Who => "user",
        SlotName::What => "edit",
        SlotName::When => "clock",
        SlotName::Where => "folder",
        SlotName::Event => "calendar",
        SlotName::Body => "file",
    }
}

fn view_rows(view: &HostedView) -> Vec<ViewRow> {
    match view {
        HostedView::Form(form) => form
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| ViewRow {
                id: format!("form-field:{:?}", f.name).to_lowercase(),
                title: f.label.clone(),
                description: Some(match f.state {
                    FieldState::Needed => match f.name {
                        SlotName::Who => "Enter a recipient",
                        SlotName::When => "Enter a date or time",
                        SlotName::Where => "Enter a location",
                        SlotName::What | SlotName::Body => "Enter the text",
                        SlotName::Event => "Enter an event",
                    }
                    .to_string(),
                    FieldState::Filled => format!("{} · {}", f.value, f.source_words()),
                }),
                icon: field_icon(f.name),
                selected: form.caret == Some(i),
                word: None,
            })
            .collect(),
        HostedView::Answer(answer) => match &answer.shape {
            None => vec![ViewRow {
                id: "answer:reading".into(),
                title: "Reading…".into(),
                description: Some(answer.question.clone()),
                icon: "search",
                selected: true,
                word: None,
            }],
            Some(Shape::Number { value, from }) => vec![ViewRow {
                id: "answer:number".into(),
                title: value.clone(),
                description: Some(format!("{} · {}", answer.question, from.words())),
                icon: "code",
                selected: true,
                word: None,
            }],
            Some(Shape::List { items }) => items
                .iter()
                .enumerate()
                .map(|(i, f)| ViewRow {
                    id: format!("answer:item:{i}"),
                    title: f.label.clone(),
                    description: Some(format!("{} · {}", f.detail, f.from.words())),
                    icon: "check",
                    selected: i == 0,
                    word: None,
                })
                .collect(),
            Some(Shape::Card { title, facts }) => std::iter::once(ViewRow {
                id: "answer:card".into(),
                title: title.clone(),
                description: facts
                    .first()
                    .map(|f| format!("{} · {}", f.detail, f.from.words())),
                icon: "clipboard",
                selected: true,
                word: None,
            })
            .chain(facts.iter().skip(1).enumerate().map(|(i, f)| ViewRow {
                id: format!("answer:fact:{i}"),
                title: f.label.clone(),
                description: Some(format!("{} · {}", f.detail, f.from.words())),
                icon: "check",
                selected: false,
                word: None,
            }))
            .collect(),
            Some(Shape::Nothing) => vec![ViewRow {
                id: "answer:nothing".into(),
                title: "Nothing matched".into(),
                description: Some(answer.question.clone()),
                icon: "search",
                selected: true,
                word: None,
            }],
        },
        HostedView::Notes(list) if list.found.is_empty() => vec![match &list.trouble {
            Some(trouble) => ViewRow {
                id: "notes-row:trouble".into(),
                title: trouble.clone(),
                description: Some("Esc goes back; reopen Notes to try again".into()),
                icon: "warning",
                selected: true,
                word: None,
            },
            None => ViewRow {
                id: "notes-row:none".into(),
                title: "No notes yet".into(),
                description: Some("Type a title to start one".into()),
                icon: "plus",
                selected: true,
                word: None,
            },
        }],
        HostedView::Notes(list) => list
            .found
            .iter()
            .enumerate()
            .map(|(i, found)| ViewRow {
                id: format!("notes-row:{i}"),
                title: found.title.clone(),
                description: Some(found.sub.clone()),
                icon: found.icon(),
                selected: list.sel == i,
                word: None,
            })
            .collect(),
        HostedView::Personal(list) if list.data.rows().len() == 0 => vec![ViewRow {
            id: "personal-list:status".into(),
            title: list.message.unwrap_or("No matching titles").into(),
            description: None,
            icon: "info",
            selected: false,
            word: None,
        }],
        HostedView::Personal(list) => list
            .data
            .rows()
            .map(|r| ViewRow {
                id: list.row_id(&r.id),
                title: r.title.clone(),
                description: r.detail.clone(),
                icon: r.icon,
                selected: list.data.selected() == Some(r),
                word: None,
            })
            .collect(),
        HostedView::Reading(reading) => reading
            .rows
            .iter()
            .map(|row| ViewRow {
                id: format!("word:{}", row.kind.label().to_lowercase()),
                title: row.value.clone(),
                description: (!row.sub.is_empty()).then(|| row.sub.clone()),
                icon: "",
                selected: false,
                word: Some(row.clone()),
            })
            .collect(),
    }
}

/// A word's chip: its dot and name in its color, one width for all seven so values line up.
fn word_chip(row: &super::super::reading::WordRow) -> AnyElement {
    let color = super::super::reading::word_color(row.kind);
    div()
        .w(px(78.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(div().size(px(7.)).rounded_full().bg(gpui::rgb(color)))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(gpui::rgb(color))
                .child(row.kind.label()),
        )
        .into_any_element()
}

/// Where a word came from: "typed" in its color, "Jev 93%", "on screen" or "your words".
fn word_meta(row: &super::super::reading::WordRow, muted: u32) -> Option<AnyElement> {
    if row.meta.is_empty() {
        return None;
    }
    let color = super::super::reading::word_color(row.kind);
    let meta = div().text_size(px(11.)).child(row.meta.clone());
    Some(if row.typed {
        meta.px(px(6.))
            .py(px(1.))
            .rounded(px(5.))
            .border_1()
            .border_color(gpui::rgba((color << 8) | 0x80))
            .text_color(gpui::rgb(color))
            .into_any_element()
    } else {
        meta.text_color(gpui::rgb(muted)).into_any_element()
    })
}

/// Scrolls a view's rows so row `ix` sits in the part of the list the footer capsules don't
/// cover, by the same rule the main list reveals its highlight with. False until the rows have
/// been laid out once, so the next draw tries again.
fn reveal_above_footer(
    scroll: &gpui::ScrollHandle,
    ix: usize,
    row_count: usize,
    row_height: f32,
) -> bool {
    let viewport = scroll.bounds().size.height;
    if viewport <= px(0.) {
        return false;
    }
    let top = row_height * ix as f32;
    if let Some(scroll_top) = crate::scrolling::list_geometry::safe_reveal_scroll_top(
        -scroll.offset().y.as_f32(),
        (viewport - crate::main_list_footer_overlay_total_padding()).as_f32(),
        row_height * row_count as f32,
        top,
        top + row_height,
    ) {
        scroll.set_offset(gpui::point(px(0.), px(-scroll_top)));
    }
    true
}

impl ScriptListApp {
    /// The open view, or the staged subject's rows while the input is empty; `None` leaves the
    /// launcher's list in place.
    pub(crate) fn one_window_view_element(&self, cx: &gpui::Context<Self>) -> Option<AnyElement> {
        let (head, rows) = if let Some(view) = self.one_window_hosted_view() {
            (view.label(), view_rows(view))
        } else if self.one_window.staged().is_some()
            && !self.one_window.in_view()
            && self.filter_text.trim().is_empty()
        {
            self.one_window_staged_rows()?
        } else {
            return None;
        };
        let colors = ListItemColors::from_theme(&self.theme);
        let theme = self.current_main_menu_theme;
        let header_h = crate::list_item::effective_first_section_header_height_for_theme(theme);
        let scroll = &self.one_window_host.view_scroll;
        let selected = rows.iter().position(|row| row.selected);
        if selected != self.one_window_host.view_scrolled_to.get()
            && selected.is_none_or(|ix| {
                reveal_above_footer(
                    scroll,
                    ix,
                    rows.len(),
                    crate::list_item::effective_list_item_height_for_theme(theme),
                )
            })
        {
            self.one_window_host.view_scrolled_to.set(selected);
        }
        let mut list = div()
            .id("one-window-view-rows")
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(scroll)
            .pb(crate::main_list_footer_overlay_total_padding());
        for (ix, row) in rows.into_iter().enumerate() {
            let selector = format!("list-row:{}", row.id);
            let form_field = row.id.starts_with("form-field:").then(|| row.id.clone());
            let personal_item = row
                .id
                .starts_with("personal-item:")
                .then(|| (row.id.clone(), row.selected, !matches!(self.one_window_hosted_view(), Some(HostedView::Personal(l)) if l.read_only())));
            let stage_action = row
                .id
                .starts_with("staged:")
                .then(|| (row.id.clone(), row.selected));
            let (title, detail, label) =
                if let Some(HostedView::Form(form)) = self.one_window_hosted_view() {
                    let field = &form.fields[ix];
                    let value = if field.state == FieldState::Needed {
                        row.description.unwrap_or_default()
                    } else {
                        field.value.clone()
                    };
                    let label = div()
                        .w(px(theme.def().list.main_hint_fragment_role_width))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(gpui::rgba(
                            crate::theme::AppChromeColors::from_theme(&self.theme).text_muted_rgba,
                        ))
                        .child(row.title)
                        .into_any_element();
                    (
                        value,
                        (field.from.is_some()).then(|| field.source_words().to_string()),
                        Some(label),
                    )
                } else {
                    (row.title, row.description, None)
                };
            let item = ListItem::new(title, colors)
                .index(ix)
                .selected(row.selected)
                .main_menu_theme(theme)
                .semantic_id_opt(Some(row.id))
                .leading_accessory_opt(label)
                .description_opt(detail);
            let item = match &row.word {
                Some(word) => item
                    .leading_accessory(word_chip(word))
                    .trailing_accessory_opt(word_meta(word, self.theme.colors.text.muted))
                    .disabled(word.dim),
                None => item.icon_kind(
                    IconKind::from_icon_hint(row.icon)
                        .unwrap_or_else(|| IconKind::Svg(row.icon.into())),
                ),
            };
            list = list.child(
                div()
                    .id(gpui::SharedString::from(selector.clone()))
                    .flex_none()
                    .debug_selector(move || selector.clone())
                    .when_some(form_field, |d, id| {
                        d.cursor_pointer()
                            .on_click(cx.listener(move |app, _, window, cx| {
                                if app.one_window_pick_form_field(&id, cx) {
                                    app.focus_main_filter(window, cx);
                                }
                            }))
                    })
                    .when_some(personal_item, |d, (id, selected, can_submit)| {
                        d.cursor_pointer().on_click(cx.listener(
                            move |app, event: &gpui::ClickEvent, _, cx| {
                                let submit = crate::ui_foundation::should_submit_selected_row_click(
                                    selected,
                                    event.click_count(),
                                );
                                let _ = app.pick_personal_view_item(&id, can_submit && submit, cx);
                            },
                        ))
                    })
                    .when_some(stage_action, |d, (id, selected)| {
                        d.cursor_pointer().on_click(cx.listener(
                            move |app, event: &gpui::ClickEvent, _, cx| {
                                let submit = crate::ui_foundation::should_submit_selected_row_click(
                                    selected,
                                    event.click_count(),
                                );
                                let _ = app.pick_visible_staged_row(&id, submit, cx);
                            },
                        ))
                    })
                    .child(item),
            );
        }
        Some(
            div()
                .relative()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .pt(crate::main_list_header_overlay_height(theme.def()))
                .child(
                    div()
                        .flex_none()
                        .debug_selector(|| "oneWindowSectionHeader:0".to_string())
                        .h(px(header_h))
                        .child(crate::list_item::render_section_header(
                            &head, None, colors, true,
                        )),
                )
                .child(list)
                // Rows scrolled under the capsules show through but can't be clicked, as in the
                // main list; the wheel still scrolls them.
                .when(
                    crate::footer_popup::main_list_scrolls_under_footer(),
                    |view| {
                        view.child(
                            div()
                                .debug_selector(|| "main-list-footer-occlusion".into())
                                .absolute()
                                .left_0()
                                .right_0()
                                .bottom_0()
                                .h(crate::main_list_footer_overlay_total_padding())
                                .block_mouse_except_scroll(),
                        )
                    },
                )
                .into_any_element(),
        )
    }

    /// The row the drawn view has selected, which is the window's selected row while it's drawn.
    pub(crate) fn one_window_view_selected_id(&self) -> Option<String> {
        let rows = match self.one_window_hosted_view() {
            Some(view) => view_rows(view),
            None if self.one_window.staged().is_some()
                && !self.one_window.in_view()
                && self.filter_text.trim().is_empty() =>
            {
                self.one_window_staged_rows()?.1
            }
            None => return None,
        };
        rows.into_iter().find(|row| row.selected).map(|row| row.id)
    }

    /// The staged subject's rows as the reducer lays them out, the first under its header.
    fn one_window_staged_rows(&self) -> Option<(String, Vec<ViewRow>)> {
        let staged = self.one_window.staged()?;
        if !matches!(self.current_view, crate::AppView::ScriptList)
            || self.one_window.in_view()
            || !self.filter_text.trim().is_empty()
        {
            return None;
        }
        let identity =
            crate::logging::log_private_user_value(&serde_json::to_string(&staged.subject).ok()?)
                .sha256;
        let list = self.one_window.list();
        let layout = list.layout();
        let head = layout.iter().find_map(|item| match item {
            sk_one_window::Item::Header(head) => Some(head.clone()),
            _ => None,
        })?;
        let rows = list
            .visible()
            .iter()
            .enumerate()
            .filter_map(|(pos, item)| {
                let row = list.row(item)?;
                Some(ViewRow {
                    id: format!(
                        "staged:{identity}:{}",
                        crate::logging::log_private_user_value(&row.key).sha256
                    ),
                    title: row.title.clone(),
                    description: row.sub.clone(),
                    icon: staged_icon(row.icon.as_deref()),
                    selected: pos == list.sel,
                    word: None,
                })
            })
            .collect();
        Some((head, rows))
    }

    /// Resolve only the drawn stage, never the covered main search result at the same index.
    pub(crate) fn pick_visible_staged_row(
        &mut self,
        id: &str,
        submit: bool,
        cx: &mut gpui::Context<Self>,
    ) -> anyhow::Result<()> {
        let (_, rows) = self
            .one_window_staged_rows()
            .ok_or_else(|| anyhow::anyhow!("stage_not_current"))?;
        let ix = rows
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| anyhow::anyhow!("staged_action_not_current"))?;
        self.one_window_handle(sk_one_window::Event::Select(ix), cx);
        anyhow::ensure!(
            self.one_window_view_selected_id().as_deref() == Some(id),
            "staged_selection_changed"
        );
        if submit {
            self.one_window_enter(cx)?;
        }
        Ok(())
    }

    /// Whether an open view, or a staged subject's rows, is drawn in place of the list.
    pub(crate) fn one_window_view_covers_list(&self) -> bool {
        self.one_window_hosted_view().is_some()
            || (self.one_window.staged().is_some()
                && !self.one_window.in_view()
                && self.filter_text.trim().is_empty())
    }

    /// The first header the window draws: an open form, answer or note's words, a work view's
    /// crumb, or the list's.
    pub(crate) fn one_window_drawn_header(&self) -> String {
        if let Some(view) = self.one_window_hosted_view() {
            view.label()
        } else if self.one_window_work_view_open() {
            self.one_window.views().crumb()
        } else {
            self.one_window.list().header()
        }
    }

    /// What automation sees while a form, answer or note covers the list: the input, the view's
    /// header and its rows, from the same rows the view draws. Row words are the user's own or
    /// read from their data, so they're redacted.
    pub(crate) fn collect_one_window_hosted_elements(
        &self,
        limit: usize,
    ) -> Option<(Vec<ElementInfo>, usize)> {
        let view = self.one_window_hosted_view();
        let (label, rows) = match view {
            Some(v) => (v.label(), view_rows(v)),
            None => self.one_window_staged_rows()?,
        };
        let mut header = header_element(0, &label);
        if matches!(view, Some(HostedView::Personal(_))) {
            // A recipe title is workspace content, including in the usual section slug.
            header.semantic_id = "personal-list:header".into();
            header = header.redact_content(ElementContentKind::UserContent);
        }
        let mut out = vec![
            ElementInfo::input(
                "filter",
                Some(self.filter_text.as_str()),
                self.focused_input != FocusedInput::None,
            ),
            header,
        ];
        out.extend(rows.into_iter().enumerate().map(|(ix, row)| {
            let mut el = ElementInfo::redacted_choice(
                ix,
                &row.title,
                if let Some(v @ HostedView::Form(_)) = view {
                    form_value(v, ix)
                } else {
                    &row.title
                },
                row.selected,
                ElementContentKind::UserContent,
            );
            el.semantic_id = row.id;
            if view.is_none() || matches!(view, Some(HostedView::Personal(_))) {
                el.role = Some("row".into());
                el.selectable = Some(el.semantic_id != "personal-list:status");
            }
            el
        }));
        let total = out.len();
        out.truncate(limit);
        Some((out, total))
    }
}

/// A view's header as automation sees it: the words `render_section_header` draws for `label`.
pub(crate) fn header_element(ix: usize, label: &str) -> ElementInfo {
    let (name, count) = label
        .split_once(" · ")
        .map_or((label, None), |(name, count)| (name, Some(count)));
    let presentation = crate::list_item::resolve_section_header_presentation(
        name,
        None,
        count,
        crate::list_item::SectionPresentationFamily::Launcher,
    );
    let text = match &presentation.count {
        Some(count) => format!("{} · {count}", presentation.display_label),
        None => presentation.display_label.to_string(),
    };
    ElementInfo {
        semantic_id: crate::protocol::generate_semantic_id("section", ix, label),
        text: Some(text),
        value: Some(label.to_string()),
        role: Some("sectionHeader".to_string()),
        kind: Some("sectionHeader".to_string()),
        selectable: Some(false),
        ..ElementInfo::panel(label)
    }
}

fn staged_icon(icon: Option<&str>) -> &'static str {
    match icon {
        Some("message" | "reply") => "message",
        Some("external-link") => "external-link",
        Some("sparkles") => "sparkles",
        Some("notebook-pen") => "notebook-pen",
        Some("text-cursor") => "text-cursor",
        Some("clipboard") => "clipboard",
        Some("send") => "send",
        Some("panel-left") => "panel-left",
        Some("panel-right") => "panel-right",
        Some("copy") => "copy",
        Some("append" | "edit") => "edit",
        Some("open") => "arrowright",
        Some("undo") => "close",
        Some("again") => "play",
        Some("script") => "code",
        _ => "check",
    }
}
