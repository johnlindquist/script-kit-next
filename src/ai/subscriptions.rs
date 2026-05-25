//! Runtime subscription registry for SDK-visible Agent Chat events.

use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Mutex};
use std::sync::{MutexGuard, OnceLock};

use uuid::Uuid;

use crate::protocol::{AiMessageInfo, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AiSubscriptionEvent {
    Message,
    StreamChunk,
    StreamComplete,
    Error,
}

impl AiSubscriptionEvent {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "message" => Some(Self::Message),
            "streamChunk" => Some(Self::StreamChunk),
            "streamComplete" => Some(Self::StreamComplete),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::StreamChunk => "streamChunk",
            Self::StreamComplete => "streamComplete",
            Self::Error => "error",
        }
    }
}

#[derive(Clone)]
struct AiSubscription {
    owner_id: String,
    events: HashSet<AiSubscriptionEvent>,
    chat_id: Option<String>,
    sender: mpsc::SyncSender<Message>,
}

#[derive(Clone)]
struct DeliveryTarget {
    subscription_id: String,
    sender: mpsc::SyncSender<Message>,
}

#[derive(Default)]
struct AiSubscriptionRegistry {
    subscriptions: HashMap<String, AiSubscription>,
}

static REGISTRY: OnceLock<Mutex<AiSubscriptionRegistry>> = OnceLock::new();

fn registry() -> &'static Mutex<AiSubscriptionRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(AiSubscriptionRegistry::default()))
}

fn registry_guard() -> MutexGuard<'static, AiSubscriptionRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn normalize_events(events: Vec<String>) -> Result<HashSet<AiSubscriptionEvent>, String> {
    if events.is_empty() {
        return Err("aiSubscribe requires at least one event".to_string());
    }

    let mut normalized = HashSet::new();
    for event in events {
        let parsed = AiSubscriptionEvent::parse(event.as_str())
            .ok_or_else(|| format!("unsupported ai event type: {event}"))?;
        normalized.insert(parsed);
    }

    Ok(normalized)
}

pub(crate) fn handle_subscribe(
    owner_id: String,
    request_id: String,
    events: Vec<String>,
    chat_id: Option<String>,
    sender: mpsc::SyncSender<Message>,
) -> Message {
    let events = match normalize_events(events) {
        Ok(events) => events,
        Err(message) => {
            return Message::AiError {
                subscription_id: None,
                request_id: Some(request_id),
                code: "INVALID_SUBSCRIPTION".to_string(),
                message,
            };
        }
    };

    let subscription_id = format!("ai-sub-{}", Uuid::new_v4());
    let confirmed_events = events
        .iter()
        .map(|event| event.as_str().to_string())
        .collect::<Vec<_>>();

    let subscription = AiSubscription {
        owner_id,
        events,
        chat_id,
        sender,
    };

    registry_guard()
        .subscriptions
        .insert(subscription_id.clone(), subscription);

    Message::AiSubscribed {
        request_id,
        subscription_id,
        events: confirmed_events,
    }
}

pub(crate) fn handle_unsubscribe(
    owner_id: &str,
    request_id: String,
    subscription_id: String,
) -> Message {
    let mut guard = registry_guard();
    let (success, error) = match guard.subscriptions.get(&subscription_id) {
        Some(subscription) if subscription.owner_id == owner_id => {
            guard.subscriptions.remove(&subscription_id);
            (true, None)
        }
        Some(_) => (false, Some("AI_SUBSCRIPTION_OWNER_MISMATCH".to_string())),
        None => (false, Some("AI_SUBSCRIPTION_NOT_FOUND".to_string())),
    };

    Message::AiUnsubscribed {
        request_id,
        subscription_id,
        success,
        error,
    }
}

pub(crate) fn cleanup_owner(owner_id: &str) -> usize {
    let mut guard = registry_guard();
    let before = guard.subscriptions.len();
    guard
        .subscriptions
        .retain(|_, subscription| subscription.owner_id != owner_id);
    before.saturating_sub(guard.subscriptions.len())
}

fn delivery_targets(event: AiSubscriptionEvent, chat_id: Option<&str>) -> Vec<DeliveryTarget> {
    let guard = registry_guard();
    guard
        .subscriptions
        .iter()
        .filter(|(_, subscription)| subscription.events.contains(&event))
        .filter(|(_, subscription)| {
            subscription
                .chat_id
                .as_deref()
                .zip(chat_id)
                .map(|(expected, actual)| expected == actual)
                .unwrap_or(true)
        })
        .map(|(subscription_id, subscription)| DeliveryTarget {
            subscription_id: subscription_id.clone(),
            sender: subscription.sender.clone(),
        })
        .collect()
}

