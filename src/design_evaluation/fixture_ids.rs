//! Data-only fixture identities shared by the compiled catalogue and native constructors.

pub(crate) fn main_fixture_ids() -> &'static [&'static str] {
    &[
        "main.script-list",
        "main.subject-tabs",
        "main.launcher-basics",
        "main.launcher-ready",
        "main.launcher-cold",
        "main.launcher-reading",
        "main.launcher-watch",
        "main.root-search-frame-stability",
        "main-search-contract",
        "main.about",
        "main.clipboard-history",
        "main.app-launcher",
        "main.window-switcher",
        "main.browser-tabs",
        "main.file-search-mini",
        "main.file-search-full",
        "main.profile-search",
        "main.theme-chooser",
        "main.emoji-picker",
        "main.script-issues",
        "main.sdk-reference",
        "main.tips",
        "main.script-template-catalog",
        "main.browse-kits",
        "main.migrate-v1",
        "main.migrate-v1-scanning",
        "main.migrate-v1-porting",
        "main.migrate-v1-done",
        "main.migrate-v1-unavailable",
        "main.installed-kits",
        "main.process-manager",
        "main.search-ai-presets",
        "main.settings",
        "main.permissions-wizard",
        "main.favorites-browse",
        "main.current-app-commands",
        "main.browser-history",
        "main.notes-browse",
        "main.notes-browse-no-match",
        "main.notes-browse-loading",
        "main.notes-browse-failed",
        "main.notes-browse-empty",
        "main.menu-syntax-trigger",
        "main.menu-syntax-object",
        "main.menu-syntax-history",
        "main.browse-kits-loading",
        "main.browse-kits-failed",
    ]
}

pub(crate) const MAIN_OVERLAY_FIXTURE_IDS: &[&str] = &[
    "main-overlay.alias",
    "main-overlay.tab-ai-saved",
    "main-overlay.logs",
    "main-overlay.loading",
    "main-overlay.effects",
    "main-overlay.debug-grid",
];

pub(crate) const PROMPT_FIXTURE_IDS: &[&str] = &[
    "prompt.arg",
    "prompt.mini",
    "prompt.micro",
    "prompt.div",
    "prompt.form",
    "prompt.fields",
    "prompt.editor",
    "prompt.select",
    "prompt.path",
    "prompt.env",
    "prompt.drop",
    "prompt.template",
    "prompt.hotkey",
    "prompt.chat",
    "prompt.term",
    "prompt.naming",
    "prompt.webcam",
    "prompt.scratch-pad",
    "prompt.quick-terminal",
    "prompt.create-preset",
    "prompt.creation-feedback",
];

pub(crate) const DAY_PAGE_FIXTURE_IDS: &[&str] = &[
    "day-page.today",
    "day-page.shelf",
    "day-page.fragment",
    "day-page.switcher",
];

pub(crate) const FLOW_FIXTURE_IDS: &[&str] = &[
    "flow.desk.flash",
    "flow.desk.dispatch",
    "flow.desk.lens",
    "flow.session",
];

pub(crate) const SDK_CHAT_FIXTURE_IDS: &[&str] = &[
    "sdk-chat.empty",
    "sdk-chat.streaming",
    "sdk-chat.retryable-failure",
];

pub(crate) const SECONDARY_FIXTURE_IDS: &[&str] = &["secondary.snap"];

pub(crate) fn fixture_ids() -> impl Iterator<Item = &'static str> {
    main_fixture_ids()
        .iter()
        .copied()
        .chain(MAIN_OVERLAY_FIXTURE_IDS.iter().copied())
        .chain(PROMPT_FIXTURE_IDS.iter().copied())
        .chain(DAY_PAGE_FIXTURE_IDS.iter().copied())
        .chain(FLOW_FIXTURE_IDS.iter().copied())
        .chain(SDK_CHAT_FIXTURE_IDS.iter().copied())
        .chain(SECONDARY_FIXTURE_IDS.iter().copied())
}
