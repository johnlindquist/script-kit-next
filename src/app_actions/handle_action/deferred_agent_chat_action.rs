/// What an action hands the model's chat: words that go now or wait in the input, and an image
/// or file that waits in the line above it.
pub enum DeferredAgentChatAction {
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
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenOnly => "open_only",
            Self::SetInput { .. } => "set_input",
            Self::SetInputWithImage { .. } => "set_input_with_image",
            Self::AddAttachment { .. } => "add_attachment",
            Self::ApplyPreset { .. } => "apply_preset",
        }
    }

    pub fn failure_message(&self, error: impl std::fmt::Display) -> String {
        match self {
            Self::OpenOnly | Self::SetInput { .. } => format!("Couldn't open the chat: {error}"),
            Self::SetInputWithImage { .. } => format!("Couldn't attach the image: {error}"),
            Self::AddAttachment { .. } => format!("Couldn't attach the file: {error}"),
            Self::ApplyPreset { .. } => format!("Couldn't apply the AI preset: {error}"),
        }
    }
}

#[cfg(test)]
mod deferred_agent_chat_action_tests {
    use super::DeferredAgentChatAction;

    #[test]
    fn each_handoff_says_what_failed_in_its_own_words() {
        let cases = [
            (
                DeferredAgentChatAction::OpenOnly,
                "open_only",
                "Couldn't open the chat: offline",
            ),
            (
                DeferredAgentChatAction::SetInput {
                    text: "hi".to_string(),
                    submit: true,
                },
                "set_input",
                "Couldn't open the chat: offline",
            ),
            (
                DeferredAgentChatAction::SetInputWithImage {
                    text: "look".to_string(),
                    image_base64: "AAAA".to_string(),
                    submit: false,
                },
                "set_input_with_image",
                "Couldn't attach the image: offline",
            ),
            (
                DeferredAgentChatAction::AddAttachment {
                    path: "/tmp/a.txt".to_string(),
                },
                "add_attachment",
                "Couldn't attach the file: offline",
            ),
            (
                DeferredAgentChatAction::ApplyPreset {
                    preset_id: "writer".to_string(),
                },
                "apply_preset",
                "Couldn't apply the AI preset: offline",
            ),
        ];
        for (action, name, message) in cases {
            assert_eq!(action.name(), name);
            assert_eq!(action.failure_message("offline"), message);
        }
    }

    #[test]
    fn the_failure_prefix_is_the_message_with_nothing_after_it() {
        let action = DeferredAgentChatAction::AddAttachment {
            path: "/tmp/a.txt".to_string(),
        };
        let prefix = action.failure_message("");
        assert_eq!(
            format!("{prefix}denied"),
            action.failure_message("denied"),
            "the line joins the prefix and the error, so the error must come last"
        );
    }
}
