use super::*;

impl ScriptListApp {
    pub(crate) fn open_about_surface(
        &mut self,
        update_state: std::sync::Arc<std::sync::RwLock<crate::updates::UpdateState>>,
        cx: &mut Context<Self>,
    ) {
        let previous = match &self.current_view {
            AppView::About { previous, .. } => previous.clone(),
            _ => Box::new(self.current_view.clone()),
        };

        self.current_view = AppView::About {
            previous,
            state: crate::about::AboutState::new(),
            update_state,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);
        crate::windows::update_automation_semantic_surface("main", Some("about".to_string()));
        cx.notify();
    }

    pub(crate) fn dismiss_about(&mut self, cx: &mut Context<Self>) {
        let previous = match std::mem::replace(&mut self.current_view, AppView::ScriptList) {
            AppView::About { previous, .. } => *previous,
            other => {
                self.current_view = other;
                return;
            }
        };

        self.current_view = previous;
        self.pending_focus = Some(match &self.current_view {
            AppView::ScriptList => FocusTarget::MainFilter,
            _ => FocusTarget::AppRoot,
        });
        self.focused_input = if matches!(self.current_view, AppView::ScriptList) {
            FocusedInput::MainFilter
        } else {
            FocusedInput::None
        };
        let semantic = semantic_surface_for_main_view(&self.current_view);
        crate::windows::update_automation_semantic_surface("main", semantic);
        cx.notify();
    }

    pub(crate) fn toggle_about_acknowledgements(&mut self, cx: &mut Context<Self>) {
        if let AppView::About { state, .. } = &mut self.current_view {
            state.acks_open = !state.acks_open;
            cx.notify();
        }
    }

    /// Push an in-window confirm prompt as a state of the main window.
    ///
    /// Replaces the current launcher view; restored to the same view on
    /// confirm/cancel via [`Self::resolve_confirm_prompt`].
    pub(crate) fn open_confirm_prompt(
        &mut self,
        options: crate::confirm::ParentConfirmOptions,
        sender: async_channel::Sender<bool>,
        cx: &mut Context<Self>,
    ) {
        let previous = match &self.current_view {
            AppView::ConfirmPrompt { previous, .. } => previous.clone(),
            _ => Box::new(self.current_view.clone()),
        };
        self.current_view = AppView::ConfirmPrompt {
            options,
            sender,
            focused_button: ConfirmFocusedButton::default(),
            previous,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);
        crate::windows::update_automation_semantic_surface("main", Some("confirmPrompt".into()));
        cx.notify();
    }
}
