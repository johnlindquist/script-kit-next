use super::types::{Action, ActionCategory};
use crate::designs::icon_variations::IconName;

/// Information about a chat prompt for action building
#[derive(Debug, Clone)]
pub struct ChatPromptInfo {
    pub current_model: Option<String>,
    pub available_models: Vec<ChatModelInfo>,
    pub has_messages: bool,
    pub has_response: bool,
}

/// Information about an available chat model
#[derive(Debug, Clone)]
pub struct ChatModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider: String,
}

/// Get actions specific to a chat prompt.
pub fn get_chat_context_actions(info: &ChatPromptInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    for model in &info.available_models {
        let is_current = info
            .current_model
            .as_ref()
            .map(|m| m == &model.display_name)
            .unwrap_or(false);

        let action = Action::new(
            format!("select_model_{}", model.id),
            if is_current {
                format!("{} ✓", model.display_name)
            } else {
                model.display_name.clone()
            },
            Some(format!("via {}", model.provider)),
            ActionCategory::ScriptContext,
        );
        actions.push(action);
    }

    actions.push(
        Action::new(
            "continue_in_chat",
            "Continue in Chat",
            Some("Open in AI Chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    if info.has_response {
        actions.push(
            Action::new(
                "copy_response",
                "Copy Last Response",
                Some("Copy the last assistant response".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘C"),
        );
    }

    if info.has_messages {
        actions.push(
            Action::new(
                "clear_conversation",
                "Clear Conversation",
                Some("Clear all messages".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌫"),
        );
    }

    actions
}

/// Get actions for the AI chat command bar (Cmd+K menu).
#[allow(dead_code)]
pub fn get_ai_command_bar_actions() -> Vec<Action> {
    vec![
        Action::new(
            "copy_response",
            "Copy Response",
            Some("Copy the last AI response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "copy_chat",
            "Copy Chat",
            Some("Copy the entire conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "copy_last_code",
            "Copy Last Code Block",
            Some("Copy the most recent code block".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⌘C")
        .with_icon(IconName::Code)
        .with_section("Response"),
        Action::new(
            "submit",
            "Submit",
            Some("Send your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
        Action::new(
            "new_chat",
            "New Chat",
            Some("Start a new conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Actions"),
        Action::new(
            "delete_chat",
            "Delete Chat",
            Some("Delete current conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫")
        .with_icon(IconName::Trash)
        .with_section("Actions"),
        Action::new(
            "add_attachment",
            "Add Attachments...",
            Some("Attach files to your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘A")
        .with_icon(IconName::Plus)
        .with_section("Attachments"),
        Action::new(
            "paste_image",
            "Paste Image from Clipboard",
            Some("Paste an image from your clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘V")
        .with_icon(IconName::File)
        .with_section("Attachments"),
        Action::new(
            "export_markdown",
            "Export as Markdown",
            Some("Export chat to clipboard as Markdown".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E")
        .with_icon(IconName::FileCode)
        .with_section("Export"),
        Action::new(
            "branch_from_last",
            "Branch from Last Message",
            Some("Create a new chat branching from the last message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Actions"),
        Action::new(
            "toggle_shortcuts_help",
            "Keyboard Shortcuts",
            Some("Show keyboard shortcuts overlay".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘/")
        .with_icon(IconName::Star)
        .with_section("Help"),
        Action::new(
            "change_model",
            "Change Model",
            Some("Select a different AI model".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Settings)
        .with_section("Settings"),
    ]
}
