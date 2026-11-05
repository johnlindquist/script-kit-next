//! Versioned, local edits to the launcher's existing controls. A single readable personal
//! document is the commit point. The private journal is written first; an orphan record is not
//! evidence of application. Operations never evaluate code, inspect subject text, or grant access.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sk_one_window::personal::{Edit, LauncherRules};
use std::path::{Path, PathBuf};
use uuid::Uuid;
#[cfg(unix)]
mod files;
#[cfg(unix)]
use files::Dir;
#[cfg(not(unix))]
#[path = "unsupported.rs"]
mod files;
#[cfg(not(unix))]
use files::Dir;
pub mod api;
pub mod speech;
pub mod views;
use views::ViewRecipe;

const RULES_DIR: &[&str] = &["plugins", "main", "customizations"];
const JOURNAL_DIR: &[&str] = &["db", "personal-changes"];
const ACTIVE: &str = "launcher.json";
const STOCK: &str = "stock";
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    WrongWorkspace,
    StaleRevision,
    UnsafePath,
    WriterBusy,
    StorageUnavailable,
    InvalidDocument,
    InvalidRequest,
    InvalidView,
    ViewLimit,
    ViewNotFound,
    RequestConflict,
    NothingToUndo,
    HistoryLimit,
    SafeMode,
    Unsupported,
}
impl Error {
    pub fn code(self) -> &'static str {
        match self {
            Self::WrongWorkspace => "wrong_workspace",
            Self::StaleRevision => "stale_revision",
            Self::UnsafePath => "unsafe_path",
            Self::WriterBusy => "writer_busy",
            Self::StorageUnavailable => "storage_unavailable",
            Self::InvalidDocument => "invalid_document",
            Self::InvalidRequest => "invalid_request",
            Self::InvalidView => "invalid_view",
            Self::ViewLimit => "view_limit",
            Self::ViewNotFound => "view_not_found",
            Self::RequestConflict => "request_conflict",
            Self::NothingToUndo => "nothing_to_undo",
            Self::HistoryLimit => "history_limit",
            Self::SafeMode => "safe_mode",
            Self::Unsupported => "unsupported",
        }
    }
}
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code())
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Author {
    Launcher,
    LocalAgent,
    ConfiguredAgent,
    ConnectedAgent,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Change {
    Edit { edit: Edit },
    PutView { view: ViewRecipe },
    RemoveView { id: String },
    Undo { of: Uuid },
    Enable,
    Disable,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub workspace: String,
    pub expected_revision: String,
    pub request_id: Uuid,
    pub change: Change,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Document {
    schema_version: u8,
    generation: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal: Option<String>,
    enabled: bool,
    rules: LauncherRules,
    // Omit an empty list so existing journal seals still serialize byte-for-byte.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    views: Vec<ViewRecipe>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            schema_version: 1,
            generation: None,
            journal: None,
            enabled: true,
            rules: LauncherRules::default(),
            views: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Observed {
    revision: String,
    doc: Document,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema_version: u8,
    request: Request,
    author: Author,
    before: Observed,
    after: Document,
    previous_undo: Option<Uuid>,
    undo: Option<Uuid>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub workspace: String,
    pub revision: String,
    pub enabled: bool,
    #[serde(serialize_with = "public_rules")]
    pub declared: LauncherRules,
    #[serde(serialize_with = "public_rules")]
    pub effective: LauncherRules,
    /// Declarations only. Disabled recipes do not open, query a source or own a key.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub views: Vec<ViewRecipe>,
    pub undo: Option<Uuid>,
    pub origin: &'static str,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Applied {
    pub request_id: Uuid,
    pub applied_revision: String,
    pub replayed: bool,
    pub state: State,
}
// On-disk defaults preserve old journal seals; authoring clients see the actual value.
fn public_rules<S: serde::Serializer>(
    rules: &LauncherRules,
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    let mut v = serde_json::to_value(rules).map_err(serde::ser::Error::custom)?;
    v["homeMode"] = json!(rules.home_mode);
    v.serialize(s)
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut v = serde_json::to_vec_pretty(v).map_err(|_| Error::InvalidDocument)?;
    v.push(b'\n');
    Ok(v)
}
fn seal(record: &Record) -> Result<String> {
    let mut r = record.clone();
    r.after.journal = None;
    Ok(hash(&bytes(&r)?))
}
fn record_name(id: Uuid) -> String {
    format!("{id}.json")
}
fn valid_revision(s: &str) -> bool {
    s == STOCK
        || (s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)))
}

pub struct Store {
    path: PathBuf,
    root: Dir,
    workspace: String,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let root = Dir::root(path)?;
        let (d, i) = root.identity()?;
        Ok(Self {
            path: path.to_owned(),
            root,
            workspace: hash(format!("kit-personal-v1:{d}:{i}").as_bytes()),
        })
    }
    /// Only the product watcher asks to create the dedicated directory. Inspect is read-only.
    pub fn watch_file(&self) -> Result<PathBuf> {
        self.root
            .path(RULES_DIR, true)?
            .ok_or(Error::StorageUnavailable)?;
        Ok(self.path.join("plugins/main/customizations").join(ACTIVE))
    }
    fn observed(&self) -> Result<Observed> {
        if Dir::root(&self.path)?.identity()? != self.root.identity()? {
            return Err(Error::WrongWorkspace);
        }
        let raw = match self.root.path(RULES_DIR, false)? {
            Some(dir) => dir.read(ACTIVE)?,
            None => None,
        };
        match raw {
            None => Ok(Observed {
                revision: STOCK.into(),
                doc: Document::default(),
            }),
            Some(raw) => {
                let doc: Document =
                    serde_json::from_slice(&raw).map_err(|_| Error::InvalidDocument)?;
                if doc.schema_version != 1 || views::validate_all(&doc.views).is_err() {
                    return Err(Error::InvalidDocument);
                }
                Ok(Observed {
                    revision: hash(&raw),
                    doc,
                })
            }
        }
    }
    fn record(&self, id: Uuid) -> Result<Option<Record>> {
        let Some(dir) = self.root.path(JOURNAL_DIR, false)? else {
            return Ok(None);
        };
        let Some(b) = dir.read(&record_name(id))? else {
            return Ok(None);
        };
        let r: Record = serde_json::from_slice(&b).map_err(|_| Error::InvalidDocument)?;
        if r.request.workspace != self.workspace {
            return Err(Error::WrongWorkspace);
        }
        if r.after.journal.as_deref() != Some(seal(&r)?.as_str())
            || r.schema_version != 1
            || r.after.schema_version != 1
            || r.before.doc.schema_version != 1
            || views::validate_all(&r.after.views).is_err()
            || views::validate_all(&r.before.doc.views).is_err()
            || r.request.request_id != id
            || r.after.generation != Some(id)
            || r.request.expected_revision != r.before.revision
        {
            return Err(Error::InvalidDocument);
        }
        Ok(Some(r))
    }
    fn head(&self, s: &Observed) -> Result<Option<Record>> {
        let Some(id) = s.doc.generation else {
            return Ok(None);
        };
        let r = match self.record(id) {
            Ok(Some(r)) => r,
            Ok(None) | Err(Error::WrongWorkspace) => return Ok(None),
            Err(e) => return Err(e),
        };
        Ok((s.doc == r.after && s.revision == hash(&bytes(&r.after)?)).then_some(r))
    }
    fn state(&self, s: &Observed) -> Result<State> {
        let head = self.head(s)?;
        Ok(State {
            workspace: self.workspace.clone(),
            revision: s.revision.clone(),
            enabled: s.doc.enabled,
            declared: s.doc.rules,
            views: s.doc.views.clone(),
            effective: if s.doc.enabled {
                s.doc.rules
            } else {
                LauncherRules::default()
            },
            undo: head.as_ref().and_then(|r| r.undo),
            origin: if head.is_some() {
                "journal"
            } else if s.revision == STOCK {
                "stock"
            } else {
                "personalFile"
            },
        })
    }
    pub fn inspect(&self) -> Result<State> {
        self.state(&self.observed()?)
    }
    fn contains(&self, from: &Observed, id: Uuid) -> Result<bool> {
        let mut s = from.clone();
        for _ in 0..256 {
            let Some(rec) = self.head(&s)? else {
                return Ok(false);
            };
            if rec.request.request_id == id {
                return Ok(true);
            }
            s = rec.before;
        }
        Err(Error::HistoryLimit)
    }
    pub fn status(&self, id: Uuid) -> Result<Value> {
        let s = self.observed()?;
        let status = if self.record(id)?.is_none() {
            "unknown"
        } else if self.contains(&s, id)? {
            "committed"
        } else {
            "notInActiveHistory"
        };
        Ok(json!({"requestId":id,"status":status,"storedRevision":s.revision}))
    }
    pub fn why(&self, target: Edit) -> Result<Value> {
        let s = self.observed()?;
        let view = self.state(&s)?;
        let mut last = view.undo;
        for _ in 0..256 {
            let Some(id) = last else {
                return Ok(
                    json!({"target":target.target(),"effective":view.effective.edit_at(target),"declared":view.declared.edit_at(target),"enabled":view.enabled,"origin":if view.origin=="personalFile"||view.declared.edit_at(target)!=LauncherRules::default().edit_at(target){"personalFile"}else{"stock"},"revision":view.revision}),
                );
            };
            let rec = self.record(id)?.ok_or(Error::InvalidDocument)?;
            if let Change::Edit { edit } = rec.request.change {
                if edit.target() == target.target() {
                    return Ok(
                        json!({"target":target.target(),"effective":view.effective.edit_at(target),"declared":view.declared.edit_at(target),"enabled":view.enabled,"origin":rec.author,"requestId":id,"revision":view.revision}),
                    );
                }
            }
            last = rec.previous_undo;
        }
        Err(Error::HistoryLimit)
    }
    pub fn why_view(&self, id: &str) -> Result<Value> {
        if !views::valid_id(id) {
            return Err(Error::InvalidView);
        }
        let observed = self.observed()?;
        let state = self.state(&observed)?;
        let recipe = state.views.iter().find(|v| v.id == id);
        let mut result = json!({"target":format!("personal.view.{id}"),"declared":recipe,"active":state.enabled && recipe.is_some(),"origin":if recipe.is_some(){"personalFile"}else{"stock"},"revision":state.revision});
        let mut last = state.undo;
        for _ in 0..256 {
            let Some(next) = last else {
                return Ok(result);
            };
            let rec = self.record(next)?.ok_or(Error::InvalidDocument)?;
            if match &rec.request.change {
                Change::PutView { view } => view.id == id,
                Change::RemoveView { id: removed } => removed == id,
                _ => false,
            } {
                result["origin"] = json!(rec.author);
                result["requestId"] = json!(next);
                return Ok(result);
            }
            last = rec.previous_undo;
        }
        Err(Error::HistoryLimit)
    }
    pub fn preview(&self, change: Change) -> Result<State> {
        let current = self.observed()?;
        let (doc, _) = self.proposed(&current, &change, None)?;
        let mut state = self.state(&current)?;
        state.enabled = doc.enabled;
        state.declared = doc.rules;
        state.views = doc.views;
        state.effective = if doc.enabled {
            doc.rules
        } else {
            LauncherRules::default()
        };
        Ok(state)
    }
    fn proposed(
        &self,
        current: &Observed,
        change: &Change,
        id: Option<Uuid>,
    ) -> Result<(Document, Option<Uuid>)> {
        let tip = self.head(current)?.and_then(|r| r.undo);
        let mut doc = current.doc.clone();
        let undo = match change {
            Change::Edit { edit } => {
                doc.rules = doc.rules.with(*edit);
                id
            }
            Change::PutView { view } => {
                view.validate()?;
                if let Some(existing) = doc.views.iter_mut().find(|v| v.id == view.id) {
                    *existing = view.clone();
                } else {
                    if doc.views.len() >= views::MAX_VIEWS {
                        return Err(Error::ViewLimit);
                    }
                    doc.views.push(view.clone());
                }
                doc.views.sort_by(|a, b| a.id.cmp(&b.id));
                id
            }
            Change::RemoveView { id: name } => {
                if !views::valid_id(name) {
                    return Err(Error::InvalidView);
                }
                let at = doc
                    .views
                    .iter()
                    .position(|v| v.id == *name)
                    .ok_or(Error::ViewNotFound)?;
                doc.views.remove(at);
                id
            }
            Change::Enable => {
                doc.enabled = true;
                id
            }
            Change::Disable => {
                doc.enabled = false;
                id
            }
            Change::Undo { of } => {
                if tip != Some(*of) {
                    return Err(Error::NothingToUndo);
                }
                let r = self.record(*of)?.ok_or(Error::InvalidDocument)?;
                if r.after.rules != doc.rules
                    || r.after.enabled != doc.enabled
                    || r.after.views != doc.views
                {
                    return Err(Error::StaleRevision);
                }
                doc.rules = r.before.doc.rules;
                doc.views = r.before.doc.views;
                doc.enabled = r.before.doc.enabled;
                r.previous_undo
            }
        };
        doc.generation = id;
        doc.journal = None;
        Ok((doc, undo))
    }
    pub fn apply(&self, request: Request, author: Author) -> Result<Applied> {
        if request.workspace != self.workspace {
            return Err(Error::WrongWorkspace);
        }
        if request.request_id.get_version_num() != 4 || !valid_revision(&request.expected_revision)
        {
            return Err(Error::InvalidRequest);
        }
        match &request.change {
            Change::PutView { view } => view.validate()?,
            Change::RemoveView { id } if !views::valid_id(id) => return Err(Error::InvalidView),
            _ => {}
        }
        let journal = self
            .root
            .path(JOURNAL_DIR, true)?
            .ok_or(Error::StorageUnavailable)?;
        journal.lock()?;
        let before = self.observed()?;
        let prior = self.record(request.request_id)?;
        if let Some(r) = &prior {
            if r.author != author || r.request != request {
                return Err(Error::RequestConflict);
            }
            if self.contains(&before, request.request_id)? {
                return Ok(Applied {
                    request_id: request.request_id,
                    applied_revision: hash(&bytes(&r.after)?),
                    replayed: true,
                    state: self.state(&before)?,
                });
            }
        }
        if request.expected_revision != before.revision {
            return Err(Error::StaleRevision);
        }
        let (after, undo) = self.proposed(&before, &request.change, Some(request.request_id))?;
        let previous_undo = self.head(&before)?.and_then(|r| r.undo);
        let mut record = Record {
            schema_version: 1,
            request: request.clone(),
            author,
            before: before.clone(),
            after,
            previous_undo,
            undo,
        };
        record.after.journal = Some(seal(&record)?);
        if let Some(r) = &prior {
            if r.before != before || r.after != record.after {
                return Err(Error::RequestConflict);
            }
        } else {
            journal.put(&record_name(request.request_id), &bytes(&record)?, true)?;
        }
        let after = record.after;
        let active = self
            .root
            .path(RULES_DIR, true)?
            .ok_or(Error::StorageUnavailable)?;
        if self.observed()? != before
            || self
                .root
                .path(JOURNAL_DIR, false)?
                .ok_or(Error::UnsafePath)?
                .identity()?
                != journal.identity()?
        {
            return Err(Error::StaleRevision);
        }
        let body = bytes(&after)?;
        active.put(ACTIVE, &body, false)?;
        let reread = self.observed()?;
        if reread.revision != hash(&body) || reread.doc != after {
            return Err(Error::StaleRevision);
        }
        Ok(Applied {
            request_id: request.request_id,
            applied_revision: reread.revision.clone(),
            replayed: false,
            state: self.state(&reread)?,
        })
    }
}

