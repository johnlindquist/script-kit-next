use super::*;

impl ScriptListApp {
    fn open_script_list_attachment_portal(
        &mut self,
        kind: crate::ai::window::context_picker::types::PortalKind,
        query: &str,
        placeholder: &str,
        _cx: &mut Context<Self>,
    ) {
        self.active_attachment_portal_kind = Some(kind);
        self.filter_text = query.to_string();
        self.computed_filter_text = query.to_string();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(placeholder.to_string());
        self.current_view = AppView::ScriptList;
        self.hovered_index = None;
        self.selected_index = 0;
        self.opened_from_main_menu = true;
        self.invalidate_grouped_cache();
        self.sync_list_state();
        self.update_window_size();
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
    }

    fn restore_attachment_portal_return_view(
        &mut self,
        return_view: AppView,
        return_focus_target: FocusTarget,
    ) {
        self.active_attachment_portal_kind = None;
        self.current_view = return_view;
        self.pending_focus = Some(return_focus_target);
        self.focused_input = match return_focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };

        // Portal views can temporarily expand the window; restore the
        // originating surface sizing before control returns to the user.
        self.update_window_size();
    }

    /// Whether the app is currently in an attachment portal (file search or
    /// clipboard history opened from the ACP chat context picker).
    pub(crate) fn is_in_attachment_portal(&self) -> bool {
        self.attachment_portal_return_view.is_some()
    }

    pub(crate) fn active_attachment_portal_kind(
        &self,
    ) -> Option<crate::ai::window::context_picker::types::PortalKind> {
        self.active_attachment_portal_kind
    }

    pub(crate) fn build_attachment_portal_part_for_selected_script_list_result(
        &mut self,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        use crate::ai::message_parts::AiContextPart;

        let result = self.get_selected_result()?;
        match &result {
            scripts::SearchResult::Script(script_match) => Some(AiContextPart::FilePath {
                path: script_match.script.path.to_string_lossy().to_string(),
                label: script_match.script.name.clone(),
            }),
            scripts::SearchResult::Scriptlet(scriptlet_match) => {
                let target = Self::tab_ai_target_from_search_result(self.selected_index, &result);
                let target = crate::ai::TabAiTargetContext {
                    metadata: Some(serde_json::json!({
                        "name": scriptlet_match.scriptlet.name,
                        "description": scriptlet_match.scriptlet.description,
                        "tool": scriptlet_match.scriptlet.tool,
                        "code": scriptlet_match.scriptlet.code,
                        "filePath": scriptlet_match.scriptlet.file_path,
                        "pluginId": scriptlet_match.scriptlet.plugin_id,
                        "pluginTitle": scriptlet_match.scriptlet.plugin_title,
                    })),
                    ..target
                };
                let label = crate::ai::format_explicit_target_chip_label(&target);
                Some(AiContextPart::FocusedTarget { target, label })
            }
            scripts::SearchResult::Skill(skill_match) => {
                let owner = if skill_match.skill.plugin_title.is_empty() {
                    skill_match.skill.plugin_id.clone()
                } else {
                    skill_match.skill.plugin_title.clone()
                };
                Some(AiContextPart::SkillFile {
                    path: skill_match.skill.path.to_string_lossy().to_string(),
                    label: skill_match.skill.title.clone(),
                    skill_name: skill_match.skill.title.clone(),
                    owner_label: owner,
                    slash_name: skill_match.skill.skill_id.clone(),
                })
            }
            _ => None,
        }
    }

    /// Open a full built-in view as an attachment portal. The user browses
    /// files or clipboard entries; Enter attaches the selection back to the
    /// ACP chat, Escape cancels and returns.
    pub(crate) fn open_attachment_portal(
        &mut self,
        kind: crate::ai::window::context_picker::types::PortalKind,
        cx: &mut Context<Self>,
    ) {
        use crate::ai::window::context_picker::types::PortalKind;

        // Prevent nesting — only one portal at a time.
        if self.is_in_attachment_portal() {
            tracing::warn!(
                target: "script_kit::acp",
                event = "attachment_portal_nested_prevented",
            );
            return;
        }

        // Save the current view and focus target for restoration on return.
        // The portal is always opened from AcpChatView, so ChatPrompt is correct.
        self.attachment_portal_return_view = Some(self.current_view.clone());
        self.attachment_portal_return_focus_target = Some(FocusTarget::ChatPrompt);
        self.active_attachment_portal_kind = Some(kind);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_opened",
            kind = ?kind,
        );

        let portal_query = if let Some(AppView::AcpChatView { entity }) =
            self.attachment_portal_return_view.as_ref()
        {
            entity.update(cx, |view, _cx| {
                view.take_pending_portal_query(kind).unwrap_or_default()
            })
        } else {
            String::new()
        };

        match kind {
            PortalKind::FileSearch => {
                self.open_file_search(portal_query, cx);
            }
            PortalKind::ClipboardHistory => {
                self.cached_clipboard_entries = crate::clipboard_history::get_cached_entries(100);
                self.open_builtin_filterable_view_with_filter(
                    AppView::ClipboardHistoryView {
                        filter: portal_query.clone(),
                        selected_index: 0,
                    },
                    &portal_query,
                    "Search clipboard history...",
                    cx,
                );
            }
            PortalKind::ScriptSearch => {
                self.open_script_list_attachment_portal(
                    kind,
                    &portal_query,
                    "Search scripts...",
                    cx,
                );
            }
            PortalKind::ScriptletSearch => {
                self.open_script_list_attachment_portal(
                    kind,
                    &portal_query,
                    "Search scriptlets...",
                    cx,
                );
            }
            PortalKind::SkillSearch => {
                self.open_script_list_attachment_portal(
                    kind,
                    &portal_query,
                    "Search skills...",
                    cx,
                );
            }
            PortalKind::NotesBrowse => {
                self.open_builtin_filterable_view_with_filter(
                    AppView::NotesBrowseView {
                        filter: portal_query.clone(),
                        selected_index: 0,
                    },
                    &portal_query,
                    "Search notes...",
                    cx,
                );
            }
            PortalKind::AcpHistory => {
                self.open_builtin_filterable_view_with_filter(
                    AppView::AcpHistoryView {
                        filter: portal_query.clone(),
                        selected_index: 0,
                    },
                    &portal_query,
                    "Search conversation history...",
                    cx,
                );
            }
        }

        cx.notify();
    }

    /// Close the attachment portal and attach the selected part to the ACP chat.
    pub(crate) fn close_attachment_portal_with_part(
        &mut self,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
        let return_view = self
            .attachment_portal_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_focus_target = self
            .attachment_portal_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_closed_with_part",
            focus_target = ?return_focus_target,
        );

        self.restore_attachment_portal_return_view(return_view.clone(), return_focus_target);

        // Stage the context part with an inline @mention token.
        // Uses the canonical token format so the mention sync system can track
        // it — deleting characters from the mention removes the part.
        if let AppView::AcpChatView { entity } = &return_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| {
                view.attach_portal_part(part.clone(), cx);
                cx.notify();
            });
        }

        cx.notify();
    }

    /// Close the attachment portal without attaching anything (Escape).
    pub(crate) fn close_attachment_portal_cancel(&mut self, cx: &mut Context<Self>) {
        let return_view = self
            .attachment_portal_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_focus_target = self
            .attachment_portal_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_cancelled",
            focus_target = ?return_focus_target,
        );

        self.restore_attachment_portal_return_view(return_view, return_focus_target);

        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn attachment_portal_source() -> String {
        fs::read_to_string("src/app_impl/attachment_portal.rs")
            .expect("Failed to read src/app_impl/attachment_portal.rs")
    }

    #[test]
    fn attachment_portal_restore_helper_reapplies_window_size_contract() {
        let source = attachment_portal_source();
        let helper_start = source
            .find("fn restore_attachment_portal_return_view(")
            .expect("restore helper must exist");
        let helper_body = &source[helper_start..];

        assert!(
            helper_body.contains("self.update_window_size();"),
            "restore helper must reapply the originating surface window size"
        );
    }

    #[test]
    fn attachment_portal_exit_paths_use_shared_restore_helper() {
        let source = attachment_portal_source();
        let helper_calls = source
            .matches("self.restore_attachment_portal_return_view(")
            .count();

        assert!(
            helper_calls >= 2,
            "attach + cancel portal exits must share the same restore helper"
        );
    }
}
