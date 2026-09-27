//! The chats from before, read from Agent Chat's history for the reducer's past-chats rows, and
//! what their rows do: open one again in the chat, copy it, export it, delete it.

use gpui::{ClipboardItem, Context};
use sk_one_window::chat::{Turn, Who};
use sk_one_window::past_chats::{PastChat, COPY, DELETE, EXPORT};
use sk_one_window::work::WorkEvent;
use sk_one_window::{Done, Event};

use crate::ai::agent_chat::ui::history::{self, AgentChatHistoryEntry, SavedConversation};
use crate::ai::agent_chat::ui::history_attachment::{
    format_history_attachment_markdown, write_history_attachment, AgentChatHistoryAttachMode,
};
use crate::ScriptListApp;

/// "Today", "Yesterday", or the day it was.
fn when(timestamp: &str, today: chrono::NaiveDate) -> String {
    let Ok(at) = chrono::DateTime::parse_from_rfc3339(timestamp) else {
        return String::new();
    };
    let day = at.with_timezone(&chrono::Local).date_naive();
    match (today - day).num_days() {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        2..=6 => day.format("%A").to_string(),
        _ => day.format("%b %-d").to_string(),
    }
}

fn past_chat(entry: &AgentChatHistoryEntry, today: chrono::NaiveDate) -> PastChat {
    PastChat {
        id: entry.session_id.clone(),
        title: entry.title_display().to_string(),
        when: when(&entry.timestamp, today),
        preview: entry.preview_display().to_string(),
        search: entry.search_text.clone(),
    }
}

/// A saved chat's turns as the chat shows them.
fn turns_of(conversation: &SavedConversation) -> Vec<Turn> {
    conversation
        .messages
        .iter()
        .filter_map(|m| {
            let who = match m.role.as_str() {
                "User" => Who::You,
                "Assistant" => Who::Model,
                "Tool" => Who::Tool,
                _ => return None,
            };
            Some(Turn::new(who, m.body.clone()))
        })
        .collect()
}

impl ScriptListApp {
    /// Opens the past chats under the main input, filtered by `query`.
    pub(crate) fn one_window_open_past_chats(&mut self, query: String, cx: &mut Context<Self>) {
        self.one_window_leave_for_chat(cx);
        self.one_window_handle(Event::Work(WorkEvent::OpenPastChats { query }), cx);
    }

    /// A chat the launcher found in the history opens again in the chat; an index entry with no
    /// saved conversation opens a chat with its first words waiting in the input.
    pub(crate) fn resume_agent_chat_conversation_from_history(
        &mut self,
        session_id: &str,
        first_words: &str,
        cx: &mut Context<Self>,
    ) {
        if !history::conversation_exists(session_id) {
            self.one_window_open_chat(
                crate::one_window::chat_entry::ChatEntry::seeded(first_words),
                cx,
            );
            return;
        }
        self.one_window_leave_for_chat(cx);
        self.one_window_reopen_chat(session_id.to_string(), cx);
    }

    pub(crate) fn one_window_list_past_chats(&mut self, cx: &mut Context<Self>) {
        let today = chrono::Local::now().date_naive();
        let chats = history::load_history()
            .iter()
            .map(|entry| past_chat(entry, today))
            .collect();
        self.one_window_work_later(WorkEvent::PastChats(chats), cx);
    }

    /// Opens a chat from before in the chat: its turns show, and the model gets its transcript
    /// so the next turn carries on from it.
    pub(crate) fn one_window_reopen_chat(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(conversation) = history::load_conversation(&id) else {
            tracing::warn!(target: "script_kit::one_window", event = "past_chat_missing");
            self.one_window_handle(Event::Work(WorkEvent::PastChatGone { id }), cx);
            return;
        };
        let transcript = write_history_attachment(&id, AgentChatHistoryAttachMode::Transcript).map(
            |(path, label)| crate::ai::message_parts::AiContextPart::FilePath {
                path: path.to_string_lossy().into_owned(),
                label,
            },
        );
        match self.one_window_fresh_chat_thread(cx) {
            Ok(model) => {
                model.thread.update(cx, |thread, cx| {
                    thread.load_saved_messages(&conversation.messages, cx);
                    match transcript {
                        Ok(part) => thread.add_context_part(part, cx),
                        Err(error) => tracing::warn!(target: "script_kit::one_window", event = "past_chat_transcript_failed", error = %error),
                    }
                });
                self.one_window_host.work.chat = Some(model);
            }
            Err(error) => {
                tracing::warn!(target: "script_kit::one_window", event = "past_chat_thread_failed", %error);
            }
        }
        self.one_window_work_later(
            WorkEvent::ChatReopened {
                id,
                turns: turns_of(&conversation),
            },
            cx,
        );
    }

