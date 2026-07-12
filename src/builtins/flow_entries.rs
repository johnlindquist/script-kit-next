use super::{BuiltInEntry, BuiltInFeature};

pub(super) fn push_flow_entries(entries: &mut Vec<BuiltInEntry>) {
    // THE flow-first entry point: the Conversation Desk (fusion-ultra
    // 2026-07-09 redesign). Every flow is an agent identity — Enter starts
    // (or resumes) a conversation in the main window, ⇧↵ runs once in the
    // background, active sessions live at the top of the same list. The
    // detached Flow Manager and the four UX variants are gone.
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flows",
        "Flows",
        "Talk to your mdflow agents — Enter converses, ⇧↵ runs once, sessions stay live",
        vec![
            "flow", "flows", "mdflow", "md", "run", "agent", "agentic", "launcher", "desk",
            "converse", "sessions",
        ],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::Flash),
        "flow",
    ));

    // Creation must be discoverable from the words a user actually types
    // ("new flow", "create a flow"), not only from the desk's trailing row.
    // Locked by tests/launcher_discoverability_contract.rs — keep the name,
    // description, and keyword phrases in step with that bar.
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/new-flow",
        "New Flow",
        "Describe an agent in plain English (md create)",
        vec![
            "new",
            "flow",
            "flows",
            "create",
            "create flow",
            "create a flow",
            "make a flow",
            "add flow",
            "agent",
            "mdflow",
            "md create",
            "wizard",
        ],
        BuiltInFeature::NewFlow,
        "plus",
    ));
}
