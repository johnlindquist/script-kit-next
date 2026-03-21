//! JSONL persistence layer for transaction flight recorder traces.
//!
//! Appends serialized `TransactionTrace` records to an append-only JSONL file
//! and reads them back for inspection by agents and diagnostic tooling.

use crate::protocol::types::batch_wait::{TransactionTrace, TransactionTraceMode};
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current epoch time in milliseconds.
pub fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Returns the default path for transaction trace logs.
pub fn default_transaction_log_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".scriptkit")
        .join("logs")
        .join("transactions.jsonl")
}

/// Append a single transaction trace to the JSONL log file.
///
/// Creates parent directories if they don't exist. Returns the path written to.
pub fn append_transaction_trace(path: Option<&Path>, trace: &TransactionTrace) -> Result<PathBuf> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_transaction_log_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    let line = serde_json::to_string(trace).context("failed to serialize transaction trace")?;
    writeln!(file, "{line}").context("failed to write transaction trace")?;

    tracing::info!(
        target: "script_kit::transaction",
        log_path = %path.display(),
        request_id = %trace.request_id,
        status = ?trace.status,
        "transaction_trace_persisted"
    );

    Ok(path)
}

/// Read the most recent transaction trace, optionally filtered by request_id.
pub fn read_latest_transaction_trace(
    path: Option<&Path>,
    request_id: Option<&str>,
) -> Result<Option<TransactionTrace>> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_transaction_log_path);

    if !path.exists() {
        return Ok(None);
    }

    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<std::io::Result<Vec<_>>>()
        .context("failed to read transaction trace log")?;

    for line in lines.into_iter().rev() {
        if line.trim().is_empty() {
            continue;
        }
        let trace: TransactionTrace =
            serde_json::from_str(&line).context("failed to deserialize transaction trace")?;
        if request_id.is_none() || request_id == Some(trace.request_id.as_str()) {
            return Ok(Some(trace));
        }
    }

    Ok(None)
}

/// Returns true when trace policy says to include the trace in the result.
pub fn should_include_trace(mode: TransactionTraceMode, success: bool) -> bool {
    matches!(mode, TransactionTraceMode::On)
        || (!success && matches!(mode, TransactionTraceMode::OnFailure))
}
