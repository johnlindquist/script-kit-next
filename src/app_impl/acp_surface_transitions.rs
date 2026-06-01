//! The single mutator for [`ScriptListApp::acp_surface_state`].
//!
//! Oracle-Session `acp-chat-state-machine-audit` PR1. Every write to
//! the placement machine goes through [`transition_acp_surface`], which
//! runs the pure reducer, emits a structured transition event, and in
//! debug builds asserts the placement agrees with
//! [`ScriptListApp::current_view`]. Raw writes to `acp_surface_state`
//! are forbidden; an audit test pins that contract.

use super::*;
use crate::ai::acp::surface_state::{reduce_acp_surface, AcpSurfaceEvent, AcpSurfaceState};

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct AcpSurfaceLifecycleReceipt {
    pub(crate) schema_version: u32,
    pub(crate) event: &'static str,
    pub(crate) source: &'static str,
    pub(crate) previous_state: String,
    pub(crate) next_state: String,
    pub(crate) previous_view: String,
    pub(crate) next_view: String,
    pub(crate) target_automation_id: String,
    pub(crate) target_kind: String,
    pub(crate) surface_kind: String,
    pub(crate) app_view_variant: String,
    pub(crate) return_view: Option<String>,
    pub(crate) return_focus_target: Option<String>,
    pub(crate) focused_input: String,
    pub(crate) main_rekeyed: bool,
    pub(crate) embedded_ai_window_visible: bool,
    pub(crate) actions_popup_cleared: bool,
    pub(crate) warnings: Vec<String>,
}

impl ScriptListApp {
    /// Switch the launcher into the embedded ACP chat surface and update
    /// every state mirror that must move in lock-step with that view.
    pub(crate) fn enter_embedded_acp_chat_surface(
        &mut self,
        entity: gpui::Entity<crate::ai::agent_chat::ui::AgentChatView>,
        cx: &mut gpui::Context<Self>,
    ) -> AcpSurfaceLifecycleReceipt {
        let previous_state = self.acp_surface_state;
        let previous_view = self.current_view.clone();
        self.embedded_acp_focus_handle = Some(entity.read(cx).focus_handle(cx));
        self.current_view = AppView::AcpChatView { entity };
        crate::windows::ensure_embedded_ai_window(true);
        let main_rekeyed = self.rekey_main_automation_surface_from_current_view();
        self.transition_acp_surface(AcpSurfaceEvent::EmbeddedOpened);
        self.focused_input = FocusedInput::None;
        self.clear_actions_popup_state();
        self.focus_coordinator
            .request(crate::focus_coordinator::FocusRequest::acp_chat());
        self.sync_coordinator_to_legacy();
        self.debug_assert_acp_surface_consistent();

        let receipt = self.acp_surface_lifecycle_receipt(
            "embedded_entry",
            "enter_embedded_acp_chat_surface",
            previous_state,
            previous_view,
            main_rekeyed,
            true,
            None,
            None,
            Vec::new(),
        );
        self.log_acp_surface_lifecycle_receipt(&receipt);
        receipt
    }

    pub(crate) fn exit_embedded_acp_chat_surface(
        &mut self,
        return_view: AppView,
        return_focus_target: FocusTarget,
        source: &'static str,
        _cx: &mut gpui::Context<Self>,
    ) -> AcpSurfaceLifecycleReceipt {
        let previous_state = self.acp_surface_state;
        let previous_view = self.current_view.clone();
        let return_view_debug = format!("{return_view:?}");
        let return_focus_debug = format!("{return_focus_target:?}");
        self.restore_current_view_with_focus(return_view.clone(), return_focus_target);
        let main_rekeyed = self.rekey_main_automation_surface_from_current_view();
        crate::windows::ensure_embedded_ai_window(false);
        self.embedded_acp_focus_handle = None;
        self.clear_actions_popup_state();
        self.transition_acp_surface(AcpSurfaceEvent::EmbeddedClosed);
        self.sync_coordinator_to_legacy();
        self.debug_assert_acp_surface_consistent();

        let receipt = self.acp_surface_lifecycle_receipt(
            "embedded_exit",
            source,
            previous_state,
            previous_view,
            main_rekeyed,
            false,
            Some(return_view_debug),
            Some(return_focus_debug),
            Vec::new(),
        );
        self.log_acp_surface_lifecycle_receipt(&receipt);
        receipt
    }

