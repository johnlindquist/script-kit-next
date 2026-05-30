use super::*;

impl ScriptListApp {
    pub(crate) fn open_creation_feedback_surface(
        &mut self,
        path: Option<std::path::PathBuf>,
        cx: &mut Context<Self>,
    ) {
        let path = path.unwrap_or_else(|| {
            std::path::PathBuf::from("/tmp/script-kit-liquid-glass-feedback-fixture.ts")
        });

        self.transition_current_view_and_rekey_main_automation_surface(AppView::CreationFeedback {
            path,
        });
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);
        cx.notify();
    }
}
