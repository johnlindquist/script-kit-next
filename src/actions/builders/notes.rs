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

/// Get actions for the Notes window command bar (Cmd+K menu).
pub fn get_notes_command_bar_actions(info: &NotesInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    actions.push(
        Action::new(
            "new_note",
            "New Note",
            Some("Create a new note".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Notes"),
    );

    if info.has_selection && !info.is_trash_view {
        actions.push(
            Action::new(
                "duplicate_note",
                "Duplicate Note",
                Some("Create a copy of the current note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘D")
            .with_icon(IconName::Copy)
            .with_section("Notes"),
        );
    }

    actions.push(
        Action::new(
            "browse_notes",
            "Browse Notes",
            Some("Open note browser/picker".to_string()),
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
                Some("Search within current note".to_string()),
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
                Some("Open formatting toolbar".to_string()),
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
                "Copy Note As...",
                Some("Copy note in a chosen format".to_string()),
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
                Some("Copy a deeplink to the note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘D")
            .with_icon(IconName::ArrowRight)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "create_quicklink",
                "Create Quicklink",
                Some("Copy a markdown quicklink".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⇧⌘L")
            .with_icon(IconName::Star)
            .with_section("Copy"),
        );

        actions.push(
            Action::new(
                "export",
                "Export...",
                Some("Export note content".to_string()),
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
                Some("Window grows/shrinks with content".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘A")
            .with_icon(IconName::Settings)
            .with_section("Settings"),
        );
    }

    actions
}

/// Build actions for the AI new chat dropdown.
#[allow(dead_code)]
pub fn get_new_chat_actions(
    last_used: &[NewChatModelInfo],
    presets: &[NewChatPresetInfo],
    models: &[NewChatModelInfo],
) -> Vec<Action> {
    let mut actions = Vec::new();

    for (idx, setting) in last_used.iter().enumerate() {
        actions.push(
            Action::new(
                format!("last_used_{}", idx),
                &setting.display_name,
                Some(setting.provider_display_name.clone()),
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
                None,
                ActionCategory::ScriptContext,
            )
            .with_section("Presets")
            .with_icon(preset.icon),
        );
    }

    for (idx, model) in models.iter().enumerate() {
        actions.push(
            Action::new(
                format!("model_{}", idx),
                &model.display_name,
                Some(model.provider_display_name.clone()),
                ActionCategory::ScriptContext,
            )
            .with_section("Models")
            .with_icon(IconName::Settings),
        );
    }

    actions
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
                Some("Press ⌘N to create a new note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_icon(IconName::Plus)
            .with_section("Notes"),
        );
    }

    actions
}
