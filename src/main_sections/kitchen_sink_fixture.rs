pub(crate) const MAIN_WINDOW_KITCHEN_SINK_FIXTURE_ID: &str = "main-window-kitchen-sink";
pub(crate) const MAIN_WINDOW_KITCHEN_SINK_QUERY: &str = "kitchen sink";
pub(crate) const MAIN_WINDOW_KITCHEN_SINK_NO_MATCH_QUERY: &str =
    "zzzz-main-window-kitchen-sink-no-match";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MainWindowKitchenSinkMode {
    Populated,
    NoMatch,
}

pub(crate) fn main_window_kitchen_sink_feature_manifest() -> &'static [&'static str] {
    &[
        "shell:content-insets",
        "search:input-text",
        "search:long-placeholder",
        "list:section-header",
        "list:first-section-header",
        "list:source-status-row",
        "list:scroll-overflow",
        "row:selected",
        "row:hover-worthy",
        "row:long-title",
        "row:long-description",
        "row:empty-description",
        "icon:svg",
        "icon:app",
        "icon:missing-fallback",
        "metadata:source",
        "metadata:badge",
        "metadata:keycap",
        "typography:section",
        "footer:run-actions-ai",
        "header-info:pills",
        "header-info:hover-worthy",
        "empty:no-match",
    ]
}

pub(crate) fn main_window_kitchen_sink_grouped_results() -> (
    Vec<crate::list_item::GroupedListItem>,
    Vec<crate::scripts::SearchResult>,
) {
    use crate::list_item::GroupedListItem;

    let mut grouped = Vec::new();
    let mut results = Vec::new();
    let sections = [
        ("Suggested", "Scripts"),
        ("Built-ins", "Built-in"),
        ("Apps", "App"),
        ("Skills", "Skill"),
        ("Files", "File"),
        ("Diagnostics", "Fallback"),
    ];

    for (section_index, (section, kind)) in sections.iter().enumerate() {
        grouped.push(GroupedListItem::SectionHeader(
            (*section).to_string(),
            Some(format!("{kind} kitchen sink rows")),
        ));
        if section_index == 0 {
            grouped.push(GroupedListItem::Status(
                crate::list_item::SourceChipStatusRow {
                    source: crate::menu_syntax::RootUnifiedSourceFilter::Scripts,
                    source_name: "Scripts".to_string(),
                    status_kind: crate::list_item::SourceChipStatusKind::Showing,
                    label: "Showing deterministic kitchen sink rows".to_string(),
                    shown: 6,
                    loaded: 6,
                    total: Some(30),
                },
            ));
        }

        for item_index in 0..5 {
            let flat_index = results.len();
            results.push(main_window_kitchen_sink_result(
                *kind,
                section_index,
                item_index,
            ));
            grouped.push(GroupedListItem::Item(flat_index));
        }
    }

    (grouped, results)
}

pub(crate) fn main_window_kitchen_sink_no_match_grouped_results() -> (
    Vec<crate::list_item::GroupedListItem>,
    Vec<crate::scripts::SearchResult>,
) {
    (Vec::new(), Vec::new())
}

