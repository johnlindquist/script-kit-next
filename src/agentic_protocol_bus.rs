//! Agentic session protocol response bus.
//!
//! When `SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH` is set (by `session.sh start`),
//! every stdin protocol response is appended as one NDJSON envelope so
//! `await-response.ts` can correlate by `requestId` without scraping `app.log`.

use serde::Serialize;
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
