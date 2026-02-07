use super::*;

#[test]
fn test_terminal_action_ids() {
    assert_eq!(TerminalAction::Clear.id(), "clear");
    assert_eq!(TerminalAction::Copy.id(), "copy");
    assert_eq!(TerminalAction::Paste.id(), "paste");
    assert_eq!(TerminalAction::SelectAll.id(), "select_all");
    assert_eq!(TerminalAction::ScrollToTop.id(), "scroll_to_top");
    assert_eq!(TerminalAction::ScrollToBottom.id(), "scroll_to_bottom");
    assert_eq!(TerminalAction::ScrollPageUp.id(), "scroll_page_up");
    assert_eq!(TerminalAction::ScrollPageDown.id(), "scroll_page_down");
    assert_eq!(TerminalAction::Find.id(), "find");
    assert_eq!(TerminalAction::Interrupt.id(), "interrupt");
    assert_eq!(TerminalAction::Kill.id(), "kill");
    assert_eq!(TerminalAction::Suspend.id(), "suspend");
    assert_eq!(TerminalAction::Quit.id(), "quit");
    assert_eq!(TerminalAction::SendEOF.id(), "send_eof");
    assert_eq!(TerminalAction::Reset.id(), "reset");
    assert_eq!(TerminalAction::NewShell.id(), "new_shell");
    assert_eq!(TerminalAction::Restart.id(), "restart");
    assert_eq!(TerminalAction::ZoomIn.id(), "zoom_in");
    assert_eq!(TerminalAction::ZoomOut.id(), "zoom_out");
    assert_eq!(TerminalAction::ResetZoom.id(), "reset_zoom");

    assert_eq!(TerminalAction::Custom("my_action".into()).id(), "my_action");
}

#[test]
fn test_terminal_action_shortcuts() {
    assert_eq!(TerminalAction::Clear.default_shortcut(), Some("⌘K"));
    assert_eq!(TerminalAction::Copy.default_shortcut(), Some("⌘C"));
    assert_eq!(TerminalAction::Paste.default_shortcut(), Some("⌘V"));
    assert_eq!(TerminalAction::Interrupt.default_shortcut(), Some("⌃C"));
    assert_eq!(TerminalAction::Suspend.default_shortcut(), Some("⌃Z"));
    assert_eq!(TerminalAction::SendEOF.default_shortcut(), Some("⌃D"));
    assert_eq!(TerminalAction::ZoomIn.default_shortcut(), Some("⌘+"));
    assert_eq!(TerminalAction::ZoomOut.default_shortcut(), Some("⌘-"));
    assert_eq!(TerminalAction::ResetZoom.default_shortcut(), Some("⌘0"));

    assert_eq!(TerminalAction::Kill.default_shortcut(), None);
    assert_eq!(TerminalAction::Reset.default_shortcut(), None);
    assert_eq!(TerminalAction::NewShell.default_shortcut(), None);
}

#[test]
fn test_terminal_action_is_signal() {
    assert!(TerminalAction::Kill.is_signal_action());
    assert!(TerminalAction::Interrupt.is_signal_action());
    assert!(TerminalAction::Suspend.is_signal_action());
    assert!(TerminalAction::Quit.is_signal_action());

    assert!(!TerminalAction::Clear.is_signal_action());
    assert!(!TerminalAction::Copy.is_signal_action());
    assert!(!TerminalAction::SendEOF.is_signal_action());
    assert!(!TerminalAction::ZoomIn.is_signal_action());
    assert!(!TerminalAction::ZoomOut.is_signal_action());
    assert!(!TerminalAction::ResetZoom.is_signal_action());
}

#[test]
fn test_terminal_action_display() {
    assert_eq!(format!("{}", TerminalAction::Clear), "clear");
    assert_eq!(format!("{}", TerminalAction::ScrollToTop), "scroll_to_top");
}

#[test]
fn test_command_item_creation() {
    let item = TerminalCommandItem::new(
        "Clear Terminal",
        "Clear the screen",
        Some("⌘K"),
        TerminalAction::Clear,
    );

    assert_eq!(item.name, "Clear Terminal");
    assert_eq!(item.description, "Clear the screen");
    assert_eq!(item.shortcut, Some("⌘K".to_string()));
    assert_eq!(item.action, TerminalAction::Clear);
    assert_eq!(item.name_lower, "clear terminal");
    assert_eq!(item.description_lower, "clear the screen");
    assert_eq!(item.action_id(), "clear");
}

#[test]
fn test_command_item_without_shortcut() {
    let item = TerminalCommandItem::new(
        "Kill Process",
        "Terminate the running process",
        None::<String>,
        TerminalAction::Kill,
    );

    assert!(item.shortcut.is_none());
    assert_eq!(item.action_id(), "kill");
}

#[test]
fn test_command_item_matches() {
    let item = TerminalCommandItem::new(
        "Clear Terminal",
        "Clear the screen and scrollback",
        Some("⌘K"),
        TerminalAction::Clear,
    );

    assert!(item.matches("clear"));
    assert!(item.matches("CLEAR"));
    assert!(item.matches("terminal"));

    assert!(item.matches("screen"));
    assert!(item.matches("scrollback"));

    assert!(item.matches("cle"));
    assert!(item.matches("scr"));

    assert!(!item.matches("paste"));
    assert!(!item.matches("xyz"));
}

#[test]
fn test_get_terminal_commands() {
    let commands = get_terminal_commands();

    assert!(commands.len() >= 17);

    let action_ids: Vec<&str> = commands.iter().map(|c| c.action_id()).collect();

    assert!(action_ids.contains(&"clear"));
    assert!(action_ids.contains(&"copy"));
    assert!(action_ids.contains(&"paste"));
    assert!(action_ids.contains(&"select_all"));
    assert!(action_ids.contains(&"scroll_to_top"));
    assert!(action_ids.contains(&"scroll_to_bottom"));
    assert!(action_ids.contains(&"find"));
    assert!(action_ids.contains(&"interrupt"));
    assert!(action_ids.contains(&"kill"));
    assert!(action_ids.contains(&"suspend"));
    assert!(action_ids.contains(&"quit"));
    assert!(action_ids.contains(&"send_eof"));
    assert!(action_ids.contains(&"reset"));
    assert!(action_ids.contains(&"new_shell"));
    assert!(action_ids.contains(&"restart"));
}

#[test]
fn test_commands_have_descriptions() {
    let commands = get_terminal_commands();

    for cmd in &commands {
        assert!(
            !cmd.description.is_empty(),
            "Command {} should have a description",
            cmd.name
        );
    }
}

#[test]
fn test_commands_have_lowercase_cache() {
    let commands = get_terminal_commands();

    for cmd in &commands {
        assert_eq!(
            cmd.name_lower,
            cmd.name.to_lowercase(),
            "name_lower should be lowercase version of name"
        );
        assert_eq!(
            cmd.description_lower,
            cmd.description.to_lowercase(),
            "description_lower should be lowercase version of description"
        );
    }
}