    /// Copy, export or delete a chat from before; the line says it happened.
    pub(crate) fn one_window_past_chat(
        &mut self,
        id: String,
        act: &'static str,
        cx: &mut Context<Self>,
    ) {
        let Some(conversation) = history::load_conversation(&id) else {
            self.one_window_handle(Event::Work(WorkEvent::PastChatGone { id }), cx);
            return;
        };
        let title = history::build_history_entry(&conversation)
            .map(|e| e.title_display().to_string())
            .unwrap_or_else(|| "the chat".to_string());
        let done = match act {
            COPY => {
                cx.write_to_clipboard(ClipboardItem::new_string(
                    format_history_attachment_markdown(
                        &conversation,
                        AgentChatHistoryAttachMode::Transcript,
                    ),
                ));
                Done {
                    icon: "copy".into(),
                    title: format!("Copied “{title}”"),
                    stay: true,
                    ..Done::default()
                }
            }
            EXPORT => {
                let markdown = format_history_attachment_markdown(
                    &conversation,
                    AgentChatHistoryAttachMode::Transcript,
                );
                let dir = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
                match crate::ai::agent_chat::ui::export::persist_private_agent_chat_export(
                    &dir, &id, &markdown,
                ) {
                    Ok(_) => Done {
                        icon: "export".into(),
                        title: format!("Exported “{title}” to Downloads"),
                        stay: true,
                        ..Done::default()
                    },
                    Err(error) => {
                        tracing::warn!(target: "script_kit::one_window", event = "past_chat_export_failed", kind = ?error.kind());
                        return;
                    }
                }
            }
            DELETE => {
                self.one_window_hold(
                    "trash",
                    format!("Delete “{title}”"),
                    Box::new(move |this, cx| {
                        if let Err(error) = history::delete_conversation(&id) {
                            tracing::warn!(target: "script_kit::one_window", event = "past_chat_delete_failed", error = %error);
                            this.one_window_notice(
                                crate::line_notice::FAILED,
                                "Couldn't delete that chat",
                                cx,
                            );
                            return;
                        }
                        this.one_window_handle(Event::Work(WorkEvent::PastChatGone { id }), cx);
                    }),
                    cx,
                );
                return;
            }
            _ => return,
        };
        self.one_window_handle(Event::Done(done), cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chat_says_when_it_was_in_words() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 9, 26).expect("date");
        let noon = |d: u32| {
            chrono::NaiveDate::from_ymd_opt(2026, 9, d)
                .expect("date")
                .and_hms_opt(12, 0, 0)
                .expect("time")
                .and_local_timezone(chrono::Local)
                .single()
                .expect("local")
                .to_rfc3339()
        };
        assert_eq!(when(&noon(26), today), "Today");
        assert_eq!(when(&noon(25), today), "Yesterday");
        assert_eq!(when(&noon(1), today), "Sep 1");
        assert_eq!(when("not a time", today), "");
    }

    #[test]
    fn saved_turns_keep_yours_the_models_and_its_tools() {
        let conversation = SavedConversation {
            session_id: "s".into(),
            timestamp: String::new(),
            messages: ["User", "System", "Tool", "Assistant"]
                .iter()
                .map(|role| history::SavedMessage {
                    role: role.to_string(),
                    body: role.to_lowercase(),
                })
                .collect(),
            custom_title: None,
        };
        let who: Vec<Who> = turns_of(&conversation).iter().map(|t| t.who).collect();
        assert_eq!(who, [Who::You, Who::Tool, Who::Model]);
    }
}