pub fn catalog() -> Value {
    json!({"schemaVersion":1,"targets":[{"id":"launcher.home.mode","values":["standard","minimal"],"default":"standard","when":"launcher.home","effect":"Minimal hides the subject tabs and shows just scripts and setup on the empty launcher. Typing still searches everything."},{"id":"launcher.header.pinned-label.textRole","values":["muted","primary"],"when":"window.pinned"},{"id":"launcher.header.pinned-label.weight","values":["regular","semibold","semiboldWhileDictating"],"when":"window.pinned"},{"id":"launcher.subject.selection.firstAction","values":[null,"ask","note","show","handoff"]},{"id":"launcher.subject.clipboard.firstAction","values":[null,"ask","note","show","open"]}],"operations":["describe","inspect","preview","apply","why","status"],"commit":"apply requires workspace, expectedRevision, a fresh v4 requestId, and change; repeat only the identical request","undo":"apply change {op:undo,of:<inspect.undo>}","limits":{"jsonBytes":65536,"historyLookup":256},"capabilities":[],"views":{"schema":views::schema(),"maxViews":views::MAX_VIEWS,"put":"apply change {op:putView,view}","remove":"apply change {op:removeView,id}","why":"personal.view.<id>","activation":"Saving or previewing does not read a source or grant its action"},"safeMode":"SCRIPT_KIT_PERSONAL_SAFE_MODE=1"})
}
