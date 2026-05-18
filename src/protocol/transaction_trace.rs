//! JSONL persistence layer for transaction flight recorder traces.
//!
//! Appends serialized `TransactionTrace` records to an append-only JSONL file
//! and reads them back for inspection by agents and diagnostic tooling.

use crate::protocol::types::batch_wait::{TransactionTrace, TransactionTraceMode};
use anyhow::{anyhow, Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TRANSACTION_TRACE_MAX_BYTES: u64 = 10 * 1024 * 1024;
const TRANSACTION_TRACE_COMPACT_KEEP: usize = 2_000;

fn trace_log_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

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
    let _guard = trace_log_mutex()
        .lock()
        .map_err(|_| anyhow!("trace log mutex poisoned"))?;
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_transaction_log_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    compact_transaction_trace_log_if_needed(&path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    ensure_jsonl_append_boundary(&mut file)?;

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

fn ensure_jsonl_append_boundary(file: &mut fs::File) -> Result<()> {
    let len = file.metadata()?.len();
    if len == 0 {
        return Ok(());
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    file.seek(SeekFrom::End(0))?;
    if last[0] != b'\n' {
        writeln!(file)?;
    }
    Ok(())
}

pub fn compact_transaction_trace_log_if_needed(path: &Path) -> Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to stat {}", path.display()))
        }
    };
    if metadata.len() <= TRANSACTION_TRACE_MAX_BYTES {
        return Ok(());
    }

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut traces = Vec::new();
    for line in reader.lines() {
        let line = line.context("failed to read transaction trace log during compaction")?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<TransactionTrace>(&line) {
            Ok(trace) => traces.push(trace),
            Err(error) => tracing::warn!(
                target: "script_kit::transaction",
                log_path = %path.display(),
                %error,
                "Skipping malformed transaction trace log entry during compaction"
            ),
        }
    }
    let start = traces.len().saturating_sub(TRANSACTION_TRACE_COMPACT_KEEP);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to compact {}", path.display()))?;
    for trace in traces.into_iter().skip(start) {
        let line = serde_json::to_string(&trace).context("failed to serialize compacted trace")?;
        writeln!(file, "{line}").context("failed to write compacted transaction trace")?;
    }
    Ok(())
}

/// Read the most recent transaction trace, optionally filtered by request_id.
pub fn read_latest_transaction_trace(
    path: Option<&Path>,
    request_id: Option<&str>,
) -> Result<Option<TransactionTrace>> {
    let _guard = trace_log_mutex()
        .lock()
        .map_err(|_| anyhow!("trace log mutex poisoned"))?;
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
    let mut latest = None;
    for line in reader.lines() {
        let line = line.context("failed to read transaction trace log")?;
        if line.trim().is_empty() {
            continue;
        }
        let trace: TransactionTrace = match serde_json::from_str(&line) {
            Ok(trace) => trace,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::transaction",
                    log_path = %path.display(),
                    %error,
                    "Skipping malformed transaction trace log entry"
                );
                continue;
            }
        };
        if request_id.is_none() || request_id == Some(trace.request_id.as_str()) {
            latest = Some(trace);
        }
    }

    Ok(latest)
}

/// Returns true when trace policy says to include the trace in the result.
pub fn should_include_trace(mode: TransactionTraceMode, success: bool) -> bool {
    matches!(mode, TransactionTraceMode::On)
        || (!success && matches!(mode, TransactionTraceMode::OnFailure))
}
