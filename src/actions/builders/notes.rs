use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;

/// Information about a model for the new chat dropdown
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NewChatModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub provider: String,
    pub provider_display_name: String,
}

/// Information about a preset for the new chat dropdown
#[derive(Debug, Clone)]
pub struct NewChatPresetInfo {
    pub id: String,
    pub name: String,
    pub icon: IconName,
}

/// Information about notes for action building
#[derive(Debug, Clone)]
pub struct NotesInfo {
    pub has_selection: bool,
    pub is_trash_view: bool,
    pub auto_sizing_enabled: bool,
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn has_invalid_new_chat_model_info(model: &NewChatModelInfo) -> bool {
    is_blank(&model.model_id)
        || is_blank(&model.display_name)
        || is_blank(&model.provider)
        || is_blank(&model.provider_display_name)
}

fn has_invalid_new_chat_preset_info(preset: &NewChatPresetInfo) -> bool {
    is_blank(&preset.id) || is_blank(&preset.name)
}

fn has_invalid_note_switcher_note_info(note: &NoteSwitcherNoteInfo) -> bool {
    is_blank(&note.id) || is_blank(&note.title)
}

/// Get actions for the Notes window command bar (Cmd+K menu).
pub fn get_notes_command_bar_actions(info: &NotesInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    actions.push(
        Action::new(
            "new_note",
            "New Note",
            Some("Creates a new note".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Notes"),
    );

    if info.has_selection {
        if info.is_trash_view {
            actions.push(
                Action::new(
                    "restore_note",
                    "Restore Note",
                    Some("Restores the current note from Trash".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘Z")
                .with_icon(IconName::Refresh)
                .with_section("Trash"),
            );

            actions.push(
                Action::new(
                    "permanently_delete_note",
                    "Delete Permanently",
                    Some("Permanently deletes the current note".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_icon(IconName::Trash)
                .with_section("Trash"),
            );
        } else {
            actions.push(
                Action::new(
                    "duplicate_note",
                    "Duplicate Note",
                    Some("Creates a copy of the current note".to_string()),
                    ActionCategory::ScriptContext,
                )
                .with_shortcut("⌘D")
                .with_icon(IconName::Copy)
                .with_section("Notes"),
            );
        }
    }

    actions.push(
        Action::new(
            "browse_notes",
            "Browse Notes",
            Some("Opens the note browser".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘P")
        .with_icon(IconName::FolderOpen)
        .with_section("Notes"),
    );

    if info.has_selection && !info.is_trash_view {
        actions.push(
            Action::new(
                "find_in_note",
                "Find in Note",
                Some("Searches within the current note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘F")
            .with_icon(IconName::MagnifyingGlass)
            .with_section("Edit"),
        );

        actions.push(
            Action::new(
                "format",
                "Format...",
                Some("Opens the formatting toolbar".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘T")
            .with_icon(IconName::Code)
            .with_section("Edit"),
        );
    }

    if info.has_selection && !info.is_trash_view {
        actions.push(
            Action::new(
                "copy_note_as",
                "Copy Note as Markdown",
                Some("Copies the note as Markdown".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘C")
            .with_icon(IconName::Copy)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "copy_deeplink",
                "Copy Deeplink",
                Some("Copies a deeplink to this note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘Y")
            .with_icon(IconName::ArrowRight)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "create_quicklink",
                "Create Quicklink",
                Some("Copies a Markdown quicklink".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘L")
            .with_icon(IconName::Star)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "export",
                "Copy as HTML",
                Some("Copies the note as HTML".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘E")
            .with_icon(IconName::ArrowRight)
            .with_section("Export"),
        );
    }

    if !info.auto_sizing_enabled {
        actions.push(
            Action::new(
                "enable_auto_sizing",
                "Enable Auto-Sizing",
                Some("Resizes the window to match content".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘A")
            .with_icon(IconName::Settings)
            .with_section("Settings"),
        );
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_notes_command_bar_actions_uses_markdown_and_html_copy_labels_when_selected() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };

        let actions = get_notes_command_bar_actions(&info);
        let copy_note_as = actions
            .iter()
            .find(|action| action.id == "copy_note_as")
            .expect("missing copy_note_as action");
        let export = actions
            .iter()
            .find(|action| action.id == "export")
            .expect("missing export action");

        assert_eq!(copy_note_as.title, "Copy Note as Markdown");
        assert_eq!(
            copy_note_as.description.as_deref(),
            Some("Copies the note as Markdown")
        );
        assert!(!copy_note_as.title.ends_with("..."));

        assert_eq!(export.title, "Copy as HTML");
        assert_eq!(
            export.description.as_deref(),
            Some("Copies the note as HTML")
        );
        assert!(!export.title.ends_with("..."));
    }

    #[test]
    fn test_get_notes_command_bar_actions_sets_non_conflicting_copy_deeplink_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };

        let actions = get_notes_command_bar_actions(&info);
        let copy_deeplink = actions
            .iter()
            .find(|action| action.id == "copy_deeplink")
            .expect("missing copy_deeplink action");

        assert_eq!(copy_deeplink.shortcut.as_deref(), Some("⇧⌘Y"));
        assert_ne!(copy_deeplink.shortcut.as_deref(), Some("⇧⌘D"));
    }

    #[test]
    fn test_get_notes_command_bar_actions_includes_trash_actions_when_selected_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };

        let actions = get_notes_command_bar_actions(&info);
        let restore_note = actions
            .iter()
            .find(|action| action.id == "restore_note")
            .expect("missing restore_note action");
        let permanently_delete_note = actions
            .iter()
            .find(|action| action.id == "permanently_delete_note")
            .expect("missing permanently_delete_note action");

        assert_eq!(restore_note.title, "Restore Note");
        assert_eq!(restore_note.shortcut.as_deref(), Some("⌘Z"));
        assert_eq!(
            restore_note.description.as_deref(),
            Some("Restores the current note from Trash")
        );
        assert_eq!(restore_note.section.as_deref(), Some("Trash"));

        assert_eq!(permanently_delete_note.title, "Delete Permanently");
        assert_eq!(permanently_delete_note.shortcut.as_deref(), None);
        assert_eq!(
            permanently_delete_note.description.as_deref(),
            Some("Permanently deletes the current note")
        );
        assert_eq!(permanently_delete_note.section.as_deref(), Some("Trash"));
    }

    #[test]
    fn test_get_notes_command_bar_actions_excludes_trash_actions_when_no_selection_in_trash() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };

        let actions = get_notes_command_bar_actions(&info);

        assert!(!actions.iter().any(|action| action.id == "restore_note"));
        assert!(!actions
            .iter()
            .any(|action| action.id == "permanently_delete_note"));
    }

    #[test]
    fn test_get_new_chat_actions_uses_provider_and_model_id_for_action_ids() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4o-mini".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let models = vec![NewChatModelInfo {
            model_id: "claude-3-5-sonnet".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];

        let actions = get_new_chat_actions(&last_used, &[], &models);

        assert!(actions
            .iter()
            .any(|action| action.id == "last_used_openai::gpt-4o-mini"));
        assert!(actions
            .iter()
            .any(|action| action.id == "model_anthropic::claude-3-5-sonnet"));
    }

    #[test]
    fn test_get_new_chat_actions_model_ids_are_stable_when_order_changes() {
        let model_a = NewChatModelInfo {
            model_id: "gpt-4o".to_string(),
            display_name: "GPT-4o".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        };
        let model_b = NewChatModelInfo {
            model_id: "claude-3-5-sonnet".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        };

        let actions_a = get_new_chat_actions(&[], &[], &[model_a.clone(), model_b.clone()]);
        let actions_b = get_new_chat_actions(&[], &[], &[model_b, model_a]);

        let mut ids_a: Vec<String> = actions_a.into_iter().map(|action| action.id).collect();
        let mut ids_b: Vec<String> = actions_b.into_iter().map(|action| action.id).collect();
        ids_a.sort();
        ids_b.sort();

        assert_eq!(
            ids_a,
            vec![
                "model_anthropic::claude-3-5-sonnet".to_string(),
                "model_openai::gpt-4o".to_string(),
            ]
        );
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn test_get_new_chat_actions_includes_descriptions_for_presets() {
        let presets = vec![NewChatPresetInfo {
            id: "focused-writing".to_string(),
            name: "Focused Writing".to_string(),
            icon: IconName::Pencil,
        }];

        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset_action = actions
            .iter()
            .find(|action| action.id == "preset_focused-writing")
            .expect("missing preset action");

        assert_eq!(
            preset_action.description.as_deref(),
            Some("Uses Focused Writing preset")
        );
    }

    #[test]
    fn test_get_new_chat_actions_returns_empty_when_last_used_model_input_is_invalid() {
        let last_used = vec![NewChatModelInfo {
            model_id: "   ".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "focused-writing".to_string(),
            name: "Focused Writing".to_string(),
            icon: IconName::Pencil,
        }];

        let actions = get_new_chat_actions(&last_used, &presets, &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_new_chat_actions_returns_empty_when_preset_input_is_invalid() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4o-mini".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "   ".to_string(),
            name: "Focused Writing".to_string(),
            icon: IconName::Pencil,
        }];

        let actions = get_new_chat_actions(&last_used, &presets, &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_new_chat_actions_returns_empty_when_model_input_is_invalid() {
        let presets = vec![NewChatPresetInfo {
            id: "focused-writing".to_string(),
            name: "Focused Writing".to_string(),
            icon: IconName::Pencil,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "claude-3-5-sonnet".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            provider: "   ".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];

        let actions = get_new_chat_actions(&[], &presets, &models);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_note_switcher_actions_returns_empty_when_note_input_is_invalid() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "   ".to_string(),
            title: "Missing ID".to_string(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "Preview".to_string(),
            relative_time: "1m ago".to_string(),
        }];

        let actions = get_note_switcher_actions(&notes);
        assert!(actions.is_empty());
    }
}

/// Build actions for the AI new chat dropdown.
#[allow(dead_code)]
pub fn get_new_chat_actions(
    last_used: &[NewChatModelInfo],
    presets: &[NewChatPresetInfo],
    models: &[NewChatModelInfo],
) -> Vec<Action> {
    let invalid_last_used_count = last_used
        .iter()
        .filter(|model| has_invalid_new_chat_model_info(model))
        .count();
    if invalid_last_used_count > 0 {
        tracing::warn!(
            target: "script_kit::actions",
            builder = "notes_new_chat",
            invalid_last_used_count,
            "Invalid last-used chat model input; returning empty actions"
        );
        return vec![];
    }

    let invalid_preset_count = presets
        .iter()
        .filter(|preset| has_invalid_new_chat_preset_info(preset))
        .count();
    if invalid_preset_count > 0 {
        tracing::warn!(
            target: "script_kit::actions",
            builder = "notes_new_chat",
            invalid_preset_count,
            "Invalid chat preset input; returning empty actions"
        );
        return vec![];
    }

    let invalid_model_count = models
        .iter()
        .filter(|model| has_invalid_new_chat_model_info(model))
        .count();
    if invalid_model_count > 0 {
        tracing::warn!(
            target: "script_kit::actions",
            builder = "notes_new_chat",
            invalid_model_count,
            "Invalid chat model input; returning empty actions"
        );
        return vec![];
    }

    let mut actions = Vec::new();

    for setting in last_used {
        actions.push(
            Action::new(
                format!("last_used_{}", new_chat_model_identifier(setting)),
                &setting.display_name,
                Some(format!("Uses {}", setting.provider_display_name)),
                ActionCategory::ScriptContext,
            )
            .with_section("Last Used Settings")
            .with_icon(IconName::BoltFilled),
        );
    }

    for preset in presets {
        actions.push(
            Action::new(
                format!("preset_{}", preset.id),
                &preset.name,
                Some(new_chat_preset_description(preset)),
                ActionCategory::ScriptContext,
            )
            .with_section("Presets")
            .with_icon(preset.icon),
        );
    }

    for model in models {
        actions.push(
            Action::new(
                format!("model_{}", new_chat_model_identifier(model)),
                &model.display_name,
                Some(format!("Uses {}", model.provider_display_name)),
                ActionCategory::ScriptContext,
            )
            .with_section("Models")
            .with_icon(IconName::Settings),
        );
    }

    actions
}

fn new_chat_model_identifier(model: &NewChatModelInfo) -> String {
    format!("{}::{}", model.provider, model.model_id)
}

fn new_chat_preset_description(preset: &NewChatPresetInfo) -> String {
    format!("Uses {} preset", preset.name)
}

/// Information about a note for the note switcher dialog
#[derive(Debug, Clone)]
pub struct NoteSwitcherNoteInfo {
    pub id: String,
    pub title: String,
    pub char_count: usize,
    pub is_current: bool,
    pub is_pinned: bool,
    pub preview: String,
    pub relative_time: String,
}

/// Get actions for the note switcher dialog (Cmd+P in Notes window).
pub fn get_note_switcher_actions(notes: &[NoteSwitcherNoteInfo]) -> Vec<Action> {
    let invalid_note_count = notes
        .iter()
        .filter(|note| has_invalid_note_switcher_note_info(note))
        .count();
    if invalid_note_count > 0 {
        tracing::warn!(
            target: "script_kit::actions",
            builder = "note_switcher",
            note_count = notes.len(),
            invalid_note_count,
            "Invalid note switcher input; returning empty actions"
        );
        return vec![];
    }

    let mut actions = Vec::new();

    for note in notes {
        let icon = if note.is_pinned {
            IconName::StarFilled
        } else if note.is_current {
            IconName::Check
        } else {
            IconName::File
        };

        let title = if note.is_current {
            format!("• {}", note.title)
        } else {
            note.title.clone()
        };

        let description = if !note.preview.is_empty() {
            let preview: String = note.preview.chars().take(60).collect();
            let preview = if note.preview.chars().count() > 60 {
                format!("{}…", preview.trim_end())
            } else {
                preview
            };
            if note.relative_time.is_empty() {
                preview
            } else {
                format!("{} · {}", preview, note.relative_time)
            }
        } else if !note.relative_time.is_empty() {
            note.relative_time.clone()
        } else {
            format!(
                "{} char{}",
                note.char_count,
                if note.char_count == 1 { "" } else { "s" }
            )
        };

        let section = if note.is_pinned { "Pinned" } else { "Recent" };

        actions.push(
            Action::new(
                format!("note_{}", note.id),
                title,
                Some(description),
                ActionCategory::ScriptContext,
            )
            .with_icon(icon)
            .with_section(section),
        );
    }

    if actions.is_empty() {
        actions.push(
            Action::new(
                "no_notes",
                "No notes yet",
                Some("Creates a new note with ⌘N".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Plus)
            .with_section("Notes"),
        );
    }

    actions
}
