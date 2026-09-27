//! Data-only catalogue over the production fixture owners. No constructor,
//! filesystem read, provider startup or window effect is allowed here.
use super::fixture_ids;
use crate::protocol::FixtureDescriptor;

const MAIN_OWNER: &str = "src/main_entry/app_run_setup.rs";
const NATIVE_EXCLUSIONS: &[&str] = &[
    "WindowServer composition and desktop capture",
    "AppKit material, native glyph pixels and calibrated native entry/exit motion",
    "Native focus, activation, IME and global input",
    "System clipboard, device access, TCC and external-app insertion",
    "Live providers, credentials, network and operator storage",
];

fn descriptor(
    id: &str,
    family: &str,
    root: &str,
    owner: &str,
    route: Option<&str>,
    presentation: &str,
) -> FixtureDescriptor {
    let (semantic_surface, controls) = semantic_contract(id, route);
    FixtureDescriptor {
        id: id.into(),
        family: family.into(),
        root: root.into(),
        owner: owner.into(),
        factory_owners: vec![owner.into()],
        parent_fixture_id: None,
        proof_boundary: "owned-production-runtime".into(),
        native_exclusions: NATIVE_EXCLUSIONS
            .iter()
            .map(|value| (*value).into())
            .collect(),
        app_view_variant: route.map(str::to_owned),
        presentation_owner: Some(presentation.into()),
        surface_variant: None,
        expected_semantic_surface: semantic_surface.into(),
        required_semantic_ids: controls.iter().map(|id| (*id).into()).collect(),
    }
}

