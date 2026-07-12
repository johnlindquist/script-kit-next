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

/// Cap for the append-only conversation-turn accumulator. Conversation
/// streaming needs a monotonic buffer (the bounded display tail front-evicts,
/// which breaks byte cursors — the 2026-07-11 audit P0); on overflow the
/// capture FREEZES (never evicts) so cursors stay valid, and the UI appends
/// a truncation caption.
pub const CONVERSATION_CAPTURE_MAX_BYTES: usize = 4 * 1024 * 1024;

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
        friendly_flow_name(&self.name)
    }

    /// Origin string for rows/detail: the concrete package/path when known,
    /// otherwise the coarse source class.
    pub fn origin_label(&self) -> &str {
        self.origin
            .as_deref()
            .unwrap_or_else(|| self.source.label())
    }
}

/// Friendly agent-identity form of a raw flow name (shared by descriptor
/// rows and registry-run rows): `flow-gmail` → `Gmail`, `flow-npm` → `NPM`.
pub fn friendly_flow_name(name: &str) -> String {
    let base = name.strip_prefix("flow-").unwrap_or(name);
    if base.is_empty() {
        return name.to_string();
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
        name.to_string()
    } else {
        words.join(" ")
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
#[derive(Debug, Clone, PartialEq)]
pub struct RunEventEnvelope {
    pub seq: u64,
    pub run_id: String,
    pub ts: u64,
    pub event: RunEvent,
}

#[derive(Debug, Clone, PartialEq)]
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
        duration_ms: Option<u64>,
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

/// Why one NDJSON line was rejected. The stream consumer fails CLOSED on any
/// of these — a malformed event must never be coerced into progress, and a
/// missing `run.completed.exitCode` is never success (protocol §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventParseError {
    NotJson,
    NotObject,
    /// `protocolVersion` missing or not [`FLOW_UX_PROTOCOL_VERSION`].
    ProtocolVersion(Option<u64>),
    /// A load-bearing field is missing or has the wrong type. Purely
    /// informational fields (`ts`, `durationMs`, `pid`, per-step exit codes)
    /// stay lenient — failing a healthy run over display-only data would be
    /// fail-closed theater.
    Field(&'static str),
}

impl std::fmt::Display for EventParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventParseError::NotJson => write!(f, "line is not valid JSON"),
            EventParseError::NotObject => write!(f, "line is not a JSON object"),
            EventParseError::ProtocolVersion(Some(version)) => {
                write!(f, "unsupported protocolVersion {version}")
            }
            EventParseError::ProtocolVersion(None) => write!(f, "missing protocolVersion"),
            EventParseError::Field(name) => write!(f, "missing or invalid field `{name}`"),
        }
    }
}

/// Parse one NDJSON line into an envelope, validating the frozen contract:
/// `protocolVersion` must match, envelope identity fields must be present,
/// and load-bearing payload fields must be present with the right type.
pub fn parse_event_line(line: &str) -> Result<RunEventEnvelope, EventParseError> {
    let value: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(|_| EventParseError::NotJson)?;
    let obj = value.as_object().ok_or(EventParseError::NotObject)?;
    match obj.get("protocolVersion").and_then(|v| v.as_u64()) {
        Some(FLOW_UX_PROTOCOL_VERSION) => {}
        other => return Err(EventParseError::ProtocolVersion(other)),
    }
    let seq = obj
        .get("seq")
        .and_then(|v| v.as_u64())
        .ok_or(EventParseError::Field("seq"))?;
    let run_id = obj
        .get("runId")
        .and_then(|v| v.as_str())
        .ok_or(EventParseError::Field("runId"))?
        .to_string();
    let ts = obj.get("ts").and_then(|v| v.as_u64()).unwrap_or(0);
    let name = obj
        .get("event")
        .and_then(|v| v.as_str())
        .ok_or(EventParseError::Field("event"))?;

    let req_str = |key: &'static str| -> Result<String, EventParseError> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or(EventParseError::Field(key))
    };

    let event = match name {
        "protocol" => RunEvent::Protocol {
            mdflow_version: req_str("mdflowVersion")?,
        },
        "run.started" => RunEvent::RunStarted {
            flow_id: req_str("flowId")?,
            pid: obj.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32),
        },
        "output.delta" => RunEvent::OutputDelta {
            channel: match obj.get("channel").and_then(|v| v.as_str()) {
                Some("stdout") => OutputChannel::Stdout,
                Some("stderr") => OutputChannel::Stderr,
                _ => return Err(EventParseError::Field("channel")),
            },
            text: req_str("text")?,
        },
        "step.started" => RunEvent::StepStarted {
            step_id: req_str("stepId")?,
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
            step_id: req_str("stepId")?,
            exit_code: obj.get("exitCode").and_then(|v| v.as_i64()),
            cached: obj.get("cached").and_then(|v| v.as_bool()).unwrap_or(false),
        },
        "run.completed" => RunEvent::RunCompleted {
            // A missing exit code is NEVER success — reject the event and
            // let the stream consumer fail the run closed.
            exit_code: obj
                .get("exitCode")
                .and_then(|v| v.as_i64())
                .ok_or(EventParseError::Field("exitCode"))?,
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()),
        },
        "run.error" => RunEvent::RunError {
            exit_code: obj.get("exitCode").and_then(|v| v.as_i64()),
            message: req_str("message")?,
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()),
        },
        "run.cancelled" => RunEvent::RunCancelled {
            signal: req_str("signal")?,
            duration_ms: obj.get("durationMs").and_then(|v| v.as_u64()),
        },
        other => RunEvent::Unknown {
            name: other.to_string(),
        },
    };

    Ok(RunEventEnvelope {
        seq,
        run_id,
        ts,
        event,
    })
}

