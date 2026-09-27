//! Source-level contract for AURP-10 actions-interceptor key intents.
//!
//! The actions interceptor must dispatch local main-window intents such as
//! Cmd+K toggle through the named intent handler.

const STARTUP: &str = include_str!("../src/app_impl/startup.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

#[test]
fn actions_key_intent_handler_preserves_existing_effects() {
    let handler = source_between(
        STARTUP,
        "fn handle_main_window_actions_key_intent(",
        "\n    pub(crate) fn from_initial_data(",
    );
    let footer_shortcut = handler
        .find(
            "self.dispatch_main_window_footer_shortcut(\"cmd+k\", window, cx, \"footer_shortcut\")",
        )
        .expect("The ToggleActions intent must first offer Cmd+K to the footer shortcut owner.");
    let toggle = handler
        .find("self.handle_cmd_k_actions_toggle(window, cx)")
        .expect("The ToggleActions intent must preserve the existing shared Cmd+K toggle helper.");
    assert!(
        footer_shortcut < toggle,
        "The shared Cmd+K toggle runs only when no footer shortcut took the key."
    );
}