    /// Apply an [`AcpSurfaceEvent`]. No-op when the reduced next state
    /// equals the current state. Emits one `acp_surface_transition`
    /// tracing event per real transition so operators can correlate
    /// placement drift with launcher-entry bugs.
    pub(crate) fn transition_acp_surface(&mut self, event: AcpSurfaceEvent) {
        let previous = self.acp_surface_state;
        let next = reduce_acp_surface(previous, event);
        if next == previous {
            tracing::trace!(
                target: "script_kit::acp",
                event = "acp_surface_transition_noop",
                from = ?previous,
                trigger = ?event,
            );
            return;
        }
        tracing::info!(
            target: "script_kit::acp",
            event = "acp_surface_transition",
            from = ?previous,
            to = ?next,
            trigger = ?event,
        );
        self.acp_surface_state = next;
    }

    /// Debug-only consistency check between the placement enum and
    /// [`AppView`]. Fires when the two disagree — the embedded state
    /// must co-occur with `AppView::AcpChatView`, a portal must co-occur
    /// with the matching portal host view, and `Hidden` must not be
    /// observed while the ACP chat view is on-screen.
    ///
    /// This is `debug_assert` so release builds pay no cost. The goal
    /// is to fail loudly in test / dev runs if a future refactor sets
    /// `current_view` without calling [`transition_acp_surface`].
    #[cfg(debug_assertions)]
    pub(crate) fn debug_assert_acp_surface_consistent(&self) {
        match self.acp_surface_state {
            AcpSurfaceState::Embedded => {
                debug_assert!(
                    matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::Embedded must agree with AppView::AcpChatView; \
                     current_view = {:?}",
                    self.current_view
                );
            }
            AcpSurfaceState::AttachmentPortal { .. } => {
                // Portal host view is one of several builtin surfaces;
                // we can only assert the *negative* half — the chat
                // view must NOT be the current view while a portal
                // owns the panel.
                debug_assert!(
                    !matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::AttachmentPortal must not observe AppView::AcpChatView"
                );
            }
            AcpSurfaceState::Hidden => {
                debug_assert!(
                    !matches!(self.current_view, AppView::AcpChatView { .. }),
                    "AcpSurfaceState::Hidden must not observe AppView::AcpChatView"
                );
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn debug_assert_acp_surface_consistent(&self) {}

    fn acp_surface_lifecycle_receipt(
        &self,
        event: &'static str,
        source: &'static str,
        previous_state: AcpSurfaceState,
        previous_view: AppView,
        main_rekeyed: bool,
        embedded_ai_window_visible: bool,
        return_view: Option<String>,
        return_focus_target: Option<String>,
        warnings: Vec<String>,
    ) -> AcpSurfaceLifecycleReceipt {
        AcpSurfaceLifecycleReceipt {
            schema_version: 1,
            event,
            source,
            previous_state: format!("{previous_state:?}"),
            next_state: format!("{:?}", self.acp_surface_state),
            previous_view: format!("{previous_view:?}"),
            next_view: format!("{:?}", self.current_view),
            target_automation_id: "main".to_string(),
            target_kind: "main".to_string(),
            surface_kind: crate::semantic_surface_for_main_view(&self.current_view)
                .unwrap_or_else(|| "unknown".to_string()),
            app_view_variant: format!("{:?}", self.current_view),
            return_view,
            return_focus_target,
            focused_input: format!("{:?}", self.focused_input),
            main_rekeyed,
            embedded_ai_window_visible,
            actions_popup_cleared: !self.show_actions_popup && self.actions_dialog.is_none(),
            warnings,
        }
    }

    fn log_acp_surface_lifecycle_receipt(&self, receipt: &AcpSurfaceLifecycleReceipt) {
        tracing::info!(
            target: "script_kit::acp",
            event = "acp_surface_lifecycle_receipt",
            receipt_json = %serde_json::to_string(receipt).unwrap_or_default(),
        );
    }
}