/// One stream-level contract violation (protocol §3). Any violation fails
/// the run closed: a stream that lies about ordering or identity can no
/// longer be trusted to report the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamViolation {
    SequenceGap { expected: u64, got: u64 },
    RunIdChanged,
    Order(&'static str),
}

impl std::fmt::Display for StreamViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamViolation::SequenceGap { expected, got } => {
                write!(f, "sequence gap (expected {expected}, got {got})")
            }
            StreamViolation::RunIdChanged => write!(f, "runId changed mid-stream"),
            StreamViolation::Order(detail) => write!(f, "{detail}"),
        }
    }
}

/// Per-run protocol state machine (protocol §3): `protocol` first, gapless
/// `seq` from 0, a stable `runId`, at most one `run.started`, no output/step
/// events before `run.started`, exactly one terminal event, nothing after
/// it. Pre-start failures (`protocol` → `run.error`/`run.cancelled` with no
/// `run.started`) are legitimate.
#[derive(Debug, Default)]
pub struct EventStreamValidator {
    next_seq: u64,
    run_id: Option<String>,
    saw_protocol: bool,
    saw_started: bool,
    saw_terminal: bool,
}

impl EventStreamValidator {
    pub fn validate(&mut self, envelope: &RunEventEnvelope) -> Result<(), StreamViolation> {
        if self.saw_terminal {
            return Err(StreamViolation::Order("event after terminal event"));
        }
        if envelope.seq != self.next_seq {
            return Err(StreamViolation::SequenceGap {
                expected: self.next_seq,
                got: envelope.seq,
            });
        }
        if let Some(run_id) = &self.run_id {
            if *run_id != envelope.run_id {
                return Err(StreamViolation::RunIdChanged);
            }
        }
        match &envelope.event {
            RunEvent::Protocol { .. } => {
                if self.saw_protocol {
                    return Err(StreamViolation::Order("duplicate protocol event"));
                }
                self.saw_protocol = true;
            }
            event => {
                if !self.saw_protocol {
                    return Err(StreamViolation::Order("first event must be `protocol`"));
                }
                match event {
                    RunEvent::RunStarted { .. } => {
                        if self.saw_started {
                            return Err(StreamViolation::Order("duplicate run.started"));
                        }
                        self.saw_started = true;
                    }
                    RunEvent::OutputDelta { .. }
                    | RunEvent::StepStarted { .. }
                    | RunEvent::StepCompleted { .. } => {
                        if !self.saw_started {
                            return Err(StreamViolation::Order(
                                "output/step event before run.started",
                            ));
                        }
                    }
                    RunEvent::RunCompleted { .. } => {
                        if !self.saw_started {
                            return Err(StreamViolation::Order("run.completed before run.started"));
                        }
                        self.saw_terminal = true;
                    }
                    // Pre-start failures legitimately terminate without a
                    // run.started (missing file, interactive rejection).
                    RunEvent::RunError { .. } | RunEvent::RunCancelled { .. } => {
                        self.saw_terminal = true;
                    }
                    // Unknown events from a newer mdflow pass through; they
                    // still consume their sequence slot.
                    RunEvent::Protocol { .. } => unreachable!("handled above"),
                    RunEvent::Unknown { .. } => {}
                }
            }
        }
        if self.run_id.is_none() {
            self.run_id = Some(envelope.run_id.clone());
        }
        self.next_seq = envelope.seq + 1;
        Ok(())
    }

    pub fn saw_terminal(&self) -> bool {
        self.saw_terminal
    }
}

