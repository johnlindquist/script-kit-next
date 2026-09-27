/// What an action hands the model's chat: words that go now or wait in the input, and an image
/// or file that waits in the line above it.
enum DeferredAgentChatAction {
    OpenOnly,
    SetInput {
        text: String,
        submit: bool,
    },
    SetInputWithImage {
        text: String,
        image_base64: String,
        submit: bool,
    },
    AddAttachment {
        path: String,
    },
    ApplyPreset {
        preset_id: String,
    },
}

impl DeferredAgentChatAction {
    fn name(&self) -> &'static str {
        match self {
            Self::OpenOnly => "open_only",
            Self::SetInput { .. } => "set_input",
            Self::SetInputWithImage { .. } => "set_input_with_image",
            Self::AddAttachment { .. } => "add_attachment",
            Self::ApplyPreset { .. } => "apply_preset",
        }
    }

    fn failure_message(&self, error: impl std::fmt::Display) -> String {
        match self {
            Self::OpenOnly | Self::SetInput { .. } => format!("Couldn't open the chat: {error}"),
            Self::SetInputWithImage { .. } => format!("Couldn't attach the image: {error}"),
            Self::AddAttachment { .. } => format!("Couldn't attach the file: {error}"),
            Self::ApplyPreset { .. } => format!("Couldn't apply the AI preset: {error}"),
        }
    }

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