fn main_window_kitchen_sink_result(
    kind: &str,
    section_index: usize,
    item_index: usize,
) -> crate::scripts::SearchResult {
    match kind {
        "Built-in" => crate::scripts::SearchResult::BuiltIn(crate::scripts::BuiltInMatch {
            entry: crate::builtins::BuiltInEntry {
                id: format!("builtin/kitchen-sink-{section_index}-{item_index}"),
                name: kitchen_sink_title(kind, section_index, item_index),
                description: kitchen_sink_description(kind, section_index, item_index),
                keywords: vec!["kitchen".to_string(), "sink".to_string(), kind.to_lowercase()],
                feature: crate::builtins::BuiltInFeature::AppLauncher,
                icon: Some(if item_index % 2 == 0 { "command" } else { "sparkles" }.to_string()),
                group: crate::builtins::BuiltInGroup::Core,
            },
            score: 100 - item_index as i32,
            match_evidence: None,
        }),
        "App" => crate::scripts::SearchResult::App(crate::scripts::AppMatch {
            app: crate::app_launcher::AppInfo {
                name: kitchen_sink_title(kind, section_index, item_index),
                path: format!("/Applications/Kitchen Sink {item_index}.app").into(),
                bundle_id: Some(format!("dev.scriptkit.kitchensink.{item_index}")),
                icon: None,
            },
            score: 90 - item_index as i32,
            match_evidence: None,
        }),
        "Skill" => crate::scripts::SearchResult::Skill(crate::scripts::SkillMatch {
            skill: std::sync::Arc::new(crate::plugins::PluginSkill {
                plugin_id: "kitchen-sink".to_string(),
                plugin_title: "Kitchen Sink Skills".to_string(),
                skill_id: format!("style-edge-case-{item_index}"),
                path: format!("/tmp/kitchen-sink/skills/style-edge-case-{item_index}/SKILL.md")
                    .into(),
                title: kitchen_sink_title(kind, section_index, item_index),
                description: kitchen_sink_description(kind, section_index, item_index),
            }),
            score: 80 - item_index as i32,
            match_indices: crate::scripts::MatchIndices::default(),
            match_evidence: None,
        }),
        "File" => crate::scripts::SearchResult::File(crate::scripts::FileMatch {
            file: crate::file_search::FileResult {
                path: format!(
                    "/Users/example/Projects/kitchen-sink/very/long/path/that/stresses/metadata/row-{item_index}.md"
                ),
                name: kitchen_sink_title(kind, section_index, item_index),
                size: 1024 * (item_index as u64 + 1),
                modified: 1_717_171_717 + item_index as u64,
                file_type: if item_index % 2 == 0 {
                    crate::file_search::FileType::Document
                } else {
                    crate::file_search::FileType::Directory
                },
            },
            score: 70 - item_index as i32,
        }),
        "Fallback" => {
            let fallback = crate::fallbacks::builtins::BuiltinFallback::new(
                "kitchen-sink-fallback",
                "Kitchen Sink Fallback",
                "Exercises fallback rows with missing icon and long metadata",
                "zap",
                crate::fallbacks::builtins::FallbackAction::CopyToClipboard,
                crate::fallbacks::builtins::FallbackCondition::Always,
                20,
            );
            crate::scripts::SearchResult::Fallback(
                crate::scripts::FallbackMatch::new(
                    crate::fallbacks::FallbackItem::Builtin(fallback),
                    0,
                )
                .with_display_overrides(
                    kitchen_sink_title(kind, section_index, item_index),
                    kitchen_sink_description(kind, section_index, item_index),
                )
                .with_stable_selection_key(format!("kitchen-sink-fallback-{item_index}")),
            )
        }
        _ => crate::scripts::SearchResult::Script(crate::scripts::ScriptMatch {
            script: std::sync::Arc::new(crate::scripts::Script {
                name: kitchen_sink_title(kind, section_index, item_index),
                path: format!("/tmp/kitchen-sink/scripts/row-{item_index}.ts").into(),
                extension: "ts".to_string(),
                description: (item_index != 3)
                    .then(|| kitchen_sink_description(kind, section_index, item_index)),
                icon: (item_index != 2).then(|| {
                    if item_index % 2 == 0 {
                        "file-code".to_string()
                    } else {
                        "terminal".to_string()
                    }
                }),
                alias: (item_index == 1).then(|| "ks".to_string()),
                shortcut: (item_index == 0).then(|| "cmd shift k".to_string()),
                plugin_id: "main".to_string(),
                plugin_title: Some("Main".to_string()),
                kit_name: Some("main".to_string()),
                body: Some("Kitchen sink fixture body for content search.".to_string()),
                ..Default::default()
            }),
            score: 120 - item_index as i32,
            filename: format!("kitchen-sink-row-{item_index}.ts"),
            match_indices: crate::scripts::MatchIndices::default(),
            match_kind: crate::scripts::ScriptMatchKind::Name,
            content_match: None,
            match_evidence: None,
        }),
    }
}

fn kitchen_sink_title(kind: &str, section_index: usize, item_index: usize) -> String {
    match (section_index, item_index) {
        (0, 1) => "A".to_string(),
        (0, 2) => "Kitchen Sink Row With A Very Long Title That Should Wrap Cleanly".to_string(),
        (2, 3) => "Kitchen Sink Punctuation ! ? / @ : Row".to_string(),
        _ => format!("{kind} Kitchen Sink Row {}", item_index + 1),
    }
}

fn kitchen_sink_description(kind: &str, section_index: usize, item_index: usize) -> String {
    if section_index == 0 && item_index == 2 {
        return "A deliberately long description used to stress row typography, metadata spacing, wrapping, selected state, hover state, footer action text, and source-chip alignment in the real main menu renderer.".to_string();
    }
    format!(
        "{kind} fixture description for style control coverage row {}",
        item_index + 1
    )
}

impl ScriptListApp {
    pub(crate) fn open_main_window_kitchen_sink_fixture(&mut self, cx: &mut gpui::Context<Self>) {
        self.install_main_window_kitchen_sink_fixture(MainWindowKitchenSinkMode::Populated, cx);
    }

