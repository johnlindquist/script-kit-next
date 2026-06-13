const FILTER_CHANGE: &str = include_str!("../src/app_impl/filter_input_change.rs");
const FILTER_CORE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const FILTER_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");

#[test]
fn live_and_setfilter_paths_share_script_list_special_entry_router() {
    assert!(
        FILTER_CHANGE.contains("route_script_list_special_entry("),
        "live filter input must route ScriptList special entries through the shared helper"
    );
    assert!(
        FILTER_UPDATES.contains("route_script_list_special_entry("),
        "setFilter must route ScriptList special entries through the shared helper"
    );
}

#[test]
fn set_filter_text_immediate_does_not_agent_chat_only_special_case_sigil_routing() {
    let sig = FILTER_UPDATES
        .find("pub(crate) fn set_filter_text_immediate")
        .expect("set_filter_text_immediate exists");
    let body = &FILTER_UPDATES[sig..sig + 3500.min(FILTER_UPDATES.len() - sig)];

    assert!(
        !body.contains("ScriptListSpecialEntry::AgentChatMentionPicker =>"),
        "setFilter must not special-case only Agent Chat mention routing"
    );
    assert!(
        !body.contains("ScriptListSpecialEntry::AgentChatProfilePicker =>"),
        "setFilter must not special-case only Agent Chat profile routing"
    );
}

#[test]
fn bare_at_stays_in_script_list_spine_route() {
    assert!(
        !FILTER_CORE.contains("AgentChatMentionPicker")
            && !FILTER_CHANGE.contains(concat!("open_tab_ai_agent_chat_with_", "mention_picker")),
        "bare @ in ScriptList must stay in the shared Spine/main-list route, not open Agent Chat"
    );
    assert!(
        FILTER_CORE.contains("\"@\"") && FILTER_CORE.contains("None"),
        "the bare @ special-entry test should pin @ as a non-handoff sigil"
    );
}

#[test]
fn bang_is_quick_terminal_sigil_and_greater_than_is_plain_filter() {
    assert!(
        FILTER_CORE.contains("\"!\" => Some(ScriptListSpecialEntry::QuickTerminal)")
            || FILTER_CORE.contains("\"!\"=>Some(ScriptListSpecialEntry::QuickTerminal)"),
        "bare ! must route to Quick Terminal through the shared ScriptList special-entry router"
    );
    assert!(
        !FILTER_CORE.contains("\">\" => Some(ScriptListSpecialEntry::QuickTerminal)")
            && !FILTER_CORE.contains("\">\"=>Some(ScriptListSpecialEntry::QuickTerminal)"),
        "bare > must not route to Quick Terminal"
    );
    assert!(
        FILTER_CORE.contains("\">\" => None") || FILTER_CORE.contains("\">\"=>None"),
        "bare > should be explicitly pinned as a non-handoff ScriptList query"
    );
}

#[test]
fn bang_prefixed_queries_are_not_quick_terminal_handoffs() {
    assert!(
        FILTER_CORE.contains("special_entry_from_script_list_filter(\"!dep\")")
            && FILTER_CORE.contains("None"),
        "!dep must remain normal query text, not a Quick Terminal handoff"
    );
}
