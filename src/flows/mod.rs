//! Flow-first launcher domain (mdflow front end).
//!
//! mdflow (the standalone CLI) owns flow discovery, resolution, and
//! execution; this module is the app-side client of the frozen contract in
//! `docs/ai/flow-ux-protocol.md`:
//!
//! - [`catalog`] — `md roster --json` per-cwd cache.
//! - [`explain_cache`] — free `md explain --json` previews (Lens).
//! - [`runner`] — `md <flow> --events` process-group spawn + NDJSON reader.
//! - [`run_registry`] — the single source of truth for run state; every
//!   Flow UX variation and the Flow Manager render from it.
//! - [`automation`] — the `flowUx` getState payload for devtools receipts.
//!
//! Design rule: variations are thin renderers over the registry/catalog.
//! Adding or deleting a variation must not touch the run lifecycle.

pub mod automation;
pub mod catalog;
pub mod explain_cache;
pub mod manager_window;
pub mod model;
pub mod run_registry;
pub mod runner;
