//! Flow catalog: invokes `md roster --json` and caches results per cwd.
//!
//! mdflow owns discovery — the app never re-implements project-root walking
//! or frontmatter parsing (protocol §1). The cache is invalidated by cwd
//! change or age; refreshes run on background threads and land via the
//! registry-style notify hook so renderers stay passive.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use super::model::{FlowDescriptor, RosterSnapshot, FLOW_UX_PROTOCOL_VERSION};

/// Roster entries older than this refetch on next access.
const ROSTER_TTL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterStatus {
    Ready,
    Loading,
    /// mdflow present but pre-protocol; only terminal `--json` runs work.
    Legacy,
    Error,
}

impl RosterStatus {
    pub fn automation_label(self) -> &'static str {
        match self {
            RosterStatus::Ready => "ready",
            RosterStatus::Loading => "loading",
            RosterStatus::Legacy => "legacy",
            RosterStatus::Error => "error",
        }
    }
}

#[derive(Clone)]
pub struct RosterEntry {
    pub status: RosterStatus,
    pub flows: Arc<Vec<FlowDescriptor>>,
    pub warnings: Vec<String>,
    pub fetched_at: Instant,
}

impl RosterEntry {
    fn empty(status: RosterStatus) -> Self {
        Self {
            status,
            flows: Arc::new(Vec::new()),
            warnings: Vec::new(),
            fetched_at: Instant::now(),
        }
    }

    pub fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() > ROSTER_TTL
    }
}

static CATALOG: Mutex<Option<Arc<FlowCatalog>>> = Mutex::new(None);

pub fn flow_catalog() -> Arc<FlowCatalog> {
    let mut guard = CATALOG.lock();
    guard
        .get_or_insert_with(|| Arc::new(FlowCatalog::default()))
        .clone()
}

/// Resolve the mdflow binary, preferring `mdflow` over `md` (`md` may shadow
/// other tools on some systems; the long name is unambiguous).
pub fn mdflow_binary() -> Option<&'static str> {
    static RESOLVED: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();
    *RESOLVED.get_or_init(|| {
        if which::which("mdflow").is_ok() {
            Some("mdflow")
        } else if which::which("md").is_ok() {
            Some("md")
        } else {
            None
        }
    })
}

#[derive(Default)]
pub struct FlowCatalog {
    entries: Mutex<HashMap<String, RosterEntry>>,
    notify: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
}

impl FlowCatalog {
    pub fn set_notify_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.notify.lock() = Some(Box::new(hook));
    }

    fn notify(&self) {
        if let Some(hook) = self.notify.lock().as_ref() {
            hook();
        }
    }

    /// Current entry for a cwd without blocking; kicks off a background
    /// refresh when missing or stale. Renderers call this every frame.
    pub fn roster_for(self: &Arc<Self>, cwd: &str) -> RosterEntry {
        let needs_refresh = {
            let entries = self.entries.lock();
            match entries.get(cwd) {
                Some(entry) => entry.is_stale() && entry.status != RosterStatus::Loading,
                None => true,
            }
        };
        if needs_refresh {
            self.spawn_refresh(cwd);
        }
        self.entries
            .lock()
            .get(cwd)
            .cloned()
            .unwrap_or_else(|| RosterEntry::empty(RosterStatus::Loading))
    }

    /// Force refresh (cwd chip changed, manual reload action).
    pub fn refresh(self: &Arc<Self>, cwd: &str) {
        self.spawn_refresh(cwd);
    }

    fn spawn_refresh(self: &Arc<Self>, cwd: &str) {
        {
            let mut entries = self.entries.lock();
            let placeholder = entries
                .entry(cwd.to_string())
                .or_insert_with(|| RosterEntry::empty(RosterStatus::Loading));
            if placeholder.status == RosterStatus::Loading && !placeholder.flows.is_empty() {
                return; // refresh already in flight with previous data showing
            }
            placeholder.status = RosterStatus::Loading;
        }
        let catalog = Arc::clone(self);
        let cwd = cwd.to_string();
        std::thread::Builder::new()
            .name("flow-roster-fetch".into())
            .spawn(move || {
                let entry = fetch_roster_blocking(&cwd);
                catalog.entries.lock().insert(cwd, entry);
                catalog.notify();
            })
            .ok();
    }

    #[cfg(test)]
    fn insert_for_test(&self, cwd: &str, entry: RosterEntry) {
        self.entries.lock().insert(cwd.to_string(), entry);
    }
}

