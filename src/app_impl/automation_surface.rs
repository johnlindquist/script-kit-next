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

    /// Log top-level launcher view transitions once per active variant.
    ///
    /// View assignment is intentionally spread across route owners. Sampling at
    /// render time records the observable surface without logging hot-path
    /// selection/filter churn.
    pub(crate) fn log_current_view_transition_if_changed(&mut self, source: &'static str) {
        let current_view = self.current_view.app_view_variant();
        if self.last_logged_app_view_variant == Some(current_view) {
            return;
        }

        let previous_view = self.last_logged_app_view_variant.unwrap_or("unknown");
        self.last_logged_app_view_variant = Some(current_view);
        let contract = self.current_view.surface_contract();
        tracing::info!(
            event_type = "main_view_transition",
            source,
            previous_view,
            current_view,
            surface_kind = ?self.current_view.surface_kind(),
            native_footer_surface = ?self.current_view.native_footer_surface(),
            surface_family = ?contract.vocabulary.family,
            input_ownership = ?contract.vocabulary.input_ownership,
            preview_role = ?contract.vocabulary.preview_role,
            focus_policy = ?contract.focus_policy,
            keyboard_policy = ?contract.keyboard_policy,
            actions_policy = ?contract.actions_policy,
            proof_policy = ?contract.proof_policy,
            visual_policy = ?contract.visual_policy,
            automation_semantic_surface = contract.automation_semantic_surface,
            "Main view transition"
        );
    }

    /// Re-key the main window automation `semanticSurface` from the active
    /// `AppView` contract without replacing the whole automation window record.
    pub(crate) fn rekey_main_automation_surface_from_current_view(&self) -> bool {
        let semantic_surface = crate::semantic_surface_for_main_view(&self.current_view);
        crate::windows::update_automation_semantic_surface("main", semantic_surface)
    }
}
