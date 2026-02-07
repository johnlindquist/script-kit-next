use super::*;

#[test]
fn test_perf_pty_read_buffer_size() {
    const _: () = assert!(PTY_READ_BUFFER_SIZE >= 4096);
    const _: () = assert!(PTY_READ_BUFFER_SIZE <= 65536);

    let size = PTY_READ_BUFFER_SIZE;
    assert!(
        (4096..=65536).contains(&size),
        "PTY buffer size {} outside 4096-65536 range",
        size
    );
}

#[test]
fn test_perf_scrollback_default() {
    const _: () = assert!(DEFAULT_SCROLLBACK_LINES >= 1000);
    const _: () = assert!(DEFAULT_SCROLLBACK_LINES <= 50000);

    let lines = DEFAULT_SCROLLBACK_LINES;
    assert!(
        (1000..=50000).contains(&lines),
        "Scrollback {} outside 1000-50000 range",
        lines
    );
}

#[test]
fn test_perf_content_method_allocations() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(terminal) = result {
        let content1 = terminal.content();
        let content2 = terminal.content();

        assert_eq!(content1.styled_lines.len(), 24, "Should have 24 rows");
        assert_eq!(content2.styled_lines.len(), 24, "Should have 24 rows");
        assert_eq!(
            content1.styled_lines[0].len(),
            80,
            "Each row should have 80 cells"
        );
    }
}

#[test]
fn test_perf_process_is_nonblocking() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(mut terminal) = result {
        let start = std::time::Instant::now();

        for _ in 0..100 {
            let _ = terminal.process();
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "process() should be non-blocking, but 100 calls took {}ms",
            elapsed.as_millis()
        );
    }
}

#[test]
fn test_perf_constants_unchanged() {
    assert_eq!(
        DEFAULT_SCROLLBACK_LINES, 10_000,
        "DEFAULT_SCROLLBACK_LINES changed!"
    );
    assert_eq!(PTY_READ_BUFFER_SIZE, 4096, "PTY_READ_BUFFER_SIZE changed!");
    assert_eq!(
        MAX_PROCESS_BYTES_PER_TICK, 1_048_576,
        "MAX_PROCESS_BYTES_PER_TICK changed!"
    );
}

#[test]
fn test_is_application_cursor_mode_default_off() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(terminal) = result {
        assert!(
            !terminal.is_application_cursor_mode(),
            "Terminal should start in normal cursor mode (DECCKM off)"
        );
    }
}

#[test]
fn test_is_application_cursor_mode_method_exists() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(terminal) = result {
        let _mode: bool = terminal.is_application_cursor_mode();
    }
}

#[test]
fn test_perf_selection_range_is_lazy() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(terminal) = result {
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let content = terminal.content();
            assert!(
                content.selected_cells.is_empty(),
                "No selection, should have no selected cells"
            );
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "content() without selection should be fast, took {}ms for 100 calls",
            elapsed.as_millis()
        );
    }
}

#[test]
fn test_resize_clamps_zero_dimensions_to_minimum() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(mut terminal) = result {
        terminal
            .resize(0, 0)
            .expect("resize should clamp zero dimensions to 1x1");
        assert_eq!(terminal.size(), (1, 1));
    }
}

#[test]
fn test_input_ignores_empty_payload() {
    let result = TerminalHandle::new(80, 24);
    if let Ok(mut terminal) = result {
        terminal
            .input(b"")
            .expect("empty input should be a no-op success");
    }
}
