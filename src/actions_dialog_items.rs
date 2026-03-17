#[derive(Clone, Copy)]
pub(crate) struct ActionsDialogItem {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) const ACTIONS_DIALOG_ITEMS: &[ActionsDialogItem] = &[
    ActionsDialogItem {
        id: "create_script",
        title: "Create Script",
        description: "Start a new script from the current launcher context.",
    },
    ActionsDialogItem {
        id: "edit_script",
        title: "Edit Script",
        description: "Open the selected script in the configured editor.",
    },
    ActionsDialogItem {
        id: "reload",
        title: "Reload",
        description: "Reload scripts, bins, and local state.",
    },
    ActionsDialogItem {
        id: "settings",
        title: "Settings",
        description: "Open Script Kit preferences.",
    },
    ActionsDialogItem {
        id: "quit",
        title: "Quit",
        description: "Close the launcher immediately.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn actions_dialog_items_have_unique_ids() {
        let mut ids = BTreeSet::new();
        for item in ACTIONS_DIALOG_ITEMS {
            assert!(ids.insert(item.id), "duplicate action id: {}", item.id);
        }
    }

    #[test]
    fn actions_dialog_items_all_have_descriptions() {
        for item in ACTIONS_DIALOG_ITEMS {
            assert!(
                !item.description.is_empty(),
                "action {} has empty description",
                item.id
            );
        }
    }

    #[test]
    fn actions_dialog_items_ids_are_snake_case() {
        for item in ACTIONS_DIALOG_ITEMS {
            assert!(
                item.id.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "action id '{}' is not snake_case",
                item.id
            );
        }
    }
}
