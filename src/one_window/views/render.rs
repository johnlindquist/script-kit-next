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
                title: "Notes couldn't be searched".into(),
                description: Some(trouble.clone()),
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
            let item = ListItem::new(row.title, colors)
                .index(ix)
                .selected(row.selected)
                .main_menu_theme(theme)
                .semantic_id_opt(Some(row.id))
                .description_opt(row.description);
            let item = match &row.word {
                Some(word) => item
                    .leading_accessory(word_chip(word))
                    .trailing_accessory_opt(word_meta(word, self.theme.colors.text.muted))
                    .disabled(word.dim),
                None => item.icon_kind(
                    IconKind::from_icon_hint(row.icon)
                        .unwrap_or_else(|| IconKind::Svg("check".into())),
                ),
            };
            list = list.child(
                div()
                    .flex_none()
                    .debug_selector(move || selector.clone())
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
                    word: None,
                })
            })
            .collect();
        Some((head, rows))
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
        let view = self.one_window_hosted_view()?;
        let label = view.label();
        let mut out = vec![
            ElementInfo::input(
                "filter",
                Some(self.filter_text.as_str()),
                self.focused_input != FocusedInput::None,
            ),
            header_element(0, &label),
        ];
        out.extend(
            view_rows(view)
                .into_iter()
                .enumerate()
                .map(|(ix, row)| ElementInfo {
                    semantic_id: row.id,
                    ..ElementInfo::redacted_choice(
                        ix,
                        &row.title,
                        &row.title,
                        row.selected,
                        ElementContentKind::UserContent,
                    )
                }),
        );
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
        Some("copy") => "copy",
        Some("append" | "edit") => "edit",
        Some("open") => "arrowright",
        Some("undo") => "close",
        Some("again") => "play",
        Some("script") => "code",
        _ => "check",
    }
}
