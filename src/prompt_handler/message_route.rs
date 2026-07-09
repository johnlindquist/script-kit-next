#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptMessageRoute {
    ConfirmDialog,
    UnhandledWarning,
    Other,
}
#[inline]
fn classify_prompt_message_route(message: &PromptMessage) -> PromptMessageRoute {
    match message {
        PromptMessage::ShowConfirm { .. } => PromptMessageRoute::ConfirmDialog,
        PromptMessage::UnhandledMessage { .. } => PromptMessageRoute::UnhandledWarning,
        _ => PromptMessageRoute::Other,
    }
}

fn prompt_message_from_protocol_message(
    message: crate::protocol::Message,
) -> Option<PromptMessage> {
    match message {
        Message::Arg {
            id,
            placeholder,
            choices,
            actions,
        } => Some(PromptMessage::ShowArg {
            id,
            placeholder,
            choices,
            actions,
        }),
        Message::Div {
            id,
            html,
            container_classes,
            actions,
            placeholder,
            hint,
            footer,
            container_bg,
            container_padding,
            opacity,
        } => Some(PromptMessage::ShowDiv {
            id,
            html,
            container_classes,
            actions,
            placeholder,
            hint,
            footer,
            container_bg,
            container_padding,
            opacity,
        }),
        Message::Form { id, html, actions } => Some(PromptMessage::ShowForm { id, html, actions }),
        Message::Fields {
            id,
            fields,
            actions,
        } => Some(PromptMessage::ShowFields {
            id,
            fields,
            actions,
        }),
        Message::Term {
            id,
            command,
            actions,
        } => Some(PromptMessage::ShowTerm {
            id,
            command,
            actions,
        }),
        Message::Editor {
            id,
            content,
            language,
            template,
            on_init: _,
            on_submit: _,
            actions,
        } => Some(PromptMessage::ShowEditor {
            id,
            content,
            language,
            template,
            actions,
        }),
        Message::Path {
            id,
            start_path,
            hint,
        } => Some(PromptMessage::ShowPath {
            id,
            start_path,
            hint,
        }),
        Message::Env {
            id,
            key,
            prompt,
            title,
            secret,
        } => Some(PromptMessage::ShowEnv {
            id,
            key,
            prompt,
            title,
            secret: secret.unwrap_or(false),
        }),
        Message::Drop { id } => Some(PromptMessage::ShowDrop {
            id,
            placeholder: None,
            hint: None,
        }),
        Message::Hotkey { id, placeholder } => Some(PromptMessage::ShowHotkey { id, placeholder }),
        Message::Template { id, template } => Some(PromptMessage::ShowTemplate { id, template }),
        Message::Select {
            id,
            placeholder,
            choices,
            multiple,
        } => Some(PromptMessage::ShowSelect {
            id,
            placeholder: Some(placeholder),
            choices,
            multiple: multiple.unwrap_or(false),
        }),
        Message::Micro {
            id,
            placeholder,
            choices,
        } => Some(PromptMessage::ShowMicro {
            id,
            placeholder,
            choices,
        }),
        Message::Chat {
            id,
            placeholder,
            messages,
            hint,
            footer,
            actions,
            model,
            models,
            save_history,
            use_builtin_ai,
        } => Some(PromptMessage::ShowChat {
            id,
            placeholder,
            messages,
            hint,
            footer,
            actions,
            model,
            models,
            save_history,
            use_builtin_ai,
        }),
        Message::ChatMessage { id, message } => Some(PromptMessage::ChatAddMessage { id, message }),
        Message::ChatStreamStart {
            id,
            message_id,
            position,
        } => Some(PromptMessage::ChatStreamStart {
            id,
            message_id,
            position,
        }),
        Message::ChatStreamChunk {
            id,
            message_id,
            chunk,
        } => Some(PromptMessage::ChatStreamChunk {
            id,
            message_id,
            chunk,
        }),
        Message::ChatStreamComplete { id, message_id } => {
            Some(PromptMessage::ChatStreamComplete { id, message_id })
        }
        Message::ChatClear { id } => Some(PromptMessage::ChatClear { id }),
        Message::ChatSetError {
            id,
            message_id,
            error,
        } => Some(PromptMessage::ChatSetError {
            id,
            message_id,
            error,
        }),
        Message::ChatClearError { id, message_id } => {
            Some(PromptMessage::ChatClearError { id, message_id })
        }
        Message::Webcam { id } => Some(PromptMessage::WebcamComingSoon { id }),
        Message::Mic { id } => Some(PromptMessage::MicComingSoon { id }),
        Message::GetState {
            request_id,
            target,
            summary_only,
        } => Some(PromptMessage::GetState {
            request_id,
            target,
            summary_only,
        }),
        Message::GetElements {
            request_id,
            limit,
            target,
        } => Some(PromptMessage::GetElements {
            request_id,
            limit,
            target,
        }),
        Message::GetAgentChatState { request_id, target } => {
            Some(PromptMessage::GetAgentChatState { request_id, target })
        }
        Message::PerformAgentChatSetupAction {
            request_id,
            action,
            agent_id,
            target,
        } => Some(PromptMessage::PerformAgentChatSetupAction {
            request_id,
            action,
            agent_id,
            target,
        }),
        Message::ResetAgentChatTestProbe { request_id, target } => {
            Some(PromptMessage::ResetAgentChatTestProbe { request_id, target })
        }
        Message::GetAgentChatTestProbe {
            request_id,
            tail,
            target,
        } => Some(PromptMessage::GetAgentChatTestProbe {
            request_id,
            tail,
            target,
        }),
        Message::GetLayoutInfo { request_id, target } => {
            Some(PromptMessage::GetLayoutInfo { request_id, target })
        }
        Message::InspectAutomationWindow {
            request_id,
            target,
            hi_dpi,
            probes,
        } => Some(PromptMessage::InspectAutomationWindow {
            request_id,
            target,
            hi_dpi,
            probes,
        }),
        Message::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
            trace,
            target,
        } => Some(PromptMessage::WaitFor {
            request_id,
            condition,
            timeout,
            poll_interval,
            trace,
            target,
        }),
        Message::Batch {
            request_id,
            commands,
            options,
            trace,
            target,
        } => Some(PromptMessage::Batch {
            request_id,
            commands,
            options,
            trace,
            target,
        }),
        Message::SimulateGpuiEvent {
            request_id,
            target,
            event,
        } => Some(PromptMessage::SimulateGpuiEvent {
            request_id,
            target,
            event,
        }),
        // Allow stdin/devtools to show HUD pills directly, matching the script
        // path; probes use this to exercise HUD stacking/dismissal behavior.
        Message::Hud { text, duration_ms } => Some(PromptMessage::ShowHud { text, duration_ms }),
        _ => None,
    }
}
