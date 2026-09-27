//! What's staged in the main window, listed in place under the main input: the reducer's header,
//! then the staged thing's actions as the main list's own rows. ⌘K in any view and a command bar
//! hosted in the main window draw their actions through this one list; nothing opens.

use std::rc::Rc;

use gpui::{
    div, list, prelude::*, px, AnyElement, App, ElementId, Entity, ListSizingBehavior, Window,
};

use crate::actions::{ActionsDialog, GroupedActionItem};
use crate::designs::icon_variations::IconName;
use crate::list_item::{render_section_header, IconKind, ListItem, ListItemColors};

pub(crate) const STAGED_ACTIONS_ID: &str = "one-window-staged-actions";

/// The semantic id a staged action's row paints and is projected under.
pub(crate) fn staged_action_semantic_id(action_id: &str) -> String {
    format!("staged-action:{action_id}")
}

/// One action as its row shows it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StagedActionRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) icon: String,
    pub(crate) selected: bool,
}

/// The dialog's actions as its row for each entry of `grouped_items`: `None` for the dialog's
/// own section headers, which the in-place list leaves out under its single header.
pub(crate) fn staged_action_slots(dialog: &ActionsDialog) -> Vec<Option<StagedActionRow>> {
    let selected = dialog.get_selected_action_id();
    dialog
        .grouped_items
        .iter()
        .map(|item| match item {
            GroupedActionItem::Item(filtered) => dialog
                .filtered_actions
                .get(*filtered)
                .and_then(|index| dialog.actions.get(*index))
                .map(|action| StagedActionRow {
                    id: action.id.clone(),
                    title: action.title.clone(),
                    description: action.description.clone().filter(|text| !text.is_empty()),
                    icon: format!("{:?}", action.icon.unwrap_or(IconName::Settings)),
                    selected: selected.as_deref() == Some(action.id.as_str()),
                }),
            GroupedActionItem::SectionHeader(_) => None,
        })
        .collect()
}

/// The staged actions in the order the keys walk them.
pub(crate) fn staged_action_rows(dialog: &ActionsDialog) -> Vec<StagedActionRow> {
    staged_action_slots(dialog).into_iter().flatten().collect()
}

/// The header and rows. `on_click` runs a row's action by id, the way Enter does. The list scrolls
/// with the dialog's own list state, so moving the selection keeps the row in view.
pub(crate) fn render_staged_actions(
    dialog: &Entity<ActionsDialog>,
    head: &str,
    theme: &crate::theme::Theme,
    on_click: impl Fn(String, &mut Window, &mut App) + 'static,
    cx: &App,
) -> AnyElement {
    let main_menu_theme = crate::designs::current_main_menu_theme();
    let colors = ListItemColors::from_theme(theme);
    let dialog = dialog.read(cx);
    let slots = Rc::new(staged_action_slots(dialog));
    let on_click = Rc::new(on_click);
    let row_height = crate::list_item::effective_list_item_height_for_theme(main_menu_theme);
    let rows = list(dialog.list_state.clone(), move |ix, _window, _cx| {
        let Some(Some(row)) = slots.get(ix) else {
            return div().into_any_element();
        };
        let semantic_id = staged_action_semantic_id(&row.id);
        let selector = format!("list-row:{semantic_id}");
        let id = row.id.clone();
        let on_click = on_click.clone();
        div()
            .id(ElementId::Name(semantic_id.clone().into()))
            .debug_selector(move || selector)
            .h(px(row_height))
            .on_click(move |_, window, cx| on_click(id.clone(), window, cx))
            .child(
                ListItem::new(row.title.clone(), colors)
                    .index(ix)
                    .semantic_id(semantic_id)
                    .icon_kind(IconKind::Svg(row.icon.clone()))
                    .description_opt(row.description.clone())
                    .selected(row.selected)
                    .main_menu_theme(main_menu_theme),
            )
            .into_any_element()
    })
    .with_sizing_behavior(ListSizingBehavior::Infer)
    .flex_1()
    .min_h(px(0.0));

    div()
        .id(STAGED_ACTIONS_ID)
        .debug_selector(|| STAGED_ACTIONS_ID.to_string())
        .flex()
        .flex_col()
        .flex_1()
        .min_h(px(0.0))
        .w_full()
        .child(
            div()
                .flex_none()
                .debug_selector(|| "oneWindowSectionHeader:0".to_string())
                .h(px(
                    crate::list_item::effective_first_section_header_height_for_theme(
                        main_menu_theme,
                    ),
                ))
                .child(render_section_header(head, None, colors, true)),
        )
        .child(rows)
        .into_any_element()
}
