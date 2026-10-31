//! One effect: replace the filename in an already-open native Save sheet. The
//! launcher and MCP both use this service; neither can supply a ref or command.
mod native;
#[cfg(any(test, feature = "owned-ui-evaluation"))]
pub mod owned;
mod sheet;
#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub type Failure = &'static str;
const TTL: Duration = Duration::from_secs(60);
const LIMIT: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub app: String,
    pub pid: u32,
    pub window_id: String,
    pub title: String,
    pub process_instance: String,
}

/// The CLI can only be asked for these four operations, never arbitrary argv.
trait Backend: Send + Sync {
    fn windows(&self, app: &str) -> Result<Vec<Window>, Failure>;
    fn sheet(&self, window: &Window) -> Result<Value, Failure>;
    fn value(&self, reference: &str) -> Result<String, Failure>;
    fn set_value(&self, reference: &str, value: &str) -> Result<(), Failure>;
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub proposal_id: Uuid,
    pub window: Window,
    pub filename: String,
    pub field: String,
    pub expires_in_seconds: u64,
}
#[derive(Clone)]
struct Grant {
    proposal: Proposal,
    reference: String,
    old: String,
}
#[derive(Clone, Copy, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Proposed,
    Applying,
    Verified,
    Refused { code: Failure },
    Uncertain { code: Failure },
}
struct Entry {
    caller: String,
    issued: Instant,
    grant: Option<Grant>,
    status: Status,
}
struct Service<B> {
    backend: B,
    entries: Mutex<HashMap<Uuid, Entry>>,
    dispatch: Mutex<()>,
}
static SERVICE: LazyLock<Service<native::Native>> =
    LazyLock::new(|| Service::new(native::Native::default()));

/// The application and agent transports share the same operations. Principal is
/// assigned by the transport, never read from a tool argument.
pub trait FilenameApi: Sync {
    fn propose(
        &self,
        caller: &str,
        app: &str,
        pid: u32,
        native_id: u32,
        title: Option<&str>,
        filename: &str,
    ) -> Result<Proposal, Failure>;
    fn apply(&self, caller: &str, id: Uuid) -> Result<Status, Failure>;
    fn status(&self, caller: &str, id: Uuid) -> Result<Status, Failure>;
}
pub fn agent_api() -> &'static dyn FilenameApi {
    &*SERVICE
}
pub fn launcher_api(production: bool) -> Result<&'static dyn FilenameApi, Failure> {
    if production && !crate::runtime_policy::is_owned_evaluation() {
        return Ok(&*SERVICE);
    }
    #[cfg(feature = "owned-ui-evaluation")]
    if !production && crate::runtime_policy::is_owned_evaluation() {
        return Ok(owned::api());
    }
    Err("native_effects_forbidden")
}

pub fn valid_filename(s: &str) -> bool {
    !s.trim().is_empty()
        && s.len() <= 255
        && !matches!(s, "." | "..")
        && !s.contains(['/', ':', '\\'])
        && !s.chars().any(char::is_control)
}

