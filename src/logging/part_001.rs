impl<S, N> FormatEvent<S, N> for JsonWithCorrelation
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut collector = JsonFieldCollector::default();
        event.record(&mut collector);
        let mut fields = collector.take();

        let message = fields
            .remove("message")
            .map(value_to_string)
            .unwrap_or_default();

        let correlation_id = fields
            .remove("correlation_id")
            .map(value_to_string)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(current_correlation_id);

        let mut root = Map::new();
        root.insert(
            "timestamp".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        );
        root.insert(
            "level".to_string(),
            Value::String(event.metadata().level().to_string()),
        );
        root.insert(
            "target".to_string(),
            Value::String(event.metadata().target().to_string()),
        );
        root.insert("correlation_id".to_string(), Value::String(correlation_id));
        root.insert("message".to_string(), Value::String(message));

        if !fields.is_empty() {
            root.insert("fields".to_string(), Value::Object(fields));
        }

        match serde_json::to_string(&Value::Object(root)) {
            Ok(json) => writeln!(writer, "{}", json),
            Err(e) => writeln!(
                writer,
                r#"{{"level":"ERROR","message":"failed to serialize log","error":"{}"}}"#,
                e
            ),
        }
    }
}
/// Wrapper to make stderr compatible with MakeWriter
struct StderrWriter;
impl<'a> MakeWriter<'a> for StderrWriter {
    type Writer = std::io::Stderr;

    fn make_writer(&'a self) -> Self::Writer {
        std::io::stderr()
    }
}
// =============================================================================
// TEE WRITER - writes to both the main log AND the session log
// =============================================================================

/// A writer that duplicates output to two `NonBlocking` writers.
struct TeeWriter {
    main: tracing_appender::non_blocking::NonBlocking,
    session: tracing_appender::non_blocking::NonBlocking,
}
impl Write for TeeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.main.write(buf)?;
        let _ = self.session.write_all(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.main.flush()?;
        let _ = self.session.flush();
        Ok(())
    }
}
/// Wrapper so tracing_subscriber can use `TeeWriter` via `MakeWriter`.
struct TeeWriterMaker {
    main: tracing_appender::non_blocking::NonBlocking,
    session: tracing_appender::non_blocking::NonBlocking,
}
impl<'a> MakeWriter<'a> for TeeWriterMaker {
    type Writer = TeeWriter;

    fn make_writer(&'a self) -> Self::Writer {
        TeeWriter {
            main: self.main.clone(),
            session: self.session.clone(),
        }
    }
}
// =============================================================================
// LEGACY SUPPORT - In-memory log buffer for UI display
// =============================================================================

static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
const MAX_LOG_LINES: usize = 50;
// =============================================================================
// LOG CAPTURE SYSTEM
// =============================================================================
// Allows capturing logs to a separate timestamped file via hotkey toggle.
// Press hotkey once to start capture, press again to stop.
// Captured logs go to ~/.scriptkit/logs/capture-<timestamp>.jsonl

