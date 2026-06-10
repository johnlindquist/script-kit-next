use super::*;

impl ScriptListApp {
    /// Shared low-level helper: create an Agent Chat setup view from a pre-built
    /// `AgentChatInlineSetupState`, wire footer callbacks, switch the current view,
    /// and notify the UI. Both error-path helpers delegate here.
    fn show_embedded_agent_chat_setup_view(
        &mut self,
        source_view: AppView,
        setup: crate::ai::agent_chat::ui::AgentChatInlineSetupState,
        cx: &mut Context<Self>,
    ) {
        let view_entity =
            cx.new(|cx| crate::ai::agent_chat::ui::AgentChatView::new_setup(setup, cx));
        self.wire_embedded_agent_chat_footer_callbacks(&view_entity, cx);
        self.tab_ai_harness_return_view = Some(source_view);
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());
        self.enter_embedded_agent_chat_surface(view_entity, cx);
        cx.notify();
    }

    /// Show a setup card when the Agent Chat agent catalog fails to load.
    pub(super) fn show_agent_chat_catalog_load_failed_setup_view(
        &mut self,
        source_view: AppView,
        error: String,
        cx: &mut Context<Self>,
    ) {
        tracing::error!(
            target: "script_kit::tab_ai",
            event = "agent_chat_catalog_load_failed",
            error = %error,
        );
        let setup = crate::ai::agent_chat::ui::AgentChatInlineSetupState {
            reason_code: "catalogLoadFailed",
            title: "Failed to load Agent Chat catalog".into(),
            body: format!("{error}").into(),
            primary_action: crate::ai::agent_chat::ui::AgentChatSetupAction::OpenCatalog,
            secondary_action: Some(crate::ai::agent_chat::ui::AgentChatSetupAction::Retry),
            selected_agent: None,
            catalog_entries: Vec::new(),
            launch_requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
        };
        self.show_embedded_agent_chat_setup_view(source_view, setup, cx);
    }

    pub(super) fn show_pi_agent_chat_unavailable_setup_view(
        &mut self,
        source_view: AppView,
        error: String,
        cx: &mut Context<Self>,
    ) {
        tracing::error!(
            target: "script_kit::tab_ai",
            event = "pi_agent_chat_unavailable",
            error = %error,
        );
        let setup = crate::ai::agent_chat::ui::AgentChatInlineSetupState {
            reason_code: "piAgentChatUnavailable",
            title: "Pi Agent Chat is unavailable".into(),
            body: error.into(),
            primary_action: crate::ai::agent_chat::ui::AgentChatSetupAction::Retry,
            secondary_action: None,
            selected_agent: None,
            catalog_entries: Vec::new(),
            launch_requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
        };
        self.show_embedded_agent_chat_setup_view(source_view, setup, cx);
    }

    pub(super) fn show_pi_agent_chat_warming_setup_view(
        &mut self,
        source_view: AppView,
        profile_name: String,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "pi_agent_chat_warming_setup",
            profile_name = %profile_name,
        );
        let setup = crate::ai::agent_chat::ui::AgentChatInlineSetupState {
            reason_code: "piAgentChatWarming",
            title: "Starting Pi Agent Chat".into(),
            body: format!("{profile_name} is starting. Try again in a moment.").into(),
            primary_action: crate::ai::agent_chat::ui::AgentChatSetupAction::Retry,
            secondary_action: None,
            selected_agent: None,
            catalog_entries: Vec::new(),
            launch_requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
        };
        self.show_embedded_agent_chat_setup_view(source_view, setup, cx);
    }

    /// Show a setup card when Agent Chat launch is blocked (missing key, model
    /// capability, etc.).
    pub(super) fn show_agent_chat_launch_blocked_setup_view(
        &mut self,
        source_view: AppView,
        agent_chat_launch_resolution: &crate::ai::agent_chat::ui::AgentChatLaunchResolution,
        requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements,
        cx: &mut Context<Self>,
    ) {
        let setup = crate::ai::agent_chat::ui::AgentChatInlineSetupState::from_resolution(
            agent_chat_launch_resolution,
            requirements,
        );
        self.show_embedded_agent_chat_setup_view(source_view, setup, cx);
    }
}
