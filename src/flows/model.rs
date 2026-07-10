//! Domain model for the flow-first launcher surfaces.
//!
//! Mirrors the frozen contract in `docs/ai/flow-ux-protocol.md` (protocol
//! version 1). The mdflow CLI owns discovery/execution; these types are the
//! app-side deserialization targets plus the run lifecycle state machine.
//!
//! Phase and engagement are deliberately independent axes: `Esc` on an
//! engaged run backgrounds it (engagement change) and must never cancel it
//! (phase change). Cancel is an explicit action on a selected run only.

use serde::Deserialize;

pub const FLOW_UX_PROTOCOL_VERSION: u64 = 1;

/// Bounds for per-run output retention in the registry.
pub const OUTPUT_TAIL_MAX_BYTES: usize = 64 * 1024;
pub const OUTPUT_TAIL_MAX_LINES: usize = 500;

// ---------------------------------------------------------------------------
// Roster (`md roster --json`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterSnapshot {
    pub protocol_version: u64,
    pub cwd: String,
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub flows: Vec<FlowDescriptor>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowDescriptor {
    /// Stable id: `<source>:<slug>`.
    pub id: String,
    pub path: String,
    pub source: FlowSource,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub engine: String,
    #[serde(default)]
    pub engine_source: Option<String>,
    #[serde(default)]
    pub inputs: Vec<FlowInput>,
    #[serde(default)]
    pub is_workflow: bool,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub mtime_ms: u64,
    /// Human origin of the flow definition — the actual package or directory
    /// it came from (e.g. `@johnlindquist/flows`, `repo flows/`), not just the
    /// coarse source class. Rows always show this so the user can discern
    /// where a flow lives.
    #[serde(default)]
    pub origin: Option<String>,
    /// Bun-linked wrapper command (e.g. `flow-gmail`) when one exists on
    /// PATH. Conversations launch the wrapper so package semantics (local
    /// `.flows` overrides, package cwd) are preserved exactly as in a shell.
    #[serde(default)]
    pub wrapper_command: Option<String>,
}

impl FlowDescriptor {
    pub fn display_title(&self) -> &str {
        self.description.as_deref().unwrap_or(&self.name)
    }

    /// Friendly agent-identity name for desk rows: `flow-gmail` → `Gmail`,
    /// `flow-npm` → `NPM`. Filenames make bad identities; rows lead with this.
    pub fn friendly_name(&self) -> String {
        let base = self.name.strip_prefix("flow-").unwrap_or(&self.name);
        if base.is_empty() {
            return self.name.clone();
        }
        let words: Vec<String> = base
            .split(['-', '_'])
            .filter(|w| !w.is_empty())
            .map(|w| {
                if w.len() <= 3 {
                    w.to_uppercase()
                } else {
                    let mut chars = w.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                }
            })
            .collect();
        if words.is_empty() {
            self.name.clone()
        } else {
            words.join(" ")
        }
    }