    pub(crate) fn open_main_window_no_match_kitchen_sink_fixture(
        &mut self,
        cx: &mut gpui::Context<Self>,
    ) {
        self.install_main_window_kitchen_sink_fixture(MainWindowKitchenSinkMode::NoMatch, cx);
    }

    fn install_main_window_kitchen_sink_fixture(
        &mut self,
        mode: MainWindowKitchenSinkMode,
        cx: &mut gpui::Context<Self>,
    ) {
        let query = match mode {
            MainWindowKitchenSinkMode::Populated => MAIN_WINDOW_KITCHEN_SINK_QUERY,
            MainWindowKitchenSinkMode::NoMatch => MAIN_WINDOW_KITCHEN_SINK_NO_MATCH_QUERY,
        };
        let (grouped_items, flat_results) = match mode {
            MainWindowKitchenSinkMode::Populated => main_window_kitchen_sink_grouped_results(),
            MainWindowKitchenSinkMode::NoMatch => {
                main_window_kitchen_sink_no_match_grouped_results()
            }
        };

        self.current_view = AppView::ScriptList;
        self.main_window_mode = MainWindowMode::Full;
        self.filter_text = query.to_string();
        self.computed_filter_text = query.to_string();
        self.selected_index = if matches!(mode, MainWindowKitchenSinkMode::Populated) {
            1
        } else {
            0
        };
        self.hovered_index = if matches!(mode, MainWindowKitchenSinkMode::Populated) {
            Some(3)
        } else {
            None
        };
        self.pending_filter_sync = true;
        self.main_menu_result_caches
            .store_filtered_results(query.to_string(), flat_results.clone());
        self.main_menu_result_caches.store_grouped_results(
            query.to_string(),
            grouped_items,
            flat_results,
            None,
            None,
        );
        crate::actions::close_actions_window(cx);
        self.clear_actions_popup_state();
        script_kit_gpui::set_main_window_visible(true);
        script_kit_gpui::mark_window_shown();
        cx.notify();
    }

    pub(crate) fn open_actions_popup_kitchen_sink_fixture(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.install_actions_popup_kitchen_sink_fixture(
            crate::actions::ActionsPopupKitchenSinkMode::Populated,
            window,
            cx,
        );
    }

    pub(crate) fn open_actions_popup_no_match_kitchen_sink_fixture(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.install_actions_popup_kitchen_sink_fixture(
            crate::actions::ActionsPopupKitchenSinkMode::NoMatch,
            window,
            cx,
        );
    }

    fn install_actions_popup_kitchen_sink_fixture(
        &mut self,
        mode: crate::actions::ActionsPopupKitchenSinkMode,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let actions = crate::actions::actions_popup_kitchen_sink_actions();
        let config = crate::actions::actions_popup_kitchen_sink_config(mode);
        let theme = std::sync::Arc::clone(&self.theme);
        let no_match = matches!(mode, crate::actions::ActionsPopupKitchenSinkMode::NoMatch);
        let dialog = cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = crate::actions::ActionsDialog::from_actions_with_context(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}),
                actions,
                None,
                None,
                theme,
                crate::designs::DesignVariant::Default,
                Some(
                    "Actions Popup Kitchen Sink - long context header for style controls"
                        .to_string(),
                ),
                config,
            );
            if no_match {
                dialog.set_search_text(
                    crate::actions::ACTIONS_POPUP_KITCHEN_SINK_NO_MATCH_QUERY.to_string(),
                    cx,
                );
            }
            dialog.set_skip_track_focus(true);
            dialog.set_match_main_window_background(true);
            dialog
        });

        self.current_view = AppView::ScriptList;
        self.clear_actions_popup_state();
        self.actions_dialog = Some(dialog.clone());
        self.begin_actions_popup_window_open(cx, window);

        let app_entity = cx.entity().clone();
        dialog.update(cx, |dialog, _cx| {
            dialog.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                ActionsDialogHost::MainList,
            ));
            dialog.set_on_close(Self::make_actions_window_on_close_callback(
                app_entity,
                ActionsDialogHost::MainList,
                "Actions kitchen sink closed, focus restored via coordinator",
            ));
        });

        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|display| display.id());
        let position = self.main_list_actions_window_position();
        crate::actions::emit_actions_popup_event(
            crate::actions::ActionsPopupEvent::OpenRequested,
            Some("mainList"),
            Some(position),
            Some(crate::actions::actions_popup_kitchen_sink_actions().len()),
            None,
            None,
        );
        Self::spawn_open_actions_window_with_parent_id(
            cx,
            window.window_handle(),
            main_bounds,
            display_id,
            dialog,
            position,
            "Actions popup kitchen sink window opened",
            "Failed to open actions popup kitchen sink window",
            None,
        );
        cx.notify();
    }
}