impl<B: Backend> Service<B> {
    fn new(backend: B) -> Self {
        Self {
            backend,
            entries: Mutex::default(),
            dispatch: Mutex::new(()),
        }
    }
}
impl<B: Backend> FilenameApi for Service<B> {
    fn propose(
        &self,
        caller: &str,
        app: &str,
        pid: u32,
        native_id: u32,
        title: Option<&str>,
        filename: &str,
    ) -> Result<Proposal, Failure> {
        if !valid_filename(filename)
            || app.trim().is_empty()
            || app.len() > 180
            || app.starts_with('-')
            || app.chars().any(char::is_control)
            || pid == 0
            || native_id == 0
        {
            return Err("invalid_target_or_filename");
        }
        let id = format!("w-{native_id}");
        let mut matches = self.backend.windows(app)?.into_iter().filter(|w| {
            // Without Screen Recording macOS may omit the CG title; the CLI
            // substitutes the app name. PID and native ID are still mandatory.
            w.pid == pid
                && w.app == app
                && id == w.window_id
                && title.is_none_or(|t| t == w.title || w.title == w.app)
        });
        let window = matches.next().ok_or("window_not_found")?;
        if matches.next().is_some() {
            return Err("ambiguous_window");
        }
        let field = sheet::filename_field(&self.backend.sheet(&window)?, &window)?;
        let old = self.backend.value(&field.reference)?;
        if !self.backend.windows(app)?.contains(&window) {
            return Err("window_changed");
        }
        let p = Proposal {
            proposal_id: Uuid::new_v4(),
            window,
            filename: filename.into(),
            field: field.label,
            expires_in_seconds: TTL.as_secs(),
        };
        let mut entries = self.entries.lock();
        entries.retain(|_, e| e.issued.elapsed() < TTL || matches!(e.status, Status::Applying));
        if entries.len() >= LIMIT {
            return Err("too_many_proposals");
        }
        entries.insert(
            p.proposal_id,
            Entry {
                caller: caller.into(),
                issued: Instant::now(),
                status: Status::Proposed,
                grant: Some(Grant {
                    proposal: p.clone(),
                    reference: field.reference,
                    old,
                }),
            },
        );
        Ok(p)
    }
    fn status(&self, caller: &str, id: Uuid) -> Result<Status, Failure> {
        self.entries
            .lock()
            .get(&id)
            .filter(|e| {
                e.caller == caller
                    && (e.issued.elapsed() < TTL || matches!(e.status, Status::Applying))
            })
            .map(|e| e.status)
            .ok_or("unknown_or_expired_proposal")
    }
    fn apply(&self, caller: &str, id: Uuid) -> Result<Status, Failure> {
        let _writer = self.dispatch.try_lock().ok_or("desktop_busy")?;
        let grant = {
            let mut entries = self.entries.lock();
            let e = entries
                .get_mut(&id)
                .filter(|e| e.caller == caller && e.issued.elapsed() < TTL)
                .ok_or("unknown_or_expired_proposal")?;
            let Some(g) = e.grant.take() else {
                return Ok(e.status);
            };
            e.status = Status::Applying;
            g
        };
        let w = &grant.proposal.window;
        let before = (|| {
            if !self.backend.windows(&w.app)?.contains(w) {
                return Err("window_changed");
            };
            if self.backend.value(&grant.reference)? != grant.old {
                return Err("field_changed");
            };
            Ok(())
        })();
        let status = if let Err(code) = before {
            Status::Refused { code }
        } else {
            // After dispatch starts, never fall back, retry, or claim a missing
            // response proves nothing happened. Read the same ref AND open sheet.
            let after = (|| {
                self.backend
                    .set_value(&grant.reference, &grant.proposal.filename)?;
                if self.backend.value(&grant.reference)? != grant.proposal.filename
                    || !self.backend.windows(&w.app)?.contains(w)
                {
                    return Err("readback_mismatch");
                }
                let current = sheet::filename_field(&self.backend.sheet(w)?, w)?;
                if current.label != grant.proposal.field
                    || self.backend.value(&current.reference)? != grant.proposal.filename
                {
                    return Err("sheet_changed");
                };
                Ok(())
            })();
            match after {
                Ok(()) => Status::Verified,
                Err(code) => Status::Uncertain { code },
            }
        };
        if let Some(e) = self.entries.lock().get_mut(&id) {
            e.status = status;
            e.issued = Instant::now();
        }
        Ok(status)
    }
}

pub fn explanation(code: Failure) -> &'static str {
    match code {
        "provider_missing" => "Set up the agent-desktop helper first",
        "provider_version" => "This version of agent-desktop is not supported",
        "permission_denied" => "The desktop helper needs Accessibility access",
        "window_not_found" | "window_changed" => "The original window is no longer available",
        "ambiguous_window" => "More than one window matches",
        "field_changed" => "The filename changed; inspect it again",
        "unknown_or_expired_proposal" => "The preview expired; inspect the filename again",
        "ambiguous_field" => "More than one filename field matches",
        "unsafe_surface" | "missing_field" => "No supported Save or Export sheet was found",
        "native_effects_forbidden" => "Desktop access is disabled in this session",
        "desktop_busy" => "Another filename change is in progress",
        _ => "Couldn’t verify the filename field",
    }
}
