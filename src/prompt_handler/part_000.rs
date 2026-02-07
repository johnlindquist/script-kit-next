fn unhandled_message_warning(message_type: &str) -> String {
    format!(
        "'{}' is not supported yet. Update the script to a supported message type or update Script Kit GPUI.",
        message_type
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptMessageRoute {
    ConfirmWindow,
    UnhandledWarning,
    Other,
}
#[inline]
fn classify_prompt_message_route(message: &PromptMessage) -> PromptMessageRoute {
    match message {
        PromptMessage::ShowConfirm { .. } => PromptMessageRoute::ConfirmWindow,
        PromptMessage::UnhandledMessage { .. } => PromptMessageRoute::UnhandledWarning,
        _ => PromptMessageRoute::Other,
    }
}