/// Blocking roster fetch — call only from background threads.
pub fn fetch_roster_blocking(cwd: &str) -> RosterEntry {
    let Some(binary) = mdflow_binary() else {
        let mut entry = RosterEntry::empty(RosterStatus::Error);
        entry
            .warnings
            .push("mdflow CLI not found on PATH (npm i -g mdflow)".to_string());
        return entry;
    };
    if !Path::new(cwd).is_dir() {
        let mut entry = RosterEntry::empty(RosterStatus::Error);
        entry.warnings.push(format!("cwd does not exist: {cwd}"));
        return entry;
    }
    let output = Command::new(binary)
        .arg("roster")
        .arg("--json")
        .current_dir(cwd)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            let mut entry = RosterEntry::empty(RosterStatus::Error);
            entry
                .warnings
                .push(format!("failed to run {binary}: {err}"));
            return entry;
        }
    };
    if !output.status.success() {
        // Pre-protocol mdflow has no `roster` subcommand → legacy mode
        // (capability handshake, protocol §3).
        return RosterEntry::empty(RosterStatus::Legacy);
    }
    parse_roster_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_roster_output(stdout: &str) -> RosterEntry {
    match serde_json::from_str::<RosterSnapshot>(stdout) {
        Ok(snapshot) if snapshot.protocol_version == FLOW_UX_PROTOCOL_VERSION => RosterEntry {
            status: RosterStatus::Ready,
            flows: Arc::new(snapshot.flows),
            warnings: snapshot.warnings,
            fetched_at: Instant::now(),
        },
        Ok(_) => RosterEntry::empty(RosterStatus::Legacy),
        Err(err) => {
            let mut entry = RosterEntry::empty(RosterStatus::Error);
            entry.warnings.push(format!("roster parse error: {err}"));
            entry
        }
    }
}

/// Simple case-insensitive subsequence filter for roster rows, ranked:
/// name prefix > name contains > description contains. Frecency integration
/// can replace this without touching renderers.
pub fn filter_flows<'a>(flows: &'a [FlowDescriptor], query: &str) -> Vec<&'a FlowDescriptor> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return flows.iter().collect();
    }
    let mut ranked: Vec<(u8, &FlowDescriptor)> = flows
        .iter()
        .filter_map(|flow| {
            let name = flow.name.to_lowercase();
            let description = flow
                .description
                .as_deref()
                .unwrap_or_default()
                .to_lowercase();
            if name.starts_with(&query) {
                Some((0u8, flow))
            } else if name.contains(&query) {
                Some((1u8, flow))
            } else if description.contains(&query) {
                Some((2u8, flow))
            } else {
                None
            }
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.cmp(&b.1.name)));
    ranked.into_iter().map(|(_, flow)| flow).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flows::model::FlowSource;

    fn descriptor(name: &str, description: Option<&str>) -> FlowDescriptor {
        serde_json::from_value(serde_json::json!({
            "id": format!("project:{name}"),
            "path": format!("/tmp/p/flows/{name}.md"),
            "source": "project",
            "name": name,
            "description": description,
            "engine": "pi",
            "inputs": [],
            "isWorkflow": false,
            "interactive": false,
            "mtimeMs": 0
        }))
        .expect("descriptor builds")
    }

    #[test]
    fn parse_roster_output_accepts_protocol_v1() {
        let entry = parse_roster_output(
            r#"{"protocolVersion":1,"cwd":"/p","projectRoot":"/p","flows":[],"warnings":["w"]}"#,
        );
        assert_eq!(entry.status, RosterStatus::Ready);
        assert_eq!(entry.warnings, vec!["w".to_string()]);
    }

    #[test]
    fn parse_roster_output_flags_future_protocol_as_legacy() {
        let entry =
            parse_roster_output(r#"{"protocolVersion":2,"cwd":"/p","flows":[],"warnings":[]}"#);
        assert_eq!(entry.status, RosterStatus::Legacy);
    }

    #[test]
    fn parse_roster_output_reports_garbage_as_error() {
        let entry = parse_roster_output("not json");
        assert_eq!(entry.status, RosterStatus::Error);
        assert!(!entry.warnings.is_empty());
    }

    #[test]
    fn filter_ranks_prefix_over_contains_over_description() {
        let flows = vec![
            descriptor("deploy", Some("ship it")),
            descriptor("redeploy", None),
            descriptor("notes", Some("deploy notes helper")),
        ];
        let hits = filter_flows(&flows, "dep");
        let names: Vec<&str> = hits.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["deploy", "redeploy", "notes"]);
        assert_eq!(flows[0].source, FlowSource::Project);
    }

    #[test]
    fn empty_query_returns_all_in_roster_order() {
        let flows = vec![descriptor("b", None), descriptor("a", None)];
        let hits = filter_flows(&flows, "  ");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].name, "b", "roster order preserved for empty query");
    }

    #[test]
    fn catalog_returns_cached_entry_without_blocking() {
        let catalog = FlowCatalog::default();
        let mut entry = RosterEntry::empty(RosterStatus::Ready);
        entry.flows = Arc::new(vec![descriptor("cached", None)]);
        catalog.insert_for_test("/tmp/cwd", entry);
        let got = catalog.entries.lock().get("/tmp/cwd").cloned().unwrap();
        assert_eq!(got.flows.len(), 1);
    }
}
