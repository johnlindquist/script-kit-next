use super::*;

impl ScriptListApp {
    /// Switch the main route and re-key the main automation `semanticSurface`
    /// from the new [`AppView`] contract.
    pub(crate) fn transition_current_view_and_rekey_main_automation_surface(
        &mut self,
        next_view: AppView,
    ) -> bool {
        self.current_view = next_view;
        self.rekey_main_automation_surface_from_current_view()
    }

    /// Restore a previously captured main route and focus target.
    ///
    /// This deliberately does not re-key automation or emit notifications.
    /// Callers that close child windows or Agent Chat surfaces still own those
    /// side-effects in their local route contract.
    pub(crate) fn restore_current_view_with_focus(
        &mut self,
        next_view: AppView,
        focus_target: FocusTarget,
    ) {
        self.current_view = next_view;
        self.pending_focus = Some(focus_target);
        self.focused_input = match focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };
    }

    /// Return the main route to ScriptList and target the shared filter input.
    ///
    /// Use this for ScriptList entries that already handle their own caches,
    /// sizing, and notifications locally. The helper still re-keys the main
    /// automation surface because the view/focus pair is observable by agents.
    pub(crate) fn show_script_list_with_main_filter_focus(&mut self) -> bool {
        self.restore_current_view_with_focus(AppView::ScriptList, FocusTarget::MainFilter);
        self.rekey_main_automation_surface_from_current_view()
    }

    /// Re-key the main window automation `semanticSurface` from the active
    /// `AppView` contract without replacing the whole automation window record.
    pub(crate) fn rekey_main_automation_surface_from_current_view(&self) -> bool {
        let semantic_surface = crate::semantic_surface_for_main_view(&self.current_view);
        crate::windows::update_automation_semantic_surface("main", semantic_surface)
    }
}
