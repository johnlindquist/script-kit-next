//! Bounded in-process mailboxes. A handoff is one frozen body for one registered principal,
//! never a reference to live selection. No private text, token, or reply goes in a Herdr notice.
use super::{commands::name_ok, herdr};
use crate::mcp_kit_tools::ToolDefinition;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};
use uuid::Uuid;
const HOUR: Duration = Duration::from_secs(3600);
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Address {
    pub name: String,
    pub session: String,
    pub core: String,
    pub terminal: String,
    pub created_ms: u64,
    pub pid: u32,
}
#[derive(Clone)]
struct Peer {
    id: Uuid,
    owner: String,
    addr: Address,
    at: Instant,
}
#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Checking,
    Offered,
    Notified,
    Taken,
    Replied,
    Refused,
    NotificationUnknown,
    Cancelled,
}
struct Ticket {
    id: Uuid,
    key: Uuid,
    sender: String,
    peer: Peer,
    at: Instant,
    digest: [u8; 32],
    body: Option<String>,
    reply: Option<String>,
    draft: Option<String>,
    phase: Phase,
    reason: Option<&'static str>,
}
#[derive(Default)]
struct Store {
    peers: Vec<Peer>,
    tickets: HashMap<Uuid, Ticket>,
}
static STORE: LazyLock<Mutex<Store>> = LazyLock::new(Default::default);
static EVENTS: LazyLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = LazyLock::new(|| async_channel::bounded(64));
pub(crate) fn events() -> async_channel::Receiver<String> {
    EVENTS.1.clone()
}
fn store() -> parking_lot::MutexGuard<'static, Store> {
    let mut s = STORE.lock();
    s.peers.retain(|p| p.at.elapsed() < HOUR * 8);
    s.tickets.retain(|_, t| t.at.elapsed() < HOUR);
    s
}
fn grant(scopes: &[String], prefix: &str, name: &str) -> bool {
    scopes.iter().any(|s| s.strip_prefix(prefix) == Some(name))
}
fn any(scopes: &[String], prefix: &str) -> bool {
    scopes
        .iter()
        .any(|s| s.strip_prefix(prefix).is_some_and(name_ok))
}
pub(crate) fn permitted(name: &str, ss: &[String]) -> bool {
    match name {
        "kit/handoff_available" | "kit/handoff_away" | "kit/handoff_take" | "kit/handoff_reply" => {
            any(ss, "handoff:receive:")
        }
        "kit/handoff_targets"
        | "kit/handoff_send"
        | "kit/handoff_status"
        | "kit/handoff_cancel" => any(ss, "handoff:send:"),
        _ => false,
    }
}
pub(super) fn definitions() -> Vec<ToolDefinition> {
    let string = || json!({"type":"string"});
    let id = || json!({"type":"string","format":"uuid"});
    let body = || json!({"type":"string","minLength":1,"maxLength":65536});
    let tool = |verb: &str, description: &str, fields: Vec<(&str, Value)>| {
        let properties: serde_json::Map<String, Value> = fields
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), schema))
            .collect();
        let required: Vec<_> = properties.keys().collect();
        ToolDefinition {
            name: format!("kit/handoff_{verb}"),
            description: description.into(),
            input_schema: json!({"type":"object","additionalProperties":false,"required":required,"properties":properties}),
        }
    };
    vec![
        tool("available", "Register your own named live local Herdr terminal.", vec![("name", string()), ("session", string()), ("core", string()), ("terminal", string()), ("createdMs", json!({"type":"integer"})), ("pid", json!({"type":"integer"}))]),
        tool("away", "Stop accepting new handoffs and close pending grants.", vec![]),
        tool("targets", "List only recipients you are explicitly allowed to send to.", vec![]),
        tool("send", "Share only the supplied text with one selected registration. A notification is not a reply.", vec![("target", id()), ("requestId", id()), ("body", body())]),
        tool("status", "Observe your original request. A response is an editable draft, not a paste.", vec![("requestId", id())]),
        tool("cancel", "Revoke future reading or replying. Cannot recall text already read.", vec![("requestId", id())]),
        tool("take", "Read the frozen text only as the designated recipient.", vec![("id", id())]),
        tool("reply", "Return one draft. Does not open a window or execute it.", vec![("id", id()), ("body", body())]),
    ]
}
fn authorized(p: &Peer) -> bool {
    #[cfg(feature = "owned-ui-evaluation")]
    if crate::runtime_policy::is_owned_evaluation() {
        return p.owner == "owned-fixture";
    }
    crate::mcp_server::recipient_grant_current(&p.addr.name, &p.owner)
}
fn valid(a: &Address) -> bool {
    name_ok(&a.name)
        && !a.session.is_empty()
        && a.session.len() <= 64
        && a.session
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        && a.core.len() == 32
        && a.core.bytes().all(|b| b.is_ascii_hexdigit())
        && a.terminal
            .strip_prefix('t')
            .and_then(|s| s.parse::<u32>().ok())
            .is_some_and(|i| i > 0 && a.terminal == format!("t{i}"))
        && a.pid > 0
        && a.created_ms > 0
}
fn close(t: &mut Ticket) {
    if t.phase != Phase::Replied {
        t.body = None;
        t.phase = Phase::Cancelled;
    }
}
fn retire(s: &mut Store, owner: &str) {
    s.peers.retain(|p| p.owner != owner);
    for t in s.tickets.values_mut().filter(|t| t.peer.owner == owner) {
        close(t)
    }
}
fn register(owner: &str, addr: Address) -> Result<Value, &'static str> {
    let mut s = store();
    if let Some(p) = s.peers.iter().find(|p| {
        p.owner == owner
            && p.addr.terminal == addr.terminal
            && p.addr.core == addr.core
            && p.addr.created_ms == addr.created_ms
            && p.addr.pid == addr.pid
            && p.addr.name == addr.name
            && p.addr.session == addr.session
    }) {
        return Ok(json!({"ok":true,"registration":p.id}));
    }
    if s.peers.len() >= 32 {
        return Err("recipient_limit");
    }
    retire(&mut s, owner);
    let id = Uuid::new_v4();
    s.peers.push(Peer {
        id,
        owner: owner.into(),
        addr,
        at: Instant::now(),
    });
    Ok(json!({"ok":true,"registration":id}))
}
pub(crate) fn recipients() -> Vec<(Uuid, String, String)> {
    let mut ps: Vec<_> = store()
        .peers
        .iter()
        .filter(|p| authorized(p))
        .map(|p| (p.id, p.addr.name.clone(), p.addr.terminal.clone()))
        .collect();
    ps.sort_by(|a, b| a.1.cmp(&b.1));
    ps
}
fn receipt(t: &Ticket) -> Value {
    json!({"ok":true,"id":t.id,"requestId":t.key,"recipient":t.peer.addr.name,"state":t.phase,"reason":t.reason,"reply":t.reply})
}
fn body_ok(b: &str) -> bool {
    !b.trim().is_empty() && b.len() <= 65536
}
pub(crate) fn send(
    sender: &str,
    scopes: Option<&[String]>,
    target: Uuid,
    key: Uuid,
    body: String,
) -> Result<Value, &'static str> {
    if !body_ok(&body) {
        return Err("invalid_body");
    }
    let digest: [u8; 32] = Sha256::digest(body.as_bytes()).into();
    let (peer, id) = {
        let mut s = store();
        if let Some(t) = s
            .tickets
            .values()
            .find(|t| t.sender == sender && t.key == key)
        {
            if scopes.is_some_and(|ss| !grant(ss, "handoff:send:", &t.peer.addr.name)) {
                return Err("scope_denied");
            }
            if t.peer.id != target || digest != t.digest {
                return Err("request_conflict");
            }
            return Ok(receipt(t));
        }
        let p = s
            .peers
            .iter()
            .find(|p| p.id == target)
            .filter(|p| authorized(p))
            .cloned()
            .ok_or("recipient_changed")?;
        if scopes.is_some_and(|ss| !grant(ss, "handoff:send:", &p.addr.name)) {
            return Err("scope_denied");
        }
        // Receipts (including refused/unknown ones) survive retries for one hour. One
        // external principal cannot consume the other agents' or the launcher's slots.
        if sender != "launcher"
            && (s.tickets.values().filter(|t| t.sender == sender).count() >= 12
                || s.tickets
                    .values()
                    .filter(|t| t.sender != "launcher")
                    .count()
                    >= 48)
        {
            return Err("sender_handoff_limit");
        }
        if s.tickets.len() >= 64 {
            return Err("handoff_limit");
        }
        let id = Uuid::new_v4();
        s.tickets.insert(
            id,
            Ticket {
                id,
                key,
                sender: sender.into(),
                peer: p.clone(),
                at: Instant::now(),
                digest,
                body: Some(body),
                reply: None,
                draft: None,
                phase: Phase::Checking,
                reason: None,
            },
        );
        (p, id)
    };
    let before = || {
        let mut s = store();
        if !s.peers.iter().any(|p| p.id == peer.id && authorized(p)) {
            return Err("recipient_changed");
        }
        let t = s.tickets.get_mut(&id).ok_or("expired")?;
        if t.phase != Phase::Checking {
            return Err("no_longer_pending");
        }
        t.phase = Phase::Offered;
        Ok(())
    };
    let result = notify(&peer, id, before);
    let mut s = store();
    let t = s.tickets.get_mut(&id).ok_or("expired")?;
    if matches!(t.phase, Phase::Offered | Phase::Checking) {
        match result {
            Ok(()) => t.phase = Phase::Notified,
            Err("notification_unknown") => {
                t.phase = Phase::NotificationUnknown;
                t.reason = Some("check_status_do_not_resend");
            }
            Err(e) => {
                t.phase = Phase::Refused;
                t.body = None;
                t.reason = Some(e);
            }
        }
    }
    Ok(receipt(t))
}
fn notify(
    peer: &Peer,
    id: Uuid,
    before: impl FnOnce() -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    #[cfg(feature = "owned-ui-evaluation")]
    if crate::runtime_policy::is_owned_evaluation() {
        if peer.owner != "owned-fixture" {
            return Err("herdr_forbidden");
        }
        return before();
    }
    herdr::notify(&peer.addr, id, before)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handle {
    id: Uuid,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    id: Uuid,
    body: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Key {
    request_id: Uuid,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Send {
    request_id: Uuid,
    target: Uuid,
    body: String,
}
fn decode<T: serde::de::DeserializeOwned>(v: Value) -> Result<T, &'static str> {
    serde_json::from_value(v).map_err(|_| "invalid_arguments")
}
pub(crate) fn call(
    name: &str,
    owner: &str,
    ss: &[String],
    v: Value,
) -> Result<Value, &'static str> {
    if !permitted(name, ss) {
        return Err("scope_denied");
    }
    match name {
        "kit/handoff_available" => {
            let a: Address = decode(v)?;
            if !valid(&a) || !grant(ss, "handoff:receive:", &a.name) {
                return Err("invalid_registration");
            }
            herdr::validate(&a)?;
            register(owner, a)
        }
        "kit/handoff_away" if v == json!({}) => {
            retire(&mut store(), owner);
            Ok(json!({"ok":true,"available":false}))
        }
        "kit/handoff_targets" if v == json!({}) => Ok(
            json!({"ok":true,"recipients":recipients().into_iter().filter(|(_,n,_)|grant(ss,"handoff:send:",n)).map(|(target,name,terminal)|json!({"target":target,"name":name,"terminal":terminal})).collect::<Vec<_>>()}),
        ),
        "kit/handoff_send" => {
            let a: Send = decode(v)?;
            let r = send(owner, Some(ss), a.target, a.request_id, a.body)?;
            match r["state"].as_str() {
                Some("refused") => Err("handoff_refused_check_status"),
                Some("notification_unknown") => Err("notification_unknown_check_status"),
                _ => Ok(r),
            }
        }
        "kit/handoff_status" | "kit/handoff_cancel" => {
            let a: Key = decode(v)?;
            let mut s = store();
            let t = s
                .tickets
                .values_mut()
                .find(|t| {
                    t.key == a.request_id
                        && t.sender == owner
                        && grant(ss, "handoff:send:", &t.peer.addr.name)
                })
                .ok_or("unknown_request")?;
            if name.ends_with("cancel") {
                close(t);
            }
            Ok(receipt(t))
        }
        "kit/handoff_take" | "kit/handoff_reply" => {
            let (id, body) = if name.ends_with("take") {
                (decode::<Handle>(v)?.id, None)
            } else {
                let p: Reply = decode(v)?;
                if !body_ok(&p.body) {
                    return Err("invalid_body");
                }
                (p.id, Some(p.body))
            };
            let mut s = store();
            let t = s
                .tickets
                .get_mut(&id)
                .filter(|t| {
                    t.peer.owner == owner && grant(ss, "handoff:receive:", &t.peer.addr.name)
                })
                .ok_or("unknown_handoff")?;
            if let Some(body) = body {
                if t.phase == Phase::Replied && t.reply.as_ref() == Some(&body) {
                    return Ok(json!({"ok":true,"state":"replied"}));
                }
                if t.phase != Phase::Taken {
                    return Err("not_awaiting_reply");
                }
                t.phase = Phase::Replied;
                t.body = None;
                t.reply = Some(body.clone());
                t.draft = Some(body);
                let _ = EVENTS.0.try_send(t.peer.addr.name.clone());
                Ok(json!({"ok":true,"state":"replied"}))
            } else if matches!(
                t.phase,
                Phase::Offered | Phase::Notified | Phase::NotificationUnknown | Phase::Taken
            ) {
                t.phase = Phase::Taken;
                Ok(json!({"ok":true,"body":t.body,"id":id,"state":"taken"}))
            } else {
                Err("no_longer_shared")
            }
        }
        _ => Err("invalid_arguments"),
    }
}
pub(crate) fn inbox() -> Vec<(Uuid, String, bool)> {
    let s = store();
    let mut a: Vec<_> = s
        .tickets
        .values()
        .filter(|t| !matches!(t.phase, Phase::Cancelled | Phase::Refused))
        .collect();
    a.sort_by_key(|t| std::cmp::Reverse(t.at));
    a.into_iter()
        .map(|t| (t.id, t.peer.addr.name.clone(), t.draft.is_some()))
        .collect()
}
pub(crate) fn cancel_local(id: Uuid) -> bool {
    if let Some(t) = store()
        .tickets
        .get_mut(&id)
        .filter(|t| t.phase != Phase::Replied)
    {
        close(t);
        return true;
    }
    false
}
pub(crate) fn local_phase(id: Uuid) -> Option<&'static str> {
    store().tickets.get(&id).and_then(|t| match t.phase {
        Phase::Taken => Some("taken"),
        Phase::Replied => Some("replied"),
        Phase::Cancelled => Some("cancelled"),
        _ => None,
    })
}
pub(crate) fn draft(id: Uuid) -> Option<String> {
    store().tickets.get(&id)?.draft.clone()
}
pub(crate) fn edit(id: Uuid, old: &[u8; 32], words: &str) -> bool {
    let mut s = store();
    let Some(t) = s.tickets.get_mut(&id) else {
        return false;
    };
    if words.len() > 65536
        || !t
            .draft
            .as_ref()
            .is_some_and(|d| <[u8; 32]>::from(Sha256::digest(d.as_bytes())) == *old)
    {
        return false;
    }
    t.draft = Some(words.into());
    true
}
#[cfg(feature = "owned-ui-evaluation")]
pub(crate) fn fixture_peer() -> Result<(), &'static str> {
    if !crate::runtime_policy::is_owned_evaluation() {
        return Err("not_owned");
    }
    register(
        "owned-fixture",
        Address {
            name: "workshop".into(),
            session: "fixture".into(),
            terminal: "t1".into(),
            core: "0".repeat(32),
            created_ms: 1,
            pid: 1,
        },
    )
    .map(drop)
}
#[cfg(feature = "owned-ui-evaluation")]
pub(crate) fn fixture_answer() -> Result<Value, &'static str> {
    if !crate::runtime_policy::is_owned_evaluation() {
        return Err("not_owned");
    }
    let id = store()
        .tickets
        .values()
        .find(|t| t.phase == Phase::Notified && t.peer.owner == "owned-fixture")
        .map(|t| t.id)
        .ok_or("no_notification")?;
    let ss = ["handoff:receive:workshop".into()];
    let taken = call("kit/handoff_take", "owned-fixture", &ss, json!({"id":id}))?;
    call(
        "kit/handoff_reply",
        "owned-fixture",
        &ss,
        json!({"id":id,"body":format!("Draft: {}",taken["body"].as_str().unwrap())}),
    )
}
