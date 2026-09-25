use super::*;

fn focus_for_about_restore(view: &AppView) -> (FocusTarget, FocusedInput) {
    match view {
        AppView::ActionsDialog => (FocusTarget::ActionsDialog, FocusedInput::ActionsSearch),
        view if matches!(
            view.surface_contract().vocabulary.input_ownership,
            LauncherSurfaceInputOwnership::LauncherFilter
        ) =>
        {
            (FocusTarget::MainFilter, FocusedInput::MainFilter)
        }
        _ => (FocusTarget::AppRoot, FocusedInput::None),
    }
}

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

        self.transition_current_view_and_rekey_main_automation_surface(AppView::About {
            previous,
            state: crate::about::AboutState::new(),
            update_state,
        });
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::AppRoot);
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

        self.transition_current_view_and_rekey_main_automation_surface(previous);
        let (focus_target, focused_input) = focus_for_about_restore(&self.current_view);
        self.pending_focus = Some(focus_target);
        self.focused_input = focused_input;
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(cx);
        }
        cx.notify();
    }

    pub(crate) fn toggle_about_acknowledgements(&mut self, cx: &mut Context<Self>) {
        if let AppView::About { state, .. } = &mut self.current_view {
            state.acks_open = !state.acks_open;
            cx.notify();
        }
    }

    /// Open a modal over the main window: `open` receives the main window
    /// once it is on screen (a script may have just asked for it to show).
    /// Deferred, so it never re-enters an update that already holds the main
    /// window and any caller can use it. Does nothing if no main window
    /// exists.
    pub(crate) fn open_main_window_modal(
        &mut self,
        open: impl FnOnce(&mut Window, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |_, cx| {
            for _ in 0..40 {
                if script_kit_gpui::is_main_window_visible() {
                    break;
                }
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(25))
                    .await;
            }
            cx.update(|cx| {
                let Some(main) = crate::get_main_window_handle() else {
                    tracing::warn!(event = "main_window_modal_without_main_window");
                    return;
                };
                if let Err(error) = main.update(cx, move |_, window, cx| open(window, cx)) {
                    tracing::warn!(event = "main_window_modal_open_failed", %error);
                }
            });
        })
        .detach();
    }

    /// Ask a yes/no question in the shared confirm modal over the main
    /// window. `on_result` gets `true` for the primary button and `false` for
    /// a dismissal. It never runs if the modal closes because
    /// `keep_open_while` turned false (say, the asking script exited).
    pub(crate) fn open_main_window_confirm(
        &mut self,
        options: crate::confirm::ParentConfirmOptions,
        keep_open_while: impl Fn() -> bool + 'static,
        on_result: impl Fn(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let on_result = std::rc::Rc::new(on_result);
        let (confirm, cancel) = (on_result.clone(), on_result);
        self.open_main_window_modal(
            move |window, cx| {
                crate::confirm::open_parent_confirm_dialog_with_lifecycle(
                    window,
                    cx,
                    options,
                    keep_open_while,
                    move |_, cx| confirm(true, cx),
                    move |_, cx| cancel(false, cx),
                );
            },
            cx,
        );
    }
}
