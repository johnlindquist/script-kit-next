//! Handing things to AI: the one chat's context from any view, the Spine's prompt plans, and the
//! Tab AI harness terminal.

use super::*;
use crate::one_window::chat_entry::ChatEntry;

mod focused_text;
mod source_classification;
mod spine_cwd;
mod types;
use focused_text::materialize_selection_context_parts;
use source_classification::{
    build_tab_ai_apply_back_hint, detect_tab_ai_source_type, detect_tab_ai_source_type_early,
};
pub(crate) use types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TabAiHarnessCloseDisposition {
    RestoreOrigin,
    CloseMainWindowStateFirst,
}

impl ScriptListApp {
    /// Upper bound for Ask Anything / ambient desktop capture before bootstrap fails open.
    const TAB_AI_DEFERRED_CAPTURE_TIMEOUT_SEC: u64 = 30;

    /// Opens the one chat about what the current view is on: its focused item, or what's on
    /// screen when it has none. Words go now. The launcher's own rows are never implied: ⌘K on a
    /// row is how the chat is opened about one.
    pub(crate) fn open_chat_about_current_view(
        &mut self,
        text: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let entry = match text.filter(|t| !t.trim().is_empty()) {
            Some(text) => ChatEntry::sending(text),
            None => ChatEntry::quick(),
        };
        let entry = match self.focused_part_of_current_view(cx) {
            Some(part) => entry.with(part),
            None if matches!(self.current_view, AppView::ScriptList) => entry,
            None => entry.capturing(crate::ai::TabAiCaptureKind::DefaultContext),
        };
        self.one_window_open_chat(entry, cx);
    }

    /// The focused item of a view other than the launcher, as a part the chat takes along.
    fn focused_part_of_current_view(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        if matches!(self.current_view, AppView::ScriptList) {
            return None;
        }
        let (snapshot, _) = self.snapshot_tab_ai_ui(cx);
        let view = self.current_view.clone();
        self.build_tab_ai_focused_part_for_view(&view, &snapshot)
    }

    /// Unified spine Enter dispatch: try the selected row's action, then fall
    /// back to submitting the prompt plan. Returns true if the spine consumed
    /// the Enter, false to let the caller's default action run.
    pub(crate) fn try_handle_spine_enter(
        &mut self,
        window: &mut Window,
        ctx: &mut Context<Self>,
    ) -> bool {
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(ctx);
            self.set_main_menu_dispatch_observation(None);
            if !self.root_search.query_is_current() {
                return false;
            }
        }
        let inputs = self.root_search.computed_query_inputs();
        if inputs.spine_parse.input != inputs.raw
            || !self.spine_projection_owns_main_list_for(
                &inputs.spine_parse,
                inputs.spine_projection.as_ref(),
            )
        {
            return false;
        }
        // Unarmed empty colon mode (`@clipboard:` showing recents with no
        // selected row): Enter is consumed but attaches nothing — a reflexive
        // double-Enter must never insert an unseen recent item. Down or a
        // click arms the selection first.
        if self.spine_empty_subsearch_selection_suppressed() {
            tracing::info!(
                target: "script_kit::spine",
                event = "empty_context_subsearch_enter_consumed",
            );
            return true;
        }
        // accept_spine_projection_row handles rich subsearch interception
        // (@file:, @clipboard:, etc.) before applying the row's action. If it
        // succeeds, the Spine consumed Enter. Otherwise (e.g. Noop placeholder,
        // no selection), fall through to plan submission.
        if self.accept_spine_projection_row(window, ctx) {
            return true;
        }
        if self.try_submit_spine_prompt_plan_from_enter(ctx) {
            return true;
        }
        // Structural guard: while a `@source:` subsearch segment is active,
        // Enter must never fall through to the launcher's default row
        // execution (opening the note / running the script / pasting the
        // clipboard entry) — that destroys the prompt being built. Consume
        // the Enter even when no row attached.
        if let Some(projection) = inputs.spine_projection.as_ref() {
            if let crate::spine::SpineSegmentKind::ContextMention {
                context_type,
                sub_query,
            } = &projection.active_segment_kind
            {
                if crate::spine::catalog_subsearch::parse_context_subsearch(
                    context_type,
                    sub_query.as_deref(),
                )
                .is_some()
                {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "spine_subsearch_enter_consumed_without_attach",
                        context_type = %context_type,
                    );
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn try_submit_spine_prompt_plan_from_enter(
        &mut self,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.spine_enabled || !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }

        let raw = self.filter_text.clone();
        if raw.trim().is_empty() {
            return false;
        }

        self.set_spine_parse_from_filter_and_cursor(&raw, raw.len());
        self.try_submit_spine_prompt_plan_from_parse(self.spine_parse.clone(), true, true, true, cx)
    }

    pub(crate) fn try_submit_spine_prompt_plan_from_parse(
        &mut self,
        parse: crate::spine::SpineParse,
        reset_launcher_filter: bool,
        clear_aliases_after_submit: bool,
        use_selection_preview_cache: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let mention_aliases = self.spine_mention_aliases.clone();
        self.try_submit_spine_prompt_plan_from_parse_with_aliases(
            parse,
            &mention_aliases,
            reset_launcher_filter,
            clear_aliases_after_submit,
            use_selection_preview_cache,
            cx,
        )
    }

    pub(crate) fn try_submit_spine_prompt_plan_from_parse_with_aliases(
        &mut self,
        parse: crate::spine::SpineParse,
        mention_aliases: &std::collections::HashMap<
            String,
            crate::ai::message_parts::AiContextPart,
        >,
        reset_launcher_filter: bool,
        clear_aliases_after_submit: bool,
        use_selection_preview_cache: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let plan = Self::spine_prompt_plan_for_aliases(&parse, mention_aliases);
        // Plain prose without sigils normally doesn't submit to chat. But when
        // the user has already established a working directory via the cwd
        // chip (typed `>` then picked a folder), Cmd+Enter on a non-empty
        // prompt should dispatch to Agent Chat with that cwd. The chip is itself a
        // context anchor.
        let cwd_anchor_authorizes_submit =
            self.spine_cwd.is_some() && !plan.normalized_prompt.trim().is_empty();
        if !plan.should_submit_to_chat() && !cwd_anchor_authorizes_submit {
            return false;
        }

        // Style-only rewrites (`.` + pick a style) route into the instant
        // rewrite mini instead of the full Agent Chat transcript: three fast
        // variations, Tab to choose, Enter to paste back — and no `.` sigil
        // surfaced anywhere. Falls back to the legacy Agent Chat handoff when
        // the focused field can't be captured over AX (e.g. web apps that
        // only work through the Cmd+C selection fallback).
        if let Some(style) = plan
            .selected_style
            .clone()
            .filter(|_| crate::spine::prompt_plan::spine_parse_is_style_only(&parse))
        {
            if let Some(part) = self.capture_focused_text_part("spine_style_rewrite") {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "spine_style_rewrite_to_chat",
                    style = %style.id,
                );
                if reset_launcher_filter {
                    self.reset_spine_launcher_filter_after_submit();
                }
                if clear_aliases_after_submit {
                    self.spine_mention_aliases.clear();
                }
                self.one_window_open_chat(ChatEntry::sending(style.instruction).with(part), cx);
                self.flush_pending_main_menu_query(cx);
                cx.notify();
                return true;
            }
        }

        let prompt = plan.normalized_prompt.trim().to_string();
        // Freeze `@selection` into real text NOW, while the launcher panel is
        // still a non-activating overlay and the source app owns focus. The
        // Selection part is otherwise a lazy `kit://context?selectedText=1`
        // resource resolved at thread submit time — by then Agent Chat has
        // taken over app activation and the AX/Cmd+C capture reads from the
        // wrong window ("No selected text was provided").
        let cached_selection = if use_selection_preview_cache {
            let preview = &self.spine_live_preview_cache.current;
            preview
                .selection_text
                .clone()
                .map(|text| crate::selected_text::CapturedText {
                    text,
                    kind: if preview.selection_is_draft {
                        crate::selected_text::CapturedTextKind::FocusedField
                    } else {
                        crate::selected_text::CapturedTextKind::Selection
                    },
                    source_app: preview
                        .selection_source_app
                        .clone()
                        .or_else(|| preview.frontmost_app_name.clone()),
                })
        } else {
            None
        };
        // Selection-or-focused-field: a user who invokes a rewrite while
        // typing (no selection) means "rewrite my draft", so fall back to the
        // focused field's text rather than submitting an empty attachment.
        let parts = materialize_selection_context_parts(
            plan.context_parts.clone(),
            cached_selection,
            || {
                crate::selected_text::capture_selection_or_focused_text()
                    .ok()
                    .flatten()
            },
        );
        let selected_profile_name = if let Some(profile) = plan.selected_profile.as_ref() {
            let mut prefs = crate::config::load_user_preferences();
            let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
            let persisted = crate::ai::agent_chat::profiles::persist_agent_chat_profile_selection(
                &mut prefs.ai,
                &profile.id,
                &ctx,
            )
            .and_then(|entry| {
                if crate::config::save_user_preferences(&prefs).is_ok() {
                    Some(entry.name)
                } else {
                    None
                }
            });
            if persisted.is_none() {
                tracing::warn!(
                    target: "script_kit::spine",
                    event = "spine_prompt_plan_profile_persist_failed",
                    profile_id = %profile.id,
                );
            } else {
                self.prewarm_selected_agent_chat_profile_for_current_cwd(
                    &prefs.ai,
                    "spine_prompt_plan_profile_selection",
                );
            }
            persisted
        } else {
            None
        };

        if prompt.is_empty() && parts.is_empty() {
            if let Some(profile_name) = selected_profile_name {
                self.one_window_notice(
                    crate::line_notice::INFO,
                    format!("Default Agent Chat profile: {profile_name}"),
                    cx,
                );
                cx.notify();
                return true;
            }
            return false;
        }

        tracing::info!(
            target: "script_kit::spine",
            event = "spine_prompt_plan_submit",
            raw_len = plan.raw_input.len(),
            prompt_len = prompt.len(),
            context_count = parts.len(),
            warning_count = plan.unknown_warnings.len(),
            profile = ?plan.selected_profile.as_ref().map(|p| p.id.as_str()),
            style = ?plan.selected_style.as_ref().map(|s| s.id.as_str()),
        );

        if reset_launcher_filter {
            self.reset_spine_launcher_filter_after_submit();
        }
        // Launcher input aliases are tied to a cleared ephemeral prompt. Day
        // Page aliases may still back visible note text, so callers decide
        // when clearing them is appropriate.
        if clear_aliases_after_submit {
            self.spine_mention_aliases.clear();
        }

        let entry = if prompt.is_empty() {
            ChatEntry::quick()
        } else {
            ChatEntry::sending(prompt)
        };
        self.one_window_open_chat(entry.with_parts(parts), cx);
        self.flush_pending_main_menu_query(cx);

        cx.notify();
        true
    }

    fn reset_spine_launcher_filter_after_submit(&mut self) {
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.pending_filter_sync = true;
        self.spine_parse = Default::default();
        self.spine_projection = None;
        self.spine_empty_subsearch_armed_for = None;
        self.spine_live_preview_cache = Default::default();
        self.invalidate_grouped_cache();
        self.accept_root_search_input_intent("");
    }

    fn spine_prompt_plan_for_aliases(
        parse: &crate::spine::SpineParse,
        mention_aliases: &std::collections::HashMap<
            String,
            crate::ai::message_parts::AiContextPart,
        >,
    ) -> crate::spine::prompt_plan::SpinePromptPlan {
        crate::spine::prompt_plan::build_spine_prompt_plan_with_aliases(parse, mention_aliases)
    }

