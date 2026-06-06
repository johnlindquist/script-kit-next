#![allow(dead_code)] // Used by the binary target through include!()-merged app code.

pub(crate) const ACTIONS_POPUP_KITCHEN_SINK_FIXTURE_ID: &str = "actions-popup-kitchen-sink";
pub(crate) const ACTIONS_POPUP_KITCHEN_SINK_NO_MATCH_QUERY: &str =
    "zzzz-actions-popup-kitchen-sink-no-match";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionsPopupKitchenSinkMode {
    Populated,
    NoMatch,
}

pub(crate) fn actions_popup_kitchen_sink_feature_manifest() -> &'static [&'static str] {
    &[
        "shell:popup-window",
        "search:visible",
        "list:scroll-overflow",
        "row:selected",
        "row:hover-worthy",
        "row:long-title",
        "row:long-description",
        "row:destructive",
        "row:unsectioned",
        "section:headers",
        "context-header:long-title",
        "shortcut:none",
        "shortcut:single-keycap",
        "shortcut:multi-token",
        "shortcut:long-string",
        "icon:present",
        "icon:absent",
        "empty:no-match",
    ]
}

pub(crate) fn actions_popup_kitchen_sink_config(
    mode: ActionsPopupKitchenSinkMode,
) -> crate::actions::ActionsDialogConfig {
    crate::actions::ActionsDialogConfig {
        search_position: crate::actions::SearchPosition::Bottom,
        section_style: crate::actions::SectionStyle::Headers,
        anchor: crate::actions::AnchorPosition::Bottom,
        show_icons: true,
        show_context_header: true,
        search_placeholder: Some(match mode {
            ActionsPopupKitchenSinkMode::Populated => "Search kitchen sink actions".to_string(),
            ActionsPopupKitchenSinkMode::NoMatch => "No matching kitchen sink actions".to_string(),
        }),
        ..crate::actions::ActionsDialogConfig::default()
    }
}

pub(crate) fn actions_popup_kitchen_sink_actions() -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};
    use crate::designs::icon_variations::IconName;

    let sections = [
        "Primary",
        "Edit",
        "Navigation",
        "Clipboard",
        "Danger Zone",
        "Diagnostics",
    ];
    let mut actions = Vec::new();
    actions.push(
        Action::new(
            "kitchen-sink-unsectioned",
            "Unsectioned Kitchen Sink Action",
            Some("This action intentionally has no section and no shortcut.".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::BoltFilled),
    );

    for (section_index, section) in sections.iter().enumerate() {
        for item_index in 0..5 {
            let action_index = section_index * 5 + item_index;
            let mut action = Action::new(
                format!("kitchen-sink-action-{action_index}"),
                actions_kitchen_sink_title(section, item_index),
                Some(actions_kitchen_sink_description(section, item_index)),
                ActionCategory::ScriptContext,
            )
            .with_section(*section);

            action = match item_index {
                0 => action.with_shortcut("cmd+k").with_icon(IconName::Terminal),
                1 => action
                    .with_shortcut("shift+cmd+k")
                    .with_icon(IconName::Pencil),
                2 => action.with_shortcut("cmd+shift+option+control+k"),
                3 => action.with_icon(IconName::Warning),
                _ => action,
            };

            if *section == "Danger Zone" && item_index == 0 {
                action.id = "delete_kitchen_sink_fixture".to_string();
                action.title = "Delete Kitchen Sink Fixture".to_string();
                action.title_lower = action.title.to_lowercase();
            }

            actions.push(action);
        }
    }

    actions
}

fn actions_kitchen_sink_title(section: &str, item_index: usize) -> String {
    match (section, item_index) {
        ("Primary", 2) => {
            "Very Long Actions Popup Kitchen Sink Title That Should Truncate Or Wrap Cleanly"
                .to_string()
        }
        ("Diagnostics", 3) => "Kitchen Sink Punctuation ! ? / @ : Action".to_string(),
        _ => format!("{section} Kitchen Sink Action {}", item_index + 1),
    }
}

fn actions_kitchen_sink_description(section: &str, item_index: usize) -> String {
    if section == "Primary" && item_index == 2 {
        return "Long description for row padding, row gap, typography, shortcut alignment, icon spacing, selected state, and hover state coverage in the real ActionsDialog renderer.".to_string();
    }
    format!("{section} fixture description row {}", item_index + 1)
}
