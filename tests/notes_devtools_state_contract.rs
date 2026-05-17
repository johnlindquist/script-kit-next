//! Source-level contract for Notes DevTools runtime state.
//!
//! Notes UX bugs often depend on active note identity, dirty state, selection,
//! focus surface, and autosize state. The DevTools protocol must expose those
//! receipts through a redacted target-scoped state envelope, not scripts that
//! guess from screenshots or hard-coded recipes.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const NOTES_NAVIGATION: &str = include_str!("../src/notes/window/navigation.rs");
const QUERY_OPS_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const QUERY_OPS_CONSTRUCTORS: &str =
    include_str!("../src/protocol/message/constructors/query_ops.rs");
const DEVTOOLS_NOTES: &str = include_str!("../scripts/devtools/notes.ts");
const DEVTOOLS_COVERAGE: &str = include_str!("../scripts/devtools/coverage.ts");
const DEVTOOLS_LAYOUT: &str = include_str!("../scripts/devtools/layout.ts");
const DEVTOOLS_SCHEMA: &str = include_str!("../scripts/devtools/schema.ts");
const NOTES_STORAGE: &str = include_str!("../src/notes/storage.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/actions/dialog.rs");
const COMMAND_BAR: &str = include_str!("../src/actions/command_bar.rs");
const NOTES_FOCUS: &str = include_str!("../src/notes/window/focus.rs");
const NOTES_WINDOW: &str = include_str!("../src/notes/window.rs");
const NOTES_PANELS: &str = include_str!("../src/notes/window/panels.rs");
const BATCH_WAIT: &str = include_str!("../src/protocol/types/batch_wait.rs");

#[test]
fn state_result_exposes_redacted_notes_envelope() {
    for needle in [
        "notes_state: Option<serde_json::Value>",
        "rename = \"notes\"",
        "notes_state",
    ] {
        assert!(
            QUERY_OPS_VARIANTS.contains(needle) || QUERY_OPS_CONSTRUCTORS.contains(needle),
            "StateResult must carry Notes runtime state through protocol field: {needle}"
        );
    }

    assert!(
        PROMPT_HANDLER.contains("GetStateTargetResolution::Notes")
            && PROMPT_HANDLER.contains("automation_state(cx)")
            && PROMPT_HANDLER.contains("\"notes\".to_string()"),
        "getState must resolve Notes targets and return a notes stateResult"
    );
}

#[test]
fn notes_automation_state_is_runtime_derived_and_redacted() {
    for needle in [
        "pub(crate) fn automation_state(&self, cx: &gpui::App)",
        "\"activeNoteId\"",
        "\"dirtyState\"",
        "\"selectionRange\"",
        "\"focusSurface\"",
        "\"autoSizingEnabled\"",
        "\"lastWindowHeight\"",
        "\"notesAcpGeneration\"",
        "\"redacted\": true",
        "\"storage\"",
        "\"commandBars\"",
        "\"shortcutRegistry\"",
        "devtools_text_fingerprint",
    ] {
        assert!(
            NOTES_NAVIGATION.contains(needle),
            "Notes automation_state must expose redacted runtime receipt field: {needle}"
        );
    }

    assert!(
        !NOTES_NAVIGATION.contains("\"content\": note.content")
            && !NOTES_NAVIGATION.contains("\"title\": note.title"),
        "Notes DevTools state must not expose raw note content or title"
    );
}

