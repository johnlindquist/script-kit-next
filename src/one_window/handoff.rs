//! A subject action freezes the exact selected text. The recipient chooser, cancellable send
//! and explicit editable review use existing one-window controls; a reply cannot take focus.
use super::ledger_view::LedgerRun;
use crate::{mcp_protocol::bridge::handoff as mail, AppView, ScriptListApp};
use gpui::{AppContext as _, Context};
use sha2::{Digest, Sha256};
use uuid::Uuid;
pub(super) struct OpenDraft {
    id: Uuid,
    editor: gpui::EntityId,
    base: [u8; 32],
}
impl ScriptListApp {
    pub(crate) fn listen_for_handoff_replies(&mut self, cx: &mut Context<Self>) {
        if std::mem::replace(&mut self.one_window_host.handoff_listening, true) {
            return;
        }
        let replies = mail::events();
        cx.spawn(async move |this, cx| {
            while let Ok(name) = replies.recv().await {
                if this
                    .update(cx, |app, cx| {
                        app.one_window_notice(
                            crate::line_notice::INFO,
                            format!("{name} replied · Open Agent Handoffs"),
                            cx,
                        )
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }
    pub(super) fn choose_handoff_recipient(&mut self, body: String, cx: &mut Context<Self>) {
        if body.trim().is_empty() || body.len() > 65536 {
            self.notice_failed_public("Select between one and 65,536 bytes of text", cx);
            return;
        }
        self.listen_for_handoff_replies(cx);
        let peers = mail::recipients();
        if peers.is_empty() {
            self.one_window_notice(
                crate::line_notice::INFO,
                "No agent is available · run scriptkit agent available in its Herdr terminal",
                cx,
            );
            return;
        }
        let count = body.chars().count();
        let body = std::sync::Arc::new(body);
        let mut choices: Vec<(String, LedgerRun)> = Vec::new();
        for (target, name, terminal) in peers {
            let label = format!("{name} · {terminal}");
            let share = body.clone();
            let hold = format!("Share {count} characters with {label}");
            let send: LedgerRun =
                Box::new(move |app, cx| app.send_frozen_selection(target, name, share, cx));
            choices.push((
                label,
                Box::new(move |app, cx| app.one_window_hold("send", hold, send, cx)),
            ));
        }
        self.one_window_ask(
            "send",
            "Share the selected text with whom?",
            format!("{count} characters · only this selection"),
            choices,
            None,
            cx,
        );
    }
    fn send_frozen_selection(
        &mut self,
        target: Uuid,
        name: String,
        body: std::sync::Arc<String>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    mail::send(
                        "launcher",
                        None,
                        target,
                        Uuid::new_v4(),
                        body.as_str().to_owned(),
                    )
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                use crate::line_notice::{FAILED, INFO};
                let (icon, words) = match result {
                    Ok(v) => {
                        let phase = v["id"]
                            .as_str()
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .and_then(mail::local_phase)
                            .or_else(|| v["state"].as_str());
                        match phase {
                            Some("taken") => (INFO, format!("{name} received the selection")),
                            Some("replied") => {
                                (INFO, format!("{name} replied · Open Agent Handoffs"))
                            }
                            Some("cancelled") => (
                                INFO,
                                "Handoff closed · already shared text cannot be recalled".into(),
                            ),
                            Some("notified") => {
                                (INFO, format!("Notified {name} · waiting for a reply"))
                            }
                            Some("notification_unknown") => (
                                INFO,
                                "Delivery unknown · check Agent Handoffs before trying again"
                                    .into(),
                            ),
                            _ => (
                                FAILED,
                                format!(
                                    "Selection not shared · {}",
                                    v["reason"]
                                        .as_str()
                                        .unwrap_or("unavailable")
                                        .replace('_', " ")
                                ),
                            ),
                        }
                    }
                    Err(e) => (
                        FAILED,
                        format!("Selection not shared · {}", e.replace('_', " ")),
                    ),
                };
                app.one_window_notice(icon, words, cx);
            });
        })
        .detach();
    }
    pub(crate) fn review_agent_handoffs(&mut self, cx: &mut Context<Self>) {
        let mut choices: Vec<(String, LedgerRun)> = Vec::new();
        for (id, name, ready) in mail::inbox() {
            let short = &id.to_string()[..8];
            if ready {
                choices.push((
                    format!("Edit reply from {name} · {short}"),
                    Box::new(move |app, cx| app.open_agent_draft(id, cx)),
                ));
            } else {
                choices.push((
                    format!("Stop sharing with {name} · {short}"),
                    Box::new(move |app, cx| {
                        let stopped = mail::cancel_local(id);
                        app.one_window_notice(
                            crate::line_notice::INFO,
                            if stopped {
                                "Closed the handoff · already shared text cannot be recalled"
                            } else {
                                "The handoff changed · check Agent Handoffs"
                            },
                            cx,
                        );
                    }),
                ));
            }
        }
        if choices.is_empty() {
            self.one_window_notice(
                crate::line_notice::INFO,
                "No pending handoffs or replies",
                cx,
            );
        } else {
            self.one_window_ask(
                "inbox",
                "Agent Handoffs",
                "Replies stay editable for one hour in this session",
                choices,
                None,
                cx,
            );
        }
    }
    fn open_agent_draft(&mut self, id: Uuid, cx: &mut Context<Self>) {
        use crate::design_evaluation::prompt_fixtures::{
            EditorPromptSeed, PromptSeed, PromptSeedCommon,
        };
        let Some(words) = mail::draft(id) else {
            self.notice_failed_public("That reply expired", cx);
            return;
        };
        if !self.prepare_shortcut_launcher(cx) {
            self.notice_failed_public("Finish the open prompt before opening a reply", cx);
            return;
        }
        let seed = PromptSeed::Editor(EditorPromptSeed {
            common: PromptSeedCommon::local("agent-handoff-draft"),
            content: words.clone(),
            language: "markdown".into(),
            template: None,
        });
        if self.construct_prompt_seed(seed, cx).is_err() {
            self.notice_failed_public("Couldn’t open the reply", cx);
            return;
        }
        if let AppView::EditorPrompt { entity, .. } = &self.current_view {
            let editor = entity.clone();
            let observed = editor.entity_id();
            self.one_window_host.handoff_draft = Some(OpenDraft {
                id,
                editor: observed,
                base: Sha256::digest(words.as_bytes()).into(),
            });
            let (tx, rx) = async_channel::bounded(1);
            editor.update(cx, |editor, _| editor.observe_local_finish(tx));
            cx.observe(&editor, move |app, _, cx| {
                if app.review_editor_id() == Some(observed) {
                    app.save_handoff_edit(false, cx);
                }
            })
            .detach();
            cx.spawn(async move |this, cx| {
                while rx.recv().await.is_ok() {
                    if this
                        .update(cx, |app, cx| {
                            if app.review_editor_id() != Some(observed) {
                                return true;
                            }
                            app.leave_handoff_editor(false, cx);
                            app.review_editor_id() != Some(observed)
                        })
                        .unwrap_or(true)
                    {
                        break;
                    }
                }
            })
            .detach();
            self.one_window_notice(
                crate::line_notice::INFO,
                "Editing an agent reply · Esc returns to handoffs",
                cx,
            );
        }
    }
    fn review_editor_id(&self) -> Option<gpui::EntityId> {
        let AppView::EditorPrompt { entity, .. } = &self.current_view else {
            return None;
        };
        self.one_window_host
            .handoff_draft
            .as_ref()
            .filter(|d| d.editor == entity.entity_id())
            .map(|d| d.editor)
    }
    fn save_handoff_edit(&mut self, closing: bool, cx: &mut Context<Self>) -> bool {
        let AppView::EditorPrompt { entity, .. } = &self.current_view else {
            return false;
        };
        let Some(d) = self
            .one_window_host
            .handoff_draft
            .as_mut()
            .filter(|d| d.editor == entity.entity_id())
        else {
            return false;
        };
        let words = entity.read(cx).content_from_app(cx);
        let next: [u8; 32] = Sha256::digest(words.as_bytes()).into();
        if !closing && next == d.base {
            return true;
        }
        if !mail::edit(d.id, &d.base, &words) {
            return false;
        }
        d.base = next;
        true
    }
    /// Esc, submit and the editor footer return to the inbox; none send edits to the agent.
    pub(crate) fn leave_handoff_editor(&mut self, escape: bool, cx: &mut Context<Self>) -> bool {
        if self.review_editor_id().is_none() {
            return false;
        }
        if !self.save_handoff_edit(true, cx) {
            let armed = escape
                && self
                    .editor_escape_armed_at
                    .is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(2));
            if !armed {
                self.editor_escape_armed_at = escape.then(std::time::Instant::now);
                self.one_window_notice(
                    crate::line_notice::FAILED,
                    "Edits not kept · copy them first, or press Esc twice to discard",
                    cx,
                );
                return true;
            }
        }
        self.editor_escape_armed_at = None;
        self.one_window_host.handoff_draft = None;
        self.reset_to_script_list(cx);
        self.review_agent_handoffs(cx);
        true
    }
}
