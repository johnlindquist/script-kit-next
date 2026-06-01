const FILTER_CHANGE: &str = include_str!("../src/app_impl/filter_input_change.rs");
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
fn set_filter_text_immediate_does_not_acp_only_special_case_sigil_routing() {
    let sig = FILTER_UPDATES
        .find("pub(crate) fn set_filter_text_immediate")
        .expect("set_filter_text_immediate exists");
    let body = &FILTER_UPDATES[sig..sig + 3500.min(FILTER_UPDATES.len() - sig)];

    assert!(
        !body.contains("ScriptListSpecialEntry::AcpMentionPicker =>"),
        "setFilter must not special-case only ACP mention routing"
    );
    assert!(
        !body.contains("ScriptListSpecialEntry::AcpProfilePicker =>"),
        "setFilter must not special-case only ACP profile routing"
    );
}