#[test]
fn notes_state_exposes_focus_owner_transition_timeline() {
    for needle in [
        "struct NotesFocusTransition",
        "focus_transition_generation",
        "focus_transition_log",
    ] {
        assert!(
            NOTES_WINDOW.contains(needle),
            "NotesApp must store focus transition timeline state: {needle}"
        );
    }

    for needle in [
        "fn record_focus_transition(",
        "\"requested\"",
        "\"drain-pending\"",
        "\"restore-after-dialog\"",
        "\"applied\"",
        "MAX_FOCUS_TRANSITIONS",
    ] {
        assert!(
            NOTES_FOCUS.contains(needle),
            "Notes focus handling must record transition phase: {needle}"
        );
    }

    for needle in [
        "fn automation_focus_transition_timeline(&self)",
        "\"focusTransitions\"",
        "\"previousSurface\"",
        "\"commandBarOpen\"",
        "\"noteSwitcherOpen\"",
        "\"hasActiveDialog\"",
    ] {
        assert!(
            NOTES_NAVIGATION.contains(needle),
            "Notes state must expose focus transition timeline field: {needle}"
        );
    }

    for needle in [
        "notesState.focusTransitions.generation",
        "notesState.focusTransitions.entries",
        "shortcutActivation.channel",
        "getState(target notes) focus owner transition timeline",
        "Notes native shortcut activation parity receipt",
    ] {
        assert!(
            DEVTOOLS_SCHEMA.contains(needle)
                || DEVTOOLS_COVERAGE.contains(needle)
                || DEVTOOLS_NOTES.contains(needle),
            "Notes DevTools CLI/schema/coverage must report focus transition timeline: {needle}"
        );
    }
}

#[test]
fn notes_state_exposes_shortcut_registry_and_focus_owner_scope() {
    for needle in [
        "fn automation_shortcut_registry(&self)",
        "\"activeScope\"",
        "\"currentFocusSurface\"",
        "\"pendingFocusSurface\"",
        "\"modalGuard\"",
        "\"actionsPanel\"",
        "\"noteSwitcher\"",
        "\"embeddedAcp\"",
        "\"Cmd+K\"",
        "\"Cmd+P\"",
        "\"Cmd+Enter\"",
    ] {
        assert!(
            NOTES_NAVIGATION.contains(needle),
            "Notes state must expose shortcut/focus-owner registry field: {needle}"
        );
    }

    for needle in [
        "notesState.shortcutRegistry.activeScope",
        "notesState.shortcutRegistry.scopes",
        "getState(target notes) shortcut registry scopes",
        "Notes native shortcut activation parity receipt",
    ] {
        assert!(
            DEVTOOLS_SCHEMA.contains(needle)
                || DEVTOOLS_COVERAGE.contains(needle)
                || DEVTOOLS_NOTES.contains(needle),
            "Notes DevTools CLI/schema/coverage must report shortcut registry field: {needle}"
        );
    }
}

#[test]
fn notes_batch_supports_target_scoped_open_actions() {
    for needle in ["OpenActions", "openActions"] {
        assert!(
            BATCH_WAIT.contains(needle) || PROMPT_HANDLER.contains(needle),
            "Batch protocol must define target-scoped Notes actions primitive: {needle}"
        );
    }

    for needle in [
        "supported_commands: &[\"setInput\", \"openActions\", \"waitFor\"]",
        "protocol::BatchCommand::OpenActions",
        "transaction_notes_open_actions",
        "window.defer(cx, move |window, cx|",
        "app.open_actions_panel(window, cx)",
    ] {
        assert!(
            PROMPT_HANDLER.contains(needle),
            "Notes batch routing must open actions on the resolved Notes target: {needle}"
        );
    }

    assert!(
        NOTES_PANELS.contains("pub(crate) fn open_actions_panel"),
        "Notes actions opener must be available to the DevTools batch route"
    );

    for needle in [
        "commands: [{ type: \"openActions\" }]",
        "protocol.batch.openActions",
        "target-scoped batch openActions receipt",
    ] {
        assert!(
            DEVTOOLS_NOTES.contains(needle) || DEVTOOLS_COVERAGE.contains(needle),
            "Notes DevTools CLI/coverage must prefer target-scoped openActions receipts: {needle}"
        );
    }
}

