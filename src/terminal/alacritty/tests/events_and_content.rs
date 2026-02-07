use super::*;
use alacritty_terminal::event::Event as AlacrittyEvent;

#[test]
fn test_event_proxy_creation() {
    let proxy = EventProxy::new();
    assert!(proxy.take_events().is_empty());
}

#[test]
fn test_event_proxy_batching() {
    let proxy = EventProxy::new();

    proxy.send_event(AlacrittyEvent::Bell);
    proxy.send_event(AlacrittyEvent::Title("Test".to_string()));

    let events = proxy.take_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], TerminalEvent::Bell));
    assert!(matches!(events[1], TerminalEvent::Title(_)));

    assert!(proxy.take_events().is_empty());
}

#[test]
fn test_terminal_size() {
    let size = TerminalSize::new(80, 24);
    assert_eq!(size.columns(), 80);
    assert_eq!(size.screen_lines(), 24);
    assert_eq!(size.total_lines(), 24);
}

#[test]
fn test_terminal_content_is_empty() {
    let empty_content = TerminalContent {
        lines: vec![],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 0,
        selected_cells: vec![],
    };
    assert!(empty_content.is_empty());

    let whitespace_content = TerminalContent {
        lines: vec!["".to_string(), "".to_string()],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 0,
        selected_cells: vec![],
    };
    assert!(whitespace_content.is_empty());

    let content_with_text = TerminalContent {
        lines: vec!["hello".to_string()],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 5,
        selected_cells: vec![],
    };
    assert!(!content_with_text.is_empty());
}

#[test]
fn test_terminal_content_line_count() {
    let content = TerminalContent {
        lines: vec!["hello".to_string(), "".to_string(), "world".to_string()],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 0,
        selected_cells: vec![],
    };
    assert_eq!(content.line_count(), 2);
}

#[test]
fn test_cursor_position_from_content() {
    let content = TerminalContent {
        lines: vec!["hello world".to_string()],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 6,
        selected_cells: vec![],
    };
    let cursor: CursorPosition = (&content).into();
    assert_eq!(cursor.line, 0);
    assert_eq!(cursor.col, 6);
}
