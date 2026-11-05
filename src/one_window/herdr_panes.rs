//! A person opening a declared view may read one deliberately paired space. The only
//! executable text is the read-only module embedded at compile time, never workspace JS.
//! Its provider-facing worker runs under a per-request deny-default macOS sandbox.
use super::personal_views::SourceSnapshot;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sk_one_window::personal_list::Item;
use sk_personal::views::{PaneDetail, PaneFilter, PaneSort};
use std::path::Path;

const PROGRAM: &str = concat!(
    include_str!("../../scripts/herdr-pane-read.mjs"),
    "\nconst [root, expectedWorkspace, query, pairId] = JSON.parse(await Bun.stdin.text());\nconsole.log(JSON.stringify(listPaired(root, expectedWorkspace, query, pairId)));\n"
);
const UNVERIFIED: &str = "Kit couldn't verify the shared pane list";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Ready {
    state: String,
    capability: String,
    source: String,
    space: String,
    workspace: String,
    pair_id: String,
    rows: Vec<Pane>,
    more: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pane {
    pane: String,
    tab: String,
    terminal: String,
    pid: u64,
    created_ms: u64,
    title: String,
    status: Option<Status>,
    focused: bool,
    zoomed: bool,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Status {
    Blocked,
    Done,
    Working,
    Idle,
}
fn token(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn local_id(s: &str) -> bool {
    (1..=64).contains(&s.len())
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}
fn project(
    bytes: &[u8],
    workspace: &str,
    expected: Option<&str>,
    filter: PaneFilter,
    detail: PaneDetail,
    limit: u8,
) -> Result<SourceSnapshot, &'static str> {
    let reply: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| UNVERIFIED)?;
    match reply["state"].as_str() {
        Some("notPaired") => return Err("No authorized Herdr pane source is connected"),
        Some("unavailable") => return Err("The shared Herdr space is unavailable"),
        Some("renewalRequired") => {
            return Err("Unpair the old pane share and make a new offer for this Kit workspace.")
        }
        Some("noFreshReply") => {
            return Err("Herdr didn't return a fresh pane list. Reopen the view to check again.")
        }
        Some("supersededRequest") => {
            return Err("A newer read replaced this one. Reopen the pane list.")
        }
        Some("pairingBusy" | "pairingChanged" | "workspaceChanged") => {
            return Err("The pane pairing changed. Reopen the view.")
        }
        Some("safeMode" | "agentContextDenied") => {
            return Err("Pane sharing is off in this context")
        }
        Some("ready") => (),
        _ => return Err(UNVERIFIED),
    }
    let r: Ready = serde_json::from_value(reply).map_err(|_| UNVERIFIED)?;
    if r.state != "ready"
        || r.workspace != workspace
        || r.capability != "herdr.panes.metadata"
        || !token(&r.pair_id)
        || !token(&r.source)
        || !local_id(&r.space)
        || expected.is_some_and(|id| id != r.pair_id)
        || r.rows.len() > usize::from(limit)
        || (r.more && r.rows.len() != usize::from(limit))
    {
        return Err(UNVERIFIED);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut items = Vec::with_capacity(r.rows.len());
    for row in r.rows {
        let digits = row.terminal.strip_prefix('t').ok_or(UNVERIFIED)?;
        if !(1..=12).contains(&digits.len())
            || digits.starts_with('0')
            || !digits.bytes().all(|c| c.is_ascii_digit())
            || row.title != format!("Pane {digits}")
            || !local_id(&row.pane)
            || !local_id(&row.tab)
            || !seen.insert(row.pane.clone())
            || row.pid == 0
            || row.created_ms == 0
            || (filter == PaneFilter::NeedsAttention
                && !matches!(row.status, Some(Status::Blocked | Status::Done)))
        {
            return Err(UNVERIFIED);
        }
        let detail = match detail {
            PaneDetail::None => None,
            PaneDetail::Space => Some(r.space.clone()),
            PaneDetail::Status => {
                let mut parts = vec![match row.status {
                    Some(Status::Blocked) => "Needs attention",
                    Some(Status::Done) => "Done",
                    Some(Status::Working) => "Working",
                    Some(Status::Idle) => "Idle",
                    None => "No reported status",
                }];
                if row.focused {
                    parts.push("Focused");
                }
                if row.zoomed {
                    parts.push("Zoomed");
                }
                Some(parts.join(" · "))
            }
        };
        // UI element IDs are observable separately from redacted row contents. Preserve the
        // exact target generation without handing a terminal address or PID to a UI inspector.
        let identity = format!(
            "{}:{}:{}:{}:{}:{}",
            r.pair_id, row.tab, row.pane, row.terminal, row.pid, row.created_ms
        );
        items.push(Item {
            id: format!("h-{:x}", Sha256::digest(identity.as_bytes())),
            title: row.title,
            detail,
            icon: "terminal",
        });
    }
    let empty = items
        .is_empty()
        .then_some(if filter == PaneFilter::NeedsAttention {
            "No panes need attention"
        } else {
            "No live panes in the shared space"
        });
    Ok(SourceSnapshot {
        items,
        pair_id: Some(r.pair_id),
        more: r.more,
        message: empty,
    })
}

/// Called on the background executor, never on the typing or paint path.
pub(super) fn read(
    root: &Path,
    workspace: &str,
    expected_pair: Option<&str>,
    filter: PaneFilter,
    sort: PaneSort,
    detail: PaneDetail,
    limit: u8,
) -> Result<SourceSnapshot, &'static str> {
    use crate::runtime_policy::{check, ExternalEffect};
    if !cfg!(target_os = "macos") {
        return Err("This pane source requires macOS");
    }
    if sk_personal::api::safe_mode()
        || [
            "SCRIPT_KIT_AGENT_CREDENTIAL",
            "SCRIPT_KIT_MCP_TOKEN",
            "SCRIPT_KIT_MCP_ENDPOINT",
            "SCRIPT_KIT_MCP_SERVER_JSON",
        ]
        .iter()
        .any(|k| std::env::var_os(k).is_some())
    {
        return Err("Pane sharing is off in this context");
    }
    // A private evaluator cannot cross into the user's Herdr, even if a fixture mentions it.
    // Leave the passive not-paired state observable without invoking Bun or probing the OS.
    let pair_dir = root.join("db/herdr-panes-v1");
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy
            .require_owned_path(&pair_dir)
            .map_err(|_| UNVERIFIED)?;
    }
    if std::fs::symlink_metadata(root.join("db")).is_ok_and(|s| !s.is_dir()) {
        return Err(UNVERIFIED);
    }
    if matches!(std::fs::symlink_metadata(pair_dir),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
    {
        return Err("No authorized Herdr pane source is connected");
    }
    if std::env::var_os("SCRIPT_KIT_NONINTERACTIVE").is_some_and(|s| s != "0")
        || std::env::var_os("CI").is_some_and(|s| s != "0" && s != "false")
        || check(ExternalEffect::ExternalStorage).is_err()
        || check(ExternalEffect::Process).is_err()
    {
        return Err("Pane sharing is off in this context");
    }
    let bun = crate::setup::resolve_bun().ok_or("The pane reader needs a direct Bun executable")?;
    let input = serde_json::to_vec(&(
        root.to_str().ok_or(UNVERIFIED)?,
        workspace,
        serde_json::json!({"filter":filter,"sort":sort,"limit":limit}),
        expected_pair,
    ))
    .map_err(|_| UNVERIFIED)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| UNVERIFIED)?;
    let out = rt
        .block_on(crate::mcp_protocol::bridge::process::output(
            &bun,
            &[
                "--config=/dev/null",
                "--no-env-file",
                "--no-install",
                "--no-macros",
                "--log-level=error",
                "-e",
                PROGRAM,
            ],
            Path::new("/"),
            &[
                ("HOME", "/dev/null".into()),
                ("PATH", "/usr/bin:/bin".into()),
                ("BUN_RUNTIME_TRANSPILER_CACHE_PATH", "0".into()),
            ],
            input,
            5,
            32768,
        ))
        .map_err(|_| "The pane reader is unavailable in this session")?;
    project(&out, workspace, expected_pair, filter, detail, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_ready_empty_space_is_not_an_unavailable_or_replaced_pair() {
        let pair = "a".repeat(32);
        let reply = json!({"state":"ready","capability":"herdr.panes.metadata","source":"b".repeat(32),"space":"fixture","workspace":"ws","pairId":pair,"rows":[],"more":false});
        let read = |value: &serde_json::Value, expected: &str| {
            project(
                &serde_json::to_vec(value).unwrap(),
                "ws",
                Some(expected),
                PaneFilter::NeedsAttention,
                PaneDetail::Status,
                2,
            )
        };
        let good = read(&reply, &pair).unwrap();
        assert!(good.items.is_empty());
        assert_eq!(good.message, Some("No panes need attention"));
        assert!(read(&reply, &"c".repeat(32)).is_err());
        for state in [
            "notPaired",
            "unavailable",
            "noFreshReply",
            "pairingChanged",
            "unverifiedSource",
        ] {
            assert!(read(&json!({"state":state}), &pair).is_err());
        }
    }
    #[test]
    fn a_reused_terminal_address_has_a_new_row_identity_and_no_private_title() {
        let mut reply = json!({"state":"ready","capability":"herdr.panes.metadata","source":"a".repeat(32),"space":"sp1","workspace":"ws","pairId":"b".repeat(32),"more":false,"rows":[{"pane":"p1","tab":"tb1","terminal":"t1","pid":2,"createdMs":3,"title":"Pane 1","status":"blocked","focused":true,"zoomed":false}]});
        let read = |v: &serde_json::Value| {
            project(
                &serde_json::to_vec(v).unwrap(),
                "ws",
                None,
                PaneFilter::NeedsAttention,
                PaneDetail::Status,
                1,
            )
        };
        let first = read(&reply).unwrap();
        assert_eq!(
            first.items[0].detail.as_deref(),
            Some("Needs attention · Focused")
        );
        reply["rows"][0]["createdMs"] = 4.into();
        assert_ne!(read(&reply).unwrap().items[0].id, first.items[0].id);
        reply["rows"][0]["title"] = "private screen text".into();
        assert!(read(&reply).is_err());
    }
}
