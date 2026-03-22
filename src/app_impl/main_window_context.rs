use super::*;

/// Request payload for launching AI from the main window context rail.
#[derive(Debug, Clone)]
pub(crate) struct MainWindowAiLaunchRequest {
    pub message: String,
    pub parts: Vec<crate::ai::message_parts::AiContextPart>,
    pub submit: bool,
}

impl ScriptListApp {
    /// Returns the default set of context parts for the main window rail.
    ///
    /// Order: Current Context, Selection, Browser URL, Focused Window.
    pub(crate) fn default_main_window_context_parts(
    ) -> Vec<crate::ai::message_parts::AiContextPart> {
        let parts = vec![
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Current Context".to_string(),
            },
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0".to_string(),
                label: "Selection".to_string(),
            },
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0".to_string(),
                label: "Browser URL".to_string(),
            },
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1".to_string(),
                label: "Focused Window".to_string(),
            },
        ];
        tracing::info!(
            event = "main_window_context_defaults_init",
            count = parts.len(),
            labels = ?parts.iter().map(|p| p.label()).collect::<Vec<_>>(),
            "Initialized default main window context parts"
        );
        parts
    }

    /// Toggle a context part: removes it if already present (by equality), appends otherwise.
    /// Preserves insertion order. Always calls `cx.notify()`.
    pub(crate) fn toggle_main_window_context_part(
        &mut self,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
        let label = part.label().to_string();
        if let Some(ix) = self
            .main_window_context_parts
            .iter()
            .position(|existing| existing == &part)
        {
            self.main_window_context_parts.remove(ix);
            tracing::info!(
                event = "main_window_context_toggle",
                action = "remove",
                label = %label,
                remaining = self.main_window_context_parts.len(),
                remaining_labels = ?self.main_window_context_parts.iter().map(|p| p.label()).collect::<Vec<_>>(),
                "Removed context part from main window rail"
            );
        } else {
            self.main_window_context_parts.push(part);
            tracing::info!(
                event = "main_window_context_toggle",
                action = "add",
                label = %label,
                total = self.main_window_context_parts.len(),
                all_labels = ?self.main_window_context_parts.iter().map(|p| p.label()).collect::<Vec<_>>(),
                "Added context part to main window rail"
            );
        }
        cx.notify();
    }

    /// Clear all selected context parts and reset preview state.
    /// Always calls `cx.notify()`.
    #[allow(dead_code)]
    pub(crate) fn clear_main_window_context_parts(&mut self, cx: &mut Context<Self>) {
        let previous_count = self.main_window_context_parts.len();
        self.main_window_context_parts.clear();
        self.main_window_context_preview_index = None;
        tracing::info!(
            event = "main_window_context_clear",
            previous_count,
            "Cleared all main window context parts"
        );
        cx.notify();
    }

    /// Build an AI launch request from the current main window context state.
    /// Returns `None` if no context parts are selected.
    pub(crate) fn build_main_window_ai_launch_request(
        &self,
        message: String,
        submit: bool,
    ) -> Option<MainWindowAiLaunchRequest> {
        if self.main_window_context_parts.is_empty() {
            tracing::info!(
                event = "main_window_ai_launch_skip",
                reason = "no_context_parts",
                "Skipping AI launch: no context parts selected"
            );
            return None;
        }
        let request = MainWindowAiLaunchRequest {
            message: message.clone(),
            parts: self.main_window_context_parts.clone(),
            submit,
        };
        tracing::info!(
            event = "main_window_ai_launch_request",
            message_len = message.len(),
            part_count = request.parts.len(),
            part_labels = ?request.parts.iter().map(|p| p.label()).collect::<Vec<_>>(),
            part_uris = ?request.parts.iter().map(|p| p.source()).collect::<Vec<_>>(),
            submit,
            "Built main window AI launch request"
        );
        Some(request)
    }
}
