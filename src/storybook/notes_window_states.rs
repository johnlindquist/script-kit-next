//! Presenter-backed Notes window states for Storybook.
//!
//! The live Notes window owns storage and focus side effects, so these fixtures
//! mirror the visual shell with deterministic data instead of constructing
//! `NotesApp` against the user's real notes database.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight};

use crate::notes::window::style::NotesWindowStyle;
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::{get_vibrancy_background, HexColorExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotesWindowStateId {
    Editor,
    HoverActions,
    Search,
    FormatToolbar,
    MarkdownPreview,
    Empty,
    Trash,
    AcpHost,
}

impl NotesWindowStateId {
    pub const ALL: [Self; 8] = [
        Self::Editor,
        Self::HoverActions,
        Self::Search,
        Self::FormatToolbar,
        Self::MarkdownPreview,
        Self::Empty,
        Self::Trash,
        Self::AcpHost,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::HoverActions => "hover-actions",
            Self::Search => "search",
            Self::FormatToolbar => "format-toolbar",
            Self::MarkdownPreview => "markdown-preview",
            Self::Empty => "empty",
            Self::Trash => "trash",
            Self::AcpHost => "acp-host",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Editor => "Editor",
            Self::HoverActions => "Hover Actions",
            Self::Search => "Search",
            Self::FormatToolbar => "Format Toolbar",
            Self::MarkdownPreview => "Markdown Preview",
            Self::Empty => "Empty",
            Self::Trash => "Trash",
            Self::AcpHost => "ACP Host",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Editor => "Normal single-note editor with quiet chrome and footer stats.",
            Self::HoverActions => "Window hover state with titlebar action affordances visible.",
            Self::Search => "Find/search bar open above the current note.",
            Self::FormatToolbar => "Formatting toolbar pinned above the editor.",
            Self::MarkdownPreview => "Rendered markdown preview mode.",
            Self::Empty => "No notes available state.",
            Self::Trash => "Deleted-note view with restore and permanent delete controls.",
            Self::AcpHost => "Embedded Agent Chat surface hosted inside the Notes window.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "editor" => Some(Self::Editor),
            "hover-actions" => Some(Self::HoverActions),
            "search" => Some(Self::Search),
            "format-toolbar" => Some(Self::FormatToolbar),
            "markdown-preview" => Some(Self::MarkdownPreview),
            "empty" => Some(Self::Empty),
            "trash" => Some(Self::Trash),
            "acp-host" => Some(Self::AcpHost),
            _ => None,
        }
    }
}

pub fn notes_window_state_story_variants() -> Vec<StoryVariant> {
    NotesWindowStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "notesWindow")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_notes_window_state_preview(stable_id: &str) -> AnyElement {
    let id = NotesWindowStateId::from_stable_id(stable_id).unwrap_or(NotesWindowStateId::Editor);
    render_notes_window_state(id, false)
}

pub fn render_notes_window_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = NotesWindowStateId::from_stable_id(stable_id).unwrap_or(NotesWindowStateId::Editor);
    render_notes_window_state(id, true)
}

