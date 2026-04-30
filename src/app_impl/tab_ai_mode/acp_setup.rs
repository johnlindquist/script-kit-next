use super::*;

impl ScriptListApp {
    /// Shared low-level helper: create an ACP setup view from a pre-built
    /// `AcpInlineSetupState`, wire footer callbacks, switch the current view,
    /// and notify the UI. Both error-path helpers delegate here.
    fn show_embedded_acp_setup_view(
        &mut self,
        source_view: AppView,
        setup: crate::ai::acp::AcpInlineSetupState,
        cx: &mut Context<Self>,
    ) {
        let view_entity = cx.new(|cx| crate::ai::acp::AcpChatView::new_setup(setup, cx));
        self.wire_embedded_acp_footer_callbacks(&view_entity, cx);
        self.tab_ai_harness_return_view = Some(source_view);
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());
        self.current_view = AppView::AcpChatView {
            entity: view_entity,
        };
        crate::windows::ensure_embedded_ai_window(true);
        crate::windows::update_automation_semantic_surface(
            "main",
            crate::semantic_surface_for_main_view(&self.current_view),
        );
        self.transition_acp_surface(
            crate::ai::acp::surface_state::AcpSurfaceEvent::EmbeddedOpened,
        );
        self.focused_input = FocusedInput::None;
        self.show_actions_popup = false;
        self.actions_dialog = None;
        self.pending_focus = Some(FocusTarget::ChatPrompt);
        cx.notify();
    }

    /// Show a setup card when the ACP agent catalog fails to load.
    pub(super) fn show_acp_catalog_load_failed_setup_view(
        &mut self,
        source_view: AppView,
        error: String,
        cx: &mut Context<Self>,
    ) {
        tracing::error!(
            target: "script_kit::tab_ai",
            event = "acp_catalog_load_failed",
            error = %error,
        );
        let setup = crate::ai::acp::AcpInlineSetupState {
            reason_code: "catalogLoadFailed",
            title: "Failed to load ACP catalog".into(),
            body: format!("{error}").into(),
            primary_action: crate::ai::acp::AcpSetupAction::OpenCatalog,
            secondary_action: Some(crate::ai::acp::AcpSetupAction::Retry),
            selected_agent: None,
            catalog_entries: Vec::new(),
            launch_requirements: crate::ai::acp::AcpLaunchRequirements::default(),
        };
        self.show_embedded_acp_setup_view(source_view, setup, cx);
    }

    /// Show a setup card when ACP launch is blocked (missing key, model
    /// capability, etc.).
    pub(super) fn show_acp_launch_blocked_setup_view(
        &mut self,
        source_view: AppView,
        acp_launch_resolution: &crate::ai::acp::AcpLaunchResolution,
        requirements: crate::ai::acp::AcpLaunchRequirements,
        cx: &mut Context<Self>,
    ) {
        let setup = crate::ai::acp::AcpInlineSetupState::from_resolution(
            acp_launch_resolution,
            requirements,
        );
        self.show_embedded_acp_setup_view(source_view, setup, cx);
    }
}