    /// Origin string for rows/detail: the concrete package/path when known,
    /// otherwise the coarse source class.
    pub fn origin_label(&self) -> &str {
        self.origin
            .as_deref()
            .unwrap_or_else(|| self.source.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowSource {
    Project,
    Global,
    Registry,
    /// Installed flows package (e.g. `@johnlindquist/flows`), discovered by
    /// the app-side package scanner rather than `md roster`.
    Package,
}

impl FlowSource {
    pub fn label(self) -> &'static str {
        match self {
            FlowSource::Project => "Project",
            FlowSource::Global => "Global",
            FlowSource::Registry => "Registry",
            FlowSource::Package => "Package",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: FlowInputType,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowInputType {
    Text,
    Select,
    Number,
    Confirm,
    Password,
}

impl FlowInputType {
    /// Password values must never surface in state, logs, or screenshots.
    pub fn is_redacted(self) -> bool {
        matches!(self, FlowInputType::Password)
    }
}

// ---------------------------------------------------------------------------
// Explain (`md explain <flow> --json`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainInfo {
    pub protocol_version: u64,
    pub flow_id: String,
    pub path: String,
    pub engine: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: String,
    pub prompt: String,
    #[serde(default)]
    pub prompt_tokens_estimate: u64,
    #[serde(default)]
    pub inputs: Vec<FlowInput>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub config_fingerprint: Option<String>,
}

// ---------------------------------------------------------------------------
// Run events (`md <flow> --events` NDJSON)
// ---------------------------------------------------------------------------

/// One NDJSON line from a `--events` run. Unknown events are preserved so a
/// newer mdflow does not break an older app.
#[derive(Debug, Clone)]
pub struct RunEventEnvelope {
    pub seq: u64,
    pub run_id: String,
    pub ts: u64,
    pub event: RunEvent,
}

#[derive(Debug, Clone)]
pub enum RunEvent {
    Protocol {
        mdflow_version: String,
    },
    RunStarted {
        flow_id: String,
        pid: Option<u32>,
    },
    OutputDelta {
        channel: OutputChannel,
        text: String,
    },
    StepStarted {
        step_id: String,
        needs: Vec<String>,
    },
    StepCompleted {
        step_id: String,
        exit_code: Option<i64>,
        cached: bool,
    },
    RunCompleted {
        exit_code: i64,
        duration_ms: u64,
    },
    RunError {
        exit_code: Option<i64>,
        message: String,
        duration_ms: Option<u64>,
    },
    RunCancelled {
        signal: String,
        duration_ms: Option<u64>,
    },
    Unknown {
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputChannel {
    Stdout,
    Stderr,
}

/// Parse one NDJSON line into an envelope. Returns `None` for lines that are
/// not valid protocol objects (callers count these as protocol violations).
pub fn parse_event_line(line: &str) -> Option<RunEventEnvelope> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let obj = value.as_object()?;
    let seq = obj.get("seq")?.as_u64()?;
    let run_id = obj.get("runId")?.as_str()?.to_string();
    let ts = obj.get("ts").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = obj.get("event")?.as_str()?;

    let str_field =
        |key: &str| -> Option<String> { obj.get(key).and_then(|v| v.as_str()).map(str::to_string) };

    let event = match name {
        "protocol" => RunEvent::Protocol {
            mdflow_version: str_field("mdflowVersion").unwrap_or_default(),
        },
        "run.started" => RunEvent::RunStarted {
            flow_id: str_field("flowId").unwrap_or_default(),
            pid: obj.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32),
        },
        "output.delta" => RunEvent::OutputDelta {
            channel: match obj.get("channel").and_then(|v| v.as_str()) {
                Some("stderr") => OutputChannel::Stderr,
                _ => OutputChannel::Stdout,
            },
            text: str_field("text").unwrap_or_default(),
        },
        "step.started" => RunEvent::StepStarted {
            step_id: str_field("stepId").unwrap_or_default(),
            needs: obj
                .get("needs")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        },
        "step.completed" => RunEvent::StepCompleted {
            step_id: str_field("stepId").unwrap_or_default(),
            exit_code: obj.get("exitCode").and_then(|v| v.as_i64()),
            cached: obj.get("cached").and_then(|v| v.as_bool()).unwrap_or(false),
        },
        "run.completed" => RunEvent::RunCompleted {
            exit_code: obj.get("exitCode").and_then(|v| v.as_i64()).unwrap_or(0),
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()).unwrap_or(0),
        },
        "run.error" => RunEvent::RunError {
            exit_code: obj.get("exitCode").and_then(|v| v.as_i64()),
            message: str_field("message").unwrap_or_default(),
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()),
        },
        "run.cancelled" => RunEvent::RunCancelled {
            signal: str_field("signal").unwrap_or_else(|| "SIGTERM".to_string()),
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()),
        },
        other => RunEvent::Unknown {
            name: other.to_string(),
        },
    };

    Some(RunEventEnvelope {
        seq,
        run_id,
        ts,
        event,
    })
}

// ---------------------------------------------------------------------------
// Run lifecycle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPhase {
    Starting,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl RunPhase {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            RunPhase::Succeeded | RunPhase::Failed | RunPhase::Cancelled
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            RunPhase::Starting => "Starting",
            RunPhase::Running => "Running",
            RunPhase::Succeeded => "Succeeded",
            RunPhase::Failed => "Failed",
            RunPhase::Cancelled => "Cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngagementMode {
    Inline,
    Background,
    ManagerFocused,
}

impl EngagementMode {
    pub fn label(self) -> &'static str {
        match self {
            EngagementMode::Inline => "Inline",
            EngagementMode::Background => "Background",
            EngagementMode::ManagerFocused => "ManagerFocused",
        }
    }
}

/// Which Flow UX variation launched a run (receipts + manager display).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowUxVariant {
    Flash,
    Dispatch,
    Lens,
    MissionControl,
}

impl FlowUxVariant {
    pub const ALL: [FlowUxVariant; 4] = [
        FlowUxVariant::Flash,
        FlowUxVariant::Dispatch,
        FlowUxVariant::Lens,
        FlowUxVariant::MissionControl,
    ];