fn send_to_targets<F>(targets: Vec<DeliveryTarget>, mut build: F)
where
    F: FnMut(String) -> Message,
{
    let mut disconnected = Vec::new();
    for target in targets {
        let subscription_id = target.subscription_id.clone();
        if target
            .sender
            .try_send(build(subscription_id.clone()))
            .is_err()
        {
            disconnected.push(subscription_id);
        }
    }

    if !disconnected.is_empty() {
        let disconnected: HashSet<String> = disconnected.into_iter().collect();
        registry_guard()
            .subscriptions
            .retain(|subscription_id, _| !disconnected.contains(subscription_id));
    }
}

pub(crate) fn publish_new_message(chat_id: &str, message: AiMessageInfo) {
    let targets = delivery_targets(AiSubscriptionEvent::Message, Some(chat_id));
    send_to_targets(targets, |subscription_id| Message::AiNewMessage {
        subscription_id,
        chat_id: chat_id.to_string(),
        message: message.clone(),
    });
}

pub(crate) fn publish_stream_chunk(chat_id: &str, chunk: String, accumulated_content: String) {
    let targets = delivery_targets(AiSubscriptionEvent::StreamChunk, Some(chat_id));
    send_to_targets(targets, |subscription_id| Message::AiStreamChunk {
        subscription_id,
        chat_id: chat_id.to_string(),
        chunk: chunk.clone(),
        accumulated_content: accumulated_content.clone(),
    });
}

pub(crate) fn publish_stream_complete(
    chat_id: &str,
    message_id: String,
    full_content: String,
    tokens_used: Option<u32>,
) {
    let targets = delivery_targets(AiSubscriptionEvent::StreamComplete, Some(chat_id));
    send_to_targets(targets, |subscription_id| Message::AiStreamComplete {
        subscription_id,
        chat_id: chat_id.to_string(),
        message_id: message_id.clone(),
        full_content: full_content.clone(),
        tokens_used,
    });
}

pub(crate) fn publish_error(chat_id: Option<&str>, code: String, message: String) {
    let targets = delivery_targets(AiSubscriptionEvent::Error, chat_id);
    send_to_targets(targets, |subscription_id| Message::AiError {
        subscription_id: Some(subscription_id),
        request_id: None,
        code: code.clone(),
        message: message.clone(),
    });
}

#[cfg(test)]
pub(crate) fn reset_for_test() {
    registry().lock().unwrap().subscriptions.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subscribe_for_test(
        owner: &str,
        events: Vec<&str>,
        chat_id: Option<&str>,
    ) -> (String, mpsc::Receiver<Message>) {
        let (tx, rx) = mpsc::sync_channel(16);
        let response = handle_subscribe(
            owner.to_string(),
            format!("req-{owner}"),
            events.into_iter().map(str::to_string).collect(),
            chat_id.map(str::to_string),
            tx,
        );
        let Message::AiSubscribed {
            subscription_id, ..
        } = response
        else {
            panic!("expected aiSubscribed response");
        };
        (subscription_id, rx)
    }

    #[test]
    fn publish_filters_by_event_and_chat() {
        reset_for_test();
        let (_sub_a, rx_a) = subscribe_for_test("owner-a", vec!["message"], Some("chat-a"));
        let (_sub_b, rx_b) = subscribe_for_test("owner-b", vec!["message"], Some("chat-b"));
        let (_sub_c, rx_c) = subscribe_for_test("owner-c", vec!["streamChunk"], Some("chat-a"));

        publish_new_message(
            "chat-a",
            AiMessageInfo {
                id: "msg-1".into(),
                role: "user".into(),
                content: "hello".into(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                tokens_used: None,
            },
        );

        assert!(matches!(rx_a.try_recv(), Ok(Message::AiNewMessage { .. })));
        assert!(rx_b.try_recv().is_err());
        assert!(rx_c.try_recv().is_err());
    }

    #[test]
    fn unsubscribe_and_owner_cleanup_prevent_delivery() {
        reset_for_test();
        let (sub_a, rx_a) = subscribe_for_test("owner-a", vec!["streamChunk"], Some("chat-a"));
        let (_sub_b, rx_b) = subscribe_for_test("owner-b", vec!["streamChunk"], Some("chat-a"));

        let _ = handle_unsubscribe("owner-a", "unsub-a".to_string(), sub_a);
        publish_stream_chunk("chat-a", "x".to_string(), "x".to_string());
        assert!(rx_a.try_recv().is_err());
        assert!(matches!(rx_b.try_recv(), Ok(Message::AiStreamChunk { .. })));

        assert_eq!(cleanup_owner("owner-b"), 1);
        publish_stream_chunk("chat-a", "y".to_string(), "xy".to_string());
        assert!(rx_b.try_recv().is_err());
    }
}