    fn quick_terminal_context_part_from_entity(
        &mut self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        let capture = entity.update(cx, |term, _cx| term.terminal_context_capture())?;
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "quick_terminal_context_part_captured",
            source = %capture.source,
            bytes = capture.text.len(),
            line_count = capture.line_count,
            truncated = capture.truncated,
        );
        crate::terminal_history::record(crate::terminal_history::TerminalHistoryEntry {
            label: capture.label.clone(),
            source: capture.source.clone(),
            text: capture.text.clone(),
            line_count: capture.line_count,
            truncated: capture.truncated,
            captured_at: chrono::Utc::now().to_rfc3339(),
        });
        Some(crate::ai::message_parts::AiContextPart::TextBlock {
            label: capture.label,
            source: capture.source,
            text: capture.text,
            mime_type: Some("text/x-terminal-transcript".to_string()),
        })
    }

    pub(crate) fn open_agent_chat_with_quick_terminal_output(
        &mut self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(part) = self.quick_terminal_context_part_from_entity(entity, cx) else {
            self.one_window_notice(
                crate::line_notice::FAILED,
                "No terminal output to attach".to_string(),
                cx,
            );
            return false;
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "quick_terminal_output_agent_chat_open",
            source = %part.source(),
            label_sha256 = %crate::logging::log_private_user_value(part.label()),
            label_bytes = part.label().len(),
        );
        self.one_window_open_chat(ChatEntry::quick().with(part), cx);
        true
    }

    pub(crate) fn attach_quick_terminal_output_to_portal_origin(
        &mut self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.active_attachment_portal_kind()
            != Some(crate::ai::context_selector::types::ContextPortalKind::Terminal)
        {
            return false;
        }

        let Some(part) = self.quick_terminal_context_part_from_entity(entity.clone(), cx) else {
            self.one_window_notice(
                crate::line_notice::FAILED,
                "No terminal output to attach".to_string(),
                cx,
            );
            return true;
        };

        entity.update(cx, |term, _cx| {
            let _ = term.terminate_session();
        });
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "quick_terminal_portal_output_attached",
            source = %part.source(),
            label_sha256 = %crate::logging::log_private_user_value(part.label()),
            label_bytes = part.label().len(),
        );
        self.close_attachment_portal_with_part(part, cx);
        true
    }

    pub(crate) fn dispatch_quick_terminal_cmd_enter(
        &mut self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::QuickTerminalView { .. }) {
            return false;
        }

        if self.attach_quick_terminal_output_to_portal_origin(entity.clone(), cx) {
            return true;
        }

        if self.quick_terminal_can_apply_back() {
            self.apply_tab_ai_result_from_terminal(entity, cx);
            return true;
        }

        self.open_agent_chat_with_quick_terminal_output(entity, cx)
    }

    pub(crate) fn route_large_script_list_paste_to_agent_chat(
        &mut self,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }

        let Ok(mut clipboard) = arboard::Clipboard::new() else {
            return false;
        };
        if let Ok(image_data) = clipboard.get_image() {
            let Ok(encoded) = crate::clipboard_history::encode_image_as_png(&image_data) else {
                return false;
            };
            let Some(png_bytes) = crate::clipboard_history::content_to_png_bytes(&encoded) else {
                return false;
            };

            if png_bytes.len() > crate::prompts::chat::MAX_IMAGE_BYTES {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "script_list_clipboard_image_rejected_too_large",
                    size_bytes = png_bytes.len(),
                    max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                );
                return false;
            }

            let Ok(temp_file) = crate::pasted_image::reserve_png_temp_file() else {
                return false;
            };
            let Ok(path) = crate::pasted_image::write_png_bytes_to_temp_file(temp_file, &png_bytes)
            else {
                return false;
            };
            let prepared = crate::pasted_image::prepare_pasted_image(&path, &[], "");
            let label = prepared.token.label.clone();
            let part = crate::ai::message_parts::AiContextPart::FilePath {
                path,
                label: label.clone(),
            };

            tracing::info!(
                target: "script_kit::tab_ai",
                event = "script_list_clipboard_image_routed_to_agent_chat",
                label_sha256 = %crate::logging::log_private_user_value(&label),
                label_bytes = label.len(),
                width = image_data.width,
                height = image_data.height,
                size_bytes = png_bytes.len(),
            );

            self.one_window_open_chat(ChatEntry::quick().with(part), cx);
            return true;
        }
        let Ok(text) = clipboard.get_text() else {
            return false;
        };

        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        // A5 decision (2026-06-09): the launcher input is single-line, so any
        // multi-line paste routes to Agent Chat instead of being newline-
        // stripped into the filter. Single-line pastes under the large-paste
        // thresholds still paste inline.
        let prepared =
            crate::pasted_text::prepare_pasted_text_for_single_line_surface(&normalized, &[]);
        let Some(token) = prepared.token else {
            return false;
        };

        let part = crate::ai::message_parts::AiContextPart::TextBlock {
            label: token.label.clone(),
            source: format!("clipboard://pasted-text/{}", uuid::Uuid::new_v4()),
            text: normalized,
            mime_type: Some("text/plain".to_string()),
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "script_list_large_paste_routed_to_agent_chat",
            label_sha256 = %crate::logging::log_private_user_value(&token.label),
            label_bytes = token.label.len(),
            char_count = token.text.chars().count(),
            line_count = token.text.lines().count().max(1),
        );

        self.one_window_open_chat(ChatEntry::quick().with(part), cx);
        true
    }

    /// A plugin skill picked in the launcher: the chat opens with the skill waiting in the line,
    /// so the next question goes with it.
    pub(crate) fn open_agent_chat_with_selected_skill(
        &mut self,
        skill: &crate::plugins::PluginSkill,
        cx: &mut Context<Self>,
    ) {
        let owner = if skill.plugin_title.is_empty() {
            &skill.plugin_id
        } else {
            &skill.plugin_title
        };
        tracing::info!(
            target: "script_kit::one_window",
            event = "chat_skill_staged",
            plugin_id = %skill.plugin_id,
            skill_id = %skill.skill_id,
        );
        let part = crate::ai::message_parts::AiContextPart::SkillFile {
            path: skill.path.to_string_lossy().to_string(),
            label: format!("/{}", skill.skill_id),
            skill_name: skill.title.clone(),
            owner_label: owner.clone(),
            slash_name: skill.skill_id.clone(),
        };
        self.one_window_open_chat(ChatEntry::quick().with(part), cx);
    }

    /// Entry point with explicit capture kind.
    ///
    /// Used by `SendScreenToAi`, `SendFocusedWindowToAi`, etc. so each
    /// command gets the appropriate screenshot/context capture behaviour
    /// instead of always defaulting to focused-window.
    pub(crate) fn open_tab_ai_chat_with_capture_kind(
        &mut self,
        entry_intent: Option<String>,
        capture_kind: crate::ai::TabAiCaptureKind,
        cx: &mut Context<Self>,
    ) {
        let entry = match entry_intent.filter(|t| !t.trim().is_empty()) {
            Some(text) => ChatEntry::sending(text),
            None => ChatEntry::quick(),
        };
        self.one_window_open_chat(entry.capturing(capture_kind), cx);
    }

    /// Route raw text (from Auto Submit fallback or dictation) through the
    /// quick-submit planner and into the harness — either an existing live
    /// session or a fresh one.
    pub(crate) fn submit_to_current_or_new_tab_ai_harness_from_text(
        &mut self,
        raw_text: String,
        source: crate::ai::TabAiQuickSubmitSource,
        cx: &mut Context<Self>,
    ) {
        let Some(plan) = crate::ai::plan_tab_ai_quick_submit(source, &raw_text) else {
            self.one_window_open_chat(ChatEntry::quick(), cx);
            return;
        };

        // If the PTY harness is already the active surface and alive, route through
        // the full structured submission builder so live-session quick submits
        // get fresh context and quick-submit metadata.
        if let Some(session) = self
            .tab_ai_harness
            .as_ref()
            .filter(|_| matches!(self.current_view, AppView::QuickTerminalView { .. }))
            .filter(|session| session.entity.read(cx).is_alive())
        {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "tab_ai_quick_submit_live_session",
                source = ?plan.source,
                kind = ?plan.kind,
                capture_kind = %plan.capture_kind,
                input_len = plan.raw_query.len(),
            );

            self.submit_live_tab_ai_harness_from_plan(session.entity.clone(), plan, cx);
            return;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_quick_submit_new_session",
            source = ?plan.source,
            kind = ?plan.kind,
            capture_kind = %plan.capture_kind,
            input_len = plan.raw_query.len(),
        );

        self.open_tab_ai_chat_with_quick_submit_plan(plan, cx);
    }

    /// A planned submission goes to the harness terminal when it needs script verification,
    /// otherwise to the one chat.
    fn open_tab_ai_chat_with_quick_submit_plan(
        &mut self,
        plan: crate::ai::TabAiQuickSubmitPlan,
        cx: &mut Context<Self>,
    ) {
        let text = plan.submission_intent().to_string();
        self.begin_tab_ai_harness_entry(
            TabAiHarnessEntryOptions {
                text: Some(text),
                submit: plan.submit,
                stage_focused: true,
                capture_kind: plan.capture_kind_enum(),
                quick_submit_plan: Some(plan),
            },
            cx,
        );
    }

    /// Submit a structured turn into an already-open, live harness session.
    ///
    /// Captures fresh desktop context, rebuilds the full flat-line context
    /// submission with quick-submit metadata, and injects via Submit mode.
    /// On build failure, shows an error toast instead of falling back to
    /// raw intent-only PTY injection.
    fn submit_live_tab_ai_harness_from_plan(
        &mut self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        plan: crate::ai::TabAiQuickSubmitPlan,
        cx: &mut Context<Self>,
    ) {
        let capture_kind = plan.capture_kind_enum();
        let source_view = self
            .tab_ai_harness_return_view
            .clone()
            .unwrap_or(AppView::ScriptList);

        let (ui_snapshot, invocation_receipt) = self.snapshot_tab_ai_ui(cx);
        self.tab_ai_harness_capture_generation += 1;

        let request = TabAiLaunchRequest {
            source_view,
            text: Some(plan.submission_intent().to_string()),
            submit: true,
            quick_submit_plan: Some(plan),
            ui_snapshot,
            invocation_receipt,
            capture_kind,
            capture_generation: self.tab_ai_harness_capture_generation,
        };

        let wait_for_readiness = Self::tab_ai_harness_needs_readiness_wait(&entity, cx);
        let capture_rx = Self::spawn_tab_ai_pre_switch_capture(request.capture_kind);
        let app_weak = cx.entity().downgrade();
        let capture_gen = request.capture_generation;

        cx.spawn(async move |_this, cx| {
            let capture_result = match capture_rx.recv().await {
                Ok(result) => result,
                Err(_) => Err("deferred capture channel closed".to_string()),
            };

            let artifacts = match capture_result {
                Ok(artifacts) => artifacts,
                Err(error) => {
                    tracing::warn!(
                        event = "tab_ai_live_quick_submit_capture_failed",
                        error = %error,
                    );
                    TabAiDeferredCaptureArtifacts::default()
                }
            };

            cx.update(|cx| {
                let Some(app) = app_weak.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| {
                    if this.tab_ai_harness_capture_generation != capture_gen {
                        tracing::debug!(
                            event = "tab_ai_live_quick_submit_stale",
                            expected = capture_gen,
                            current = this.tab_ai_harness_capture_generation,
                        );
                        return;
                    }

                    let resolved = this.build_tab_ai_context_from(
                        request.entry_text().unwrap_or_default().to_string(),
                        request.source_view.clone(),
                        request.ui_snapshot.clone(),
                        artifacts.desktop,
                        request.invocation_receipt.clone(),
                        cx,
                    );

                    let source_type = detect_tab_ai_source_type(
                        &request.source_view,
                        &resolved.context.desktop,
                        resolved.context.focused_target.as_ref(),
                    );
                    let apply_back_hint = build_tab_ai_apply_back_hint(source_type.as_ref());

                    this.tab_ai_harness_apply_back_route = source_type
                        .clone()
                        .zip(apply_back_hint.clone())
                        .map(|(source_type, hint)| crate::ai::TabAiApplyBackRoute {
                            source_type,
                            hint,
                            focused_target: resolved.context.focused_target.clone(),
                        });

                    let context = resolved.context.with_deferred_capture_fields(
                        source_type,
                        artifacts.screenshot_path,
                        apply_back_hint,
                    );

                    match crate::ai::build_tab_ai_harness_submission(
                        &context,
                        request.entry_text(),
                        crate::ai::TabAiHarnessSubmissionMode::Submit,
                        request.quick_submit_plan.as_ref(),
                        Some(&resolved.invocation_receipt),
                        &resolved.suggested_intents,
                    ) {
                        Ok(submission) => {
                            this.inject_tab_ai_harness_submission(
                                entity.clone(),
                                submission,
                                wait_for_readiness,
                                true,
                                cx,
                            );
                        }
                        Err(error) => {
                            tracing::warn!(
                                event = "tab_ai_live_quick_submit_build_failed",
                                error = %error,
                            );
                            this.one_window_notice(
                                crate::line_notice::FAILED,
                                format!("Failed to build quick-submit context: {error}"),
                                cx,
                            );
                            cx.notify();
                        }
                    }
                });
            });
        })
        .detach();
    }

    /// Map a target kind to its human-readable chip prefix.
    pub(crate) fn tab_ai_chip_prefix_for_kind(kind: &str) -> &'static str {
        match kind {
            "file" => "File",
            "directory" => "Folder",
            "search_query" => "Search",
            "input" => "Input",
            "clipboard_entry" => "Clipboard",
            "script" | "scriptlet" | "builtin" => "Command",
            "window" => "Window",
            "app" => "App",
            "process" => "Process",
            "menu_command" => "Menu Command",
            "action" => "Action",
            "note" => "Note",
            "agent" => "Agent",
            "fallback" => "Suggestion",
            _ => "Selection",
        }
    }

    /// Format a canonical chip label from a resolved target.
    ///
    /// Delegates to the shared `format_explicit_target_chip_label` so Notes,
    /// actions, and the main-window Agent Chat openings all produce the same text.
    fn format_tab_ai_focused_chip_label(target: &crate::ai::TabAiTargetContext) -> String {
        crate::ai::format_explicit_target_chip_label(target)
    }

    /// Resolve targets for a view and emit a structured audit log.
    fn resolve_tab_ai_targets_with_audit_for_view(
        &self,
        view: &AppView,
        ui: &crate::ai::TabAiUiSnapshot,
        phase: &str,
    ) -> (
        Option<crate::ai::TabAiTargetContext>,
        Vec<crate::ai::TabAiTargetContext>,
    ) {
        let (focused_target, visible_targets) =
            self.resolve_tab_ai_surface_targets_for_view(view, ui);
        crate::ai::TabAiTargetAudit::from_targets(
            &ui.prompt_type,
            &focused_target,
            &visible_targets,
        )
        .emit_with_phase(phase);
        (focused_target, visible_targets)
    }

    /// Returns `true` when the current view should treat global `Cmd+Enter`
    /// as launcher-style "send this context to AI".
    fn supports_global_cmd_enter_ai_entry(view: &AppView) -> bool {
        matches!(
            view,
            AppView::ScriptList
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::ThemeChooserView { .. }
                | AppView::EmojiPickerView { .. }
                | AppView::BrowseKitsView { .. }
                | AppView::MigrateV1View { .. }
                | AppView::InstalledKitsView { .. }
                | AppView::ProcessManagerView { .. }
                | AppView::SearchAiPresetsView { .. }
                | AppView::CreateAiPresetView { .. }
                | AppView::SettingsView { .. }
                | AppView::FavoritesBrowseView { .. }
                | AppView::CurrentAppCommandsView { .. }
                | AppView::FileSearchView { .. }
        )
    }

    /// Route a launcher-style `Cmd+Enter` press from the current non-AI source
    /// surface into Agent Chat.
    ///
    /// Returns `true` when the keypress was consumed and Agent Chat launch began.
    /// Run 12 Pass 11 — Cmd+Enter while composing power syntax generates a
    /// deterministic stub proposal, sets `pending_menu_syntax_ai_proposal`,
    /// and triggers a snapshot rebuild. Returns `true` when the chord was
    /// consumed so the legacy Agent Chat route doesn't also fire.
    pub(crate) fn try_route_cmd_enter_to_menu_syntax_ai(&mut self, cx: &mut Context<Self>) -> bool {
        use crate::menu_syntax::{builtin_schema, MenuSyntaxActionState};
        let raw = self.filter_text().to_string();
        let mode = &self.menu_syntax_mode;
        let pending = if let Some(invocation) = mode.capture_for(&raw) {
            let target = invocation.target.clone();
            let schema = builtin_schema(&target);
            let state = MenuSyntaxActionState::CaptureComposer {
                target: &target,
                payload: invocation,
                schema: schema.as_ref(),
            };
            Some(crate::menu_syntax_ai::PendingMenuSyntaxAiProposal::new(
                raw.clone(),
                Some(target.clone()),
                crate::menu_syntax_ai::stub_proposal_for(&state),
            ))
        } else if let Some(argv) = mode.command_for(&raw) {
            let origin_target = Some(format!("!{}", argv.head));
            let state = MenuSyntaxActionState::CommandComposer {
                head: &argv.head,
                argv: &argv.argv,
            };
            Some(crate::menu_syntax_ai::PendingMenuSyntaxAiProposal::new(
                raw.clone(),
                origin_target,
                crate::menu_syntax_ai::stub_proposal_for(&state),
            ))
        } else if let Some(query) = mode.advanced_query_for(&raw) {
            let state = MenuSyntaxActionState::RefineQuery { query };
            Some(crate::menu_syntax_ai::PendingMenuSyntaxAiProposal::new(
                raw.clone(),
                Some("?".to_string()),
                crate::menu_syntax_ai::stub_proposal_for(&state),
            ))
        } else {
            None
        };
        let Some(pending) = pending else {
            return false;
        };
        tracing::info!(
            target: "script_kit::menu_syntax_ai",
            event = "menu_syntax_ai_proposal_generated",
            title_sha256 = %crate::logging::log_private_user_value(&pending.proposal.title),
            title_bytes = pending.proposal.title.len(),
            actionable = pending.proposal.is_actionable(),
            origin_target = ?pending.origin.target,
        );
        self.pending_menu_syntax_ai_proposal = Some(pending);
        cx.notify();
        true
    }

    /// Run 12 Pass 13 — `ai-proposal-accept-dismiss` UI handler. When the
    /// inline AI proposal hint card is up (i.e.
    /// `pending_menu_syntax_ai_proposal.is_some()`) and the user keys
    /// Tab/Enter (Accept) or Esc (Dismiss), thread the keypress through
    /// the pure `apply_proposal` decision layer (Run 11 Pass 35) and
    /// dispatch the resulting `ProposalEffect`:
    ///
    /// - `SetFilterText { new_text }` → write through `set_filter_text_immediate`
    ///   (which also clears `pending_menu_syntax_ai_proposal` as a side
    ///   effect, keeping the surface in sync).
    /// - `Dismiss` → clear `pending_menu_syntax_ai_proposal` and
    ///   `cx.notify()` to drop the card from the next render.
    ///
    /// Returns `true` when the keypress was consumed (caller should
    /// `stop_propagation` so the legacy Tab/Enter/Esc handlers don't also
    /// fire).
    pub(crate) fn try_apply_pending_menu_syntax_ai_proposal(
        &mut self,
        action: crate::menu_syntax_ai_apply::ProposalApplyAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pending) = self.pending_menu_syntax_ai_proposal.clone() else {
            return false;
        };
        let current_input = self.filter_text().to_string();
        let origin_matches = pending.is_current_for(&current_input);
        let effect =
            crate::menu_syntax_ai_apply::apply_pending_proposal(&current_input, &pending, action);
        tracing::info!(
            target: "script_kit::menu_syntax_ai",
            event = "menu_syntax_ai_proposal_resolved",
            action = ?action,
            origin_matches,
            origin_target = ?pending.origin.target,
            effect_kind = match &effect {
                crate::menu_syntax_ai_apply::ProposalEffect::SetFilterText { .. } => "set_filter_text",
                crate::menu_syntax_ai_apply::ProposalEffect::Dismiss => "dismiss",
            },
        );
        match effect {
            crate::menu_syntax_ai_apply::ProposalEffect::SetFilterText { new_text } => {
                self.set_filter_text_immediate(new_text, window, cx);
            }
            crate::menu_syntax_ai_apply::ProposalEffect::Dismiss => {
                self.pending_menu_syntax_ai_proposal = None;
                cx.notify();
            }
        }
        true
    }

    pub(crate) fn try_route_global_cmd_enter_to_agent_chat_context_capture(
        &mut self,
        cx: &mut Context<Self>,
    ) -> bool {
        if matches!(self.current_view, AppView::ScriptList) && self.one_window_owns_list() {
            return self
                .one_window_key(sk_one_window::Key::CmdEnter, cx)
                .taken();
        }
        let explicit_spine_prompt_plan =
            self.script_list_cmd_enter_has_explicit_spine_prompt_plan();
        if explicit_spine_prompt_plan && self.try_submit_spine_prompt_plan_from_enter(cx) {
            return true;
        }
        // Run 12 Pass 11 — `cmd-enter-inline-ai-proposal`. When the user is
        // composing power syntax, Cmd+Enter generates an INLINE proposal
        // shown in the hint card (NOT a full Agent Chat chat handoff). Intercept
        // BEFORE the legacy Agent Chat-route guards.
        if self.try_route_cmd_enter_to_menu_syntax_ai(cx) {
            return true;
        }
        if self.show_actions_popup {
            return false;
        }
        if !Self::supports_global_cmd_enter_ai_entry(&self.current_view) {
            return false;
        }
        // Same attachment-portal guard as the plain Tab path: the portal
        // temporarily hosts a builtin view, but Cmd+Enter there must not
        // reopen / refocus the Agent Chat chat behind the portal.
        if self.is_in_attachment_portal() {
            tracing::debug!(
                target: "script_kit::tab_ai",
                event = "tab_ai_global_cmd_enter_suppressed_in_attachment_portal",
                source_view = %self.app_view_name(),
            );
            return false;
        }

        if let Some(intent) =
            self.script_list_cmd_enter_plain_prompt_intent(explicit_spine_prompt_plan)
        {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "tab_ai_global_cmd_enter_plain_prompt_routed_to_agent_chat",
                input_len = intent.len(),
            );
            self.one_window_open_chat(ChatEntry::sending(intent), cx);
            return true;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_global_cmd_enter_routed_to_chat",
            source_view = %self.app_view_name(),
        );
        self.open_chat_about_current_view(None, cx);
        true
    }

    fn script_list_cmd_enter_has_explicit_spine_prompt_plan(&mut self) -> bool {
        if !self.spine_enabled || !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }

        let raw = self.filter_text.clone();
        if raw.trim().is_empty() {
            return false;
        }

        self.set_spine_parse_from_filter_and_cursor(&raw, raw.len());
        crate::spine::parse_has_prompt_builder_segments(&self.spine_parse)
    }

    fn script_list_cmd_enter_plain_prompt_intent(
        &self,
        explicit_spine_prompt_plan: bool,
    ) -> Option<String> {
        if explicit_spine_prompt_plan || !matches!(self.current_view, AppView::ScriptList) {
            return None;
        }

        let intent = self.filter_text.trim();
        if intent.is_empty() {
            return None;
        }

        Some(intent.to_string())
    }

    /// Build a focused-target `AiContextPart` from the current view's
    /// resolved focus, if any. Returns `None` when the active surface has
    /// no resolvable focused item (e.g. empty list, generic prompt).
    fn build_tab_ai_focused_part_for_view(
        &self,
        source_view: &AppView,
        ui_snapshot: &crate::ai::TabAiUiSnapshot,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        let (focused_target, _visible_targets) =
            self.resolve_tab_ai_targets_with_audit_for_view(source_view, ui_snapshot, "pre_open");
        focused_target.map(|target| {
            let label = Self::format_tab_ai_focused_chip_label(&target);
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "tab_ai_focused_chip_label_built",
                item_source = %target.source,
                item_kind = %target.kind,
                label_chars = label.chars().count(),
            );
            crate::ai::message_parts::AiContextPart::FocusedTarget { target, label }
        })
    }

    /// Returns `true` when the Tab press should route to the Ask Anything
    /// fallback (ambient desktop context) instead of the focused-target chip
    /// path. This happens when no focused item is resolvable from the active
    /// surface.
    fn should_use_tab_ai_ask_anything_fallback(
        &self,
        source_view: &AppView,
        ui_snapshot: &crate::ai::TabAiUiSnapshot,
    ) -> bool {
        self.build_tab_ai_focused_part_for_view(source_view, ui_snapshot)
            .is_none()
    }

    /// Deferred-capture entry point: build a launch request from pre-switch
    /// state, start background capture, then immediately open the harness.
    ///
    /// The harness terminal appears within one frame of the Tab keypress.
    /// Context capture (desktop snapshot, screenshot-to-file) runs in the
    /// background and is injected into the live PTY once complete.
    fn begin_tab_ai_harness_entry(
        &mut self,
        options: TabAiHarnessEntryOptions,
        cx: &mut Context<Self>,
    ) {
        self.begin_tab_ai_harness_entry_from_source_view(self.current_view.clone(), options, cx);
    }

    /// Deferred-capture entry point with an explicit source view.
    ///
    /// This is the inner implementation that `begin_tab_ai_harness_entry`
    /// delegates to. The `source_view` parameter allows callers like the
    /// Agent Chat live-submit reroute to preserve the original source surface
    /// instead of using `self.current_view` (which may already be Agent Chat).
    fn begin_tab_ai_harness_entry_from_source_view(
        &mut self,
        source_view: AppView,
        options: TabAiHarnessEntryOptions,
        cx: &mut Context<Self>,
    ) {
        let TabAiHarnessEntryOptions {
            text,
            submit,
            stage_focused,
            quick_submit_plan,
            capture_kind,
        } = options;
        let snapshot_started_at = std::time::Instant::now();
        let (ui_snapshot, invocation_receipt) = if matches!(source_view, AppView::ScriptList) {
            self.snapshot_tab_ai_ui_fast_for_script_list()
        } else {
            self.snapshot_tab_ai_ui(cx)
        };
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_open_stage",
            stage = "snapshot_tab_ai_ui",
            stage_ms = snapshot_started_at.elapsed().as_millis() as u64,
            source_view = match &source_view {
                AppView::ScriptList => "ScriptList",
                _ => "Other",
            },
        );
        // Emit the receipt as a standalone structured log line for agent/test consumption
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_invocation_receipt",
            prompt_type = %invocation_receipt.prompt_type,
            input_status = %invocation_receipt.input_status,
            focus_status = %invocation_receipt.focus_status,
            elements_status = %invocation_receipt.elements_status,
            has_input_text = invocation_receipt.has_input_text,
            has_focus_target = invocation_receipt.has_focus_target,
            element_count = invocation_receipt.element_count,
            warning_count = invocation_receipt.warning_count,
            rich = invocation_receipt.rich,
            degradation_reasons = ?invocation_receipt.degradation_reasons,
            receipt_json = %serde_json::to_string(&invocation_receipt).unwrap_or_default(),
        );

        self.tab_ai_harness_capture_generation += 1;

        let request = TabAiLaunchRequest {
            source_view,
            text,
            submit,
            quick_submit_plan,
            ui_snapshot,
            invocation_receipt,
            capture_kind,
            capture_generation: self.tab_ai_harness_capture_generation,
        };

        // Compute surface preference from the shared verification markers
        // *before* branching so the decision is logged and deterministic.
        let effective_intent = Self::tab_ai_effective_submission_intent(&request);
        let submit_now = request
            .quick_submit_plan
            .as_ref()
            .map(|plan| plan.submit)
            .unwrap_or_else(|| request.requests_submission());
        let submission_mode = if submit_now {
            crate::ai::TabAiHarnessSubmissionMode::Submit
        } else {
            crate::ai::TabAiHarnessSubmissionMode::PasteOnly
        };
        let surface_preference = crate::ai::harness::tab_ai_surface_preference_for_prompt(
            &request.ui_snapshot.prompt_type,
            effective_intent.as_deref(),
            submission_mode,
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_surface_selected",
            surface = if surface_preference.use_quick_terminal { "quick_terminal" } else { "agent_chat" },
            reason = if surface_preference.use_quick_terminal { "script_verification_required" } else { "default_agent_chat" },
            prompt_type = %request.ui_snapshot.prompt_type,
            has_entry_intent = request.entry_text().is_some(),
            submit_now,
            includes_script_authoring_skill = surface_preference.includes_script_authoring_skill,
            includes_bun_build_verification = surface_preference.includes_bun_build_verification,
            includes_bun_execute_verification = surface_preference.includes_bun_execute_verification,
        );

        // Resolve whether we have a focused target or need the Ask Anything
        // fallback *before* spawning background capture.
        let should_stage_focused_part =
            Self::should_stage_focused_part_for_request(&request.source_view, stage_focused);
        let use_ask_anything_fallback = self
            .should_use_tab_ai_ask_anything_fallback(&request.source_view, &request.ui_snapshot);
        let use_ask_anything_fallback = should_stage_focused_part && use_ask_anything_fallback;

        // Explicit AI commands (screen, focused window, selected text, browser tab)
        // must force ambient capture even when the source surface has a focused item.
        let explicit_ambient_chip_label =
            Self::tab_ai_explicit_ambient_chip_label(&request.capture_kind).map(str::to_string);

        let focused_part = if use_ask_anything_fallback
            || explicit_ambient_chip_label.is_some()
            || !should_stage_focused_part
        {
            None
        } else {
            self.build_tab_ai_focused_part_for_view(&request.source_view, &request.ui_snapshot)
        };

        let captures = use_ask_anything_fallback || explicit_ambient_chip_label.is_some();
        if surface_preference.use_quick_terminal {
            // Only the Ask Anything fallback and explicit capture commands read the screen;
            // a focused-target Tab skips desktop snapshots and screenshots.
            let capture_rx = if captures {
                Self::spawn_tab_ai_pre_switch_capture(request.capture_kind)
            } else {
                async_channel::bounded::<Result<TabAiDeferredCaptureArtifacts, String>>(1).1
            };
            self.open_tab_ai_harness_terminal_from_request(request, capture_rx, cx);
            return;
        }
        let text = effective_intent.unwrap_or_default();
        let entry = if submit_now && !text.is_empty() {
            ChatEntry::sending(text)
        } else {
            ChatEntry::seeded(text)
        };
        let entry = entry.with_parts(focused_part);
        self.one_window_open_chat(
            if captures {
                entry.capturing(request.capture_kind)
            } else {
                entry
            },
            cx,
        );
    }

    /// Start background capture immediately on a dedicated OS thread.
    ///
    /// Captures the desktop context snapshot and (best-effort) focused window
    /// screenshot. Returns a channel receiver that delivers the results.
    ///
    /// Uses `std::thread::spawn` instead of `cx.spawn` + background executor
    /// so the expensive AX/screenshot work begins *immediately* — before the
    /// view switch can steal focus from the frontmost app.
    pub(crate) fn spawn_tab_ai_pre_switch_capture(
        capture_kind: crate::ai::TabAiCaptureKind,
    ) -> TabAiDeferredCaptureRx {
        let (tx, rx) = async_channel::bounded::<Result<TabAiDeferredCaptureArtifacts, String>>(1);

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                // Capture desktop context (text-safe, no screenshots in the blob)
                let desktop = crate::context_snapshot::capture_context_snapshot(
                    &crate::context_snapshot::CaptureContextOptions::tab_ai_submit(),
                );

                // Best-effort screenshot-to-file capture, branching on the
                // requested capture kind so explicit AI commands get the right
                // screenshot type instead of always defaulting to focused-window.
                let screenshot_path = match capture_kind {
                    crate::ai::TabAiCaptureKind::DefaultContext
                    | crate::ai::TabAiCaptureKind::FocusedWindow => {
                        match crate::ai::harness::capture_tab_ai_focused_window_screenshot_file() {
                            Ok(Some(file)) => Some(file.path),
                            Ok(None) => None,
                            Err(error) => {
                                tracing::debug!(
                                    event = "tab_ai_deferred_screenshot_failed",
                                    capture_kind = "focused_window",
                                    error = %error,
                                );
                                None
                            }
                        }
                    }
                    crate::ai::TabAiCaptureKind::FullScreen => {
                        match crate::ai::harness::capture_tab_ai_screen_screenshot_file() {
                            Ok(Some(file)) => Some(file.path),
                            Ok(None) => None,
                            Err(error) => {
                                tracing::debug!(
                                    event = "tab_ai_deferred_screenshot_failed",
                                    capture_kind = "full_screen",
                                    error = %error,
                                );
                                None
                            }
                        }
                    }
                    // Text-only captures (selected text, browser tab) skip screenshots.
                    crate::ai::TabAiCaptureKind::SelectedText
                    | crate::ai::TabAiCaptureKind::BrowserTab => None,
                };

                TabAiDeferredCaptureArtifacts {
                    desktop,
                    screenshot_path,
                }
            })
            .map_err(|_| "tab_ai_deferred_capture_panicked".to_string());

            let send_result = match result {
                Ok(artifacts) => tx.send_blocking(Ok(artifacts)),
                Err(error) => tx.send_blocking(Err(error)),
            };
            if send_result.is_err() {
                tracing::debug!(event = "tab_ai_deferred_capture_receiver_dropped");
            }
        });

        rx
    }

    /// Open the Agent Chat chat view immediately, then spawn a task that waits
    /// Compute the canonical submission intent for Agent Chat launches, matching the
    /// PTY path's normalization: prefer `quick_submit_plan.submission_intent()`
    /// over raw `entry_intent`, trim whitespace, and drop empty strings.
    fn tab_ai_effective_submission_intent(request: &TabAiLaunchRequest) -> Option<String> {
        request
            .quick_submit_plan
            .as_ref()
            .map(|plan| plan.submission_intent())
            .or(request.entry_text())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// Return a chip label for entry modes that must always use ambient capture,
    /// even if the source view has a resolvable focused target.
    fn tab_ai_explicit_ambient_chip_label(
        capture_kind: &crate::ai::TabAiCaptureKind,
    ) -> Option<&'static str> {
        match capture_kind {
            crate::ai::TabAiCaptureKind::DefaultContext => None,
            crate::ai::TabAiCaptureKind::FullScreen => Some("Full Screen"),
            crate::ai::TabAiCaptureKind::FocusedWindow => Some("Focused Window"),
            crate::ai::TabAiCaptureKind::SelectedText => Some("Selected Text"),
            crate::ai::TabAiCaptureKind::BrowserTab => Some("Browser Tab"),
        }
    }

    /// The launcher's rows are never implied: ⌘K on a row is how the chat is opened about one.
    fn should_stage_focused_part_for_request(source_view: &AppView, stage_focused: bool) -> bool {
        stage_focused && !matches!(source_view, AppView::ScriptList)
    }

    /// Extract a `TabAiTargetContext` from an `AiContextPart::FocusedTarget`,
    /// returning `None` for any other variant or `None` input.
    fn tab_ai_focused_target_from_part(
        part: Option<&crate::ai::message_parts::AiContextPart>,
    ) -> Option<crate::ai::TabAiTargetContext> {
        match part {
            Some(crate::ai::message_parts::AiContextPart::FocusedTarget { target, .. }) => {
                Some(target.clone())
            }
            _ => None,
        }
    }

    /// Seed `tab_ai_harness_apply_back_route` synchronously from the source
    /// view, UI snapshot, and an optional focused part.  When `focused_part`
    /// is `AiContextPart::FocusedTarget`, the concrete target metadata is
    /// preserved in the route so downstream apply-back consumers can identify
    /// what the user's chip refers to.
    fn seed_tab_ai_apply_back_route(
        &mut self,
        source_view: &AppView,
        ui_snapshot: &crate::ai::TabAiUiSnapshot,
        focused_part: Option<&crate::ai::message_parts::AiContextPart>,
    ) {
        let early_source_type = detect_tab_ai_source_type_early(source_view, ui_snapshot);
        let focused_target = Self::tab_ai_focused_target_from_part(focused_part);

        self.tab_ai_harness_apply_back_route = early_source_type.clone().and_then(|source_type| {
            build_tab_ai_apply_back_hint(Some(&source_type)).map(|hint| {
                crate::ai::TabAiApplyBackRoute {
                    source_type,
                    hint,
                    focused_target: focused_target.clone(),
                }
            })
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_apply_back_route_seeded",
            source_view = match source_view {
                AppView::ScriptList => "ScriptList",
                _ => "Other",
            },
            has_early_source_type = early_source_type.is_some(),
            has_focused_target = focused_target.is_some(),
            focused_target_kind = ?focused_target.as_ref().map(|t| t.kind.as_str()),
            focused_target_source = ?focused_target.as_ref().map(|t| t.source.as_str()),
        );
    }

    /// Open the harness terminal immediately, then spawn a task that waits
    /// for the deferred capture result and injects the enriched context.
    ///
    /// **Contract:** `AppView::QuickTerminalView` and `cx.notify()` happen
    /// *before* any deferred-capture await. The user sees the terminal cursor
    /// within one frame.
    fn open_tab_ai_harness_terminal_from_request(
        &mut self,
        request: TabAiLaunchRequest,
        capture_rx: TabAiDeferredCaptureRx,
        cx: &mut Context<Self>,
    ) {
        // Reuse a silently prewarmed session exactly once; otherwise force fresh.
        let reuse_fresh_prewarm = self
            .tab_ai_harness
            .as_ref()
            .map(|session| session.is_fresh_prewarm() && session.entity.read(cx).is_alive())
            .unwrap_or(false);

        if reuse_fresh_prewarm {
            if let Some(session) = self.tab_ai_harness.as_mut() {
                session.mark_consumed();
            }
        }

        let (entity, _was_cold_start) = match self
            .ensure_tab_ai_harness_terminal(!reuse_fresh_prewarm, cx)
        {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(
                    event = "tab_ai_harness_start_failed",
                    error = %error,
                );
                self.one_window_notice(crate::line_notice::FAILED, format!("Failed to start harness: {error}. Install the configured CLI or update claudeCode.path in ~/.scriptkit/config.ts"), cx);
                cx.notify();
                return;
            }
        };

        // Determine readiness based on actual PTY output, not cold-start flag.
        let wait_for_readiness = Self::tab_ai_harness_needs_readiness_wait(&entity, cx);

        tracing::debug!(
            event = "tab_ai_harness_submission_planned",
            wait_for_readiness,
            has_entry_intent = request.entry_text().is_some(),
        );

        // Save the originating surface so Escape and re-entry can use it
        self.tab_ai_harness_return_view = Some(request.source_view.clone());
        self.tab_ai_harness_return_focus_target = Some(self.tab_ai_return_focus_target());

        // Seed apply-back route synchronously so the footer shows the correct
        // Cmd+Enter label on first render and the first ⌘↩ press works without
        // waiting for the deferred capture.  PTY path has no focused part.
        self.seed_tab_ai_apply_back_route(&request.source_view, &request.ui_snapshot, None);

        // --- View switch FIRST: user sees the terminal immediately ---
        self.current_view = AppView::QuickTerminalView {
            entity: entity.clone(),
        };
        self.note_main_route_changed();
        self.focused_input = FocusedInput::None;
        self.clear_actions_popup_state();
        self.pending_focus = Some(FocusTarget::TermPrompt);

        // Deferred resize to avoid RefCell borrow error
        cx.spawn(async move |_this, _cx| {
            resize_to_view_sync(ViewType::TermPrompt, 0);
        })
        .detach();
        cx.notify();

        // --- Spawn deferred context injection task ---
        // This waits for the background capture, builds the full context blob
        // with source type / screenshot / apply-back hint, then injects.
        let app_weak = cx.entity().downgrade();
        let capture_gen = request.capture_generation;

        cx.spawn(async move |_this, cx| {
            // Wait for deferred capture to complete
            let capture_result = match capture_rx.recv().await {
                Ok(result) => result,
                Err(_) => Err("deferred capture channel closed".to_string()),
            };

            let artifacts = match capture_result {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!(
                        event = "tab_ai_deferred_capture_failed",
                        error = %e,
                    );
                    TabAiDeferredCaptureArtifacts::default()
                }
            };

            // Apply the captured context
            cx.update(|cx| {
                let Some(app) = app_weak.upgrade() else {
                    return;
                };
                app.update(cx, |this, cx| {
                    // Stale generation check
                    if this.tab_ai_harness_capture_generation != capture_gen {
                        tracing::debug!(
                            event = "tab_ai_deferred_capture_stale",
                            expected = capture_gen,
                            current = this.tab_ai_harness_capture_generation,
                        );
                        return;
                    }

                    let resolved = this.build_tab_ai_context_from(
                        request.entry_text().unwrap_or_default().to_string(),
                        request.source_view.clone(),
                        request.ui_snapshot.clone(),
                        artifacts.desktop,
                        request.invocation_receipt.clone(),
                        cx,
                    );

                    let source_type = detect_tab_ai_source_type(
                        &request.source_view,
                        &resolved.context.desktop,
                        resolved.context.focused_target.as_ref(),
                    );
                    let apply_back_hint = build_tab_ai_apply_back_hint(source_type.as_ref());

                    // Persist the apply-back route so ⌘⏎ can execute it later.
                    // Carry the focused target metadata so apply-back can route
                    // results without rediscovering UI state after the harness closes.
                    this.tab_ai_harness_apply_back_route = source_type
                        .clone()
                        .zip(apply_back_hint.clone())
                        .map(|(source_type, hint)| crate::ai::TabAiApplyBackRoute {
                            source_type,
                            hint,
                            focused_target: resolved.context.focused_target.clone(),
                        });

                    let context = resolved.context.with_deferred_capture_fields(
                        source_type,
                        artifacts.screenshot_path,
                        apply_back_hint,
                    );

                    let effective_intent = request
                        .quick_submit_plan
                        .as_ref()
                        .map(|plan| plan.submission_intent())
                        .or(request.entry_text());

                    let submit_now = request
                        .quick_submit_plan
                        .as_ref()
                        .map(|plan| plan.submit)
                        .unwrap_or_else(|| request.requests_submission());

                    let submission_mode = if submit_now {
                        crate::ai::TabAiHarnessSubmissionMode::Submit
                    } else {
                        crate::ai::TabAiHarnessSubmissionMode::PasteOnly
                    };

                    match crate::ai::build_tab_ai_harness_submission(
                        &context,
                        effective_intent,
                        submission_mode,
                        request.quick_submit_plan.as_ref(),
                        Some(&resolved.invocation_receipt),
                        &resolved.suggested_intents,
                    ) {
                        Ok(submission) => {
                            this.inject_tab_ai_harness_submission(
                                entity.clone(),
                                submission,
                                wait_for_readiness,
                                submit_now,
                                cx,
                            );
                        }
                        Err(error) => {
                            tracing::warn!(
                                event = "tab_ai_harness_context_build_failed",
                                error = %error,
                            );
                            this.one_window_notice(
                                crate::line_notice::FAILED,
                                format!("Failed to build harness context: {error}"),
                                cx,
                            );
                            cx.notify();
                        }
                    }
                });
            });
        })
        .detach();
    }

    /// Prewarm the configured harness at app startup so the first Tab press
    /// reuses a live PTY instead of paying spawn cost.
    ///
    /// This must be silent: no view switch, no focus change, no toast.
    /// User-facing errors still belong to the explicit Tab path.
    ///
    /// When `respect_startup_opt_out` is `true`, the `warmOnStartup` config
    /// flag is honoured — set for startup prewarm only.  The post-close
    /// reseed path passes `false` so the next Tab always feels instant even
    /// when startup warming is disabled.
    fn warm_tab_ai_harness_silently(
        &mut self,
        respect_startup_opt_out: bool,
        cx: &mut Context<Self>,
    ) {
        if let Some(existing) = &self.tab_ai_harness {
            if existing.entity.read(cx).is_alive() {
                tracing::debug!(
                    event = "tab_ai_harness_prewarm_skipped",
                    reason = "already_alive",
                );
                return;
            }
        }

        let config = match crate::ai::read_tab_ai_harness_config() {
            Ok(config) => config,
            Err(error) => {
                tracing::debug!(
                    event = "tab_ai_harness_prewarm_skipped",
                    reason = "config_read_failed",
                    error = %error,
                );
                return;
            }
        };

        if respect_startup_opt_out && !config.warm_on_startup {
            tracing::debug!(
                event = "tab_ai_harness_prewarm_skipped",
                reason = "disabled",
            );
            return;
        }

        if let Err(error) = crate::ai::validate_tab_ai_harness_config(&config) {
            tracing::debug!(
                event = "tab_ai_harness_prewarm_skipped",
                reason = "invalid_config",
                error = %error,
            );
            return;
        }

        match self.ensure_tab_ai_harness_terminal(false, cx) {
            Ok((_entity, was_cold_start)) => {
                // Tag newly created sessions as FreshPrewarm so the next Tab
                // reuses them exactly once instead of immediately killing them.
                if was_cold_start {
                    if let Some(session) = self.tab_ai_harness.as_mut() {
                        session.mark_fresh_prewarm();
                    }
                }
                tracing::info!(
                    event = "tab_ai_harness_prewarmed",
                    backend = ?config.backend,
                    command = %config.command,
                    was_cold_start,
                    source = if respect_startup_opt_out { "startup" } else { "post_close" },
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "tab_ai_harness_prewarm_failed",
                    error = %error,
                );
            }
        }
    }

    /// Startup prewarm: respects `warmOnStartup=false`.
    pub(crate) fn warm_tab_ai_harness_on_startup(&mut self, cx: &mut Context<Self>) {
        self.warm_tab_ai_harness_silently(true, cx);
    }

    /// Post-close prewarm: bypasses the startup opt-out so the next Tab
    /// always gets an instant fresh session after a close cycle.
    fn warm_tab_ai_harness_after_close(&mut self, cx: &mut Context<Self>) {
        self.warm_tab_ai_harness_silently(false, cx);
    }

    /// Ensure a harness terminal session exists and is alive.
    /// Returns the entity and whether this was a cold start (newly created).
    ///
    /// When `force_fresh` is `true`, any existing alive session is terminated
    /// first so the caller gets a brand-new PTY.  The startup and post-close
    /// prewarm paths pass `false` to seed a reusable session; the first
    /// explicit Tab press may reuse that `FreshPrewarm` once, and later
    /// explicit opens force a clean PTY again.
    fn ensure_tab_ai_harness_terminal(
        &mut self,
        force_fresh: bool,
        cx: &mut Context<Self>,
    ) -> Result<(gpui::Entity<crate::term_prompt::TermPrompt>, bool), String> {
        if force_fresh {
            // Kill the existing session so the user gets a clean slate.
            // Terminate FIRST, then clear the handle — if termination fails
            // the handle stays in app state so we don't lose track of a live PTY.
            if let Some(existing) = self.tab_ai_harness.as_ref() {
                existing.entity.update(cx, |term, _cx| {
                    term.terminate_session().map_err(|e| e.to_string())
                })?;
            }
            if self.tab_ai_harness.is_some() {
                self.tab_ai_harness = None;
                tracing::info!(event = "tab_ai_harness_old_session_terminated");
            }
        } else {
            // Reuse existing session if alive (prewarm path)
            if let Some(existing) = &self.tab_ai_harness {
                if existing.entity.read(cx).is_alive() {
                    return Ok((existing.entity.clone(), false));
                }
            }
        }

        let config = crate::ai::read_tab_ai_harness_config()?;
        crate::ai::validate_tab_ai_harness_config(&config)?;

        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {});

        let term_height = crate::window_resize::layout::MAX_HEIGHT
            - gpui::px(crate::window_resize::layout::FOOTER_HEIGHT);

        let mut prompt = crate::term_prompt::TermPrompt::with_height(
            "tab-ai-harness".to_string(),
            Some(config.command_line()),
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        )
        .map_err(|e| format!("tab_ai_harness_terminal_create_failed: {e}"))?;

        // Let the smart routing in scroll_to_pty() decide: when the TUI
        // enables mouse mode (Claude Code, etc.), scroll events are forwarded
        // as escape sequences so the TUI can handle scrolling internally.
        // When mouse mode is off, scroll falls back to local display buffer.
        prompt.prefer_buffer_scroll_on_wheel = false;

        let entity = cx.new(|_| prompt);

        tracing::info!(
            event = "tab_ai_harness_terminal_created",
            backend = ?config.backend,
            command = %config.command,
        );

        self.tab_ai_harness = Some(crate::ai::TabAiHarnessSessionState::new(
            config,
            entity.clone(),
            "tab-ai-harness",
        ));

        Ok((entity, true))
    }

    /// Maximum time (ms) to wait for a cold-started harness to produce its
    /// first output before injecting context anyway.
    const HARNESS_READINESS_TIMEOUT_MS: u64 = 2000;

    /// Interval (ms) between readiness polls during cold-start wait.
    const HARNESS_READINESS_POLL_MS: u64 = 20;

    /// Check whether a harness session needs to wait for prompt readiness
    /// before context paste.  Returns `true` when the PTY has not yet
    /// produced any output — regardless of whether it was cold-started or
    /// prewarmed.
    fn tab_ai_harness_needs_readiness_wait(
        entity: &gpui::Entity<crate::term_prompt::TermPrompt>,
        cx: &Context<Self>,
    ) -> bool {
        !entity.read(cx).has_received_output
    }

    /// Inject the context submission into the harness PTY, with a readiness
    /// gate that fires whenever the PTY has not yet produced output.
    ///
    /// Polls `has_received_output` on the `TermPrompt` entity up to
    /// [`HARNESS_READINESS_TIMEOUT_MS`] before injecting.  Falls back
    /// deterministically if the harness does not produce output in time.
    /// This applies to both cold-started and prewarmed sessions that have
    /// not yet rendered their first prompt.
    ///
    /// When `submit` is true, the payload is sent as a full line (appends CR).
    /// When false, the payload is pasted without a trailing CR so the user
    /// can type their intent before pressing Enter.
    fn inject_tab_ai_harness_submission(
        &self,
        entity: gpui::Entity<crate::term_prompt::TermPrompt>,
        submission: String,
        wait_for_readiness: bool,
        submit: bool,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity().downgrade();
        let entity_weak = entity.downgrade();

        cx.spawn(async move |_this, cx| {
            // Wait until the harness has produced output (its prompt/banner),
            // with a bounded timeout as fallback.  This fires for both
            // cold-started and prewarmed sessions that are not yet ready.
            if wait_for_readiness {
                let poll_interval =
                    std::time::Duration::from_millis(Self::HARNESS_READINESS_POLL_MS);
                let deadline = std::time::Instant::now()
                    + std::time::Duration::from_millis(Self::HARNESS_READINESS_TIMEOUT_MS);

                loop {
                    let is_ready = cx.update(|cx| {
                        entity_weak
                            .upgrade()
                            .map(|e| e.read(cx).has_received_output)
                            .unwrap_or(true) // entity gone → skip waiting
                    });

                    if is_ready {
                        tracing::debug!(
                            event = "tab_ai_harness_readiness_detected",
                            elapsed_ms = %std::time::Instant::now()
                                .duration_since(deadline - std::time::Duration::from_millis(Self::HARNESS_READINESS_TIMEOUT_MS))
                                .as_millis(),
                        );
                        break;
                    }
                    if std::time::Instant::now() >= deadline {
                        tracing::warn!(
                            event = "tab_ai_harness_readiness_timeout",
                            timeout_ms = Self::HARNESS_READINESS_TIMEOUT_MS,
                        );
                        break;
                    }
                    cx.background_executor().timer(poll_interval).await;
                }
            }

            cx.update(|cx| {
                let Some(entity) = entity_weak.upgrade() else {
                    return;
                };
                let result = entity.update(cx, |term, _cx| {
                    if submit {
                        term.send_line(&submission).map_err(|e| e.to_string())
                    } else {
                        term.send_text_as_paste(&submission)
                            .map_err(|e| e.to_string())
                    }
                });
                if let Err(error) = result {
                    if let Some(app) = app.upgrade() {
                        app.update(cx, |this, cx| {
                            this.one_window_notice(crate::line_notice::FAILED, format!("Failed to inject Tab AI context: {error}"), cx);
                            cx.notify();
                        });
                    }
                }
            });
        })
        .detach();
    }

    /// Terminate any active PTY harness session without restoring views.
    ///
    /// Used at Agent Chat open to kill a stale prewarm session, and by close to
    /// tear down the PTY for `QuickTerminalView`.
    fn terminate_tab_ai_harness_session(&mut self, cx: &mut Context<Self>) {
        if let Some(session) = self.tab_ai_harness.as_ref() {
            let result = session.entity.update(cx, |term, _cx| {
                term.terminate_session().map_err(|e| e.to_string())
            });
            match result {
                Ok(()) => {
                    self.tab_ai_harness = None;
                }
                Err(error) => {
                    tracing::warn!(
                        event = "tab_ai_harness_terminal_kill_failed",
                        error = %error,
                    );
                }
            }
        }
    }

    /// Close the Tab AI harness surface.
    ///
    /// **Close semantics contract:**
    /// - `Cmd+W` closes the Quick Terminal main window state-first (handled in `render_prompts/term.rs`).
    /// - Restore-origin callers still use the saved return view/focus for apply-back and embedded Agent Chat close.
    /// - Plain `Escape` is forwarded to the PTY so the harness TUI can handle it.
    /// - The footer advertises the same Close affordance as Cmd+W for Quick Terminal.
    ///
    /// **Lifecycle contract:** tears down the PTY, clears the harness, schedules prewarm.
    fn clear_transient_script_list_trigger_on_return(
        &mut self,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.current_view, AppView::ScriptList) {
            self.tab_ai_harness_script_list_trigger = None;
            return;
        }

        let Some(trigger) = self.tab_ai_harness_script_list_trigger.take() else {
            return;
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "script_list_transient_trigger_cleared",
            trigger = %trigger,
            via_window = window.is_some(),
        );

        if let Some(window) = window {
            self.clear_filter(window, cx);
        } else {
            self.reset_script_list_filter_and_selection_state(cx);
        }
    }

    fn close_tab_ai_harness_terminal_impl(
        &mut self,
        window: Option<&mut Window>,
        disposition: TabAiHarnessCloseDisposition,
        cx: &mut Context<Self>,
    ) {
        let pending_script_list_trigger = self.tab_ai_harness_script_list_trigger;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_close_with_window_decision",
            return_is_script_list = self
                .tab_ai_harness_return_view
                .as_ref()
                .is_none_or(|v| matches!(v, AppView::ScriptList)),
            has_pending_script_list_trigger = pending_script_list_trigger.is_some(),
            pending_script_list_trigger = ?pending_script_list_trigger,
            filter_sha256 = %crate::logging::log_private_user_value(&self.filter_text),
            filter_bytes = self.filter_text.len(),
            current_view = ?self.current_view,
        );
        if !matches!(self.current_view, AppView::QuickTerminalView { .. }) {
            return;
        }

        // Invalidate any in-flight deferred capture so late results cannot
        // inject stale context after the surface has been closed.
        self.tab_ai_harness_capture_generation += 1;

        // Clear the apply-back route so stale targets cannot leak across sessions.
        self.tab_ai_harness_apply_back_route = None;

        self.terminate_tab_ai_harness_session(cx);

        let return_view = self
            .tab_ai_harness_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_is_script_list = matches!(return_view, AppView::ScriptList);
        let return_focus_target = self
            .tab_ai_harness_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            event = "tab_ai_harness_terminal_close",
            focus_target = %format!("{return_focus_target:?}"),
            session_cleared = true,
            capture_generation = self.tab_ai_harness_capture_generation,
            disposition = ?disposition,
        );

        if disposition == TabAiHarnessCloseDisposition::CloseMainWindowStateFirst {
            self.tab_ai_harness_script_list_trigger = None;

            self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx);
            self.close_and_reset_window(cx);
            cx.notify();
            return;
        }

        self.restore_current_view_with_focus(return_view, return_focus_target);

        // Returning to the launcher root consumes the "opened from main menu"
        // origin. Leaving it set makes the NEXT Escape on the empty ScriptList
        // run a no-op go_back_or_close instead of hiding the window (the
        // "brain item → Agent Chat → Escape → Escape doesn't close" bug).
        if matches!(self.current_view, AppView::ScriptList) && !self.is_in_attachment_portal() {
            self.opened_from_main_menu = false;
        }

        if return_is_script_list {
            self.clear_transient_script_list_trigger_on_return(window, cx);
        } else {
            self.tab_ai_harness_script_list_trigger = None;
        }

        self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx);
        cx.notify();
    }

    pub(crate) fn close_tab_ai_harness_terminal_with_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_tab_ai_harness_terminal_impl(
            Some(window),
            TabAiHarnessCloseDisposition::RestoreOrigin,
            cx,
        );
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(cx);
        }
    }

    pub(crate) fn close_tab_ai_harness_terminal(&mut self, cx: &mut Context<Self>) {
        // close_tab_ai_harness_terminal_impl(None, cx) owns the real contract:
        // - it guards the restore path to AppView::QuickTerminalView
        // - tab_ai_harness_capture_generation += 1
        // - self.tab_ai_harness_apply_back_route = None;
        // - self.terminate_tab_ai_harness_session(cx);
        // - self.restore_current_view_with_focus(return_view, return_focus_target);
        // - the close log records session_cleared = closing_quick_terminal
        // - it requeues schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx)
        // - it ends with cx.notify()
        self.close_tab_ai_harness_terminal_impl(
            None,
            TabAiHarnessCloseDisposition::RestoreOrigin,
            cx,
        );
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(cx);
        }
    }

    pub(crate) fn close_quick_terminal_main_window_state_first(&mut self, cx: &mut Context<Self>) {
        // State-first Quick Terminal close intentionally does not restore the
        // saved return view while the main panel is visible. `close_and_reset_window`
        // sets windowVisible=false before prompt cancellation/reset can produce
        // a hidden ScriptList state.
        self.close_tab_ai_harness_terminal_impl(
            None,
            TabAiHarnessCloseDisposition::CloseMainWindowStateFirst,
            cx,
        );
    }

    /// Schedule a deferred prewarm of the Tab AI harness so the next Tab
    /// press gets an instant fresh session after a close cycle.
    fn schedule_tab_ai_harness_prewarm(
        &mut self,
        delay: std::time::Duration,
        cx: &mut Context<Self>,
    ) {
        let app_weak = cx.entity().downgrade();
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|cx| {
                if let Some(app) = app_weak.upgrade() {
                    app.update(cx, |this, cx| {
                        this.warm_tab_ai_harness_after_close(cx);
                    });
                }
            });
        })
        .detach();
    }

    /// Build context from explicit inputs, resolving targets and clipboard
    /// against the provided `source_view` (the view that was active when Tab
    /// was pressed) rather than `self.current_view` (which may now differ).
    fn build_tab_ai_context_from(
        &self,
        intent_for_lookup: String,
        source_view: AppView,
        ui: crate::ai::TabAiUiSnapshot,
        desktop: crate::context_snapshot::AiContextSnapshot,
        invocation_receipt: crate::ai::TabAiInvocationReceipt,
        _cx: &Context<Self>,
    ) -> TabAiResolvedContext {
        let bundle_id = desktop
            .frontmost_app
            .as_ref()
            .map(|app| app.bundle_id.clone());
        let recent_inputs = self.input_history.recent_entries(5);
        let clipboard_selected_index = match &source_view {
            AppView::ClipboardHistoryView { selected_index, .. } => Some(*selected_index),
            _ => None,
        };
        let clipboard_history = self.resolve_tab_ai_clipboard_history(
            clipboard_selected_index,
            TAB_AI_CLIPBOARD_HISTORY_LIMIT,
        );
        let clipboard = clipboard_history
            .first()
            .map(|entry| crate::ai::TabAiClipboardContext {
                content_type: entry.content_type.clone(),
                preview: entry.preview.clone(),
                ocr_text: entry.ocr_text.clone(),
            });
        let prior_automations = match crate::ai::resolve_tab_ai_prior_automations_for_entry(
            &intent_for_lookup,
            bundle_id.as_deref(),
            3,
        ) {
            Ok(items) => items,
            Err(error) => {
                tracing::warn!(event = "tab_ai_prior_automation_lookup_failed", error = %error);
                Vec::new()
            }
        };
        let (focused_target, visible_targets) =
            self.resolve_tab_ai_targets_with_audit_for_view(&source_view, &ui, "submit_context");

        let suggested_intents = crate::ai::build_tab_ai_suggested_intents(
            focused_target.as_ref(),
            clipboard.as_ref(),
            &prior_automations,
        );

        let context = crate::ai::TabAiContextBlob::from_parts_with_targets(
            ui,
            focused_target,
            visible_targets,
            desktop,
            recent_inputs,
            clipboard,
            clipboard_history,
            prior_automations,
            chrono::Utc::now().to_rfc3339(),
        );

        TabAiResolvedContext {
            context,
            invocation_receipt,
            suggested_intents,
        }
    }

    pub(super) fn agent_chat_input_return_state(&self, cx: &App) -> AgentChatInputReturnState {
        AgentChatInputReturnState {
            value: self.filter_text.clone(),
            selection: self.gpui_input_state.read(cx).selected_range(),
            focused_input: self.focused_input,
            pending_focus: self.pending_focus,
        }
    }

    pub(crate) fn tab_ai_return_focus_target_for_view(source_view: &AppView) -> FocusTarget {
        match source_view {
            AppView::ScriptList
            | AppView::ClipboardHistoryView { .. }
            | AppView::AppLauncherView { .. }
            | AppView::WindowSwitcherView { .. }
            | AppView::BrowserTabsView { .. }
            | AppView::FileSearchView { .. }
            | AppView::ProfileSearchView { .. }
            | AppView::ThemeChooserView { .. }
            | AppView::EmojiPickerView { .. }
            | AppView::BrowseKitsView { .. }
            | AppView::MigrateV1View { .. }
            | AppView::InstalledKitsView { .. }
            | AppView::ProcessManagerView { .. }
            | AppView::FlowUxView { .. }
            | AppView::SearchAiPresetsView { .. }
            | AppView::CreateAiPresetView { .. }
            | AppView::SettingsView { .. }
            | AppView::FavoritesBrowseView { .. }
            | AppView::BrowserHistoryView { .. }
            | AppView::NotesBrowseView { .. }
            | AppView::CurrentAppCommandsView { .. }
            | AppView::CreationFeedback { .. }
            | AppView::ScriptIssuesView { .. }
            | AppView::SdkReferenceView { .. }
            | AppView::TipsView { .. }
            | AppView::ScriptTemplateCatalogView { .. }
            | AppView::ActionsDialog => FocusTarget::MainFilter,

            AppView::About { .. } => FocusTarget::AppRoot,

            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. }
            | AppView::DivPrompt { .. }
            | AppView::WebcamView { .. } => FocusTarget::AppRoot,

            AppView::FormPrompt { .. } => FocusTarget::FormPrompt,

            AppView::EditorPrompt { .. } | AppView::ScratchPadView { .. } => {
                FocusTarget::EditorPrompt
            }

            AppView::SelectPrompt { .. } => FocusTarget::SelectPrompt,
            AppView::PathPrompt { .. } => FocusTarget::PathPrompt,
            AppView::EnvPrompt { .. } => FocusTarget::EnvPrompt,
            AppView::DropPrompt { .. } => FocusTarget::DropPrompt,
            AppView::TemplatePrompt { .. } => FocusTarget::TemplatePrompt,

            AppView::TermPrompt { .. } | AppView::QuickTerminalView { .. } => {
                FocusTarget::TermPrompt
            }

            AppView::ChatPrompt { .. } => FocusTarget::ChatPrompt,
            // Flow sessions compose in the shared MAIN input.
            AppView::FlowSessionView { .. } => FocusTarget::MainFilter,
            AppView::DayPage { .. } => FocusTarget::EditorPrompt,
            AppView::NamingPrompt { .. } => FocusTarget::NamingPrompt,

            AppView::PermissionsWizardView { .. } => FocusTarget::AppRoot,
        }
    }

    /// Return the correct `FocusTarget` for the originating surface so that
    /// closing the Tab AI overlay restores focus to the right place.
    fn tab_ai_return_focus_target(&self) -> FocusTarget {
        Self::tab_ai_return_focus_target_for_view(&self.current_view)
    }

    /// Resolve a bounded clipboard history with hydrated content from the
    /// clipboard database. The selected entry (if any) is always first,
    /// followed by other recent entries in order, deduplicated by content.
    fn resolve_tab_ai_clipboard_history(
        &self,
        selected_index: Option<usize>,
        limit: usize,
    ) -> Vec<crate::ai::TabAiClipboardHistoryEntry> {
        let mut ordered: Vec<&crate::clipboard_history::ClipboardEntryMeta> = Vec::new();

        // Selected entry goes first
        if let Some(selected) =
            selected_index.and_then(|index| self.cached_clipboard_entries.get(index))
        {
            ordered.push(selected);
        }

        for entry in &self.cached_clipboard_entries {
            if ordered.iter().any(|candidate| candidate.id == entry.id) {
                continue;
            }
            ordered.push(entry);
            if ordered.len() >= limit {
                break;
            }
        }

        let mut last_dedupe_key: Option<String> = None;
        let mut result = Vec::new();

        for entry in ordered {
            // Hydrate text content from the database for text-like entries
            let full_text = match entry.content_type {
                crate::clipboard_history::ContentType::Text
                | crate::clipboard_history::ContentType::Link
                | crate::clipboard_history::ContentType::File
                | crate::clipboard_history::ContentType::Color => {
                    crate::clipboard_history::get_entry_content(&entry.id)
                        .filter(|content| !content.trim().is_empty())
                        .map(|content| {
                            crate::ai::truncate_tab_ai_text(&content, TAB_AI_CLIPBOARD_TEXT_LIMIT)
                        })
                }
                crate::clipboard_history::ContentType::Image => None,
            };

            let preview_source = full_text
                .clone()
                .or_else(|| entry.ocr_text.clone())
                .unwrap_or_else(|| entry.display_preview());

            // Deduplicate consecutive identical entries
            let dedupe_key = format!("{}::{}", entry.content_type.as_str(), preview_source);
            if last_dedupe_key.as_deref() == Some(dedupe_key.as_str()) {
                continue;
            }
            last_dedupe_key = Some(dedupe_key);

            result.push(crate::ai::TabAiClipboardHistoryEntry {
                id: entry.id.clone(),
                content_type: entry.content_type.as_str().to_string(),
                timestamp: entry.timestamp,
                preview: crate::ai::truncate_tab_ai_text(
                    &preview_source,
                    TAB_AI_CLIPBOARD_TEXT_LIMIT,
                ),
                full_text,
                ocr_text: entry
                    .ocr_text
                    .clone()
                    .filter(|text| !text.trim().is_empty())
                    .map(|text| {
                        crate::ai::truncate_tab_ai_text(&text, TAB_AI_CLIPBOARD_TEXT_LIMIT)
                    }),
                image_width: entry.image_width,
                image_height: entry.image_height,
            });
        }

        result
    }

    fn tab_ai_target_from_element(
        prompt_type: &str,
        element: &crate::protocol::ElementInfo,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: prompt_type.to_string(),
            kind: format!("{:?}", element.element_type).to_lowercase(),
            semantic_id: element.semantic_id.clone(),
            label: element
                .text
                .clone()
                .or_else(|| element.value.clone())
                .unwrap_or_else(|| element.semantic_id.clone()),
            metadata: Some(serde_json::json!({
                "text": element.text.clone(),
                "value": element.value.clone(),
                "selected": element.selected,
                "focused": element.focused,
                "index": element.index,
            })),
        }
    }

    /// Convert a `SearchResult` from the Script List into a `TabAiTargetContext`
    /// with script-native metadata (name, path, description, type).
    /// The launcher's highlighted row as something the chat can be about.
    pub(crate) fn one_window_highlighted_part(
        &self,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        let row = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.grouped_index == self.selected_index)?;
        let target = self.tab_ai_target_from_main_menu_row(row)?;
        let label = Self::format_tab_ai_focused_chip_label(&target);
        Some(crate::ai::message_parts::AiContextPart::FocusedTarget { target, label })
    }

    fn tab_ai_target_from_main_menu_row(
        &self,
        row: &crate::MainMenuRowProjection,
    ) -> Option<crate::ai::TabAiTargetContext> {
        match row.subject {
            crate::MainMenuRowSubject::SearchResult { flat_index } => {
                Some(Self::tab_ai_target_from_search_result(
                    row,
                    self.main_menu_committed_results().get(flat_index)?,
                ))
            }
            crate::MainMenuRowSubject::Calculator => {
                let result = self.main_menu_committed_calculator()?;
                Some(crate::ai::TabAiTargetContext {
                    source: "ScriptList".to_string(),
                    kind: "calculator".to_string(),
                    semantic_id: row.semantic_id.clone(),
                    label: result.formatted.clone(),
                    metadata: Some(
                        serde_json::json!({"expression": result.normalized_expr, "result": result.formatted}),
                    ),
                })
            }
        }
    }

    pub(crate) fn tab_ai_target_from_search_result(
        row: &crate::MainMenuRowProjection,
        result: &scripts::SearchResult,
    ) -> crate::ai::TabAiTargetContext {
        let name = result.name().to_string();
        let kind = match result {
            scripts::SearchResult::Script(_) => "script",
            scripts::SearchResult::Scriptlet(_) => "scriptlet",
            scripts::SearchResult::BuiltIn(_) => "builtin",
            scripts::SearchResult::App(_) => "app",
            scripts::SearchResult::Window(_) => "window",
            scripts::SearchResult::File(_) => "file",
            scripts::SearchResult::Note(_) => "note",
            scripts::SearchResult::BrainHit(_) => "brain",
            scripts::SearchResult::BrainInboxItem(_) => "brainInbox",
            scripts::SearchResult::Todo(_) => "todo",
            scripts::SearchResult::AgentChatHistory(_) => "agent_chatHistory",
            scripts::SearchResult::AiVault(_) => "aiVault",
            scripts::SearchResult::ClipboardHistory(_) => "clipboardHistory",
            scripts::SearchResult::DictationHistory(_) => "dictationHistory",
            scripts::SearchResult::BrowserTab(_) => "browserTab",
            scripts::SearchResult::BrowserHistory(_) => "browserHistory",
            scripts::SearchResult::Agent(_) => "agent",
            scripts::SearchResult::Skill(_) => "skill",
            scripts::SearchResult::Flow(_) => "flow",
            scripts::SearchResult::Fallback(_) => "fallback",
            scripts::SearchResult::ScriptIssue(_) => "scriptIssue",
            scripts::SearchResult::Intent(_) => "intent",
            scripts::SearchResult::OneWindow(_) => "oneWindow",
            scripts::SearchResult::SpineProjection(_) => "spineProjection",
        };
        let metadata = match result {
            scripts::SearchResult::Script(m) => serde_json::json!({
                "name": m.script.name,
                "path": m.script.path.to_string_lossy(),
                "description": m.script.description,
                "shortcut": m.script.shortcut,
                "alias": m.script.alias,
            }),
            scripts::SearchResult::Scriptlet(m) => serde_json::json!({
                "name": m.scriptlet.name,
                "description": m.scriptlet.description,
                "filePath": m.scriptlet.file_path,
            }),
            scripts::SearchResult::BuiltIn(m) => serde_json::json!({
                "id": m.entry.id,
                "name": m.entry.name,
                "description": m.entry.description,
            }),
            scripts::SearchResult::App(m) => serde_json::json!({
                "name": m.app.name,
                "path": m.app.path.to_string_lossy(),
                "bundleId": m.app.bundle_id,
            }),
            scripts::SearchResult::Window(m) => serde_json::json!({
                "app": m.window.app,
                "title": m.window.title,
            }),
            scripts::SearchResult::File(m) => serde_json::json!({
                "name": m.file.name,
                "path": m.file.path,
                "fileType": format!("{:?}", m.file.file_type),
            }),
            scripts::SearchResult::Note(m) => serde_json::json!({
                "noteId": m.hit.id.as_str(),
                "title": m.title,
                "updatedAt": m.hit.updated_at.to_rfc3339(),
                "isPinned": m.hit.is_pinned,
                "charCount": m.hit.char_count,
            }),
            scripts::SearchResult::BrainHit(m) => serde_json::json!({
                "source": m.hit.source.as_str(),
                "sourceId": m.hit.source_id,
                "title": m.hit.title,
                "excerpt": m.hit.excerpt,
            }),
            scripts::SearchResult::BrainInboxItem(m) => serde_json::json!({
                "id": m.item.id,
                "kind": m.item.kind.as_str(),
                "title": m.item.title,
                "detail": m.item.detail,
                "source": m.item.source,
                "sourceId": m.item.source_id,
            }),
            scripts::SearchResult::Todo(m) => serde_json::json!({
                "stableKey": m.hit.stable_key,
                "title": m.hit.title,
                "body": m.hit.body,
                "subtitle": m.hit.subtitle,
                "tags": m.hit.tags,
                "priority": m.hit.priority,
                "due": m.hit.due,
                "createdAt": m.hit.created_at,
            }),
            scripts::SearchResult::AgentChatHistory(m) => serde_json::json!({
                "sessionId": m.entry.session_id,
                "title": m.entry.title_display(),
                "preview": m.entry.preview_display(),
                "messageCount": m.entry.message_count,
                "matchedField": format!("{:?}", m.matched_field),
            }),
            scripts::SearchResult::AiVault(m) => serde_json::json!({
                "stableKey": m.hit.stable_key,
                "provider": m.hit.provider,
                "providerDisplayName": m.hit.provider_display_name,
                "sessionId": m.hit.session_id,
                "sourceKind": m.hit.source_kind,
                "safeTitle": m.hit.safe_title,
                "workspacePath": m.hit.workspace_path,
                "model": m.hit.model,
                "modifiedAt": m.hit.modified_at,
                "matchedField": format!("{:?}", m.hit.matched_field),
            }),
            scripts::SearchResult::ClipboardHistory(m) => serde_json::json!({
                "id": m.entry.id,
                "contentType": m.entry.content_type.as_str(),
                "preview": m.title,
                "pinned": m.entry.pinned,
                "timestamp": m.entry.timestamp,
            }),
            scripts::SearchResult::DictationHistory(m) => serde_json::json!({
                "id": m.id,
                "preview": m.preview,
                "target": m.target,
                "timestamp": m.timestamp,
                "audioDurationMs": m.audio_duration_ms,
                "matchedField": format!("{:?}", m.matched_field),
            }),
            scripts::SearchResult::BrowserTab(m) => serde_json::json!({
                "stableKey": m.hit.stable_key,
                "title": m.hit.title,
                "url": m.hit.url,
                "domain": m.hit.domain,
                "provider": m.hit.provider_label,
                "windowIndex": m.hit.tab.window_index,
                "tabIndex": m.hit.tab.tab_index,
            }),
            scripts::SearchResult::BrowserHistory(m) => serde_json::json!({
                "stableKey": m.hit.stable_key,
                "title": m.hit.title,
                "url": m.hit.url,
                "domain": m.hit.domain,
                "provider": m.hit.provider_label,
                "profile": m.hit.profile_label,
                "lastVisitUnixMs": m.hit.last_visit_unix_ms,
                "visitCount": m.hit.visit_count,
            }),
            scripts::SearchResult::Agent(m) => serde_json::json!({
                "name": m.agent.name,
                "path": m.agent.path.to_string_lossy(),
                "description": m.agent.description,
            }),
            scripts::SearchResult::Flow(m) => serde_json::json!({
                "flowId": m.flow.as_ref().map(|flow| flow.id.clone()),
                "name": m.flow.as_ref().map(|flow| flow.name.clone()),
                "displayName": m.display_name,
                "path": m.flow.as_ref().map(|flow| flow.path.clone()),
                "description": m.flow.as_ref().and_then(|flow| flow.description.clone()),
                "engine": m.flow.as_ref().map(|flow| flow.engine.clone()),
                "origin": m.flow.as_ref().map(|flow| flow.origin_label()),
                "conversationId": m
                    .conversation_id()
                    .map(crate::ai::conversations::ConversationSessionId::automation_id),
            }),
            scripts::SearchResult::Skill(m) => serde_json::json!({
                "pluginId": m.skill.plugin_id,
                "skillId": m.skill.skill_id,
                "title": m.skill.title,
                "description": m.skill.description,
                "path": m.skill.path.to_string_lossy(),
            }),
            scripts::SearchResult::Fallback(m) => serde_json::json!({
                "name": m.display_label(),
                "description": m.display_description(),
            }),
            scripts::SearchResult::ScriptIssue(m) => serde_json::json!({
                "title": m.title,
                "description": m.description,
                "failedCount": m.failed_count,
                "fatalCount": m.fatal_count,
                "warningCount": m.warning_count,
            }),
            scripts::SearchResult::OneWindow(m) => serde_json::json!({
                "title": m.row.title,
                "key": m.row.key,
            }),
            scripts::SearchResult::Intent(m) => serde_json::json!({
                "title": m.title,
                "pathway": m.plan.as_ref().map(|plan| plan.pathway.key()),
                "typed": m.plan.as_ref().and_then(|plan| plan.typed.clone()),
            }),
            scripts::SearchResult::SpineProjection(_) => serde_json::json!({}),
        };
        crate::ai::TabAiTargetContext {
            source: "ScriptList".to_string(),
            kind: kind.to_string(),
            semantic_id: row.semantic_id.clone(),
            label: name,
            metadata: Some(metadata),
        }
    }

    fn tab_ai_target_from_app_launcher_row(
        display_index: usize,
        source_index: usize,
        app: &app_launcher::AppInfo,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: "AppLauncher".to_string(),
            kind: "app".to_string(),
            semantic_id: crate::protocol::generate_semantic_id("choice", display_index, &app.name),
            label: app.name.clone(),
            metadata: Some(serde_json::json!({
                "name": app.name.clone(),
                "path": app.path.to_string_lossy(),
                "bundleId": app.bundle_id.clone(),
                "sourceIndex": source_index,
            })),
        }
    }

    fn tab_ai_target_from_process_manager_row(
        display_index: usize,
        source_index: usize,
        process: &crate::process_manager::ProcessInfo,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: "ProcessManager".to_string(),
            kind: "process".to_string(),
            semantic_id: crate::protocol::generate_semantic_id(
                "choice",
                display_index,
                &process.script_path,
            ),
            label: process.script_path.clone(),
            metadata: Some(serde_json::json!({
                "pid": process.pid,
                "scriptPath": process.script_path.clone(),
                "startedAt": process.started_at.to_rfc3339(),
                "sourceIndex": source_index,
            })),
        }
    }

    fn tab_ai_target_from_sdk_reference_row(
        row: crate::mcp_resources::SdkReferenceVisibleRow<'_>,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: "SdkReference".to_string(),
            kind: "sdk_function".to_string(),
            semantic_id: crate::protocol::generate_semantic_id(
                "choice",
                row.display_index,
                &row.entry.name,
            ),
            label: row.entry.name.clone(),
            metadata: Some(serde_json::json!({
                "name": row.entry.name.clone(),
                "signature": row.entry.signature.clone(),
                "description": row.entry.description.clone(),
                "category": row.entry.category.clone(),
                "support": row.entry.support,
                "unsupportedNote": row.entry.unsupported_note.clone(),
                "sourceIndex": row.source_index,
            })),
        }
    }

    fn tab_ai_target_from_script_template_catalog_row(
        row: crate::mcp_resources::ScriptTemplateCatalogVisibleRow<'_>,
    ) -> crate::ai::TabAiTargetContext {
        crate::ai::TabAiTargetContext {
            source: "ScriptTemplateCatalog".to_string(),
            kind: "script_template".to_string(),
            semantic_id: crate::protocol::generate_semantic_id(
                "choice",
                row.display_index,
                &row.template.title,
            ),
            label: row.template.title.clone(),
            metadata: Some(serde_json::json!({
                "id": row.template.id.clone(),
                "title": row.template.title.clone(),
                "description": row.template.description.clone(),
                "category": row.template.category.clone(),
                "filenameHint": row.template.filename_hint.clone(),
                "sourceIndex": row.source_index,
            })),
        }
    }

    /// Return the first `limit` file search results in display order.
    fn visible_file_search_results(
        &self,
        limit: usize,
    ) -> Vec<(usize, &crate::file_search::FileResult)> {
        (0..self.file_search_display_indices.len())
            .take(limit)
            .filter_map(|display_index| {
                self.file_search_result_at_display_index(display_index)
                    .map(|result| (display_index, result))
            })
            .collect()
    }

    /// Determine the query mode label for file search AI context.
    fn file_search_query_mode(query: &str) -> &'static str {
        if crate::file_search::parse_directory_path(query).is_some() {
            "path-browse"
        } else if crate::file_search::looks_like_advanced_mdquery(query) {
            "spotlight-advanced"
        } else {
            "spotlight-basic"
        }
    }

    /// Build surface-level metadata for the file search view, shared across
    /// all targets so the AI can reason about the query and visible result set.
    fn file_search_surface_metadata(
        &self,
        query: &str,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut metadata = serde_json::Map::new();

        let visible_results: Vec<serde_json::Value> = self
            .visible_file_search_results(TAB_AI_VISIBLE_TARGET_LIMIT)
            .into_iter()
            .map(|(display_index, entry)| {
                serde_json::json!({
                    "displayIndex": display_index,
                    "name": entry.name.clone(),
                    "path": entry.path.clone(),
                    "fileType": format!("{:?}", entry.file_type),
                })
            })
            .collect();

        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        metadata.insert(
            "queryMode".to_string(),
            serde_json::Value::String(Self::file_search_query_mode(query).to_string()),
        );
        metadata.insert(
            "visibleResultCount".to_string(),
            serde_json::json!(self.file_search_display_indices.len()),
        );
        metadata.insert(
            "visibleResults".to_string(),
            serde_json::Value::Array(visible_results),
        );

        if let Some(parsed) = crate::file_search::parse_directory_path(query) {
            metadata.insert(
                "directory".to_string(),
                serde_json::Value::String(parsed.directory),
            );
            metadata.insert(
                "directoryFilter".to_string(),
                match parsed.filter {
                    Some(filter) => serde_json::Value::String(filter),
                    None => serde_json::Value::Null,
                },
            );
        }

        metadata
    }

    /// Convert a file search result into a `TabAiTargetContext`, enriched
    /// with surface-level metadata about the query mode and visible results.
    fn tab_ai_target_from_file_search_result(
        display_index: usize,
        entry: &crate::file_search::FileResult,
        surface_metadata: &serde_json::Map<String, serde_json::Value>,
    ) -> crate::ai::TabAiTargetContext {
        let mut metadata = surface_metadata.clone();
        metadata.insert("displayIndex".to_string(), serde_json::json!(display_index));
        metadata.insert(
            "path".to_string(),
            serde_json::Value::String(entry.path.clone()),
        );
        metadata.insert(
            "fileType".to_string(),
            serde_json::Value::String(format!("{:?}", entry.file_type)),
        );
        metadata.insert("size".to_string(), serde_json::json!(entry.size));
        metadata.insert("modified".to_string(), serde_json::json!(entry.modified));

        crate::ai::TabAiTargetContext {
            source: "FileSearch".to_string(),
            kind: if entry.file_type == crate::file_search::FileType::Directory {
                "directory".to_string()
            } else {
                "file".to_string()
            },
            semantic_id: crate::protocol::generate_semantic_id(
                "choice",
                display_index,
                &entry.name,
            ),
            label: entry.name.clone(),
            metadata: Some(serde_json::Value::Object(metadata)),
        }
    }

    /// Convert a search query into a `TabAiTargetContext`.
    fn tab_ai_target_from_search_query(
        source: &str,
        query: &str,
        metadata: serde_json::Value,
    ) -> crate::ai::TabAiTargetContext {
        let trimmed = query.trim();
        let label = crate::ai::truncate_tab_ai_text(trimmed, 96);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_search_query_target_resolved",
            item_source = %source,
            label_chars = label.chars().count(),
        );
        crate::ai::TabAiTargetContext {
            source: source.to_string(),
            kind: "search_query".to_string(),
            semantic_id: crate::protocol::generate_semantic_id("query", 0, trimmed),
            label,
            metadata: Some(metadata),
        }
    }

    /// Convert raw input text into a lightweight `TabAiTargetContext`.
    fn tab_ai_target_from_input_text(
        source: &str,
        input_text: &str,
    ) -> crate::ai::TabAiTargetContext {
        let trimmed = input_text.trim();
        let label = crate::ai::truncate_tab_ai_text(trimmed, 96);
        let preview = crate::ai::truncate_tab_ai_text(trimmed, 400);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_input_target_resolved",
            item_source = %source,
            label_chars = label.chars().count(),
        );
        crate::ai::TabAiTargetContext {
            source: source.to_string(),
            kind: "input".to_string(),
            semantic_id: crate::protocol::generate_semantic_id("input", 0, trimmed),
            label,
            metadata: Some(serde_json::json!({
                "inputPreview": preview,
                "inputLength": trimmed.chars().count(),
            })),
        }
    }

    /// Source-view-aware target resolution: resolves targets against an explicit view
    /// instead of `self.current_view`. Used at submit time when `current_view`
    /// has already switched to the harness terminal.
    fn resolve_tab_ai_surface_targets_for_view(
        &self,
        view: &AppView,
        ui: &crate::ai::TabAiUiSnapshot,
    ) -> (
        Option<crate::ai::TabAiTargetContext>,
        Vec<crate::ai::TabAiTargetContext>,
    ) {
        match view {
            AppView::ClipboardHistoryView { selected_index, .. } => {
                let focused_target =
                    self.cached_clipboard_entries
                        .get(*selected_index)
                        .map(|entry| {
                            let preview = if entry.content_type.as_str() == "image" {
                                entry
                                    .ocr_text
                                    .clone()
                                    .filter(|text| !text.trim().is_empty())
                                    .unwrap_or_else(|| entry.display_preview())
                            } else {
                                entry.display_preview()
                            };
                            crate::ai::TabAiTargetContext {
                                source: "ClipboardHistory".to_string(),
                                kind: "clipboard_entry".to_string(),
                                semantic_id: crate::protocol::generate_semantic_id(
                                    "choice",
                                    *selected_index,
                                    &entry.text_preview,
                                ),
                                label: preview.clone(),
                                metadata: Some(serde_json::json!({
                                    "id": entry.id.clone(),
                                    "timestamp": entry.timestamp,
                                    "contentType": entry.content_type.as_str(),
                                    "preview": preview,
                                    "ocrText": entry.ocr_text.clone(),
                                    "imageWidth": entry.image_width,
                                    "imageHeight": entry.image_height,
                                })),
                            }
                        });
                let visible_targets = self
                    .cached_clipboard_entries
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| {
                        let preview = if entry.content_type.as_str() == "image" {
                            entry
                                .ocr_text
                                .clone()
                                .filter(|text| !text.trim().is_empty())
                                .unwrap_or_else(|| entry.display_preview())
                        } else {
                            entry.display_preview()
                        };
                        crate::ai::TabAiTargetContext {
                            source: "ClipboardHistory".to_string(),
                            kind: "clipboard_entry".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice",
                                index,
                                &entry.text_preview,
                            ),
                            label: preview.clone(),
                            metadata: Some(serde_json::json!({
                                "id": entry.id.clone(),
                                "timestamp": entry.timestamp,
                                "contentType": entry.content_type.as_str(),
                                "preview": preview,
                                "ocrText": entry.ocr_text.clone(),
                                "imageWidth": entry.image_width,
                                "imageHeight": entry.image_height,
                            })),
                        }
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::FileSearchView {
                query,
                selected_index,
                ..
            } => {
                let surface_metadata = self.file_search_surface_metadata(query);
                let focused_target = self
                    .selected_file_search_result(*selected_index)
                    .map(|(display_index, entry)| {
                        Self::tab_ai_target_from_file_search_result(
                            display_index,
                            entry,
                            &surface_metadata,
                        )
                    })
                    .or_else(|| {
                        let trimmed = query.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(Self::tab_ai_target_from_search_query(
                                "FileSearch",
                                trimmed,
                                serde_json::Value::Object(surface_metadata.clone()),
                            ))
                        }
                    });
                let visible_targets = self
                    .visible_file_search_results(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .into_iter()
                    .map(|(display_index, entry)| {
                        Self::tab_ai_target_from_file_search_result(
                            display_index,
                            entry,
                            &surface_metadata,
                        )
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::WindowSwitcherView { selected_index, .. } => {
                let focused_target = self.cached_windows.get(*selected_index).map(|entry| {
                    let label = format!("{} — {}", entry.app, entry.title);
                    crate::ai::TabAiTargetContext {
                        source: "WindowSwitcher".to_string(),
                        kind: "window".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            *selected_index,
                            &label,
                        ),
                        label,
                        metadata: Some(serde_json::json!({
                            "app": entry.app.clone(),
                            "title": entry.title.clone(),
                        })),
                    }
                });
                let visible_targets = self
                    .cached_windows
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| {
                        let label = format!("{} — {}", entry.app, entry.title);
                        crate::ai::TabAiTargetContext {
                            source: "WindowSwitcher".to_string(),
                            kind: "window".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice", index, &label,
                            ),
                            label,
                            metadata: Some(serde_json::json!({
                                "app": entry.app.clone(),
                                "title": entry.title.clone(),
                            })),
                        }
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                let focused_target = self
                    .app_launcher_selected_visible_entry(filter, *selected_index)
                    .map(|(source_index, app)| {
                        Self::tab_ai_target_from_app_launcher_row(
                            *selected_index,
                            source_index,
                            app,
                        )
                    });
                let visible_targets = self
                    .app_launcher_visible_target_rows(filter, TAB_AI_VISIBLE_TARGET_LIMIT)
                    .into_iter()
                    .map(|(display_index, source_index, app)| {
                        Self::tab_ai_target_from_app_launcher_row(display_index, source_index, app)
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::ProcessManagerView {
                filter,
                selected_index,
            } => {
                let focused_target = self
                    .process_manager_selected_visible_entry(filter, *selected_index)
                    .map(|(source_index, process)| {
                        Self::tab_ai_target_from_process_manager_row(
                            *selected_index,
                            source_index,
                            process,
                        )
                    });
                let visible_targets = self
                    .process_manager_visible_target_rows(filter, TAB_AI_VISIBLE_TARGET_LIMIT)
                    .into_iter()
                    .map(|(display_index, source_index, process)| {
                        Self::tab_ai_target_from_process_manager_row(
                            display_index,
                            source_index,
                            process,
                        )
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::CurrentAppCommandsView { selected_index, .. } => {
                let focused_target =
                    self.cached_current_app_entries
                        .get(*selected_index)
                        .map(|entry| crate::ai::TabAiTargetContext {
                            source: "CurrentAppCommands".to_string(),
                            kind: "menu_command".to_string(),
                            semantic_id: crate::protocol::generate_semantic_id(
                                "choice",
                                *selected_index,
                                &entry.name,
                            ),
                            label: entry.name.clone(),
                            metadata: Some(serde_json::json!({
                                "name": entry.name.clone(),
                            })),
                        });
                let visible_targets = self
                    .cached_current_app_entries
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .enumerate()
                    .map(|(index, entry)| crate::ai::TabAiTargetContext {
                        source: "CurrentAppCommands".to_string(),
                        kind: "menu_command".to_string(),
                        semantic_id: crate::protocol::generate_semantic_id(
                            "choice",
                            index,
                            &entry.name,
                        ),
                        label: entry.name.clone(),
                        metadata: Some(serde_json::json!({
                            "name": entry.name.clone(),
                        })),
                    })
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::SdkReferenceView {
                filter,
                selected_index,
                entries,
            } => {
                let focused_target = crate::mcp_resources::sdk_reference_selected_visible_entry(
                    entries,
                    filter,
                    *selected_index,
                )
                .map(Self::tab_ai_target_from_sdk_reference_row);
                let visible_targets = crate::mcp_resources::sdk_reference_visible_target_rows(
                    entries,
                    filter,
                    TAB_AI_VISIBLE_TARGET_LIMIT,
                )
                .into_iter()
                .map(Self::tab_ai_target_from_sdk_reference_row)
                .collect();
                (focused_target, visible_targets)
            }
            AppView::TipsView { .. } => (None, Vec::new()),
            AppView::ScriptTemplateCatalogView {
                filter,
                selected_index,
                templates,
            } => {
                let focused_target =
                    crate::mcp_resources::script_template_catalog_selected_visible_template(
                        templates,
                        filter,
                        *selected_index,
                    )
                    .map(Self::tab_ai_target_from_script_template_catalog_row);
                let visible_targets =
                    crate::mcp_resources::script_template_catalog_visible_target_rows(
                        templates,
                        filter,
                        TAB_AI_VISIBLE_TARGET_LIMIT,
                    )
                    .into_iter()
                    .map(Self::tab_ai_target_from_script_template_catalog_row)
                    .collect();
                (focused_target, visible_targets)
            }
            AppView::ScriptList => {
                let focused_target = self.resolved_main_menu_selected_subject().and_then(|subject| {
                    let row = match subject { crate::ResolvedMainMenuSelection::SearchResult { row, .. } | crate::ResolvedMainMenuSelection::Calculator { row, .. } => row };
                    self.tab_ai_target_from_main_menu_row(row)
                })
                    .or_else(|| {
                        let trimmed = self.filter_text.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(Self::tab_ai_target_from_search_query(
                                "ScriptList",
                                trimmed,
                                serde_json::json!({
                                    "query": trimmed,
                                    "visibleResultCount": self.main_menu_result_caches.grouped_flat_result_count(),
                                }),
                            ))
                        }
                    });

                let visible_targets = self
                    .main_menu_committed_rows()
                    .iter()
                    .filter(|row| row.eligibility.selectable)
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .filter_map(|row| self.tab_ai_target_from_main_menu_row(row))
                    .collect();

                (focused_target, visible_targets)
            }
            _ => {
                let visible_targets: Vec<crate::ai::TabAiTargetContext> = ui
                    .visible_elements
                    .iter()
                    .take(TAB_AI_VISIBLE_TARGET_LIMIT)
                    .map(|element| Self::tab_ai_target_from_element(&ui.prompt_type, element))
                    .collect();

                let focused_target = ui
                    .selected_semantic_id
                    .as_deref()
                    .or(ui.focused_semantic_id.as_deref())
                    .and_then(|semantic_id| {
                        visible_targets
                            .iter()
                            .find(|target| target.semantic_id == semantic_id)
                            .cloned()
                            .or_else(|| {
                                ui.visible_elements
                                    .iter()
                                    .find(|element| element.semantic_id == semantic_id)
                                    .map(|element| {
                                        Self::tab_ai_target_from_element(&ui.prompt_type, element)
                                    })
                            })
                    })
                    .or_else(|| {
                        ui.input_text
                            .as_deref()
                            .map(str::trim)
                            .filter(|text| !text.is_empty())
                            .map(|text| Self::tab_ai_target_from_input_text(&ui.prompt_type, text))
                    });

                (focused_target, visible_targets)
            }
        }
    }

    /// Format visible file search results for AI context injection.
    fn format_file_search_ai_visible_results(
        &self,
        selected_display_index: Option<usize>,
        limit: usize,
    ) -> String {
        self.visible_file_search_results(limit)
            .into_iter()
            .map(|(display_index, entry)| {
                let marker = if Some(display_index) == selected_display_index {
                    "*"
                } else {
                    "-"
                };
                let kind = if entry.file_type == crate::file_search::FileType::Directory {
                    "directory"
                } else {
                    "file"
                };
                format!("{marker} [{}] {} ({kind})", display_index, entry.path)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build a query-level AI intent when no valid row is selected.
    /// Falls back to the current query and visible result set.
    pub(crate) fn build_file_search_ai_query_intent(
        &self,
        query: &str,
        plan_mode: bool,
    ) -> Option<String> {
        let query = query.trim();
        if query.is_empty() && self.file_search_display_indices.is_empty() {
            return None;
        }

        let nearby = self.format_file_search_ai_visible_results(None, 6);
        let nearby = if nearby.is_empty() {
            "- (no visible results yet)".to_string()
        } else {
            nearby
        };

        let presentation = match &self.current_view {
            AppView::FileSearchView {
                presentation: FileSearchPresentation::Mini,
                ..
            } => "mini",
            AppView::FileSearchView {
                presentation: FileSearchPresentation::Full,
                ..
            } => "full",
            _ => "unknown",
        };

        let query_mode = if query.is_empty() {
            "empty"
        } else {
            Self::file_search_query_mode(query)
        };

        Some(if plan_mode {
            format!(
                "I am browsing files in Script Kit.\n\
                 File-search presentation: {presentation}\n\
                 Current file-search query: {query}\n\
                 Query mode: {query_mode}\n\
                 Visible results:\n\
                 {nearby}\n\n\
                 Use the current search as the primary context.\n\
                 Propose a concrete next-step plan, including which files or directories to inspect next,\n\
                 how to refine the query, and how to verify the result."
            )
        } else {
            format!(
                "I am browsing files in Script Kit.\n\
                 File-search presentation: {presentation}\n\
                 Current file-search query: {query}\n\
                 Query mode: {query_mode}\n\
                 Visible results:\n\
                 {nearby}\n\n\
                 Use the current search as the primary context.\n\
                 Summarize what this search is likely showing, point out the most important pattern,\n\
                 and tell me the highest-leverage next search or file to inspect."
            )
        })
    }

    /// Build an entry intent string for launching the AI harness from file
    /// search.  Returns `None` when no file is selected.
    ///
    /// `plan_mode`:
    /// - `false` (⌘↵) — "explain this file/directory"
    /// - `true`  (⌘⇧↵) — "propose a plan using this selection + query"
    pub(crate) fn build_file_search_ai_entry_intent(
        &self,
        query: &str,
        selected_index: usize,
        plan_mode: bool,
    ) -> Option<String> {
        let (display_index, selected) = self.selected_file_search_result(selected_index)?;

        let subject = if selected.file_type == crate::file_search::FileType::Directory {
            "directory"
        } else {
            "file"
        };

        let nearby = self.format_file_search_ai_visible_results(Some(display_index), 6);

        let presentation = match &self.current_view {
            AppView::FileSearchView {
                presentation: FileSearchPresentation::Mini,
                ..
            } => "mini",
            AppView::FileSearchView {
                presentation: FileSearchPresentation::Full,
                ..
            } => "full",
            _ => "unknown",
        };

        Some(if plan_mode {
            format!(
                "I am browsing files in Script Kit.\n\
                 File-search presentation: {presentation}\n\
                 Current file-search query: {query}\n\
                 Selected {subject}: {}\n\
                 Nearby visible results:\n\
                 {nearby}\n\n\
                 Use the selected item as the primary target. Use nearby results only as supporting context.\n\
                 Propose a concrete plan, related files to inspect, and verification steps.",
                selected.path
            )
        } else {
            format!(
                "I selected this {subject} in Script Kit file search.\n\
                 File-search presentation: {presentation}\n\
                 Current file-search query: {query}\n\
                 Selected {subject}: {}\n\
                 Nearby visible results:\n\
                 {nearby}\n\n\
                 Explain what it is, summarize why it matters, and tell me the highest-leverage next thing to inspect or change.",
                selected.path
            )
        })
    }

    /// Open the AI harness with the currently selected file-search result
    /// routed through the quick-submit plan path (richer harness hints).
    ///
    /// Returns `false` when there is no valid selection or intent, so the
    /// caller can fall through to default key handling.
    pub(crate) fn open_file_search_selection_in_tab_ai(
        &mut self,
        query: &str,
        selected_index: usize,
        plan_mode: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((_display_index, selected)) = self.selected_file_search_result(selected_index)
        else {
            return false;
        };

        let Some(intent) = self.build_file_search_ai_entry_intent(query, selected_index, plan_mode)
        else {
            return false;
        };

        let plan = crate::ai::TabAiQuickSubmitPlan {
            source: crate::ai::TabAiQuickSubmitSource::FileSearch,
            kind: crate::ai::TabAiQuickSubmitKind::FileDrop,
            raw_query: selected.path.clone(),
            normalized_query: selected.path.clone(),
            synthesized_intent: intent,
            capture_kind: "defaultContext".to_string(),
            submit: true,
        };

        self.open_tab_ai_chat_with_quick_submit_plan(plan, cx);
        true
    }

    /// Open the AI harness from file search, falling back to a query-level
    /// intent when no valid row is selected.
    ///
    /// Returns `false` only when no useful context exists at all (empty query
    /// and no visible results), so `⌘↵` / `⌘⇧↵` is never a dead keypress.
    pub(crate) fn open_file_search_selection_or_query_in_tab_ai(
        &mut self,
        query: &str,
        selected_index: usize,
        plan_mode: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.open_file_search_selection_in_tab_ai(query, selected_index, plan_mode, cx) {
            return true;
        }

        let Some(intent) = self.build_file_search_ai_query_intent(query, plan_mode) else {
            return false;
        };

        self.open_chat_about_current_view(Some(intent), cx);
        true
    }

    /// Fast-path UI snapshot for `AppView::ScriptList`.
    ///
    /// Skips the expensive `collect_visible_elements()` tree walk because the
    /// downstream target resolution in `resolve_tab_ai_surface_targets_for_view`
    /// already uses `cached_grouped_items` / `cached_grouped_flat_results`.
    fn snapshot_tab_ai_ui_fast_for_script_list(
        &self,
    ) -> (
        crate::ai::TabAiUiSnapshot,
        crate::ai::TabAiInvocationReceipt,
    ) {
        let prompt_type = "ScriptList".to_string();
        let input_text = if self.filter_text.is_empty() {
            None
        } else {
            Some(self.filter_text.clone())
        };

        let focused_semantic_id =
            self.resolved_main_menu_selected_subject()
                .map(|subject| match subject {
                    crate::ResolvedMainMenuSelection::SearchResult { row, .. }
                    | crate::ResolvedMainMenuSelection::Calculator { row, .. } => {
                        row.semantic_id.clone()
                    }
                });
        let selected_semantic_id = focused_semantic_id.clone();

        let snapshot = crate::ai::TabAiUiSnapshot {
            prompt_type: prompt_type.clone(),
            input_text: input_text.clone(),
            focused_semantic_id: focused_semantic_id.clone(),
            selected_semantic_id: selected_semantic_id.clone(),
            visible_elements: Vec::new(),
        };

        let receipt = crate::ai::TabAiInvocationReceipt::from_snapshot(
            &prompt_type,
            &input_text,
            &focused_semantic_id,
            &selected_semantic_id,
            0,
            &[],
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_snapshot_fast_script_list",
            has_input_text = snapshot.input_text.is_some(),
            has_focus_target = snapshot.focused_semantic_id.is_some(),
            selected_index = self.selected_index,
        );

        (snapshot, receipt)
    }

    /// Capture a snapshot of the current UI state for context assembly.
    ///
    /// Returns the snapshot and a machine-readable invocation receipt that
    /// identifies whether UI context was rich or degraded with explicit
    /// reason codes.
    #[allow(dead_code)]
    fn snapshot_tab_ai_ui(
        &self,
        cx: &Context<Self>,
    ) -> (
        crate::ai::TabAiUiSnapshot,
        crate::ai::TabAiInvocationReceipt,
    ) {
        let prompt_type = self.app_view_name();

        // Collect visible elements
        let outcome = self.collect_visible_elements(TAB_AI_VISIBLE_ELEMENT_LIMIT, cx);

        let input_text = self.current_input_text(cx);
        let focused_id = outcome.focused_semantic_id();
        let selected_id = outcome.selected_semantic_id();

        // Build the machine-readable invocation receipt
        let receipt = crate::ai::TabAiInvocationReceipt::from_snapshot(
            &prompt_type,
            &input_text,
            &focused_id,
            &selected_id,
            outcome.elements.len(),
            &outcome.warnings,
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "tab_ai_snapshot_captured",
            prompt_type = %prompt_type,
            input_status = %receipt.input_status,
            focus_status = %receipt.focus_status,
            elements_status = %receipt.elements_status,
            has_input_text = receipt.has_input_text,
            has_focus_target = receipt.has_focus_target,
            element_count = receipt.element_count,
            warning_count = receipt.warning_count,
            rich = receipt.rich,
            degradation_reasons = ?receipt.degradation_reasons,
            "tab ai snapshot captured"
        );

        let snapshot = crate::ai::TabAiUiSnapshot {
            prompt_type,
            input_text,
            focused_semantic_id: focused_id,
            selected_semantic_id: selected_id,
            visible_elements: outcome.elements,
        };

        (snapshot, receipt)
    }

    /// Return a human-readable name for the current `AppView` variant.
    pub(crate) fn app_view_name(&self) -> String {
        match &self.current_view {
            AppView::ScriptList => "ScriptList".to_string(),
            AppView::About { .. } => "About".to_string(),
            AppView::ArgPrompt { .. } => "ArgPrompt".to_string(),
            AppView::MiniPrompt { .. } => "MiniPrompt".to_string(),
            AppView::MicroPrompt { .. } => "MicroPrompt".to_string(),
            AppView::DivPrompt { .. } => "DivPrompt".to_string(),
            AppView::FormPrompt { .. } => "FormPrompt".to_string(),
            AppView::TermPrompt { .. } => "TermPrompt".to_string(),
            AppView::EditorPrompt { .. } => "EditorPrompt".to_string(),
            AppView::SelectPrompt { .. } => "SelectPrompt".to_string(),
            AppView::PathPrompt { .. } => "PathPrompt".to_string(),
            AppView::EnvPrompt { .. } => "EnvPrompt".to_string(),
            AppView::DropPrompt { .. } => "DropPrompt".to_string(),
            AppView::TemplatePrompt { .. } => "TemplatePrompt".to_string(),
            AppView::ChatPrompt { .. } => "ChatPrompt".to_string(),
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory".to_string(),
            AppView::AppLauncherView { .. } => "AppLauncher".to_string(),
            AppView::WindowSwitcherView { .. } => "WindowSwitcher".to_string(),
            AppView::BrowserTabsView { .. } => "BrowserTabs".to_string(),
            AppView::FileSearchView { .. } => "FileSearch".to_string(),
            AppView::ProfileSearchView { .. } => "ProfileSearch".to_string(),
            AppView::ThemeChooserView { .. } => "ThemeChooser".to_string(),
            AppView::EmojiPickerView { .. } => "EmojiPicker".to_string(),
            AppView::WebcamView { .. } => "Webcam".to_string(),
            AppView::ScratchPadView { .. } => "ScratchPad".to_string(),
            AppView::QuickTerminalView { .. } => "QuickTerminal".to_string(),
            AppView::FlowSessionView { .. } => "FlowSessionView".to_string(),
            AppView::NamingPrompt { .. } => "NamingPrompt".to_string(),
            AppView::CreationFeedback { .. } => "CreationFeedback".to_string(),
            AppView::ActionsDialog => "ActionsDialog".to_string(),
            AppView::BrowseKitsView { .. } => "BrowseKits".to_string(),
            AppView::MigrateV1View { .. } => "MigrateV1".to_string(),
            AppView::InstalledKitsView { .. } => "InstalledKits".to_string(),
            AppView::ProcessManagerView { .. } => "ProcessManager".to_string(),
            AppView::FlowUxView { .. } => "FlowUx".to_string(),
            AppView::SearchAiPresetsView { .. } => "SearchAiPresets".to_string(),
            AppView::CreateAiPresetView { .. } => "CreateAiPreset".to_string(),
            AppView::SettingsView { .. } => "Settings".to_string(),
            AppView::PermissionsWizardView { .. } => "PermissionsWizard".to_string(),
            AppView::FavoritesBrowseView { .. } => "FavoritesBrowse".to_string(),
            AppView::CurrentAppCommandsView { .. } => "CurrentAppCommands".to_string(),
            AppView::BrowserHistoryView { .. } => "BrowserHistoryView".to_string(),
            AppView::NotesBrowseView { .. } => "NotesBrowse".to_string(),
            AppView::DayPage { .. } => "DayPage".to_string(),
            AppView::ScriptIssuesView { .. } => "ScriptIssuesView".to_string(),
            AppView::SdkReferenceView { .. } => "SdkReferenceView".to_string(),
            AppView::TipsView { .. } => "TipsView".to_string(),
            AppView::ScriptTemplateCatalogView { .. } => "ScriptTemplateCatalogView".to_string(),
        }
    }

    /// Return the current input text from whichever view is active.
    ///
    /// Returns `Some(text)` when the view has user-editable text that is
    /// non-empty, `None` otherwise.  Entity-based prompts are read via
    /// `entity.read(cx)` so this method requires a context reference.
    #[allow(dead_code)]
    fn current_input_text(&self, cx: &Context<Self>) -> Option<String> {
        let non_empty = |s: String| if s.is_empty() { None } else { Some(s) };

        match &self.current_view {
            AppView::ScriptList => non_empty(self.filter_text.clone()),

            // Flow sessions compose in the shared MAIN input — the draft
            // message lives in filter_text.
            AppView::FlowSessionView { .. } => non_empty(self.filter_text.clone()),

            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. } => non_empty(self.arg_input.text().to_string()),

            AppView::ClipboardHistoryView { filter, .. }
            | AppView::AppLauncherView { filter, .. }
            | AppView::WindowSwitcherView { filter, .. }
            | AppView::BrowserTabsView { filter, .. }
            | AppView::ThemeChooserView { filter, .. }
            | AppView::ProfileSearchView { filter, .. }
            | AppView::EmojiPickerView { filter, .. }
            | AppView::ProcessManagerView { filter, .. }
            | AppView::FlowUxView { filter, .. }
            | AppView::SettingsView { filter, .. }
            | AppView::SearchAiPresetsView { filter, .. }
            | AppView::FavoritesBrowseView { filter, .. }
            | AppView::CurrentAppCommandsView { filter, .. }
            | AppView::BrowserHistoryView { filter, .. }
            | AppView::SdkReferenceView { filter, .. }
            | AppView::TipsView { filter, .. }
            | AppView::ScriptTemplateCatalogView { filter, .. }
            | AppView::MigrateV1View { filter, .. } => non_empty(filter.clone()),

            AppView::NotesBrowseView { search } => non_empty(search.query.clone()),

            AppView::FileSearchView { query, .. } => non_empty(query.clone()),

            AppView::BrowseKitsView { query, .. } => non_empty(query.clone()),

            // --- Entity-based prompts ---
            AppView::EditorPrompt { entity, .. } => {
                entity.read_with(cx, |editor, app| non_empty(editor.content_from_app(app)))
            }
            AppView::ScratchPadView { entity, .. } => {
                entity.read_with(cx, |editor, app| non_empty(editor.content_from_app(app)))
            }
            AppView::DayPage { entity } => {
                non_empty(entity.read(cx).notes_editor.read(cx).content(cx))
            }
            AppView::ChatPrompt { entity, .. } => {
                non_empty(entity.read(cx).input.text().to_string())
            }
            AppView::PathPrompt { entity, .. } => {
                let p = entity.read(cx);
                // Prefer active filter text; fall back to current directory path
                non_empty(p.filter_text.clone()).or_else(|| non_empty(p.current_path.clone()))
            }
            AppView::EnvPrompt { entity, .. } => {
                let p = entity.read(cx);
                // Return the user-entered value (masked text is still useful
                // for "is something typed?" without revealing secrets)
                if p.secret {
                    // For secret fields, report presence but not content
                    let text = p.input_text();
                    if text.is_empty() {
                        None
                    } else {
                        Some("[secret]".to_string())
                    }
                } else {
                    non_empty(p.input_text().to_string())
                }
            }
            AppView::SelectPrompt { entity, .. } => non_empty(entity.read(cx).filter_text.clone()),
            AppView::NamingPrompt { entity, .. } => {
                non_empty(entity.read(cx).friendly_name.clone())
            }
            AppView::TemplatePrompt { entity, .. } => {
                let p = entity.read(cx);
                // Return the value of the currently focused template input
                p.values
                    .get(p.current_input)
                    .and_then(|v| non_empty(v.clone()))
            }
            AppView::CreateAiPresetView {
                name,
                system_prompt,
                model,
                active_field,
            } => {
                // Return whichever field is active
                match active_field {
                    0 => non_empty(name.clone()),
                    1 => non_empty(system_prompt.clone()),
                    2 => non_empty(model.clone()),
                    _ => non_empty(name.clone()),
                }
            }

            // --- Views with no meaningful user-editable text ---
            // DivPrompt: script-rendered HTML, no user input
            // FormPrompt: multi-field form — field values are in elements,
            //   not a single "input text" (use visible_elements instead)
            // TermPrompt/QuickTerminal: terminal content, not user text input
            // DropPrompt: file drop zone, no typed text
            // WebcamView: camera feed, no text
            // CreationFeedback: read-only confirmation
            // ActionsDialog: transient overlay, not a primary surface
            // InstalledKitsView: navigation-only, no free text
            AppView::DivPrompt { .. }
            | AppView::About { .. }
            | AppView::FormPrompt { .. }
            | AppView::TermPrompt { .. }
            | AppView::QuickTerminalView { .. }
            | AppView::DropPrompt { .. }
            | AppView::WebcamView { .. }
            | AppView::CreationFeedback { .. }
            | AppView::ScriptIssuesView { .. }
            | AppView::ActionsDialog
            | AppView::InstalledKitsView { .. }
            | AppView::PermissionsWizardView { .. } => None,
        }
    }

    /// Complete the pending Tab AI execution after the script actually exits.
    ///
    /// Gates memory write-back, keeping the script, and temp-file cleanup on real
    /// completion status — never at dispatch time.
    ///
    /// Called from the prompt-handler `ScriptExit` / `ScriptError` paths
    /// once the ephemeral process terminates. Uses `take()` on the pending
    /// record so only the first caller does work — subsequent calls are no-ops.
    pub(crate) fn complete_tab_ai_execution(
        &mut self,
        success: bool,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(record) = self.pending_tab_ai_execution.take() else {
            return;
        };

        let cleanup_attempted = true;
        let cleanup_succeeded = crate::ai::cleanup_tab_ai_temp_script(&record.temp_script_path);

        let status = if success {
            crate::ai::TabAiExecutionStatus::Succeeded
        } else {
            crate::ai::TabAiExecutionStatus::Failed
        };

        let receipt = crate::ai::build_tab_ai_execution_receipt(
            &record,
            status,
            cleanup_attempted,
            cleanup_succeeded,
            error.clone(),
        );

        if let Err(audit_error) = crate::ai::append_tab_ai_execution_receipt(&receipt) {
            tracing::warn!(
                event = "tab_ai_execution_audit_write_failed",
                error = %audit_error,
            );
        }

        if success {
            if let Err(memory_error) = crate::ai::write_tab_ai_memory_entry(&record) {
                tracing::warn!(
                    event = "tab_ai_memory_writeback_failed",
                    error = %memory_error,
                );
            }

            if crate::ai::should_offer_save(&record) {
                self.save_tab_ai_script(record, cx);
            }
        } else {
            let message = error.unwrap_or_else(|| "Tab AI script failed".to_string());
            self.one_window_notice(crate::line_notice::FAILED, message, cx);
            cx.notify();
        }
    }
    // ── Tab AI script, kept automatically ──────────────────────────────

    fn tab_ai_default_save_name(record: &crate::ai::TabAiExecutionRecord) -> String {
        let derived = super::prompt_ai::derive_script_name_from_description(&record.intent);
        if derived == "ai-generated-script" || derived.is_empty() {
            record.slug.clone()
        } else {
            derived
        }
    }

    /// Keeps the script a successful Tab AI run produced: it is saved at once and the line
    /// above the input says "Saved as <file>"; ⌘Z deletes exactly that file.
    pub(crate) fn save_tab_ai_script(
        &mut self,
        record: crate::ai::TabAiExecutionRecord,
        cx: &mut Context<Self>,
    ) {
        let filename_stem = Self::tab_ai_default_save_name(&record);
        // Route through the contract pipeline (extraction, convention
        // enforcement, concurrent-prompt-API rejection, receipt, background
        // Bun verification) instead of a bare create + write — a generated
        // script that the tested generator would reject must not land on
        // disk unflagged.
        let prompt = if record.intent.trim().is_empty() {
            filename_stem.as_str()
        } else {
            record.intent.as_str()
        };
        let created_path =
            match crate::ai::script_generation::save_generated_script_from_response_with_slug(
                prompt,
                &record.generated_source,
                Some(&filename_stem),
            ) {
                Ok(path) => path,
                Err(error) => {
                    tracing::warn!(
                        event = "tab_ai_save_create_failed",
                        error_sha256 = %crate::logging::log_private_user_value(&error.to_string()),
                        filename_stem_sha256 = %crate::logging::log_private_user_value(
                            &filename_stem
                        ),
                    );
                    self.one_window_notice(
                        crate::line_notice::FAILED,
                        format!("Couldn't save {filename_stem}.ts: {error}"),
                        cx,
                    );
                    return;
                }
            };
        let written = if created_path.is_absolute() {
            created_path
        } else {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(&created_path),
                Err(_) => created_path,
            }
        };
        tracing::info!(
            event = "tab_ai_script_saved",
            filename_stem_sha256 = %crate::logging::log_private_user_value(&filename_stem),
            filename_stem_bytes = filename_stem.len(),
            path_sha256 = %crate::logging::log_private_user_value(&written.to_string_lossy()),
        );
        let file_name = written
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("{filename_stem}.ts"));
        self.one_window_done_undoable(
            "script",
            format!("Saved as {file_name}"),
            Box::new(move |app, cx| app.remove_saved_tab_ai_script(&written, cx)),
            cx,
        );
        cx.notify();
    }

    /// ⌘Z after a Tab AI script was kept: the file it wrote goes away.
    fn remove_saved_tab_ai_script(&mut self, written: &std::path::Path, cx: &mut Context<Self>) {
        match std::fs::remove_file(written) {
            Ok(()) => tracing::info!(
                event = "tab_ai_script_save_undone",
                path_sha256 = %crate::logging::log_private_user_value(&written.to_string_lossy()),
            ),
            Err(error) => {
                let name = written
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.one_window_notice(
                    crate::line_notice::FAILED,
                    format!("Couldn't remove {name}: {error}"),
                    cx,
                );
            }
        }
    }
}

include!("apply_back.rs");

#[cfg(test)]
include!("mod_tests.rs");