// Requirements name controls emitted by the production collectors, not fixture
// render substitutes. A trailing * denotes an instance-specific semantic suffix.
fn semantic_contract(id: &str, route: Option<&str>) -> (&'static str, &'static [&'static str]) {
    match id {
        "prompt.fields" => return ("fields-prompt", &["input:fields-*"]),
        "main-overlay.alias" => return ("scriptList", &["alias-*"]),
        "main-overlay.tab-ai-save-offer" => return ("scriptList", &["tab-ai-save-offer"]),
        "main-overlay.root-dialog" => return ("scriptList", &["root-dialog:*"]),
        "main-overlay.logs" => return ("scriptList", &["main-log-panel"]),
        "main-overlay.loading" => return ("scriptList", &["main-loading"]),
        "main-overlay.effects" => return ("scriptList", &["main-background-effect"]),
        "main-overlay.debug-grid" => return ("scriptList", &["main-debug-grid"]),
        _ => {}
    }
    match route {
        Some("ScriptList") => ("scriptList", &["input:filter"]),
        Some("About") => ("about", &["panel:about"]),
        Some("ArgPrompt") => ("argPrompt", &["input:filter", "choice:*"]),
        Some("MiniPrompt") => ("miniPrompt", &["input:filter", "choice:*"]),
        Some("MicroPrompt") => ("microPrompt", &["input:filter"]),
        Some("DivPrompt") => ("divPrompt", &["panel:div-prompt"]),
        Some("FormPrompt") => ("form-prompt", &["input:form-*"]),
        Some("EditorPrompt") => ("editor-prompt", &["input:editor-language"]),
        Some("SelectPrompt") => ("selectPrompt", &["input:*", "choice:*"]),
        Some("PathPrompt") => ("path-prompt", &["input:*"]),
        Some("EnvPrompt") => ("env-prompt", &["input:*"]),
        Some("DropPrompt") => (
            "drop-prompt",
            &["list:dropped-files", "choice:dropped-file-*"],
        ),
        Some("TemplatePrompt") => ("template-prompt", &["input:*"]),
        Some("HotkeyPrompt") => ("hotkey-prompt", &["panel:hotkey-capture"]),
        Some("ChatPrompt") => ("chat-prompt", &["input:*", "list:chat-messages"]),
        Some("TermPrompt") => ("term-prompt", &["list:term-lines", "choice:*"]),
        Some("NamingPrompt") => ("naming-prompt", &["input:*"]),
        Some("WebcamView") => ("webcam", &["panel:webcam"]),
        Some("ScratchPadView") => ("scratch-pad", &["input:scratch-pad-language"]),
        Some("QuickTerminalView") => ("quick-terminal", &["list:quick-terminal-lines", "choice:*"]),
        Some("CreateAiPresetView") => ("createAiPreset", &["panel:create-ai-preset"]),
        Some("CreationFeedback") => ("creationFeedback", &["creation-feedback:*"]),
        Some("ClipboardHistoryView") => (
            "clipboardHistory",
            &["input:clipboard-filter", "list:clipboard-history"],
        ),
        Some("AppLauncherView") => ("appLauncher", &["input:app-filter", "list:apps"]),
        Some("WindowSwitcherView") => ("windowSwitcher", &["input:window-filter", "list:windows"]),
        Some("BrowserTabsView") => (
            "browserTabs",
            &["input:browser-tabs-filter", "list:browser-tabs"],
        ),
        Some("FileSearchView") => (
            "fileSearch",
            &["input:file-search-input", "list:file-results", "choice:*"],
        ),
        Some("ProfileSearchView") => (
            "profileSearch",
            &["input:profile-search-input", "profile-search-row:*"],
        ),
        Some("ThemeChooserView") => (
            "themeChooser",
            &["input:theme-filter", "control:theme-chooser:panel-mode"],
        ),
        Some("EmojiPickerView") => ("emojiPicker", &["input:emoji-filter", "list:emoji-results"]),
        Some("ScriptIssuesView") => ("scriptIssues", &["panel:script-issues"]),
        Some("SdkReferenceView") => (
            "sdkReference",
            &["input:sdk-reference-filter", "list:sdk-functions"],
        ),
        Some("TipsView") => ("tips", &["input:tips-filter", "list:tips"]),
        Some("ScriptTemplateCatalogView") => (
            "scriptTemplateCatalog",
            &["input:*", "list:script-templates"],
        ),
        Some("BrowseKitsView") => ("browseKits", &["input:kit-search", "list:kit-results"]),
        Some("MigrateV1View") => (
            "migrateV1",
            &["input:migrate-v1-filter", "list:migrate-v1-results"],
        ),
        Some("InstalledKitsView") => ("installedKits", &["input:*", "list:installed-kits"]),
        Some("ProcessManagerView") => (
            "processManager",
            &["input:process-filter", "list:processes"],
        ),
        Some("SearchAiPresetsView") => (
            "searchAiPresets",
            &["input:ai-presets-filter", "list:ai-presets"],
        ),
        Some("SettingsView") => ("settings", &["input:settings-filter", "list:settings"]),
        Some("PermissionsWizardView") => ("permissionsWizard", &["panel:permissions-wizard"]),
        Some("FavoritesBrowseView") => (
            "favoritesBrowse",
            &["input:favorites-filter", "list:favorites"],
        ),
        Some("CurrentAppCommandsView") => {
            ("currentAppCommands", &["input:*", "list:menu-commands"])
        }
        Some("BrowserHistoryView") => ("browserHistory", &["input:*", "list:browser-history"]),
        Some("NotesBrowseView") => ("notesBrowse", &["input:notes-browse-filter"]),
        Some("DayPage") => ("dayPage", &["input:day-page-editor"]),
        Some("FlowUxView") => ("flowDesk", &["input:flow-ux-filter", "flow-desk:state"]),
        Some("FlowSessionView") => ("flow-session", &["input:*"]),
        None => match id {
            "notes.editor" => ("notes", &["input:notes-editor"]),
            "notes.root-dialog" => ("notes", &["root-dialog:*"]),
            "notes.root-notification" => ("notes", &["root-notification:*"]),
            "notes.actions" | "notes.recent-switcher" | "day-page.switcher" => {
                ("actionsDialog", &["input:actions-search", "list:actions"])
            }
            "secondary.snap" => ("snapOverlay", &["panel:snap-overlay", "snap:target:*"]),
            _ => panic!("missing fixture semantic contract: {id}"),
        },
        _ => panic!("missing fixture route contract: {id}"),
    }
}

fn main_descriptor(id: &str, family: &str, route: &str, presentation: &str) -> FixtureDescriptor {
    descriptor(id, family, "main", MAIN_OWNER, Some(route), presentation)
}

fn declared_descriptor(id: &str) -> Option<FixtureDescriptor> {
    let (route, owner) = match id {
        "main.script-list"
        | "main.root-search-frame-stability"
        | "main-search-contract"
        | "main.menu-syntax-trigger"
        | "main.menu-syntax-object"
        | "main.menu-syntax-history" => ("ScriptList", "src/render_script_list"),
        "main-overlay.alias" => (
            "ScriptList",
            "src/app_impl/alias_input.rs::show_alias_input",
        ),
        "main-overlay.tab-ai-save-offer" => (
            "ScriptList",
            "src/app_impl/agent_handoff/mod.rs::open_tab_ai_save_offer",
        ),
        "main-overlay.root-dialog" => (
            "ScriptList",
            "vendor/gpui-kit/gpui-component/src/root.rs::Root::render_dialog_layer",
        ),
        "main-overlay.logs" => ("ScriptList", "src/render_script_list/mod.rs::log_panel"),
        "main-overlay.loading" => (
            "ScriptList",
            "src/components/braille_loading.rs::constellation_loading_layer",
        ),
        "main-overlay.effects" => ("ScriptList", "src/effects.rs::background_effect_layer"),
        "main-overlay.debug-grid" => ("ScriptList", "src/debug_grid/mod.rs::render_grid_overlay"),
        "main.about" => ("About", "src/about/render.rs"),
        "main.clipboard-history" => ("ClipboardHistoryView", "src/render_builtins/clipboard.rs"),
        "main.app-launcher" => ("AppLauncherView", "src/render_builtins/app_launcher.rs"),
        "main.window-switcher" => (
            "WindowSwitcherView",
            "src/render_builtins/window_switcher.rs",
        ),
        "main.browser-tabs" => ("BrowserTabsView", "src/render_builtins/browser_tabs.rs"),
        "main.file-search-mini" | "main.file-search-full" => {
            ("FileSearchView", "src/render_builtins/file_search.rs")
        }
        "main.profile-search" => ("ProfileSearchView", "src/render_builtins/profile_search.rs"),
        "main.theme-chooser" => ("ThemeChooserView", "src/render_builtins/theme_chooser.rs"),
        "main.emoji-picker" => ("EmojiPickerView", "src/render_builtins/emoji_picker.rs"),
        "main.script-issues" => (
            "ScriptIssuesView",
            "src/render_prompts/other.rs::render_script_issues_view",
        ),
        "main.sdk-reference" => ("SdkReferenceView", "src/render_builtins/sdk_reference.rs"),
        "main.tips" => ("TipsView", "src/render_builtins/tips.rs"),
        "main.script-template-catalog" => (
            "ScriptTemplateCatalogView",
            "src/render_builtins/script_templates.rs",
        ),
        "main.browse-kits" | "main.browse-kits-loading" | "main.browse-kits-failed" => (
            "BrowseKitsView",
            "src/render_builtins/kit_store.rs::render_browse_kits",
        ),
        "main.migrate-v1"
        | "main.migrate-v1-scanning"
        | "main.migrate-v1-porting"
        | "main.migrate-v1-done"
        | "main.migrate-v1-unavailable" => ("MigrateV1View", "src/render_builtins/migrate_v1.rs"),
        "main.installed-kits" => (
            "InstalledKitsView",
            "src/render_builtins/kit_store.rs::render_installed_kits",
        ),
        "main.process-manager" => (
            "ProcessManagerView",
            "src/render_builtins/process_manager.rs",
        ),
        "main.search-ai-presets" => ("SearchAiPresetsView", "src/render_builtins/ai_presets.rs"),
        "main.settings" => ("SettingsView", "src/render_builtins/settings.rs"),
        "main.permissions-wizard" => (
            "PermissionsWizardView",
            "src/render_builtins/permissions_wizard.rs",
        ),
        "main.favorites-browse" => ("FavoritesBrowseView", "src/render_builtins/favorites.rs"),
        "main.current-app-commands" => (
            "CurrentAppCommandsView",
            "src/render_builtins/current_app_commands.rs",
        ),
        "main.browser-history" => (
            "BrowserHistoryView",
            "src/render_builtins/browser_history.rs",
        ),
        "main.notes-browse"
        | "main.notes-browse-no-match"
        | "main.notes-browse-loading"
        | "main.notes-browse-failed"
        | "main.notes-browse-empty" => ("NotesBrowseView", "src/render_builtins/notes_browse.rs"),
        "prompt.arg" => ("ArgPrompt", "src/render_prompts/arg/render.rs"),
        "prompt.mini" => ("MiniPrompt", "src/render_prompts/mini.rs"),
        "prompt.micro" => ("MicroPrompt", "src/render_prompts/micro.rs"),
        "prompt.div" => ("DivPrompt", "src/prompts/div"),
        "prompt.form" | "prompt.fields" => ("FormPrompt", "src/form_prompt.rs"),
        "prompt.editor" => ("EditorPrompt", "src/editor"),
        "prompt.select" => ("SelectPrompt", "src/prompts/select"),
        "prompt.path" => ("PathPrompt", "src/prompts/path"),
        "prompt.env" => ("EnvPrompt", "src/prompts/env"),
        "prompt.drop" => ("DropPrompt", "src/prompts/drop.rs"),
        "prompt.template" => ("TemplatePrompt", "src/prompts/template"),
        "prompt.hotkey" => ("HotkeyPrompt", "src/components/shortcut_recorder.rs"),
        "prompt.chat" => ("ChatPrompt", "src/prompts/chat"),
        "prompt.term" => ("TermPrompt", "src/term_prompt"),
        "prompt.naming" => ("NamingPrompt", "src/prompts/naming"),
        "prompt.webcam" => ("WebcamView", "src/prompts/webcam.rs"),
        "prompt.scratch-pad" => ("ScratchPadView", "src/editor"),
        "prompt.quick-terminal" => ("QuickTerminalView", "src/term_prompt"),
        "prompt.create-preset" => (
            "CreateAiPresetView",
            "src/render_builtins/ai_presets.rs::render_create_ai_preset",
        ),
        "prompt.creation-feedback" => (
            "CreationFeedback",
            "src/render_prompts/other.rs::render_creation_feedback",
        ),
        "day-page.today" | "day-page.shelf" | "day-page.fragment" => {
            ("DayPage", "src/day_page/render.rs")
        }
        "flow.desk.flash" | "flow.desk.dispatch" | "flow.desk.lens" => {
            ("FlowUxView", "src/render_builtins/flow_ux.rs")
        }
        "flow.session" => ("FlowSessionView", "src/flows/session.rs"),
        "sdk-chat.empty" | "sdk-chat.streaming" | "sdk-chat.retryable-failure" => {
            ("ChatPrompt", "src/prompts/chat")
        }
        _ => return secondary_descriptor(id),
    };
    let family = if fixture_ids::MAIN_OVERLAY_FIXTURE_IDS.contains(&id) {
        "mainOverlay"
    } else if fixture_ids::PROMPT_FIXTURE_IDS.contains(&id) {
        "prompt"
    } else if fixture_ids::DAY_PAGE_FIXTURE_IDS.contains(&id) {
        "dayPage"
    } else if fixture_ids::FLOW_FIXTURE_IDS.contains(&id) {
        "flow"
    } else if fixture_ids::SDK_CHAT_FIXTURE_IDS.contains(&id) {
        "sdkChat"
    } else {
        "main"
    };
    let mut fixture = main_descriptor(id, family, route, owner);
    if family == "mainOverlay" {
        fixture.parent_fixture_id = Some("main.script-list".into());
    }
    fixture.surface_variant = match id {
        "main.file-search-mini" => Some("FileSearchMini".into()),
        "main.file-search-full" => Some("FileSearchFull".into()),
        _ => None,
    };
    Some(fixture)
}

fn secondary_descriptor(id: &str) -> Option<FixtureDescriptor> {
    let (family, root, owner, presentation, parent) = match id {
        "notes.editor" => (
            "notes",
            "notes",
            "src/notes/window/window_ops.rs",
            "src/notes/window/render.rs",
            None,
        ),
        "notes.actions" => (
            "notesAuxiliary",
            "secondary",
            "src/components/staged_actions.rs",
            "src/notes/window/panels.rs::open_actions_panel",
            Some("notes.editor"),
        ),
        "notes.recent-switcher" => (
            "notesAuxiliary",
            "secondary",
            "src/components/staged_actions.rs",
            "src/notes/window/panels.rs::open_browse_panel",
            Some("notes.editor"),
        ),
        "day-page.switcher" => (
            "dayPageAuxiliary",
            "secondary",
            "src/components/staged_actions.rs",
            "src/main_sections/day_page_switcher.rs::open_day_switcher",
            Some("day-page.today"),
        ),
        "notes.root-dialog" => (
            "notesAuxiliary",
            "notes",
            "src/notes/window/window_ops.rs",
            "vendor/gpui-kit/gpui-component/src/root.rs::Root::render_dialog_layer",
            Some("notes.editor"),
        ),
        "notes.root-notification" => (
            "notesAuxiliary",
            "notes",
            "src/notes/window/window_ops.rs",
            "vendor/gpui-kit/gpui-component/src/root.rs::Root::render_notification_layer",
            Some("notes.editor"),
        ),
        "secondary.snap" => (
            "snap",
            "secondary",
            "src/window_control/snap_overlay.rs",
            "src/window_control/snap_overlay.rs::SnapOverlayView",
            None,
        ),
        _ => return None,
    };
    let mut fixture = descriptor(id, family, root, owner, None, presentation);
    fixture.parent_fixture_id = parent.map(str::to_owned);
    Some(fixture)
}

pub(crate) fn fixtures() -> Vec<FixtureDescriptor> {
    let mut seen = std::collections::BTreeSet::new();
    fixture_ids::fixture_ids()
        .map(|id| {
            assert!(seen.insert(id), "duplicate production fixture ID: {id}");
            declared_descriptor(id)
                .unwrap_or_else(|| panic!("production fixture missing catalogue mapping: {id}"))
        })
        .collect()
}

pub(crate) fn fixture(id: &str) -> Option<FixtureDescriptor> {
    fixture_ids::fixture_ids()
        .find(|candidate| *candidate == id)
        .and_then(declared_descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_catalogue_matches_surface_contract_snapshot() {
        // The generated matrix is runtime test data, not an executable input.
        // Read its current bytes so regeneration does not require recompiling
        // an otherwise source-current library harness.
        let snapshot = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/ai/contracts/surface-contracts.json"
        ))
        .expect("surface contract matrix must be readable");
        let matrix: serde_json::Value =
            serde_json::from_str(&snapshot).expect("surface contract matrix must be valid JSON");
        assert_eq!(matrix["schemaVersion"], 2);
        assert_eq!(
            serde_json::to_value(fixtures()).expect("catalogue must serialize"),
            matrix["fixtures"],
            "compiled fixture catalogue changed; regenerate surface-contracts.json with design discover and --catalogue <compiled-catalogue.json> --write"
        );
    }

    #[test]
    fn catalogue_describes_live_owners_without_claiming_native_pixels() {
        let catalogue = fixtures();
        assert!(!catalogue.is_empty());
        for item in &catalogue {
            assert_eq!(item.proof_boundary, "owned-production-runtime");
            assert!(item.owner.starts_with("src/"));
            assert!(item.presentation_owner.as_ref().is_some_and(|owner| owner
                .starts_with("src/")
                || owner.starts_with("vendor/gpui-kit/")));
            assert!(!item.native_exclusions.is_empty());
            if let Some(parent) = &item.parent_fixture_id {
                assert!(fixture(parent).is_some());
            }
        }
        assert_eq!(
            fixture("main.file-search-mini")
                .unwrap()
                .surface_variant
                .as_deref(),
            Some("FileSearchMini")
        );
        assert_eq!(
            fixture("main.file-search-full")
                .unwrap()
                .surface_variant
                .as_deref(),
            Some("FileSearchFull")
        );
        assert!(fixture("unknown-fixture").is_none());
    }

    #[test]
    fn named_live_overlays_have_finite_executable_owner_descriptors() {
        for id in fixture_ids::MAIN_OVERLAY_FIXTURE_IDS {
            let fixture = fixture(id).expect("declared main overlay");
            assert_eq!(fixture.family, "mainOverlay");
            assert_eq!(fixture.root, "main");
            assert_eq!(
                fixture.parent_fixture_id.as_deref(),
                Some("main.script-list")
            );
            assert_eq!(fixture.app_view_variant.as_deref(), Some("ScriptList"));
        }
        for id in [
            "notes.actions",
            "notes.recent-switcher",
            "notes.root-dialog",
            "notes.root-notification",
        ] {
            let fixture = fixture(id).expect("declared Notes auxiliary presentation");
            assert_eq!(fixture.family, "notesAuxiliary");
            assert_eq!(fixture.parent_fixture_id.as_deref(), Some("notes.editor"));
        }
        assert_eq!(
            fixture("day-page.switcher")
                .unwrap()
                .parent_fixture_id
                .as_deref(),
            Some("day-page.today")
        );
        assert_eq!(
            fixture("day-page.switcher").unwrap().family,
            "dayPageAuxiliary"
        );
        assert_eq!(fixture("day-page.switcher").unwrap().root, "secondary");
        assert_eq!(fixture("notes.actions").unwrap().root, "secondary");
        assert_eq!(fixture("notes.root-dialog").unwrap().root, "notes");
    }
}
