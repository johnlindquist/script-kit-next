//! Agentic session protocol response bus.
//!
//! When `SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH` is set (by `session.sh start`),
//! every stdin protocol response is appended as one NDJSON envelope so
//! `await-response.ts` can correlate by `requestId` without scraping `app.log`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;
const KIND: &str = "protocolResponse";

static BUS_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Serialize)]
struct ProtocolResponseEnvelope<'a> {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    kind: &'a str,
    session: &'a str,
    #[serde(rename = "sessionGeneration")]
    session_generation: &'a str,
    #[serde(rename = "requestId")]
    request_id: &'a str,
    #[serde(rename = "responseType")]
    response_type: &'a str,
    #[serde(rename = "correlationId")]
    correlation_id: String,
    #[serde(rename = "finishedAtMs")]
    finished_at_ms: u128,
    response: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolResponseHistoryEntry {
    pub schema_version: u32,
    pub kind: String,
    pub session: String,
    pub session_generation: String,
    pub request_id: String,
    pub response_type: String,
    pub correlation_id: String,
    pub finished_at_ms: u128,
    pub response: Value,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolResponseHistorySummary {
    pub request_id: String,
    pub response_type: String,
    pub session: String,
    pub session_generation: String,
    pub finished_at_ms: u128,
    pub status: Option<String>,
    pub classification: Option<String>,
    pub surface_kind: Option<String>,
    pub automation_id: Option<String>,
    pub preview: String,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn bus_path_from_env() -> Option<std::path::PathBuf> {
    std::env::var_os("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH")
        .map(std::path::PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

pub fn protocol_response_history_path() -> Option<std::path::PathBuf> {
    bus_path_from_env()
}

fn session_name_from_env() -> String {
    std::env::var("SCRIPT_KIT_AGENTIC_SESSION_NAME").unwrap_or_else(|_| "default".to_string())
}

fn session_generation_from_env() -> String {
    std::env::var("SCRIPT_KIT_AGENTIC_SESSION_GENERATION").unwrap_or_else(|_| "unknown".to_string())
}

fn request_id_from_json(value: &Value) -> Option<String> {
    value
        .get("requestId")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn response_type_from_json(value: &Value) -> String {
    value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn nested_string_field(value: &Value, object_key: &str, key: &str) -> Option<String> {
    value
        .get(object_key)
        .and_then(Value::as_object)
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn response_surface_kind(response: &Value) -> Option<String> {
    nested_string_field(response, "resolvedTarget", "surfaceKind")
        .or_else(|| nested_string_field(response, "target", "surfaceKind"))
        .or_else(|| nested_string_field(response, "surfaceContract", "surfaceKind"))
}

fn response_automation_id(response: &Value) -> Option<String> {
    nested_string_field(response, "resolvedTarget", "automationId")
        .or_else(|| nested_string_field(response, "target", "automationId"))
        .or_else(|| string_field(response, "automationId"))
}

fn summarize_protocol_response(
    entry: ProtocolResponseHistoryEntry,
) -> ProtocolResponseHistorySummary {
    let status =
        string_field(&entry.response, "status").or_else(|| string_field(&entry.response, "result"));
    let classification = string_field(&entry.response, "classification");
    let surface_kind = response_surface_kind(&entry.response);
    let automation_id = response_automation_id(&entry.response);
    let preview = format!(
        "{} · {} · {}",
        entry.response_type,
        classification.as_deref().unwrap_or("unclassified"),
        surface_kind.as_deref().unwrap_or("no surface")
    );

    ProtocolResponseHistorySummary {
        request_id: entry.request_id,
        response_type: entry.response_type,
        session: entry.session,
        session_generation: entry.session_generation,
        finished_at_ms: entry.finished_at_ms,
        status,
        classification,
        surface_kind,
        automation_id,
        preview,
    }
}

pub fn load_recent_protocol_response_history(limit: usize) -> Vec<ProtocolResponseHistoryEntry> {
    let Some(path) = protocol_response_history_path() else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    raw.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<ProtocolResponseHistoryEntry>(line).ok())
        .take(limit)
        .collect()
}

pub fn load_recent_protocol_response_summaries(
    limit: usize,
) -> Vec<ProtocolResponseHistorySummary> {
    load_recent_protocol_response_history(limit)
        .into_iter()
        .map(summarize_protocol_response)
        .collect()
}

pub fn find_protocol_response_by_request_id(
    request_id: &str,
) -> Option<ProtocolResponseHistoryEntry> {
    load_recent_protocol_response_history(500)
        .into_iter()
        .find(|entry| entry.request_id == request_id)
}

/// Append a serialized protocol JSON line to the agentic response bus when configured.
pub fn append_from_json_line(json: &str) {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return;
    };
    let Some(request_id) = request_id_from_json(&value) else {
        return;
    };
    if request_id.is_empty() {
        return;
    }

    let Some(path) = bus_path_from_env() else {
        return;
    };

    let envelope = ProtocolResponseEnvelope {
        schema_version: SCHEMA_VERSION,
        kind: KIND,
        session: &session_name_from_env(),
        session_generation: &session_generation_from_env(),
        request_id: &request_id,
        response_type: &response_type_from_json(&value),
        correlation_id: format!("stdin:req:{request_id}"),
        finished_at_ms: now_ms(),
        response: value,
    };

    if let Err(error) = append_envelope(&path, &envelope) {
        tracing::warn!(
            category = "STDIN",
            error = %error,
            path = %path.display(),
            request_id = %request_id,
            "Failed to append protocol response to agentic bus"
        );
    }
}

fn append_envelope(path: &Path, envelope: &ProtocolResponseEnvelope<'_>) -> std::io::Result<()> {
    let _guard = BUS_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| std::io::Error::other("agentic protocol bus mutex poisoned"))?;

    let mut line = serde_json::to_vec(envelope)?;
    line.push(b'\n');

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&line)?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_env_lock<T>(f: impl FnOnce() -> T) -> T {
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().expect("env lock");
        f()
    }

    #[test]
    fn append_from_json_line_writes_protocol_response_envelope() {
        with_env_lock(|| {
            let dir = tempdir().expect("tempdir");
            let path = dir.path().join("protocol-responses.ndjson");
            std::env::set_var("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH", &path);
            std::env::set_var("SCRIPT_KIT_AGENTIC_SESSION_NAME", "test-session");
            std::env::set_var("SCRIPT_KIT_AGENTIC_SESSION_GENERATION", "gen-1");

            append_from_json_line(
                r#"{"type":"stateResult","requestId":"req-1","promptType":"none"}"#,
            );

            let raw = fs::read_to_string(&path).expect("bus file");
            assert!(raw.contains(r#""kind":"protocolResponse""#));
            assert!(raw.contains(r#""requestId":"req-1""#));
            assert!(raw.contains(r#""responseType":"stateResult""#));
            assert!(raw.contains(r#""correlationId":"stdin:req:req-1""#));

            let summaries = load_recent_protocol_response_summaries(10);
            assert_eq!(summaries.len(), 1);
            assert_eq!(summaries[0].request_id, "req-1");
            assert_eq!(summaries[0].response_type, "stateResult");
            assert_eq!(
                summaries[0].preview,
                "stateResult · unclassified · no surface"
            );

            let found = find_protocol_response_by_request_id("req-1").expect("history entry");
            assert_eq!(found.request_id, "req-1");

            std::env::remove_var("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH");
            std::env::remove_var("SCRIPT_KIT_AGENTIC_SESSION_NAME");
            std::env::remove_var("SCRIPT_KIT_AGENTIC_SESSION_GENERATION");
        });
    }

    #[test]
    fn append_from_json_line_skips_messages_without_request_id() {
        with_env_lock(|| {
            let dir = tempdir().expect("tempdir");
            let path = dir.path().join("protocol-responses.ndjson");
            std::env::set_var("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH", &path);

            append_from_json_line(r#"{"type":"beep"}"#);

            assert!(!path.exists());

            std::env::remove_var("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH");
        });
    }
}
