use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgentChatEntryOrigin {
    MainLauncher,
    LauncherTab,
    FileSearch,
    ActionsDialog,
    PluginSkill { skill_id: String },
    Notes,
    Dictation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgentChatThreadTarget {
    ExistingDetachedOrEmbedded,
    CurrentHostEmbedded,
    FreshEmbedded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgentChatSeedPolicy {
    ComposerOnly,
    AutoSubmitFirstTurn,
    PreserveDraft,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum AgentChatContextStaging {
    #[default]
    AmbientOrFocused,
    SuppressFocused,
    Parts {
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        source: &'static str,
    },
    FileSearchSelection,
    ActionsPayload {
        target: crate::ai::TabAiTargetContext,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct AgentChatEntryRequest {
    pub(crate) origin: AgentChatEntryOrigin,
    pub(crate) target: AgentChatThreadTarget,
    pub(crate) seed_text: Option<String>,
    pub(crate) ui_variant: crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant,
    pub(crate) seed_policy: AgentChatSeedPolicy,
    pub(crate) suppress_focused_part: bool,
    pub(crate) context_staging: AgentChatContextStaging,
    pub(crate) return_origin: Option<AppView>,
}

impl AgentChatEntryRequest {
    pub(crate) fn main_launcher(seed_text: Option<String>, suppress_focused_part: bool) -> Self {
        Self::main_launcher_with_variant(
            seed_text,
            suppress_focused_part,
            crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::Standard,
        )
    }

    pub(crate) fn main_launcher_with_variant(
        seed_text: Option<String>,
        suppress_focused_part: bool,
        ui_variant: crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant,
    ) -> Self {
        Self {
            origin: AgentChatEntryOrigin::MainLauncher,
            target: AgentChatThreadTarget::ExistingDetachedOrEmbedded,
            seed_policy: if seed_text
                .as_ref()
                .is_some_and(|text| !text.trim().is_empty())
            {
                AgentChatSeedPolicy::AutoSubmitFirstTurn
            } else {
                AgentChatSeedPolicy::ComposerOnly
            },
            seed_text,
            ui_variant,
            suppress_focused_part,
            context_staging: if suppress_focused_part {
                AgentChatContextStaging::SuppressFocused
            } else {
                AgentChatContextStaging::AmbientOrFocused
            },
            return_origin: None,
        }
    }
}

impl ScriptListApp {
    pub(crate) fn open_agent_chat_from_entry_request(
        &mut self,
        req: AgentChatEntryRequest,
        cx: &mut Context<Self>,
    ) {
        if self.agent_chat_surface_state.blocks_launcher_ai_entry() {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_entry_request_blocked_by_portal",
                origin = ?req.origin,
            );
            return;
        }

        let source_view = req
            .return_origin
            .clone()
            .unwrap_or_else(|| self.current_view.clone());
        self.seed_agent_chat_return_origin_for_view(&source_view);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_entry_request_open",
            origin = ?req.origin,
            target = ?req.target,
            agent_chat_ui_variant = req.ui_variant.state_id(),
            seed_policy = ?req.seed_policy,
            suppress_focused_part = req.suppress_focused_part,
            source_view = ?source_view,
        );

        match req.context_staging.clone() {
            AgentChatContextStaging::ActionsPayload { target } => {
                self.open_tab_ai_agent_chat_with_explicit_target(target, cx);
            }
            AgentChatContextStaging::Parts { parts, source } if parts.len() == 1 => {
                if let Some(part) = parts.into_iter().next() {
                    self.open_tab_ai_agent_chat_with_context_part(part, source, cx);
                }
            }
            _ => {
                self.open_tab_ai_agent_chat_with_options(
                    req.seed_text,
                    req.suppress_focused_part,
                    req.ui_variant,
                    cx,
                );
            }
        }
    }
}