fn render_notes_window_state(id: NotesWindowStateId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let style = NotesWindowStyle::current();
    let width = if compact { 420.0 } else { 620.0 };
    let height = if compact { 300.0 } else { 420.0 };

    let is_trash = id == NotesWindowStateId::Trash;
    let is_preview = id == NotesWindowStateId::MarkdownPreview;
    let is_hovered = matches!(
        id,
        NotesWindowStateId::HoverActions
            | NotesWindowStateId::Search
            | NotesWindowStateId::FormatToolbar
            | NotesWindowStateId::MarkdownPreview
            | NotesWindowStateId::Trash
            | NotesWindowStateId::AcpHost
    );

    let root = div()
        .w_full()
        .min_h(px(if compact { 320.0 } else { 460.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .h(px(height))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x80))
                .when_some(get_vibrancy_background(&theme), |d, bg| d.bg(bg))
                .flex()
                .flex_col()
                .text_color(theme.colors.text.primary.to_rgb())
                .when(id == NotesWindowStateId::AcpHost, |d| {
                    d.child(render_acp_titlebar(style, compact))
                        .child(render_acp_body(compact))
                        .child(render_acp_footer(compact))
                })
                .when(id != NotesWindowStateId::AcpHost, |d| {
                    d.child(render_notes_titlebar(
                        id, style, is_hovered, is_trash, is_preview, compact,
                    ))
                    .when(id == NotesWindowStateId::Search, |d| {
                        d.child(render_search_bar(compact))
                    })
                    .when(id == NotesWindowStateId::FormatToolbar, |d| {
                        d.child(render_format_toolbar(compact))
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .px(px(style.editor_padding_x))
                            .py(px(style.editor_padding_y))
                            .child(render_notes_body(id, compact)),
                    )
                    .when(id != NotesWindowStateId::Empty, |d| {
                        d.child(render_notes_footer(id, style, is_hovered, compact))
                    })
                }),
        );

    root.into_any_element()
}

fn render_notes_titlebar(
    id: NotesWindowStateId,
    style: NotesWindowStyle,
    hovered: bool,
    is_trash: bool,
    is_preview: bool,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let title = match id {
        NotesWindowStateId::Empty => "No note selected",
        NotesWindowStateId::Trash => "Trash / Deleted draft",
        _ => "Daily launch notes",
    };
    let right_width = if compact { 86.0 } else { 100.0 };

    div()
        .h(px(style.titlebar_height))
        .px(px(12.0))
        .flex()
        .items_center()
        .when(is_trash, |d| {
            d.border_b_1()
                .border_color(rgba((theme.colors.ui.warning << 8) | 0x59))
        })
        .child(div().w(px(60.0)).flex_shrink_0())
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(4.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_sm()
                .text_color(rgba(
                    (theme.colors.text.secondary << 8) | if hovered { 0xFF } else { 0xB3 },
                ))
                .when(id == NotesWindowStateId::HoverActions, |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(theme.colors.accent.selected.to_rgb())
                            .child("●"),
                    )
                })
                .child(title),
        )
        .child(render_titlebar_actions(
            right_width,
            hovered,
            is_trash,
            is_preview,
            compact,
        ))
        .into_any_element()
}

fn render_titlebar_actions(
    width: f32,
    hovered: bool,
    is_trash: bool,
    is_preview: bool,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let mut actions = div()
        .w(px(width))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(if compact { 3.0 } else { 6.0 }));

    if is_trash {
        actions = actions
            .child(action_chip("Restore", true))
            .child(icon_chip("⌫", false));
    } else if hovered {
        actions = actions
            .child(icon_chip("⌘", false))
            .child(icon_chip("≡", false))
            .child(icon_chip(if is_preview { "MD" } else { "TXT" }, is_preview))
            .child(icon_chip("+", false));
    }

    actions
        .text_color(rgba((theme.colors.text.secondary << 8) | 0xB3))
        .into_any_element()
}

fn render_search_bar(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .px(px(12.0))
        .pt(px(8.0))
        .pb(px(8.0))
        .child(
            div()
                .h(px(30.0))
                .rounded(px(8.0))
                .bg(rgba((theme.colors.background.search_box << 8) | 0xCC))
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x44))
                .px(px(10.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgba((theme.colors.text.muted << 8) | 0xB3))
                        .child("⌕"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .font_family(crate::list_item::FONT_MONO)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(if compact { "daily" } else { "daily launch" }),
                )
                .child(
                    div()
                        .h(px(18.0))
                        .px(px(7.0))
                        .rounded(px(9.0))
                        .bg(rgba((theme.colors.accent.selected_subtle << 8) | 0x80))
                        .text_xs()
                        .text_color(rgba((theme.colors.text.secondary << 8) | 0xB3))
                        .flex()
                        .items_center()
                        .child("2/4"),
                ),
        )
        .into_any_element()
}

fn render_format_toolbar(compact: bool) -> AnyElement {
    let labels: &[&str] = if compact {
        &["B", "I", "H", "•", "1.", "`", "☐"]
    } else {
        &[
            "B", "I", "H", "•", "1.", "`", "```", "S̶", "☐", "↗", "―", ">",
        ]
    };

    let mut toolbar = div()
        .w_full()
        .px(px(12.0))
        .pb(px(8.0))
        .flex()
        .items_center()
        .gap(px(4.0));

    for label in labels {
        toolbar = toolbar.child(action_chip(label, false));
    }

    toolbar.into_any_element()
}

