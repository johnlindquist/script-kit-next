//! Draws the open view, or what the staged subject offers, in place of the list: a header, then
//! rows in the shared list-row anatomy. A form's fields and an answer's facts are rows too, so the
//! window keeps one row language; the caret field is the selected row.

use gpui::{div, prelude::*, px, AnyElement};
use sk_one_window::answer::Shape;
use sk_one_window::form::{FieldState, SlotName};

use super::HostedView;
use crate::list_item::{IconKind, ListItem, ListItemColors};
use crate::ScriptListApp;

/// One drawn row: its id, words, icon, and whether it's the one Enter or typing acts on.
struct ViewRow {
    id: String,
    title: String,
    description: Option<String>,
    icon: &'static str,
    selected: bool,
}

fn field_icon(name: SlotName) -> &'static str {
    match name {
        SlotName::Who => "message",
        SlotName::What => "edit",
        SlotName::When => "bolt",
        SlotName::Where => "folder",
        SlotName::Event => "star",
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
                title: if f.state == FieldState::Needed {
                    format!("{} · Needs you", f.label)
                } else {
                    f.value.clone()
                },
                description: Some(match f.state {
                    FieldState::Needed => "Type it here".to_string(),
                    FieldState::Filled => format!("{} · {}", f.label, f.source_words()),
                }),
                icon: field_icon(f.name),
                selected: form.caret == Some(i),
            })
            .collect(),
        HostedView::Answer(answer) => match &answer.shape {
            None => vec![ViewRow {
                id: "answer:reading".into(),
                title: "Reading…".into(),
                description: Some(answer.question.clone()),
                icon: "search",
                selected: true,
            }],
            Some(Shape::Number { value, from }) => vec![ViewRow {
                id: "answer:number".into(),
                title: value.clone(),
                description: Some(format!("{} · {}", answer.question, from.words())),
                icon: "code",
                selected: true,
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
            })
            .chain(facts.iter().skip(1).enumerate().map(|(i, f)| ViewRow {
                id: format!("answer:fact:{i}"),
                title: f.label.clone(),
                description: Some(format!("{} · {}", f.detail, f.from.words())),
                icon: "check",
                selected: false,
            }))
            .collect(),
            Some(Shape::Nothing) => vec![ViewRow {
                id: "answer:nothing".into(),
                title: "Nothing matched".into(),
                description: Some(answer.question.clone()),
                icon: "search",
                selected: true,
            }],
        },
        HostedView::Note(note) => note
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| ViewRow {
                id: format!("note-line:{i}"),
                title: line.clone(),
                description: None,
                icon: "file",
                selected: false,
            })
            .chain(std::iter::once(ViewRow {
                id: "note-line:new".into(),
                title: if note.line.trim().is_empty() {
                    "Write the line…".into()
                } else {
                    note.line.clone()
                },
                description: Some("↵ adds it".into()),
                icon: "edit",
                selected: true,
            }))
            .collect(),
    }
}

impl ScriptListApp {
    /// The open view, or the staged subject's rows while the input is empty; `None` leaves the
    /// launcher's list in place.
    pub(crate) fn one_window_view_element(&self) -> Option<AnyElement> {
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
        let mut list = div().w_full().h_full().flex().flex_col().child(
            div()
                .flex_none()
                .debug_selector(|| "oneWindowSectionHeader:0".to_string())
                .h(px(header_h))
                .child(crate::list_item::render_section_header(
                    &head, None, colors, true,
                )),
        );
        for (ix, row) in rows.into_iter().enumerate() {
            let selector = format!("list-row:{}", row.id);
            let icon =
                IconKind::from_icon_hint(row.icon).unwrap_or_else(|| IconKind::Svg("check".into()));
            list = list.child(
                div()
                    .flex_none()
                    .debug_selector(move || selector.clone())
                    .child(
                        ListItem::new(row.title, colors)
                            .index(ix)
                            .selected(row.selected)
                            .main_menu_theme(theme)
                            .semantic_id_opt(Some(row.id))
                            .description_opt(row.description)
                            .icon_kind(icon),
                    ),
            );
        }
        Some(list.into_any_element())
    }

    /// The staged subject's rows as the reducer lays them out, the first under its header.
    fn one_window_staged_rows(&self) -> Option<(String, Vec<ViewRow>)> {
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
                    id: format!("staged:{}", row.key),
                    title: row.title.clone(),
                    description: row.sub.clone(),
                    icon: staged_icon(row.icon.as_deref()),
                    selected: pos == list.sel,
                })
            })
            .collect();
        Some((head, rows))
    }

    /// The open view's natural height: the line above the input, the header, its rows and the
    /// floating footer's zone.
    pub(crate) fn one_window_view_height(&self) -> Option<f32> {
        let rows = match self.one_window_hosted_view() {
            Some(view) => view.rows(),
            None if self.one_window.staged().is_some()
                && !self.one_window.in_view()
                && self.filter_text.trim().is_empty() =>
            {
                self.one_window.list().visible().len()
            }
            None => return None,
        };
        let theme = self.current_main_menu_theme;
        let row_h = crate::list_item::effective_list_item_height_for_theme(theme);
        let header_h = crate::list_item::effective_first_section_header_height_for_theme(theme);
        Some(
            sk_one_window::contract::app::SLOT_LINE_PX
                + f32::from(crate::main_list_header_overlay_height(theme.def()))
                + header_h
                + row_h * rows as f32
                + crate::footer_popup::main_window_footer_reservation_height(),
        )
    }
}

fn staged_icon(icon: Option<&str>) -> &'static str {
    match icon {
        Some("message" | "reply") => "message",
        Some("copy") => "copy",
        Some("append" | "edit") => "edit",
        Some("open") => "arrowright",
        Some("undo") => "close",
        Some("again") => "play",
        Some("script") => "code",
        _ => "check",
    }
}
