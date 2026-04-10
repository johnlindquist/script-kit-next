//! Source-contract tests for launcher AI hint visibility.
//!
//! Locks the invariant that the launcher header badge advertises both
//! Tab and Cmd+Enter for ACP Chat entry.

use std::fs;

#[test]
fn launcher_header_advertises_tab_and_cmd_enter() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains("Ask"),
        "Launcher header must show 'Ask' label"
    );
    assert!(
        source.contains(".child(\"⇥\")"),
        "Launcher header must show Tab badge"
    );
    assert!(
        source.contains("⌘↩"),
        "Launcher header must show Cmd+Enter badge"
    );
}