fn render_notes_body(id: NotesWindowStateId, compact: bool) -> AnyElement {
    match id {
        NotesWindowStateId::Empty => render_empty_body(compact),
        NotesWindowStateId::MarkdownPreview => render_markdown_preview_body(compact),
        NotesWindowStateId::Trash => render_editor_text_body(
            "# Deleted draft\n\nThis note is in trash and can be restored before it is permanently removed.",
            0x99,
            compact,
        ),
        _ => render_editor_text_body(
            "# Daily launch notes\n\n- Verify Storybook canonical states\n- Check popup surfaces\n- Commit logical slices\n\nNext: cover Notes and ACP windows.",
            0xE6,
            compact,
        ),
    }
}

fn render_empty_body(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(if compact { 8.0 } else { 12.0 }))
        .child(
            div()
                .text_base()
                .text_color(rgba((theme.colors.text.secondary << 8) | 0xB3))
                .child("No notes yet"),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.accent.selected.to_rgb())
                .child("Create your first note"),
        )
        .when(!compact, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.0))
                    .pt(px(8.0))
                    .text_xs()
                    .text_color(rgba((theme.colors.text.muted << 8) | 0xB3))
                    .child("⌘N  new")
                    .child("⌘⇧N  from clipboard")
                    .child("⌘/  shortcuts"),
            )
        })
        .into_any_element()
}

fn render_editor_text_body(content: &str, alpha: u8, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let mut body = div()
        .w_full()
        .h_full()
        .overflow_hidden()
        .font_family(crate::list_item::FONT_MONO)
        .text_size(px(if compact { 12.0 } else { 13.0 }))
        .line_height(px(if compact { 18.0 } else { 20.0 }))
        .text_color(rgba((theme.colors.text.primary << 8) | u32::from(alpha)))
        .flex()
        .flex_col()
        .gap(px(5.0));

    for line in content.lines() {
        body = body.child(div().child(line.to_string()));
    }

    body.into_any_element()
}

fn render_markdown_preview_body(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .h_full()
        .overflow_hidden()
        .flex()
        .flex_col()
        .gap(px(if compact { 8.0 } else { 12.0 }))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child("Daily launch notes"),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(20.0))
                .text_color(rgba((theme.colors.text.secondary << 8) | 0xE6))
                .child("Verify the state catalog, then keep moving through app windows."),
        )
        .child(render_preview_bullet(
            "Storybook catalog reports canonical coverage.",
        ))
        .child(render_preview_bullet(
            "Popup and compact surfaces have deterministic fixtures.",
        ))
        .when(!compact, |d| {
            d.child(
                div()
                    .mt(px(4.0))
                    .rounded(px(6.0))
                    .bg(rgba((theme.colors.background.search_box << 8) | 0x88))
                    .border_1()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x44))
                    .p(px(10.0))
                    .font_family(crate::list_item::FONT_MONO)
                    .text_xs()
                    .text_color(rgba((theme.colors.text.secondary << 8) | 0xCC))
                    .child("target/debug/storybook --catalog-json"),
            )
        })
        .into_any_element()
}

fn render_preview_bullet(label: &'static str) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .flex()
        .items_start()
        .gap(px(8.0))
        .text_sm()
        .text_color(rgba((theme.colors.text.secondary << 8) | 0xDD))
        .child(
            div()
                .text_color(theme.colors.accent.selected.to_rgb())
                .child("•"),
        )
        .child(label)
        .into_any_element()
}

fn render_notes_footer(
    id: NotesWindowStateId,
    style: NotesWindowStyle,
    hovered: bool,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let text_alpha = if hovered { 0xFF } else { 0x80 };
    let middle = match id {
        NotesWindowStateId::Trash => "128 chars",
        NotesWindowStateId::MarkdownPreview => "6 min read · 612 chars",
        NotesWindowStateId::Search => "Ln 3/8 · 2 matches",
        _ => "Ln 6/9 · 24 words · 168 chars",
    };

    div()
        .h(px(style.footer_height))
        .px(px(12.0))
        .border_t_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x33))
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_xs()
        .text_color(rgba((theme.colors.text.secondary << 8) | text_alpha))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(5.0))
                .child(if id == NotesWindowStateId::Trash {
                    "1/3"
                } else {
                    "2/8"
                })
                .when(!compact, |d| d.child("‹").child("›")),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .justify_center()
                .child(middle),
        )
        .child(
            div()
                .flex_shrink_0()
                .child(if id == NotesWindowStateId::Trash {
                    "deleted Jan 24"
                } else {
                    "updated ↓"
                }),
        )
        .into_any_element()
}

