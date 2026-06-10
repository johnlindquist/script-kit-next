//! Telegram bridge: opt-in remote access to the brain for the owner.
//!
//! Lets the user ask their own local memory questions from their phone via a
//! private Telegram bot. Strictly allowlisted: the bridge only starts when
//! the config enables it, provides a bot token, AND lists at least one
//! numeric Telegram user id. An empty allowlist disables the bot entirely —
//! unlisted users are never answered beyond a "not authorized" notice that
//! echoes their own id (so the owner can discover his id during setup).
//!
//! Design constraints:
//! - One background thread ("brain-telegram") long-polling `getUpdates`;
//!   never panics, never hot-loops (errors back off up to a cap).
//! - Answers are grounded in [`super::recall_context_block`] and produced by
//!   the same one-shot pi completion the curator uses ([`super::pi_oneshot`]).
//! - The bot token is a secret: it is never logged, and because Telegram API
//!   URLs embed it, every logged error string is passed through
//!   [`redact_token`] first.
//! - The update offset persists in `brain_meta` (key
//!   `telegram_update_offset`) so restarts neither replay nor drop messages.

use super::store;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::time::Duration;

/// `brain_meta` key holding the last processed Telegram update id.
const OFFSET_KEY: &str = "telegram_update_offset";
/// Telegram caps messages at 4096 chars; stay safely under it.
const REPLY_MAX_CHARS: usize = 4_000;
/// Long-poll wait requested from Telegram (seconds).
const POLL_TIMEOUT_SECS: u64 = 50;
/// HTTP timeout — must exceed the long-poll wait.
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);
/// First sleep after an error; doubles per consecutive error.
const ERROR_BACKOFF_START: Duration = Duration::from_secs(30);
/// Backoff never exceeds this.
const ERROR_BACKOFF_CAP: Duration = Duration::from_secs(300);

const HELLO_REPLY: &str = "Script Kit brain bridge. Send me a question and I'll \
answer it from your local Script Kit memory (notes, chats, recent activity). \
Plain-text answers only.";
const NO_MEMORY_REPLY: &str = "Nothing in memory about that.";
const NO_AGENT_REPLY: &str = "No agent backend available.";
const LOOKUP_FAILED_REPLY: &str = "Memory lookup failed.";

/// Start the Telegram bridge if (and only if) the config activates it.
/// Spawns one background thread; returns immediately either way.
pub fn start_telegram_bridge() {
    let remote = crate::config::load_config()
        .brain_remote
        .unwrap_or_default();
    if !remote.is_active() {
        tracing::debug!(
            target: "script_kit::brain",
            "telegram bridge inactive (needs enabled + bot token + non-empty allowlist)"
        );
        return;
    }
    let token = remote
        .telegram_bot_token
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_string();
    let allowed = remote.telegram_allowed_user_ids.clone();
    let allowed_count = allowed.len();
    let spawned = std::thread::Builder::new()
        .name("brain-telegram".to_string())
        .spawn(move || poll_loop(token, allowed));
    match spawned {
        Ok(_) => tracing::info!(
            target: "script_kit::brain",
            allowed_users = allowed_count,
            "telegram bridge started"
        ),
        Err(error) => tracing::warn!(
            target: "script_kit::brain",
            error = %error,
            "telegram bridge thread failed to spawn"
        ),
    }
}

/// The long-poll loop. Errors are logged (token-redacted) at debug and
/// retried with capped exponential backoff; this function never returns.
fn poll_loop(token: String, allowed: Vec<i64>) {
    let agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(HTTP_TIMEOUT))
        .build()
        .new_agent();
    let mut backoff = ERROR_BACKOFF_START;
    loop {
        let offset = store::meta_get(OFFSET_KEY)
            .ok()
            .flatten()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        let url = format!(
            "https://api.telegram.org/bot{token}/getUpdates?timeout={POLL_TIMEOUT_SECS}&offset={}",
            offset + 1
        );
        let batch = agent
            .get(&url)
            .call()
            .map_err(anyhow::Error::from)
            .and_then(|mut response| {
                let body = response.body_mut().read_to_string()?;
                parse_updates_json(&body)
            });
        match batch {
            Ok(updates) => {
                backoff = ERROR_BACKOFF_START;
                for message in incoming_messages(&updates) {
                    handle_message(&agent, &token, &allowed, &message);
                }
                if let Some(last_update_id) = next_offset(&updates) {
                    if let Err(error) = store::meta_set(OFFSET_KEY, &last_update_id.to_string()) {
                        tracing::debug!(
                            target: "script_kit::brain",
                            error = %redact_token(&token, &error.to_string()),
                            "telegram offset persist failed"
                        );
                    }
                }
            }
            Err(error) => {
                tracing::debug!(
                    target: "script_kit::brain",
                    error = %redact_token(&token, &format!("{error:#}")),
                    "telegram poll failed; backing off"
                );
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(ERROR_BACKOFF_CAP);
            }
        }
    }
}

/// Route one incoming message: allowlist gate, `/start` hello, or a brain
/// question answered from memory.
fn handle_message(agent: &ureq::Agent, token: &str, allowed: &[i64], message: &IncomingMessage) {
    let reply = if !is_authorized(message.user_id, allowed) {
        tracing::info!(
            target: "script_kit::brain",
            user_id = message.user_id,
            "telegram message from unlisted user rejected"
        );
        format!(
            "Not authorized. Your Telegram user id is {}.",
            message.user_id
        )
    } else if message.text.trim() == "/start" {
        HELLO_REPLY.to_string()
    } else {
        answer_question(&message.text)
    };
    send_reply(agent, token, message.chat_id, &reply);
}