/// Whether log capture is currently active
static CAPTURE_ENABLED: AtomicBool = AtomicBool::new(false);
/// Current capture session state
struct CaptureSession {
    file: File,
    path: PathBuf,
    start_time: SystemTime,
}
/// Active capture session (if any)
static CAPTURE_SESSION: OnceLock<Mutex<Option<CaptureSession>>> = OnceLock::new();
fn capture_session() -> &'static Mutex<Option<CaptureSession>> {
    CAPTURE_SESSION.get_or_init(|| Mutex::new(None))
}
/// Check if log capture is currently enabled
pub fn is_capture_enabled() -> bool {
    CAPTURE_ENABLED.load(Ordering::Relaxed)
}
/// Start capturing logs to a new timestamped file.
/// Returns the path to the capture file.
pub fn start_capture() -> anyhow::Result<PathBuf> {
    let log_dir = get_log_dir();
    fs::create_dir_all(&log_dir)?;

    // Create timestamped filename: capture-2026-01-11T08-37-28.jsonl
    let now = SystemTime::now();
    let timestamp = chrono::DateTime::<chrono::Utc>::from(now)
        .format("%Y-%m-%dT%H-%M-%S")
        .to_string();
    let filename = format!("capture-{}.jsonl", timestamp);
    let path = log_dir.join(&filename);

    let file = OpenOptions::new().create(true).append(true).open(&path)?;

    let session = CaptureSession {
        file,
        path: path.clone(),
        start_time: now,
    };

    {
        let mut guard = capture_session().lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(session);
    }

    CAPTURE_ENABLED.store(true, Ordering::Relaxed);

    tracing::info!(
        event_type = "log_capture",
        action = "started",
        capture_file = %path.display(),
        "Log capture started"
    );

    Ok(path)
}
/// Stop capturing logs and close the capture file.
/// Returns the path to the capture file and duration in seconds.
pub fn stop_capture() -> Option<(PathBuf, u64)> {
    CAPTURE_ENABLED.store(false, Ordering::Relaxed);

    let session = {
        let mut guard = capture_session().lock().unwrap_or_else(|e| e.into_inner());
        guard.take()
    };

    if let Some(session) = session {
        let duration_secs = session
            .start_time
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0);

        tracing::info!(
            event_type = "log_capture",
            action = "stopped",
            capture_file = %session.path.display(),
            duration_secs = duration_secs,
            "Log capture stopped"
        );

        Some((session.path, duration_secs))
    } else {
        None
    }
}
/// Toggle log capture on/off.
/// Returns (is_now_capturing, capture_file_path_if_relevant).
/// When starting: returns (true, Some(path_to_new_capture_file))
/// When stopping: returns (false, Some(path_to_completed_capture_file))
pub fn toggle_capture() -> (bool, Option<PathBuf>) {
    if is_capture_enabled() {
        // Stop capture
        if let Some((path, _duration)) = stop_capture() {
            (false, Some(path))
        } else {
            (false, None)
        }
    } else {
        // Start capture
        match start_capture() {
            Ok(path) => (true, Some(path)),
            Err(e) => {
                tracing::error!(
                    event_type = "log_capture",
                    action = "start_failed",
                    error = %e,
                    "Failed to start log capture"
                );
                (false, None)
            }
        }
    }
}
/// Write a log line to the capture file if capture is enabled.
/// This is called internally by the logging system.
fn write_to_capture(line: &str) {
    if !CAPTURE_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    if let Ok(mut guard) = capture_session().lock() {
        if let Some(ref mut session) = *guard {
            // Write line with newline
            let _ = writeln!(session.file, "{}", line);
            // Flush to ensure logs are immediately visible
            let _ = session.file.flush();
        }
    }
}
/// Guard that must be kept alive for the duration of the program.
/// Dropping this guard will flush and close the log files.
pub struct LoggingGuard {
    _file_guard: WorkerGuard,
    _session_guard: WorkerGuard,
}
/// Static storage for the logging guard to ensure it lives for the entire program.
/// This prevents the common mistake of calling `logging::init()` without storing the guard.
static LOGGING_GUARD: OnceLock<LoggingGuard> = OnceLock::new();
/// Initialize the global logging system.
///
/// This is the preferred way to initialize logging - it stores the guard internally
/// so callers cannot accidentally drop it. Safe to call multiple times (subsequent
/// calls are no-ops).
///
/// # Example
/// ```ignore
/// logging::init();  // Guard stored internally, can't be dropped
/// // ... rest of program
/// ```
pub fn init() {
    // Only initialize once - subsequent calls are no-ops
    LOGGING_GUARD.get_or_init(init_internal);
}
/// Internal initialization that returns a LoggingGuard.
/// This is used by init() to store the guard in LOGGING_GUARD.
fn init_internal() -> LoggingGuard {
    // Initialize legacy log buffer for UI display
    let _ = LOG_BUFFER.set(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));

    // Initialize global correlation_id (used as fallback when no contextual ID is set)
    let _ = DEFAULT_CORRELATION_ID.get_or_init(|| Uuid::new_v4().to_string());

    // Check for AI compact log mode
    let ai_log_mode = std::env::var("SCRIPT_KIT_AI_LOG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // Create log directory
    let log_dir = get_log_dir();
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("[LOGGING] Failed to create log directory: {}", e);
    }

    let log_path = log_dir.join("script-kit-gpui.jsonl");
    let session_path = log_dir.join("latest-session.jsonl");

    // Store session log path for panic hook and public access
    let _ = SESSION_LOG_PATH.set(session_path.clone());

    // Initialize session ID
    let sid = SESSION_ID
        .get_or_init(|| Uuid::new_v4().to_string())
        .clone();

    // Always print session log path (useful in both AI and non-AI modes)
    eprintln!("========================================");
    eprintln!("[SCRIPT-KIT-GPUI] Session log: {}", session_path.display());
    eprintln!("[SCRIPT-KIT-GPUI] Full log:    {}", log_path.display());
    eprintln!(
        "[SCRIPT-KIT-GPUI] Copy for AI:  cat {} | pbcopy",
        session_path.display()
    );
    eprintln!("========================================");

    // Open append-forever log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|e| {
            eprintln!("[LOGGING] Failed to open log file: {}", e);
            OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .expect("Failed to open /dev/null")
        });

    // Open session log file (truncated on each launch)
    let session_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&session_path)
        .unwrap_or_else(|e| {
            eprintln!("[LOGGING] Failed to open session log file: {}", e);
            OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .expect("Failed to open /dev/null")
        });

    // Create non-blocking writers for both files
    let (non_blocking_append, file_guard) = tracing_appender::non_blocking(file);
    let (non_blocking_session, session_guard) = tracing_appender::non_blocking(session_file);

    // Tee writer: every JSONL line goes to both files
    let tee = TeeWriterMaker {
        main: non_blocking_append,
        session: non_blocking_session,
    };

    // Environment filter - default to info, allow override via RUST_LOG
    let rust_log_value = std::env::var("RUST_LOG").unwrap_or_else(|_| "default".to_string());
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,gpui::window=off,gpui=warn,hyper=warn,reqwest=warn")
    });

    // JSONL layer for file output (goes to both append + session via tee)
    let json_layer = fmt::layer()
        .event_format(JsonWithCorrelation)
        .with_writer(tee)
        .with_ansi(false);

    if ai_log_mode {
        let ai_layer = fmt::layer()
            .with_writer(StderrWriter)
            .with_ansi(false)
            .event_format(CompactAiFormatter);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .with(ai_layer)
            .init();
    } else {
        let pretty_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .compact();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .with(pretty_layer)
            .init();
    }

    // ---- Session preamble: rich context for AI agents ----
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    let build_profile = option_env!("BUILD_PROFILE").unwrap_or("unknown");

    tracing::info!(
        event_type = "session_start",
        session_id = %sid,
        git_hash = git_hash,
        build_profile = build_profile,
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        pid = std::process::id(),
        working_dir = %std::env::current_dir().unwrap_or_default().display(),
        rust_log = %rust_log_value,
        ai_log_mode = ai_log_mode,
        "Session started"
    );

    // ---- Panic hook: capture panics to JSONL before process dies ----
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        // Log via tracing (goes to both JSONL files)
        tracing::error!(
            event_type = "panic",
            panic_message = %message,
            location = %location,
            "PANIC: {} at {}",
            message,
            location
        );

        // Safety net: write directly to session log (tracing may not flush)
        if let Some(path) = SESSION_LOG_PATH.get() {
            let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let escaped_msg = message.replace('\\', "\\\\").replace('"', "\\\"");
            let json = format!(
                r#"{{"timestamp":"{}","level":"ERROR","target":"panic","correlation_id":"panic","message":"PANIC: {} at {}","fields":{{"event_type":"panic","location":"{}"}}}}"#,
                timestamp, escaped_msg, location, location
            );
            if let Ok(mut f) = OpenOptions::new().append(true).open(path) {
                let _ = writeln!(f, "{}", json);
            }
        }

        // Call original hook (prints to stderr)
        default_hook(info);
    }));

    LoggingGuard {
        _file_guard: file_guard,
        _session_guard: session_guard,
    }
}
/// Get the log directory path (~/.scriptkit/logs/)
fn get_log_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".scriptkit").join("logs"))
        .unwrap_or_else(|| std::env::temp_dir().join("script-kit-logs"))
}
/// Get the path to the JSONL log file
pub fn log_path() -> PathBuf {
    get_log_dir().join("script-kit-gpui.jsonl")
}