    pub fn automation_id(self) -> &'static str {
        match self {
            FlowUxVariant::Flash => "flash",
            FlowUxVariant::Dispatch => "dispatch",
            FlowUxVariant::Lens => "lens",
            FlowUxVariant::MissionControl => "missionControl",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            FlowUxVariant::Flash => "Flow UX — Flash",
            FlowUxVariant::Dispatch => "Flow UX — Dispatch",
            FlowUxVariant::Lens => "Flow UX — Lens",
            FlowUxVariant::MissionControl => "Flow UX — Mission Control",
        }
    }

    pub fn next(self) -> Self {
        match self {
            FlowUxVariant::Flash => FlowUxVariant::Dispatch,
            FlowUxVariant::Dispatch => FlowUxVariant::Lens,
            FlowUxVariant::Lens => FlowUxVariant::MissionControl,
            FlowUxVariant::MissionControl => FlowUxVariant::Flash,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FlowUxVariant::Flash => FlowUxVariant::MissionControl,
            FlowUxVariant::Dispatch => FlowUxVariant::Flash,
            FlowUxVariant::Lens => FlowUxVariant::Dispatch,
            FlowUxVariant::MissionControl => FlowUxVariant::Lens,
        }
    }
}

/// Launch/latency receipts surfaced in automation state (milliseconds from
/// the launch keystroke).
#[derive(Debug, Clone, Copy, Default)]
pub struct RunTimings {
    pub launch_ack_ms: Option<u64>,
    pub spawn_ms: Option<u64>,
    pub first_output_ms: Option<u64>,
}

/// Bounded output tail: newest wins, capped by bytes and lines.
#[derive(Debug, Clone, Default)]
pub struct OutputTail {
    lines: std::collections::VecDeque<String>,
    bytes: usize,
    pending: String,
}

