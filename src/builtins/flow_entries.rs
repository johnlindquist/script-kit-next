use super::{BuiltInEntry, BuiltInFeature};

pub(super) fn push_flow_entries(entries: &mut Vec<BuiltInEntry>) {
    // THE visible flow-first entry point (fusion-ultra 2026-07-09: one
    // visible "Flows" entry seeded from Flash beats four hidden built-ins).
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flows",
        "Flows",
        "Find and run mdflow flows — Enter runs, ⇧↵ backgrounds, ⌘↵ supervises",
        vec![
            "flow", "flows", "mdflow", "md", "run", "agent", "agentic", "launcher",
        ],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::Flash),
        "zap",
    ));

    // Hidden query-only variations for by-feel comparison.
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flow-ux-flash",
        "Flow UX — Flash",
        "Fastest list: Enter runs the flow inline, Esc backgrounds it",
        vec!["flow", "flows", "mdflow", "flash", "launcher", "ux"],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::Flash),
        "zap",
    ));
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flow-ux-dispatch",
        "Flow UX — Dispatch",
        "Fire-and-forget: Enter backgrounds the run and keeps you in the list",
        vec!["flow", "flows", "mdflow", "dispatch", "launcher", "ux"],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::Dispatch),
        "send",
    ));
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flow-ux-lens",
        "Flow UX — Lens",
        "Confidence first: split list with a free resolved-command preview",
        vec![
            "flow", "flows", "mdflow", "lens", "preview", "launcher", "ux",
        ],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::Lens),
        "eye",
    ));
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flow-ux-mission-control",
        "Flow UX — Mission Control",
        "Runs-first workspace: opens the Flow Manager with a compact picker",
        vec![
            "flow", "flows", "mdflow", "mission", "control", "runs", "ux",
        ],
        BuiltInFeature::FlowUxVariant(crate::flows::model::FlowUxVariant::MissionControl),
        "layout-grid",
    ));
    entries.push(BuiltInEntry::new_with_icon(
        "builtin/flow-manager",
        "Flow Manager",
        "Supervise running flows: output, steps, cancel, rerun",
        vec!["flow", "flows", "runs", "manager", "agents", "mdflow"],
        BuiltInFeature::FlowManager,
        "list-checks",
    ));
}
