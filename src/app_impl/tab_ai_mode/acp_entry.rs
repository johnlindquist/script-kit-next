use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpEntryOrigin {
    MainLauncher,
    LauncherTab,
    FileSearch,
    ActionsDialog,
    PluginSkill { skill_id: String },
    Notes,
    Dictation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpThreadTarget {
    ExistingDetachedOrEmbedded,
    CurrentHostEmbedded,
    FreshEmbedded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpSeedPolicy {
    ComposerOnly,
    AutoSubmitFirstTurn,
    PreserveDraft,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum AcpContextStaging {
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
pub(crate) struct AcpEntryRequest {
    pub(crate) origin: AcpEntryOrigin,
    pub(crate) target: AcpThreadTarget,
    pub(crate) seed_text: Option<String>,
    pub(crate) seed_policy: AcpSeedPolicy,
    pub(crate) suppress_focused_part: bool,
    pub(crate) context_staging: AcpContextStaging,
    pub(crate) return_origin: Option<AppView>,
}

impl AcpEntryRequest {
    pub(crate) fn main_launcher(seed_text: Option<String>, suppress_focused_part: bool) -> Self {
        Self {
            origin: AcpEntryOrigin::MainLauncher,
            target: AcpThreadTarget::ExistingDetachedOrEmbedded,
            seed_policy: if seed_text.as_ref().is_some_and(|text| !text.trim().is_empty()) {
                AcpSeedPolicy::AutoSubmitFirstTurn
            } else {
                AcpSeedPolicy::ComposerOnly
            },
            seed_text,
            suppress_focused_part,
            context_staging: if suppress_focused_part {
                AcpContextStaging::SuppressFocused
            } else {
                AcpContextStaging::AmbientOrFocused
            },
            return_origin: None,
        }
    }
}

impl ScriptListApp {
    pub(crate) fn open_acp_chat_from_entry_request(
        &mut self,
        req: AcpEntryRequest,
        cx: &mut Context<Self>,
    ) {
        if self.acp_surface_state.blocks_launcher_ai_entry() {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_entry_request_blocked_by_portal",
                origin = ?req.origin,
            );
            return;
        }

        let source_view = req.return_origin.clone().unwrap_or_else(|| self.current_view.clone());
        self.seed_acp_return_origin_for_view(&source_view);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_entry_request_open",
            origin = ?req.origin,
            target = ?req.target,
            seed_policy = ?req.seed_policy,
            suppress_focused_part = req.suppress_focused_part,
            source_view = ?source_view,
        );

        match req.context_staging.clone() {
            AcpContextStaging::ActionsPayload { target } => {
                self.open_tab_ai_acp_with_explicit_target(target, cx);
            }
            AcpContextStaging::Parts { parts, source } if parts.len() == 1 => {
                if let Some(part) = parts.into_iter().next() {
                    self.open_tab_ai_acp_with_context_part(part, source, cx);
                }
            }
            _ => {
                self.open_tab_ai_acp_with_options(req.seed_text, req.suppress_focused_part, cx);
            }
        }
    }
}