/// Answer a brain question: recall memory, ask pi to answer from it only.
/// Always returns something sendable; failures degrade to terse notices.
fn answer_question(question: &str) -> String {
    let context = match super::recall_context_block(question) {
        Ok(Some(context)) => context,
        Ok(None) => return NO_MEMORY_REPLY.to_string(),
        Err(error) => {
            tracing::debug!(
                target: "script_kit::brain",
                error = %error,
                "telegram recall failed"
            );
            return LOOKUP_FAILED_REPLY.to_string();
        }
    };
    match super::pi_oneshot(&build_answer_prompt(question, &context)) {
        Ok(Some(answer)) if !answer.trim().is_empty() => truncate_reply(&answer),
        Ok(Some(_)) => NO_MEMORY_REPLY.to_string(),
        Ok(None) => NO_AGENT_REPLY.to_string(),
        Err(error) => {
            tracing::debug!(
                target: "script_kit::brain",
                error = %error,
                "telegram answer generation failed"
            );
            LOOKUP_FAILED_REPLY.to_string()
        }
    }
}

/// POST `sendMessage`. Failures are logged (token-redacted) and dropped —
/// the poll loop must keep running.
fn send_reply(agent: &ureq::Agent, token: &str, chat_id: i64, text: &str) {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let payload = serde_json::json!({ "chat_id": chat_id, "text": text });
    if let Err(error) = agent.post(&url).send_json(&payload) {
        tracing::debug!(
            target: "script_kit::brain",
            error = %redact_token(token, &error.to_string()),
            "telegram sendMessage failed"
        );
    }
}

// ============================================================
// Pure core (unit-tested without a network)
// ============================================================

/// One update from `getUpdates`. Unknown update kinds (no `message`) still
/// parse so their `update_id` advances the offset.
#[derive(Debug, Deserialize)]
pub(crate) struct TelegramUpdate {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<TelegramMessageBody>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct TelegramMessageBody {
    pub text: Option<String>,
    pub chat: Option<TelegramPeer>,
    pub from: Option<TelegramPeer>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TelegramPeer {
    pub id: i64,
}

/// A text message worth routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IncomingMessage {
    pub update_id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub text: String,
}

/// Parse a `getUpdates` response body. Errors when the body is not JSON or
/// Telegram reports `ok: false`; individual updates that fail to parse are
/// skipped rather than failing the batch.
pub(crate) fn parse_updates_json(body: &str) -> Result<Vec<TelegramUpdate>> {
    let envelope: serde_json::Value =
        serde_json::from_str(body).context("telegram getUpdates response is not JSON")?;
    if !envelope
        .get("ok")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        anyhow::bail!("telegram getUpdates returned ok=false");
    }
    let Some(result) = envelope.get("result").and_then(|value| value.as_array()) else {
        return Ok(Vec::new());
    };
    Ok(result
        .iter()
        .filter_map(|raw| serde_json::from_value(raw.clone()).ok())
        .collect())
}

/// The text messages in a batch. Updates without a message, text, chat, or
/// sender are skipped (service messages, edits, channel posts, ...).
pub(crate) fn incoming_messages(updates: &[TelegramUpdate]) -> Vec<IncomingMessage> {
    updates
        .iter()
        .filter_map(|update| {
            let message = update.message.as_ref()?;
            let text = message.text.as_deref()?.trim();
            if text.is_empty() {
                return None;
            }
            Some(IncomingMessage {
                update_id: update.update_id,
                chat_id: message.chat.as_ref()?.id,
                user_id: message.from.as_ref()?.id,
                text: text.to_string(),
            })
        })
        .collect()
}

/// The offset to persist after a batch: the highest update id seen across
/// ALL updates (message or not), so nothing is redelivered. `None` for an
/// empty batch (keep the stored offset).
pub(crate) fn next_offset(updates: &[TelegramUpdate]) -> Option<i64> {
    updates.iter().map(|update| update.update_id).max()
}

/// Allowlist gate. An empty allowlist authorizes NOBODY — there is no open
/// mode (the bridge also refuses to start without an allowlist).
pub(crate) fn is_authorized(user_id: i64, allowed_user_ids: &[i64]) -> bool {
    !allowed_user_ids.is_empty() && allowed_user_ids.contains(&user_id)
}

/// The answer prompt: ground the model in recalled memory only, terse plain
/// text (Telegram renders no markdown here).
pub(crate) fn build_answer_prompt(question: &str, context_block: &str) -> String {
    format!(
        "You are answering a question about the user's personal knowledge \
         base, relayed over Telegram. Answer using ONLY the memory context \
         below — no outside knowledge, no guessing. Be terse: a few short \
         plain-text sentences, no markdown formatting. If the context has \
         nothing relevant, say the memory has nothing relevant.\n\n\
         QUESTION:\n{question}\n\n\
         MEMORY CONTEXT:\n{context_block}"
    )
}

/// Trim and cap a reply under Telegram's 4096-char message limit, marking
/// truncation with an ellipsis.
pub(crate) fn truncate_reply(reply: &str) -> String {
    let trimmed = reply.trim();
    if trimmed.chars().count() <= REPLY_MAX_CHARS {
        return trimmed.to_string();
    }
    let mut capped: String = trimmed.chars().take(REPLY_MAX_CHARS - 1).collect();
    capped.push('…');
    capped
}

/// Replace every occurrence of the bot token in `text` (Telegram API URLs
/// embed it, so transport errors would otherwise leak it into logs).
pub(crate) fn redact_token(token: &str, text: &str) -> String {
    if token.is_empty() {
        return text.to_string();
    }
    text.replace(token, "<redacted-token>")
}
