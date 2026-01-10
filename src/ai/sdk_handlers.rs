//! AI SDK Protocol Handlers
//!
//! This module handles AI SDK protocol messages from scripts.
//! It converts between protocol types and storage/window operations.

use anyhow::Result;
use tracing::{debug, error, info};

use crate::protocol::{AiChatInfo, AiMessageInfo, Message};

use super::model::{Chat, ChatId, Message as AiMessage, MessageRole};
use super::storage;

/// Convert a Chat from storage to AiChatInfo for protocol
fn chat_to_info(chat: &Chat, message_count: usize) -> AiChatInfo {
    AiChatInfo {
        id: chat.id.as_str(),
        title: chat.title.clone(),
        model_id: chat.model_id.clone(),
        provider: chat.provider.clone(),
        created_at: chat.created_at.to_rfc3339(),
        updated_at: chat.updated_at.to_rfc3339(),
        is_deleted: chat.deleted_at.is_some(),
        preview: None, // Could add first message preview later
        message_count,
    }
}

/// Convert a Message from storage to AiMessageInfo for protocol
fn message_to_info(msg: &AiMessage) -> AiMessageInfo {
    AiMessageInfo {
        id: msg.id.to_string(),
        role: msg.role.as_str().to_string(),
        content: msg.content.clone(),
        created_at: msg.created_at.to_rfc3339(),
        tokens_used: msg.tokens_used,
    }
}

/// Handle AiIsOpen request - check if AI window is open
pub fn handle_ai_is_open(request_id: String) -> Message {
    let is_open = super::is_ai_window_open();
    // TODO: Get active chat ID from window state
    let active_chat_id = None;

    debug!(request_id = %request_id, is_open = is_open, "AiIsOpen handled");

    Message::AiIsOpenResult {
        request_id,
        is_open,
        active_chat_id,
    }
}

/// Handle AiGetActiveChat request - get info about active chat
pub fn handle_ai_get_active_chat(request_id: String) -> Message {
    // TODO: Get active chat ID from window state
    // For now, return the most recently updated chat
    let chat = match storage::get_all_chats() {
        Ok(chats) => chats.into_iter().next(),
        Err(e) => {
            error!(error = %e, "Failed to get chats for AiGetActiveChat");
            None
        }
    };

    let chat_info = chat.map(|c| {
        let msg_count = storage::get_chat_messages(&c.id)
            .map(|msgs| msgs.len())
            .unwrap_or(0);
        chat_to_info(&c, msg_count)
    });

    debug!(request_id = %request_id, has_chat = chat_info.is_some(), "AiGetActiveChat handled");

    Message::AiActiveChatResult {
        request_id,
        chat: chat_info,
    }
}

/// Handle AiListChats request - list all chats
pub fn handle_ai_list_chats(
    request_id: String,
    limit: Option<usize>,
    include_deleted: bool,
) -> Message {
    let mut chats = match storage::get_all_chats() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to list chats");
            return Message::AiChatListResult {
                request_id,
                chats: vec![],
                total_count: 0,
            };
        }
    };

    if include_deleted {
        if let Ok(deleted) = storage::get_deleted_chats() {
            chats.extend(deleted);
        }
    }

    let total_count = chats.len();

    // Apply limit
    if let Some(limit) = limit {
        chats.truncate(limit);
    }

    let chat_infos: Vec<AiChatInfo> = chats
        .iter()
        .map(|c| {
            let msg_count = storage::get_chat_messages(&c.id)
                .map(|msgs| msgs.len())
                .unwrap_or(0);
            chat_to_info(c, msg_count)
        })
        .collect();

    debug!(
        request_id = %request_id,
        count = chat_infos.len(),
        total = total_count,
        "AiListChats handled"
    );

    Message::AiChatListResult {
        request_id,
        chats: chat_infos,
        total_count,
    }
}