fn render_acp_titlebar(style: NotesWindowStyle, compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .h(px(style.titlebar_height))
        .px(px(12.0))
        .flex()
        .items_center()
        .child(div().w(px(60.0)).flex_shrink_0())
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .text_sm()
                .child(
                    div()
                        .text_color(rgba((theme.colors.text.secondary << 8) | 0xB3))
                        .child("Notes"),
                )
                .child(
                    div()
                        .text_color(rgba((theme.colors.text.muted << 8) | 0x80))
                        .child("/"),
                )
                .child(
                    div()
                        .text_color(theme.colors.accent.selected.to_rgb())
                        .child("Agent"),
                ),
        )
        .child(
            div()
                .w(px(if compact { 76.0 } else { 100.0 }))
                .flex()
                .justify_end()
                .child(icon_chip("⌘", false)),
        )
        .into_any_element()
}

fn render_acp_body(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .flex_1()
        .min_h(px(0.0))
        .px(px(if compact { 14.0 } else { 20.0 }))
        .py(px(if compact { 10.0 } else { 16.0 }))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(chat_line("user", "Summarize this note and pull out the next actions."))
        .child(chat_line(
            "assistant",
            "The note is a Storybook coverage pass. Next actions: verify catalog JSON, capture visual states, and commit each slice.",
        ))
        .child(div().flex_1())
        .child(
            div()
                .h(px(36.0))
                .rounded(px(8.0))
                .bg(rgba((theme.colors.background.search_box << 8) | 0xAA))
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x44))
                .px(px(12.0))
                .flex()
                .items_center()
                .text_sm()
                .text_color(rgba((theme.colors.text.muted << 8) | 0xB3))
                .child("Ask about this note..."),
        )
        .into_any_element()
}

fn render_acp_footer(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .h(px(if compact { 28.0 } else { 32.0 }))
        .px(px(12.0))
        .border_t_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x33))
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(rgba((theme.colors.text.secondary << 8) | 0x99))
        .child("Claude Sonnet")
        .child("↵ send · ⌘↵ run")
        .into_any_element()
}

fn chat_line(role: &'static str, body: &'static str) -> AnyElement {
    let theme = get_cached_theme();
    let is_user = role == "user";

    div()
        .w_full()
        .flex()
        .when(is_user, |d| d.justify_end())
        .when(!is_user, |d| d.justify_start())
        .child(
            div()
                .max_w(px(430.0))
                .rounded(px(8.0))
                .px(px(10.0))
                .py(px(8.0))
                .bg(rgba(
                    ((if is_user {
                        theme.colors.accent.selected_subtle
                    } else {
                        theme.colors.background.title_bar
                    }) << 8)
                        | 0xAA,
                ))
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x33))
                .text_sm()
                .line_height(px(19.0))
                .text_color(rgba((theme.colors.text.primary << 8) | 0xE6))
                .child(body),
        )
        .into_any_element()
}

fn icon_chip(label: &'static str, active: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .min_w(px(24.0))
        .h(px(24.0))
        .px(px(5.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(if active {
            rgba((theme.colors.accent.selected << 8) | 0xFF)
        } else {
            rgba((theme.colors.text.secondary << 8) | 0xB3)
        })
        .when(active, |d| {
            d.bg(rgba((theme.colors.accent.selected_subtle << 8) | 0x66))
        })
        .child(label)
        .into_any_element()
}

fn action_chip(label: &'static str, active: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .h(px(24.0))
        .px(px(8.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(if active {
            rgba((theme.colors.accent.selected << 8) | 0xFF)
        } else {
            rgba((theme.colors.text.secondary << 8) | 0xB3)
        })
        .when(active, |d| {
            d.bg(rgba((theme.colors.accent.selected_subtle << 8) | 0x66))
        })
        .child(label)
        .into_any_element()
}
