//! Canonical Actions Dialog state fixtures.
//!
//! Visual-style variants remain adoptable separately; this module captures
//! supported command-bar states with the shared live presenter.

use gpui::{div, prelude::*, px, AnyElement, SharedString};

use crate::storybook::{
    render_actions_dialog_presentation, resolve_actions_dialog_style,
    ActionsDialogPresentationAction, ActionsDialogPresentationItem, ActionsDialogPresentationModel,
    StoryVariant,
};
use crate::theme::get_cached_theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionsDialogStateId {
    DefaultList,
    SearchFiltered,
    DestructiveSelection,
    EmptySearch,
    MouseHover,
    BottomSearch,
}

impl ActionsDialogStateId {
    pub const ALL: [Self; 6] = [
        Self::DefaultList,
        Self::SearchFiltered,
        Self::DestructiveSelection,
        Self::EmptySearch,
        Self::MouseHover,
        Self::BottomSearch,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DefaultList => "default-list",
            Self::SearchFiltered => "search-filtered",
            Self::DestructiveSelection => "destructive-selection",
            Self::EmptySearch => "empty-search",
            Self::MouseHover => "mouse-hover",
            Self::BottomSearch => "bottom-search",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::DefaultList => "Default List",
            Self::SearchFiltered => "Search Filtered",
            Self::DestructiveSelection => "Destructive Selection",
            Self::EmptySearch => "Empty Search",
            Self::MouseHover => "Mouse Hover",
            Self::BottomSearch => "Bottom Search",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::DefaultList => "Full command list with the first action selected.",
            Self::SearchFiltered => "Search query filters the list to matching actions.",
            Self::DestructiveSelection => "Danger section with a destructive action selected.",
            Self::EmptySearch => "No matching actions state.",
            Self::MouseHover => "Mouse-input state with a hovered row separate from selection.",
            Self::BottomSearch => "Legacy bottom-anchored search layout.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "default-list" => Some(Self::DefaultList),
            "search-filtered" => Some(Self::SearchFiltered),
            "destructive-selection" => Some(Self::DestructiveSelection),
            "empty-search" => Some(Self::EmptySearch),
            "mouse-hover" => Some(Self::MouseHover),
            "bottom-search" => Some(Self::BottomSearch),
            _ => None,
        }
    }
}

pub fn actions_dialog_state_story_variants() -> Vec<StoryVariant> {
    ActionsDialogStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "actionsDialog")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_actions_dialog_state_preview(stable_id: &str) -> AnyElement {
    let id = ActionsDialogStateId::from_stable_id(stable_id)
        .unwrap_or(ActionsDialogStateId::DefaultList);
    render_actions_dialog_state(id, false)
}

pub fn render_actions_dialog_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = ActionsDialogStateId::from_stable_id(stable_id)
        .unwrap_or(ActionsDialogStateId::DefaultList);
    render_actions_dialog_state(id, true)
}

fn render_actions_dialog_state(id: ActionsDialogStateId, compact: bool) -> AnyElement {
    let (style, _) = resolve_actions_dialog_style(Some("current"));
    let theme = get_cached_theme();
    let width = if compact { 360.0 } else { 430.0 };
    let model = state_model(id);

    div()
        .w_full()
        .min_h(px(if compact { 220.0 } else { 300.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .child(render_actions_dialog_presentation(&model, style, &theme)),
        )
        .into_any_element()
}

fn state_model(id: ActionsDialogStateId) -> ActionsDialogPresentationModel {
    match id {
        ActionsDialogStateId::DefaultList => base_model("", 0, None, true, true, false),
        ActionsDialogStateId::SearchFiltered => ActionsDialogPresentationModel {
            search_text: "copy".into(),
            selected_index: 0,
            items: vec![
                section("Clipboard"),
                action(
                    "Copy Note As",
                    Some("Copy current note in another format"),
                    Some("⌘C"),
                    false,
                ),
                action(
                    "Copy Markdown Link",
                    Some("Copy note deeplink"),
                    Some("⌘⇧C"),
                    false,
                ),
            ],
            ..base_model("copy", 0, None, true, true, false)
        },
        ActionsDialogStateId::DestructiveSelection => base_model("", 3, None, true, true, false),
        ActionsDialogStateId::EmptySearch => ActionsDialogPresentationModel {
            search_text: "zzzz".into(),
            selected_index: 0,
            cursor_visible: true,
            items: vec![section("No matching actions")],
            ..base_model("zzzz", 0, None, true, true, false)
        },
        ActionsDialogStateId::MouseHover => base_model("", 1, Some(2), true, true, true),
        ActionsDialogStateId::BottomSearch => base_model("open", 0, None, true, false, false),
    }
}

fn base_model(
    search_text: &'static str,
    selected_index: usize,
    hovered_index: Option<usize>,
    show_search: bool,
    search_at_top: bool,
    input_mode_mouse: bool,
) -> ActionsDialogPresentationModel {
    ActionsDialogPresentationModel {
        context_title: Some("Current script".into()),
        search_text: search_text.into(),
        search_placeholder: "Search actions".into(),
        cursor_visible: true,
        show_search,
        search_at_top,
        show_footer: false,
        selected_index,
        hovered_index,
        input_mode_mouse,
        items: vec![
            section("Script"),
            action("Run", Some("Execute the selected script"), Some("↵"), false),
            action(
                "Open in Editor",
                Some("Edit the script source"),
                Some("⌘O"),
                false,
            ),
            action(
                "Reveal in Finder",
                Some("Show the script file"),
                Some("⌘R"),
                false,
            ),
            section("Danger"),
            action("Move to Trash", Some("Remove this script"), None, true),
        ],
    }
}

fn section(label: &'static str) -> ActionsDialogPresentationItem {
    ActionsDialogPresentationItem::SectionHeader(SharedString::from(label))
}

fn action(
    title: &'static str,
    subtitle: Option<&'static str>,
    shortcut: Option<&'static str>,
    is_destructive: bool,
) -> ActionsDialogPresentationItem {
    ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
        title: title.into(),
        subtitle: subtitle.map(SharedString::from),
        shortcut: shortcut.map(SharedString::from),
        icon_svg_path: None,
        is_destructive,
    })
}
