include!("deferred_agent_chat_action.rs");

impl DeferredAgentChatAction {
    fn words(text: String, submit: bool) -> crate::one_window::chat_entry::ChatEntry {
        use crate::one_window::chat_entry::ChatEntry;
        if submit && !text.trim().is_empty() {
            ChatEntry::sending(text)
        } else {
            ChatEntry::seeded(text)
        }
    }

    fn entry(self) -> Result<crate::one_window::chat_entry::ChatEntry, String> {
        use crate::one_window::chat_entry::ChatEntry;
        Ok(match self {
            Self::OpenOnly => ChatEntry::quick(),
            Self::SetInput { text, submit } => Self::words(text, submit),
            Self::SetInputWithImage {
                text,
                image_base64,
                submit,
            } => {
                use base64::Engine as _;
                let png_bytes = base64::engine::general_purpose::STANDARD
                    .decode(image_base64)
                    .map_err(|error| error.to_string())?;
                let path = std::env::temp_dir().join(format!(
                    "script-kit-agent_chat-clipboard-{}.png",
                    uuid::Uuid::new_v4()
                ));
                std::fs::write(&path, png_bytes).map_err(|error| error.to_string())?;
                Self::words(text, submit).with(crate::ai::AiContextPart::FilePath {
                    path: path.to_string_lossy().into_owned(),
                    label: "Clipboard image".to_string(),
                })
            }
            Self::AddAttachment { path } => {
                let label = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| path.clone());
                ChatEntry::quick().with(crate::ai::AiContextPart::FilePath { path, label })
            }
            Self::ApplyPreset { preset_id } => ChatEntry::seeded(
                crate::ai::presets::resolve_agent_chat_preset(&preset_id)?.system_prompt,
            ),
        })
    }
}