#[test]
fn notes_state_exposes_redacted_command_bar_runtime_state() {
    for needle in [
        "pub(crate) fn automation_state(&self, surface: &str)",
        "\"route\"",
        "\"stack\"",
        "\"filteredCount\"",
        "\"selectedActionId\"",
        "\"visibleSample\"",
        "\"searchTextFingerprint\"",
        "\"registeredDrillDownRouteCount\"",
    ] {
        assert!(
            ACTIONS_DIALOG.contains(needle) || COMMAND_BAR.contains(needle),
            "Actions/CommandBar DevTools state must expose redacted runtime field: {needle}"
        );
    }

    for needle in [
        "\"commandBars\"",
        "self.command_bar.automation_state(\"notes.actions\", cx)",
        "self.note_switcher.automation_state(\"notes.switcher\", cx)",
    ] {
        assert!(
            NOTES_NAVIGATION.contains(needle),
            "Notes state must include command bar runtime state: {needle}"
        );
    }

    for needle in [
        "notesState.commandBars.actions.dialog.route.stack",
        "notesState.commandBars.actions.dialog.actions.filteredCount",
        "getState(target notes) command bar route stack",
    ] {
        assert!(
            DEVTOOLS_SCHEMA.contains(needle)
                || DEVTOOLS_COVERAGE.contains(needle)
                || DEVTOOLS_NOTES.contains(needle),
            "Notes DevTools CLI/schema/coverage must report command bar field: {needle}"
        );
    }
}

#[test]
fn notes_state_exposes_redacted_storage_generation() {
    for needle in [
        "NOTES_STORAGE_GENERATION",
        "pub(crate) fn automation_storage_identity()",
        "\"generation\"",
        "\"rootSearchCacheGeneration\"",
        "\"dbPathFingerprint\"",
        "\"testSandbox\"",
        "invalidate_root_notes_search_cache()",
    ] {
        assert!(
            NOTES_STORAGE.contains(needle),
            "Notes storage must expose redacted generation/sandbox identity: {needle}"
        );
    }

    for needle in [
        "notesState.storage.generation",
        "notesState.commandBars.actions.dialog.route.stack",
        "notesState.shortcutRegistry.activeScope",
        "notesState.focusTransitions.generation",
        "shortcutActivation.assertions.actionsPanelOpened",
        "receipts.layout",
        "notes storage generation and redacted sandbox identity",
    ] {
        assert!(
            DEVTOOLS_SCHEMA.contains(needle)
                || DEVTOOLS_NOTES.contains(needle)
                || DEVTOOLS_COVERAGE.contains(needle),
            "Notes DevTools must report storage/layout receipt field: {needle}"
        );
    }
}

#[test]
fn notes_target_has_target_scoped_layout_info() {
    for needle in [
        "pub(crate) fn automation_layout_info(",
        "LayoutInfo",
        "NotesWindow",
        "NotesTitlebar",
        "NotesEditor",
        "NotesFooter",
        "NotesActionsPanel",
        "NotesBrowsePanel",
    ] {
        assert!(
            NOTES_NAVIGATION.contains(needle),
            "Notes must expose target-scoped layout component: {needle}"
        );
    }

    assert!(
        PROMPT_HANDLER.contains("AutomationWindowKind::Notes")
            && PROMPT_HANDLER.contains("automation_layout_info(&resolved)")
            && !PROMPT_HANDLER.contains("resolve_main_only_target(&request_id, \"getLayoutInfo\""),
        "getLayoutInfo must route Notes targets to Notes automation layout instead of main-only rejection"
    );

    for needle in [
        "window: {",
        "viewport: {",
        "pressure: {",
        "clientHeight",
        "scrollHeight",
        "canScrollY",
        "hiddenContentHeight",
    ] {
        assert!(
            DEVTOOLS_LAYOUT.contains(needle),
            "layout CLI must expose Chrome-style target-scoped layout field: {needle}"
        );
    }
}

#[test]
fn notes_cli_consumes_runtime_state_instead_of_guessing_everything_from_elements() {
    for needle in [
        "type: \"getState\"",
        "runtimeState.notes",
        "notesState.selectionRange",
        "notesState.layout.editorRegion",
        "notesState.storage.generation",
        "receipts.state",
        "receipts.layout",
        "redacted runtime Notes state",
    ] {
        assert!(
            DEVTOOLS_NOTES.contains(needle)
                || DEVTOOLS_COVERAGE.contains(needle)
                || DEVTOOLS_SCHEMA.contains(needle),
            "notes CLI/coverage must consume target-scoped runtime state: {needle}"
        );
    }

    assert!(
        DEVTOOLS_COVERAGE.contains("getState(target notes) redacted active note"),
        "coverage should mark active note, dirty state, and selection as protocol-supported"
    );
}
