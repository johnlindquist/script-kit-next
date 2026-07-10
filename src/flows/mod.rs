//! Flow-first launcher domain (mdflow front end).
//!
//! mdflow (the standalone CLI) owns flow discovery, resolution, and
//! execution; this module is the app-side client of the frozen contract in
//! `docs/ai/flow-ux-protocol.md`:
//!
//! - [`catalog`] — `md roster --json` per-cwd cache.
//! - [`package_source`] — installed `@johnlindquist/flows` corpus with true
//!   provenance and bun-linked wrapper commands.
//! - [`explain_cache`] — free `md explain --json` previews.
//! - [`runner`] — `md <flow> --events` run-once spawn + NDJSON reader.
//! - [`run_registry`] — the single source of truth for run-once state.
//! - [`session`] — conversational session metadata (Enter = converse; the
//!   live PTY entities live on `ScriptListApp`).
//! - [`router`] — Tab flow router: free text → best flow.
//! - [`automation`] — the `flowUx` getState payload for devtools receipts.
//!
//! Design rule: the desk is a thin renderer over the registry/catalog.
//! UI changes must not touch the run lifecycle.

pub mod automation;
pub mod catalog;
pub mod explain_cache;
pub mod manager_window;
pub mod model;
pub mod package_source;
pub mod router;
pub mod run_registry;
pub mod runner;
pub mod session;

/// THE cwd resolver for every flow surface (Flow UX variations, Flow
/// Manager, automation snapshots). Precedence:
/// 1. `SCRIPT_KIT_FLOW_UX_CWD` (probe/test seam),
/// 2. the caller's context cwd (spine chip) when it has one,
/// 3. the last cwd a flow was launched from,
/// 4. `$HOME`.
/// Surfaces that disagree about cwd show different flow lists for no visible
/// reason — never resolve cwd any other way.
pub fn resolve_flow_cwd(context_cwd: Option<String>) -> String {
    if let Ok(dir) = std::env::var("SCRIPT_KIT_FLOW_UX_CWD") {
        if !dir.is_empty() {
            return dir;
        }
    }
    if let Some(cwd) = context_cwd {
        if !cwd.is_empty() {
            return cwd;
        }
    }
    if let Some(remembered) = manager_window::last_flow_cwd() {
        return remembered;
    }
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}
