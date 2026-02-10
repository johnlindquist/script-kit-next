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

fn has_missing_chat_model_fields(model: &ChatModelInfo) -> bool {
    model.id.trim().is_empty()
        || model.display_name.trim().is_empty()
        || model.provider.trim().is_empty()
}

/// Get actions specific to a chat prompt.
pub fn get_chat_context_actions(info: &ChatPromptInfo) -> Vec<Action> {
    let has_blank_current_model = info
        .current_model
        .as_ref()
        .map(|model| model.trim().is_empty())
        .unwrap_or(false);
    if has_blank_current_model {
        tracing::warn!(
            target: "script_kit::actions",
            model_count = info.available_models.len(),
            has_messages = info.has_messages,
            has_response = info.has_response,
            "Invalid chat prompt info: current model name is blank; returning no actions"
        );
        return Vec::new();
    }

    let invalid_model_count = info
        .available_models
        .iter()
        .filter(|model| has_missing_chat_model_fields(model))
        .count();
    if invalid_model_count > 0 {
        tracing::warn!(
            target: "script_kit::actions",
            invalid_model_count,
            model_count = info.available_models.len(),
            has_messages = info.has_messages,
            has_response = info.has_response,
            "Invalid chat prompt info: model metadata missing required fields; returning no actions"
        );
        return Vec::new();
    }

    let mut actions = Vec::new();

    for model in &info.available_models {
        let is_current = info
            .current_model
            .as_ref()
            .map(|m| m == &model.display_name)
            .unwrap_or(false);

        let action = Action::new(
            format!("chat:select_model_{}", model.id),
            if is_current {
                format!("{} ✓", model.display_name)
            } else {
                model.display_name.clone()
            },
            Some(format!("Uses {}", model.provider)),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Settings);
        actions.push(action);
    }

    actions.push(
        Action::new(
            "chat:continue_in_chat",
            "Continue in Chat",
            Some("Opens the AI chat window".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵")
        .with_icon(IconName::MessageCircle),
    );

    if info.has_response {
        actions.push(
            Action::new(
                "chat:copy_response",
                "Copy Last Response",
                Some("Copies the last assistant response".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘C")
            .with_icon(IconName::Copy),
        );
    }

    if info.has_messages {
        actions.push(
            Action::new(
                "chat:clear_conversation",
                "Clear Conversation",
                Some("Clears all messages".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌫")
            .with_icon(IconName::Trash),
        );
    }

    actions
}

/// Get actions for the AI chat command bar (Cmd+K menu).
#[allow(dead_code)]
pub fn get_ai_command_bar_actions() -> Vec<Action> {
    vec![
        Action::new(
            "chat:copy_response",
            "Copy Response",
            Some("Copies the latest AI response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "chat:copy_chat",
            "Copy Chat",
            Some("Copies the full conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        Action::new(
            "chat:copy_last_code",
            "Copy Last Code Block",
            Some("Copies the latest code block".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⌘C")
        .with_icon(IconName::Code)
        .with_section("Response"),
        Action::new(
            "chat:submit",
            "Submit",
            Some("Sends your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
        Action::new(
            "chat:new_chat",
            "New Chat",
            Some("Starts a new conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Actions"),
        Action::new(
            "chat:delete_chat",
            "Delete Chat",
            Some("Deletes the current conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫")
        .with_icon(IconName::Trash)
        .with_section("Actions"),
        Action::new(
            "chat:add_attachment",
            "Add Attachments...",
            Some("Attaches files to your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘A")
        .with_icon(IconName::Plus)
        .with_section("Attachments"),
        Action::new(
            "chat:paste_image",
            "Paste Image from Clipboard",
            Some("Pastes an image from the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘V")
        .with_icon(IconName::File)
        .with_section("Attachments"),
        Action::new(
            "chat:export_markdown",
            "Export as Markdown",
            Some("Exports chat as Markdown to the clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘E")
        .with_icon(IconName::FileCode)
        .with_section("Export"),
        Action::new(
            "chat:branch_from_last",
            "Branch from Last Message",
            Some("Creates a new chat from the last message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::ArrowRight)
        .with_section("Actions"),
        Action::new(
            "chat:toggle_shortcuts_help",
            "Keyboard Shortcuts",
            Some("Shows keyboard shortcuts".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘/")
        .with_icon(IconName::Star)
        .with_section("Help"),
        Action::new(
            "chat:change_model",
            "Change Model",
            Some("Selects a different AI model".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Settings)
        .with_section("Settings"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prompt_info() -> ChatPromptInfo {
        ChatPromptInfo {
            current_model: Some("GPT-5".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-5".to_string(),
                    display_name: "GPT-5".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "claude-4".to_string(),
                    display_name: "Claude 4".to_string(),
                    provider: "Anthropic".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        }
    }

    #[test]
    fn test_get_chat_context_actions_prefixes_ids_with_chat_namespace() {
        let actions = get_chat_context_actions(&sample_prompt_info());
        assert!(actions.iter().all(|action| action.id.starts_with("chat:")));
    }

    #[test]
    fn test_get_ai_command_bar_actions_prefixes_ids_with_chat_namespace() {
        let actions = get_ai_command_bar_actions();
        assert!(actions.iter().all(|action| action.id.starts_with("chat:")));
    }

    #[test]
    fn test_get_chat_context_actions_returns_empty_when_model_metadata_missing() {
        let mut info = sample_prompt_info();
        info.available_models[0].id = "   ".to_string();

        let actions = get_chat_context_actions(&info);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_chat_context_actions_returns_empty_when_current_model_is_blank() {
        let mut info = sample_prompt_info();
        info.current_model = Some("   ".to_string());

        let actions = get_chat_context_actions(&info);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_get_chat_context_actions_assigns_consistent_primary_icons() {
        let actions = get_chat_context_actions(&sample_prompt_info());

        let select_model = actions
            .iter()
            .find(|action| action.id == "chat:select_model_gpt-5")
            .expect("missing select_model action");
        let continue_in_chat = actions
            .iter()
            .find(|action| action.id == "chat:continue_in_chat")
            .expect("missing continue_in_chat action");
        let copy_response = actions
            .iter()
            .find(|action| action.id == "chat:copy_response")
            .expect("missing copy_response action");
        let clear_conversation = actions
            .iter()
            .find(|action| action.id == "chat:clear_conversation")
            .expect("missing clear_conversation action");

        assert_eq!(select_model.icon, Some(IconName::Settings));
        assert_eq!(continue_in_chat.icon, Some(IconName::MessageCircle));
        assert_eq!(copy_response.icon, Some(IconName::Copy));
        assert_eq!(clear_conversation.icon, Some(IconName::Trash));
    }
}