// ---------------------------------------------------------------------------
// Run lifecycle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPhase {
    Starting,
    Running,
    /// Cancel was requested but the process group is not yet confirmed dead.
    /// NOT terminal: claiming "Cancelled" before the signal is known to have
    /// worked would be a lie. The authoritative `run.cancelled` event, the
    /// reader-thread EOF fallback, or the kill-escalation watcher settles it.
    Cancelling,
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
            RunPhase::Cancelling => "Cancelling",
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

    // ---- Strict parsing (2026-07-11 audit P0: no false success) ----

    /// A `run.completed` without an exit code must be REJECTED, never
    /// defaulted to 0 — "a missing run.completed.exitCode is never success"
    /// (protocol §3). This was a real false-success bug.
    #[test]
    fn missing_exit_code_is_rejected_not_success() {
        let line = r#"{"protocolVersion":1,"seq":3,"runId":"r-1","ts":4,"event":"run.completed","durationMs":10}"#;
        assert_eq!(
            parse_event_line(line),
            Err(EventParseError::Field("exitCode"))
        );
    }

    #[test]
    fn wrong_or_missing_protocol_version_is_rejected() {
        let wrong = r#"{"protocolVersion":2,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"5.0.0"}"#;
        assert_eq!(
            parse_event_line(wrong),
            Err(EventParseError::ProtocolVersion(Some(2)))
        );
        let missing =
            r#"{"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#;
        assert_eq!(
            parse_event_line(missing),
            Err(EventParseError::ProtocolVersion(None))
        );
    }

    #[test]
    fn invalid_or_missing_channel_is_rejected_not_defaulted() {
        let bad = r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":3,"event":"output.delta","channel":"trace","text":"x"}"#;
        assert_eq!(
            parse_event_line(bad),
            Err(EventParseError::Field("channel"))
        );
        let missing = r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":3,"event":"output.delta","text":"x"}"#;
        assert_eq!(
            parse_event_line(missing),
            Err(EventParseError::Field("channel"))
        );
    }

    #[test]
    fn non_json_and_missing_envelope_fields_are_typed_errors() {
        assert_eq!(parse_event_line("hello"), Err(EventParseError::NotJson));
        assert_eq!(
            parse_event_line(r#""just a string""#),
            Err(EventParseError::NotObject)
        );
        assert_eq!(
            parse_event_line(
                r#"{"protocolVersion":1,"runId":"r-1","event":"protocol","mdflowVersion":"4"}"#
            ),
            Err(EventParseError::Field("seq"))
        );
        assert_eq!(
            parse_event_line(
                r#"{"protocolVersion":1,"seq":0,"event":"protocol","mdflowVersion":"4"}"#
            ),
            Err(EventParseError::Field("runId"))
        );
    }

    // ---- Stream state machine (protocol §3 ordering) ----

    fn env(line: &str) -> RunEventEnvelope {
        parse_event_line(line).expect("test line parses")
    }

    #[test]
    fn validator_accepts_a_conforming_stream() {
        let mut validator = EventStreamValidator::default();
        for line in [
            r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            r#"{"protocolVersion":1,"seq":1,"runId":"r-1","ts":2,"event":"run.started","flowId":"project:x","pid":9}"#,
            r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":3,"event":"output.delta","channel":"stdout","text":"hi\n"}"#,
            r#"{"protocolVersion":1,"seq":3,"runId":"r-1","ts":4,"event":"run.completed","exitCode":0,"durationMs":10}"#,
        ] {
            validator.validate(&env(line)).expect("conforming stream");
        }
        assert!(validator.saw_terminal());
    }

    #[test]
    fn validator_rejects_sequence_gaps() {
        let mut validator = EventStreamValidator::default();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            ))
            .unwrap();
        assert_eq!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":2,"event":"run.started","flowId":"f"}"#,
            )),
            Err(StreamViolation::SequenceGap {
                expected: 1,
                got: 2
            })
        );
    }

    #[test]
    fn validator_requires_protocol_first_and_started_before_output() {
        let mut validator = EventStreamValidator::default();
        assert!(matches!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"run.started","flowId":"f"}"#,
            )),
            Err(StreamViolation::Order(_))
        ));

        let mut validator = EventStreamValidator::default();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            ))
            .unwrap();
        assert!(matches!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":1,"runId":"r-1","ts":2,"event":"output.delta","channel":"stdout","text":"x"}"#,
            )),
            Err(StreamViolation::Order(_))
        ));
    }

    /// Pre-start failures (protocol → run.error, no run.started, no pid) are
    /// a legitimate stream shape (protocol §3).
    #[test]
    fn validator_allows_pre_start_error_and_rejects_events_after_terminal() {
        let mut validator = EventStreamValidator::default();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            ))
            .unwrap();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":1,"runId":"r-1","ts":2,"event":"run.error","exitCode":null,"message":"interactive flow requires a terminal","durationMs":1}"#,
            ))
            .expect("pre-start error is a legal terminal");
        assert!(validator.saw_terminal());
        assert!(matches!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":2,"runId":"r-1","ts":3,"event":"output.delta","channel":"stdout","text":"late"}"#,
            )),
            Err(StreamViolation::Order(_))
        ));
    }

    #[test]
    fn validator_rejects_run_id_changes_and_completed_without_started() {
        let mut validator = EventStreamValidator::default();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            ))
            .unwrap();
        assert_eq!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":1,"runId":"r-OTHER","ts":2,"event":"run.started","flowId":"f"}"#,
            )),
            Err(StreamViolation::RunIdChanged)
        );

        let mut validator = EventStreamValidator::default();
        validator
            .validate(&env(
                r#"{"protocolVersion":1,"seq":0,"runId":"r-1","ts":1,"event":"protocol","mdflowVersion":"4.1.0"}"#,
            ))
            .unwrap();
        assert!(matches!(
            validator.validate(&env(
                r#"{"protocolVersion":1,"seq":1,"runId":"r-1","ts":2,"event":"run.completed","exitCode":0,"durationMs":1}"#,
            )),
            Err(StreamViolation::Order(_))
        ));
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
