    use super::*;
    use std::io::Write as IoWrite;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::{fmt as fmt_sub, EnvFilter};
    #[derive(Clone)]
    struct BufferWriter(Arc<Mutex<Vec<u8>>>);
    struct BufferGuard<'a> {
        buf: &'a Arc<Mutex<Vec<u8>>>,
    }
    impl<'a> IoWrite for BufferGuard<'a> {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            let mut buf = self.buf.lock().unwrap_or_else(|e| e.into_inner());
            buf.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> MakeWriter<'a> for BufferWriter {
        type Writer = BufferGuard<'a>;

        fn make_writer(&'a self) -> Self::Writer {
            BufferGuard { buf: &self.0 }
        }
    }
    #[test]
    fn json_formatter_injects_correlation_id() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = fmt_sub()
            .json()
            .with_writer(BufferWriter(buffer.clone()))
            .event_format(JsonWithCorrelation)
            .with_env_filter(EnvFilter::new("info"))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("hello-json-correlation");
        });

        let output =
            String::from_utf8(buffer.lock().unwrap_or_else(|e| e.into_inner()).clone()).unwrap();
        let line = output.lines().next().unwrap();
        let value: serde_json::Value = serde_json::from_str(line).unwrap();

        let cid = value
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        assert!(
            !cid.is_empty(),
            "correlation_id should be present and non-empty"
        );
    }
    #[test]
    fn compact_formatter_includes_correlation_id_token() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = fmt_sub()
            .with_writer(BufferWriter(buffer.clone()))
            .event_format(CompactAiFormatter)
            .with_env_filter(EnvFilter::new("info"))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("hello-compact-correlation");
        });

        let output =
            String::from_utf8(buffer.lock().unwrap_or_else(|e| e.into_inner()).clone()).unwrap();
        let line = output.lines().next().unwrap_or("");
        assert!(
            line.contains("cid="),
            "compact log should include cid token: {}",
            line
        );
    }
    // -------------------------------------------------------------------------
    // category_to_code tests - using real category strings from logs
    // -------------------------------------------------------------------------

    #[test]
    fn test_category_to_code_position() {
        // From: "CALCULATING WINDOW POSITION FOR MOUSE DISPLAY"
        assert_eq!(category_to_code("POSITION"), 'P');
        assert_eq!(category_to_code("position"), 'P');
        assert_eq!(category_to_code("Position"), 'P');
    }
    #[test]
    fn test_category_to_code_app() {
        // From: "Application logging initialized", "GPUI Application starting"
        assert_eq!(category_to_code("APP"), 'A');
        assert_eq!(category_to_code("app"), 'A');
    }
    #[test]
    fn test_category_to_code_stdin() {
        // From: "External command listener started", "Received: {\"type\": \"run\"..."
        assert_eq!(category_to_code("STDIN"), 'S');
    }
    #[test]
    fn test_category_to_code_hotkey() {
        // From: "Registered global hotkey meta+Digit0", "Tray icon initialized"
        assert_eq!(category_to_code("HOTKEY"), 'H');
        assert_eq!(category_to_code("TRAY"), 'H'); // Tray maps to H
    }
    #[test]
    fn test_category_to_code_visibility() {
        // From: "HOTKEY TRIGGERED - TOGGLE WINDOW", "WINDOW_VISIBLE set to: true"
        assert_eq!(category_to_code("VISIBILITY"), 'V');
    }
    #[test]
    fn test_category_to_code_exec() {
        // From: "Executing script: hello-world", "Script execution complete"
        assert_eq!(category_to_code("EXEC"), 'E');
    }
    #[test]
    fn test_category_to_code_theme() {
        // From: "Theme file not found, using defaults based on system appearance"
        assert_eq!(category_to_code("THEME"), 'T');
    }
    #[test]
    fn test_category_to_code_window_mgr() {
        // From: "Searching for main window among 2 windows"
        assert_eq!(category_to_code("WINDOW_MGR"), 'W');
    }
    #[test]
    fn test_category_to_code_config() {
        // From: "Successfully loaded config from ~/.scriptkit/kit/config.ts"
        assert_eq!(category_to_code("CONFIG"), 'N');
        assert_eq!(category_to_code("config"), 'N');
        assert_eq!(category_to_code("Config"), 'N');
    }
    #[test]
    fn test_category_to_code_perf() {
        // From: "Startup loading: 33.30ms total (331 scripts in 5.03ms)"
        assert_eq!(category_to_code("PERF"), 'R');
    }
    #[test]
    fn test_category_to_code_all_categories() {
        // Complete mapping verification
        let mappings = [
            ("POSITION", 'P'),
            ("APP", 'A'),
            ("UI", 'U'),
            ("STDIN", 'S'),
            ("HOTKEY", 'H'),
            ("VISIBILITY", 'V'),
            ("EXEC", 'E'),
            ("KEY", 'K'),
            ("FOCUS", 'F'),
            ("THEME", 'T'),
            ("CACHE", 'C'),
            ("PERF", 'R'),
            ("WINDOW_MGR", 'W'),
            ("ERROR", 'X'),
            ("MOUSE_HOVER", 'M'),
            ("SCROLL_STATE", 'L'),
            ("SCROLL_PERF", 'Q'),
            ("SCRIPT", 'G'), // Changed from B to G
            ("CONFIG", 'N'),
            ("RESIZE", 'Z'),
            ("DESIGN", 'D'),
            ("BENCH", 'B'), // New: Benchmark timing
            ("CHAT", 'U'),
            ("AI", 'U'),
            ("ACTIONS", 'U'),
            ("WINDOW_STATE", 'W'),
            ("DEBUG_GRID", 'D'),
            ("MCP", 'S'),
            ("WARN", 'X'),
            ("SCRIPTLET_PARSE", 'G'),
        ];

        for (category, expected_code) in mappings {
            assert_eq!(
                category_to_code(category),
                expected_code,
                "Category '{}' should map to '{}'",
                category,
                expected_code
            );
        }
    }
    #[test]
    fn test_category_to_code_unknown() {
        assert_eq!(category_to_code("UNKNOWN_CATEGORY"), '-');
        assert_eq!(category_to_code(""), '-');
        assert_eq!(category_to_code("foobar"), '-');
    }
    // -------------------------------------------------------------------------
    // level_to_char tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_level_to_char() {
        assert_eq!(level_to_char(Level::ERROR), 'e');
        assert_eq!(level_to_char(Level::WARN), 'w');
        assert_eq!(level_to_char(Level::INFO), 'i');
        assert_eq!(level_to_char(Level::DEBUG), 'd');
        assert_eq!(level_to_char(Level::TRACE), 't');
    }
    // -------------------------------------------------------------------------
    // infer_category_from_target tests - using real module paths
    // -------------------------------------------------------------------------

    #[test]
    fn test_infer_category_executor() {
        // From: script_kit_gpui::executor
        assert_eq!(infer_category_from_target("script_kit_gpui::executor"), 'E');
    }
    #[test]
    fn test_infer_category_theme() {
        // From: "script_kit_gpui::theme: Theme file not found"
        assert_eq!(infer_category_from_target("script_kit_gpui::theme"), 'T');
    }
    #[test]
    fn test_infer_category_config() {
        // From: "script_kit_gpui::config: Successfully loaded config"
        assert_eq!(infer_category_from_target("script_kit_gpui::config"), 'N');
    }
    #[test]
    fn test_infer_category_clipboard() {
        // From: "script_kit_gpui::clipboard_history: Initializing clipboard history"
        assert_eq!(
            infer_category_from_target("script_kit_gpui::clipboard_history"),
            'A'
        );
    }
    #[test]
    fn test_infer_category_logging() {
        // From: "script_kit_gpui::logging: Application logging initialized"
        assert_eq!(infer_category_from_target("script_kit_gpui::logging"), 'A');
    }
    #[test]
    fn test_infer_category_protocol() {
        // From: "script_kit_gpui::protocol" (stdin message handling)
        assert_eq!(infer_category_from_target("script_kit_gpui::protocol"), 'S');
    }
    #[test]
    fn test_infer_category_prompts() {
        // UI components
        assert_eq!(infer_category_from_target("script_kit_gpui::prompts"), 'U');
        assert_eq!(infer_category_from_target("script_kit_gpui::editor"), 'U');
        assert_eq!(infer_category_from_target("script_kit_gpui::panel"), 'U');
    }
    #[test]
    fn test_infer_category_scripts() {
        // From: "Loaded 331 scripts from ~/.scriptkit/scripts"
        assert_eq!(infer_category_from_target("script_kit_gpui::scripts"), 'G');
        assert_eq!(
            infer_category_from_target("script_kit_gpui::file_search"),
            'G'
        );
    }
    #[test]
    fn test_infer_category_hotkey() {
        // From: "Registered global hotkey meta+Digit0"
        assert_eq!(infer_category_from_target("script_kit_gpui::hotkey"), 'H');
        assert_eq!(infer_category_from_target("script_kit_gpui::tray"), 'H');
    }
    #[test]
    fn test_infer_category_window() {
        assert_eq!(
            infer_category_from_target("script_kit_gpui::window_manager"),
            'W'
        );
        assert_eq!(
            infer_category_from_target("script_kit_gpui::window_control"),
            'W'
        );
        assert_eq!(
            infer_category_from_target("script_kit_gpui::window_state"),
            'W'
        );
    }
    #[test]
    fn test_infer_category_unknown() {
        assert_eq!(infer_category_from_target("script_kit_gpui::main"), 'A');
        assert_eq!(infer_category_from_target("script_kit_gpui::ai"), 'U');
        assert_eq!(
            infer_category_from_target("script_kit_gpui::mcp_server"),
            'S'
        );
        assert_eq!(infer_category_from_target("unknown::module"), '-');
    }
    #[test]
    fn test_legacy_level_for_category() {
        assert_eq!(legacy_level_for_category("ERROR"), LegacyLogLevel::Error);
        assert_eq!(legacy_level_for_category("WARN"), LegacyLogLevel::Warn);
        assert_eq!(legacy_level_for_category("WARNING"), LegacyLogLevel::Warn);
        assert_eq!(legacy_level_for_category("DEBUG"), LegacyLogLevel::Debug);
        assert_eq!(legacy_level_for_category("TRACE"), LegacyLogLevel::Trace);
        assert_eq!(legacy_level_for_category("UI"), LegacyLogLevel::Info);
    }
    // -------------------------------------------------------------------------
    // get_minute_timestamp tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_minute_timestamp_format() {
        let ts = get_minute_timestamp();
        // Format should be "SS.mmm" - 2 digits, dot, 3 digits
        assert_eq!(ts.len(), 6, "Timestamp '{}' should be 6 chars", ts);
        assert!(ts.contains('.'), "Timestamp '{}' should contain '.'", ts);

        let parts: Vec<&str> = ts.split('.').collect();
        assert_eq!(parts.len(), 2);

        let seconds: u32 = parts[0].parse().expect("seconds should be numeric");
        let millis: u32 = parts[1].parse().expect("millis should be numeric");

        assert!(seconds < 60, "Seconds {} should be < 60", seconds);
        assert!(millis < 1000, "Millis {} should be < 1000", millis);
    }
    #[test]
    fn test_get_minute_timestamp_changes() {
        // Two calls in quick succession should produce similar timestamps
        let ts1 = get_minute_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let ts2 = get_minute_timestamp();

        // Parse both
        let parse = |ts: &str| -> u64 {
            let parts: Vec<&str> = ts.split('.').collect();
            let secs: u64 = parts[0].parse().unwrap();
            let millis: u64 = parts[1].parse().unwrap();
            secs * 1000 + millis
        };

        let diff = parse(&ts2).saturating_sub(parse(&ts1));
        // Should be at least 5ms apart (we slept 5ms)
        assert!(
            diff >= 4,
            "Timestamps should be at least 4ms apart, got {}ms",
            diff
        );
        // But not more than 100ms (reasonable execution time)
        assert!(
            diff < 100,
            "Timestamps should be less than 100ms apart, got {}ms",
            diff
        );
    }
    // -------------------------------------------------------------------------
    // Compact format output validation (pattern matching)
    // -------------------------------------------------------------------------

    #[test]
    fn test_compact_format_pattern() {
        // Real example from logs:
        // "11.697|i|A|Application logging initialized event_type=app_lifecycle..."
        let example = "11.697|i|A|Application logging initialized";

        let parts: Vec<&str> = example.split('|').collect();
        assert_eq!(parts.len(), 4, "Compact format should have 4 parts");

        // Part 0: timestamp (SS.mmm)
        assert_eq!(parts[0].len(), 6);
        assert!(parts[0].contains('.'));

        // Part 1: level (single char)
        assert_eq!(parts[1].len(), 1);
        assert!("iwedtIWEDT".contains(parts[1]));

        // Part 2: category (single char)
        assert_eq!(parts[2].len(), 1);

        // Part 3: message (rest)
        assert!(!parts[3].is_empty());
    }
    #[test]
    fn test_compact_format_real_examples() {
        // Real log lines from test run
        let examples = [
            ("11.697|i|A|Application logging initialized", "i", "A"),
            ("11.717|i|N|Successfully loaded config", "i", "N"),
            ("11.741|i|H|Registered global hotkey meta+Digit0", "i", "H"),
            ("11.779|i|P|Available displays: 1", "i", "P"),
        ];

        for (line, expected_level, expected_cat) in examples {
            let parts: Vec<&str> = line.split('|').collect();
            assert_eq!(
                parts[1], expected_level,
                "Line '{}' should have level '{}'",
                line, expected_level
            );
            assert_eq!(
                parts[2], expected_cat,
                "Line '{}' should have category '{}'",
                line, expected_cat
            );
        }
    }