impl OutputTail {
    pub fn push_text(&mut self, text: &str) {
        self.pending.push_str(text);
        while let Some(idx) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=idx).collect();
            self.push_line(line.trim_end_matches('\n').to_string());
        }
        // An unterminated pending fragment still counts against the byte cap.
        if self.pending.len() > OUTPUT_TAIL_MAX_BYTES {
            let excess = self.pending.len() - OUTPUT_TAIL_MAX_BYTES;
            let mut cut = excess.min(self.pending.len());
            while cut < self.pending.len() && !self.pending.is_char_boundary(cut) {
                cut += 1;
            }
            self.pending.drain(..cut);
        }
    }

    fn push_line(&mut self, mut line: String) {
        // A single line larger than the byte cap must be truncated, not
        // dropped — the eviction loop below would otherwise pop the very
        // line that was just pushed and the output would silently vanish.
        if line.len() > OUTPUT_TAIL_MAX_BYTES {
            let mut cut = OUTPUT_TAIL_MAX_BYTES.saturating_sub(16);
            while cut > 0 && !line.is_char_boundary(cut) {
                cut -= 1;
            }
            line.truncate(cut);
            line.push_str("… [truncated]");
        }
        self.bytes += line.len();
        self.lines.push_back(line);
        while self.lines.len() > OUTPUT_TAIL_MAX_LINES
            || (self.bytes > OUTPUT_TAIL_MAX_BYTES && self.lines.len() > 1)
        {
            if let Some(dropped) = self.lines.pop_front() {
                self.bytes = self.bytes.saturating_sub(dropped.len());
            } else {
                break;
            }
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines
            .iter()
            .map(String::as_str)
            .chain(if self.pending.is_empty() {
                None
            } else {
                Some(self.pending.as_str())
            })
    }

    pub fn last_line(&self) -> Option<&str> {
        if !self.pending.is_empty() {
            Some(self.pending.as_str())
        } else {
            self.lines.back().map(String::as_str)
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len() + usize::from(!self.pending.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepState {
    pub completed: bool,
    pub exit_code: Option<i64>,
    pub cached: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_roster_snapshot() {
        let json = r#"{
            "protocolVersion": 1,
            "cwd": "/tmp/p",
            "projectRoot": "/tmp/p",
            "flows": [{
                "id": "project:review",
                "path": "/tmp/p/flows/review.md",
                "source": "project",
                "name": "review",
                "description": "Review changes",
                "engine": "pi",
                "engineSource": "default",
                "inputs": [{"name": "target", "type": "text", "message": null, "options": [], "default": null}],
                "isWorkflow": false,
                "interactive": false,
                "mtimeMs": 1752000000000
            }],
            "warnings": []
        }"#;
        let snapshot: RosterSnapshot = serde_json::from_str(json).expect("roster parses");
        assert_eq!(snapshot.protocol_version, FLOW_UX_PROTOCOL_VERSION);
        assert_eq!(snapshot.flows.len(), 1);
        let flow = &snapshot.flows[0];
        assert_eq!(flow.id, "project:review");
        assert_eq!(flow.source, FlowSource::Project);
        assert_eq!(flow.inputs[0].input_type, FlowInputType::Text);
    }

    #[test]
    fn parses_event_lines_in_order() {
        let lines = [
            r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            r#"{"protocolVersion":1,"seq":1,"runId":"r-1","ts":2,"event":"run.started","flowId":"project:x","path":"/p","engine":"pi","command":"pi","args":[],"cwd":"/p","pid":123}"#,
            r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":3,"event":"output.delta","channel":"stdout","text":"hello\n"}"#,
            r#"{"protocolVersion":1,"seq":3,"runId":"r-1","ts":4,"event":"run.completed","exitCode":0,"durationMs":10}"#,
        ];
        let envelopes: Vec<_> = lines
            .iter()
            .map(|l| parse_event_line(l).expect("line parses"))
            .collect();
        assert!(matches!(envelopes[0].event, RunEvent::Protocol { .. }));
        assert!(matches!(
            envelopes[1].event,
            RunEvent::RunStarted { pid: Some(123), .. }
        ));
        assert!(matches!(
            envelopes[3].event,
            RunEvent::RunCompleted { exit_code: 0, .. }
        ));
        let seqs: Vec<u64> = envelopes.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2, 3]);
    }

    #[test]
    fn unknown_events_are_preserved_not_dropped() {
        let line =
            r#"{"protocolVersion":1,"seq":5,"runId":"r-2","ts":9,"event":"future.thing","x":1}"#;
        let envelope = parse_event_line(line).expect("parses");
        assert!(matches!(envelope.event, RunEvent::Unknown { ref name } if name == "future.thing"));
    }

    #[test]
    fn output_tail_enforces_line_and_byte_bounds() {
        let mut tail = OutputTail::default();
        for i in 0..(OUTPUT_TAIL_MAX_LINES + 50) {
            tail.push_text(&format!("line {i}\n"));
        }
        assert_eq!(tail.line_count(), OUTPUT_TAIL_MAX_LINES);
        assert_eq!(
            tail.last_line(),
            Some(format!("line {}", OUTPUT_TAIL_MAX_LINES + 49).as_str())
        );

        let mut big = OutputTail::default();
        let chunk = "x".repeat(1024);
        for _ in 0..100 {
            big.push_text(&format!("{chunk}\n"));
        }
        let total: usize = big.lines().map(str::len).sum();
        assert!(total <= OUTPUT_TAIL_MAX_BYTES);
    }

    #[test]
    fn phase_terminality_and_engagement_are_independent() {
        assert!(!RunPhase::Starting.is_terminal());
        assert!(!RunPhase::Running.is_terminal());
        assert!(RunPhase::Succeeded.is_terminal());
        assert!(RunPhase::Failed.is_terminal());
        assert!(RunPhase::Cancelled.is_terminal());
        // Engagement carries no phase semantics; labels only.
        assert_eq!(EngagementMode::Background.label(), "Background");
    }

    #[test]
    fn variant_cycle_is_a_complete_ring() {
        let mut v = FlowUxVariant::Flash;
        let mut seen = Vec::new();
        for _ in 0..4 {
            seen.push(v);
            v = v.next();
        }
        assert_eq!(v, FlowUxVariant::Flash);
        assert_eq!(seen.len(), 4);
        for variant in FlowUxVariant::ALL {
            assert_eq!(variant.next().prev(), variant);
        }
    }

    /// A single line larger than the byte cap is truncated for display,
    /// never dropped (fusion-ultra 2026-07-09: the eviction loop used to pop
    /// the very line that was just pushed).
    #[test]
    fn giant_single_line_is_truncated_not_dropped() {
        let mut tail = OutputTail::default();
        let giant = format!("{}END", "x".repeat(OUTPUT_TAIL_MAX_BYTES * 2));
        tail.push_text(&format!("{giant}\n"));
        assert_eq!(tail.line_count(), 1);
        let line = tail.last_line().unwrap();
        assert!(
            line.ends_with("… [truncated]"),
            "got tail: {:?}",
            &line[line.len().saturating_sub(40)..]
        );
        assert!(line.len() <= OUTPUT_TAIL_MAX_BYTES);
    }
}