/// Handle AiGetConversation request - get messages from a chat
pub fn handle_ai_get_conversation(
    request_id: String,
    chat_id: Option<String>,
    limit: Option<usize>,
) -> Message {
    // Get chat ID - use provided or try to get active
    let target_chat_id = match chat_id {
        Some(id) => match ChatId::parse(&id) {
            Some(cid) => cid,
            None => {
                error!(chat_id = %id, "Invalid chat ID format");
                return Message::AiConversationResult {
                    request_id,
                    chat_id: id,
                    messages: vec![],
                    has_more: false,
                };
            }
        },
        None => {
            // Try to get most recent chat
            match storage::get_all_chats() {
                Ok(chats) => match chats.into_iter().next() {
                    Some(c) => c.id,
                    None => {
                        return Message::AiConversationResult {
                            request_id,
                            chat_id: String::new(),
                            messages: vec![],
                            has_more: false,
                        };
                    }
                },
                Err(e) => {
                    error!(error = %e, "Failed to get chats for conversation");
                    return Message::AiConversationResult {
                        request_id,
                        chat_id: String::new(),
                        messages: vec![],
                        has_more: false,
                    };
                }
            }
        }
    };

    let messages = match limit {
        Some(lim) => storage::get_recent_messages(&target_chat_id, lim),
        None => storage::get_chat_messages(&target_chat_id),
    };

    let (message_infos, has_more) = match messages {
        Ok(msgs) => {
            let total = msgs.len();
            let infos: Vec<AiMessageInfo> = msgs.iter().map(message_to_info).collect();
            let has_more = limit.map(|l| total >= l).unwrap_or(false);
            (infos, has_more)
        }
        Err(e) => {
            error!(error = %e, chat_id = %target_chat_id, "Failed to get messages");
            (vec![], false)
        }
    };

    debug!(
        request_id = %request_id,
        chat_id = %target_chat_id,
        message_count = message_infos.len(),
        "AiGetConversation handled"
    );

    Message::AiConversationResult {
        request_id,
        chat_id: target_chat_id.as_str(),
        messages: message_infos,
        has_more,
    }
}

/// Handle AiDeleteChat request - delete a chat
pub fn handle_ai_delete_chat(request_id: String, chat_id: String, permanent: bool) -> Message {
    let parsed_id = match ChatId::parse(&chat_id) {
        Some(id) => id,
        None => {
            return Message::AiChatDeleted {
                request_id,
                success: false,
                error: Some(format!("Invalid chat ID: {}", chat_id)),
            };
        }
    };

    let result = if permanent {
        storage::delete_chat_permanently(&parsed_id)
    } else {
        storage::delete_chat(&parsed_id)
    };

    match result {
        Ok(()) => {
            info!(chat_id = %chat_id, permanent = permanent, "Chat deleted via SDK");
            Message::AiChatDeleted {
                request_id,
                success: true,
                error: None,
            }
        }
        Err(e) => {
            error!(error = %e, chat_id = %chat_id, "Failed to delete chat");
            Message::AiChatDeleted {
                request_id,
                success: false,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Handle AiFocus request - focus the AI window
pub fn handle_ai_focus(request_id: String) -> Option<Message> {
    // This needs to be handled by the UI thread, return None to signal forwarding needed
    debug!(request_id = %request_id, "AiFocus needs UI thread handling");
    None
}

/// Handle AiGetStreamingStatus request
pub fn handle_ai_get_streaming_status(request_id: String, _chat_id: Option<String>) -> Message {
    // TODO: Get streaming status from window state
    // For now, return not streaming
    Message::AiStreamingStatusResult {
        request_id,
        is_streaming: false,
        chat_id: None,
        partial_content: None,
    }
}

/// Check if a message is an AI SDK message that can be handled directly
/// Returns Some(response) if handled, None if needs UI thread
pub fn try_handle_ai_message(msg: &Message) -> Option<Message> {
    match msg {
        Message::AiIsOpen { request_id } => Some(handle_ai_is_open(request_id.clone())),

        Message::AiGetActiveChat { request_id } => {
            Some(handle_ai_get_active_chat(request_id.clone()))
        }

        Message::AiListChats {
            request_id,
            limit,
            include_deleted,
        } => Some(handle_ai_list_chats(
            request_id.clone(),
            *limit,
            *include_deleted,
        )),

        Message::AiGetConversation {
            request_id,
            chat_id,
            limit,
        } => Some(handle_ai_get_conversation(
            request_id.clone(),
            chat_id.clone(),
            *limit,
        )),

        Message::AiDeleteChat {
            request_id,
            chat_id,
            permanent,
        } => Some(handle_ai_delete_chat(
            request_id.clone(),
            chat_id.clone(),
            *permanent,
        )),

        Message::AiGetStreamingStatus {
            request_id,
            chat_id,
        } => Some(handle_ai_get_streaming_status(
            request_id.clone(),
            chat_id.clone(),
        )),

        // These need UI thread - return None
        Message::AiFocus { .. }
        | Message::AiStartChat { .. }
        | Message::AiAppendMessage { .. }
        | Message::AiSendMessage { .. }
        | Message::AiSetSystemPrompt { .. }
        | Message::AiSubscribe { .. }
        | Message::AiUnsubscribe { .. } => None,

        // Not an AI message
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_is_open_when_closed() {
        let response = handle_ai_is_open("test-123".to_string());
        match response {
            Message::AiIsOpenResult {
                request_id,
                is_open,
                ..
            } => {
                assert_eq!(request_id, "test-123");
                assert!(!is_open); // Window not open in tests
            }
            _ => panic!("Expected AiIsOpenResult"),
        }
    }
}
