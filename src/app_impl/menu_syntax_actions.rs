//! Bridges [[src/menu_syntax/actions.rs#current_actions]] (pure spec) into
//! the Cmd+K actions-dialog (live UI). The dialog calls
//! [[power_syntax_action_section]] to get a fully-prepared section row set
//! plus replace-vs-prepend semantics.
//!
//! Story: cmdk-actions-section-render. This module ships the pure adapter
//! that lets the actions dialog render these rows with replace-vs-prepend
//! semantics while the app-side live-state caller remains a follow-up.
//!
//! Receipt: `cargo test --lib app_impl::menu_syntax_actions`.
//!

use crate::actions::{Action, ActionCategory};
use crate::menu_syntax::{current_menu_syntax_actions, MenuSyntaxAction, MenuSyntaxActionState};

/// How the Power Syntax section relates to the dialog's normal selected-row
/// actions: `Replace` swaps them out entirely (capture / command — when the
/// user is composing one of these, normal row actions are not relevant);
/// `Prepend` adds the section above the normal row actions (refine — the
/// user is still narrowing a list, so the row actions remain useful).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionMode {
    Replace,
    Prepend,
}

/// Fully-prepared section the actions-dialog can render directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSyntaxActionSection {
    pub title: String,
    pub mode: SectionMode,
    pub actions: Vec<MenuSyntaxAction>,
}

/// Compute the Power Syntax section to render for the current composer
/// state. The section title is constant; the mode depends on the state
/// variant.
pub fn power_syntax_action_section(state: &MenuSyntaxActionState<'_>) -> PowerSyntaxActionSection {
    let mode = match state {
        MenuSyntaxActionState::CaptureComposer { .. } => SectionMode::Replace,
        MenuSyntaxActionState::CommandComposer { .. } => SectionMode::Replace,
        MenuSyntaxActionState::RefineQuery { .. } => SectionMode::Prepend,
    };
    PowerSyntaxActionSection {
        title: "Power Syntax".to_string(),
        mode,
        actions: current_menu_syntax_actions(state),
    }
}

/// Convert a Power Syntax section into the dialog's regular action row type.
///
/// Disabled rows stay out of the dialog so unavailable affordances cannot be
/// selected by keyboard or mouse.
pub fn power_syntax_section_to_actions(section: &PowerSyntaxActionSection) -> Vec<Action> {
    section
        .actions
        .iter()
        .filter(|action| action.enabled)
        .map(|action| {
            Action::new(
                format!("menu_syntax:{}", action.id),
                action.label.clone(),
                None,
                ActionCategory::ScriptContext,
            )
            .with_section("Power Syntax")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture_schema::builtin_schema;
    use crate::menu_syntax::parse_advanced_query;
    use crate::menu_syntax::payload::{CaptureAlias, CaptureInvocation};
    use crate::menu_syntax::MenuSyntaxActionKind;

    fn capture_payload(target: &str, body: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: body.to_string(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: format!("+{target} {body}"),
        }
    }

    #[test]
    fn capture_state_yields_replace_mode() {
        let payload = capture_payload("todo", "Buy milk");
        let schema = builtin_schema("todo").unwrap();
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "todo",
            payload: &payload,
            schema: Some(&schema),
        };
        let section = power_syntax_action_section(&state);
        assert_eq!(section.mode, SectionMode::Replace);
        assert_eq!(section.title, "Power Syntax");
        assert!(!section.actions.is_empty());
    }

    #[test]
    fn command_state_yields_replace_mode() {
        let argv = vec!["--prod".to_string()];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        let section = power_syntax_action_section(&state);
        assert_eq!(section.mode, SectionMode::Replace);
    }

    #[test]
    fn refine_state_yields_prepend_mode() {
        let query = parse_advanced_query(":foo bar");
        let state = MenuSyntaxActionState::RefineQuery { query: &query };
        let section = power_syntax_action_section(&state);
        assert_eq!(
            section.mode,
            SectionMode::Prepend,
            "refine still shows the selected row's normal actions BELOW the Power Syntax section"
        );
    }

    #[test]
    fn section_title_is_constant() {
        let payload = capture_payload("todo", "x");
        let schema = builtin_schema("todo").unwrap();
        let states: Vec<MenuSyntaxActionState> = vec![
            MenuSyntaxActionState::CaptureComposer {
                target: "todo",
                payload: &payload,
                schema: Some(&schema),
            },
            MenuSyntaxActionState::CommandComposer {
                head: "deploy",
                argv: &[],
            },
        ];
        for state in &states {
            assert_eq!(power_syntax_action_section(state).title, "Power Syntax");
        }
    }

    #[test]
    fn section_actions_match_pure_spec_output() {
        // The adapter must NOT reorder or filter — it just attaches the
        // title and mode. Pinned against any future "smart" filtering that
        // would diverge from the pure spec.
        let payload = capture_payload("cal", "Design review");
        let schema = builtin_schema("cal").unwrap();
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &payload,
            schema: Some(&schema),
        };
        assert_eq!(
            power_syntax_action_section(&state).actions,
            current_menu_syntax_actions(&state),
        );
    }

    mod app_impl {
        mod menu_syntax_actions {
            use super::super::*;

            #[test]
            fn section_to_actions_maps_enabled_rows_to_dialog_actions() {
                let section = PowerSyntaxActionSection {
                    title: "Power Syntax".to_string(),
                    mode: SectionMode::Replace,
                    actions: vec![MenuSyntaxAction {
                        id: "capture.cancel".to_string(),
                        label: "Cancel without saving".to_string(),
                        kind: MenuSyntaxActionKind::Cancel,
                        enabled: true,
                    }],
                };

                let actions = power_syntax_section_to_actions(&section);

                assert_eq!(actions.len(), 1);
                assert_eq!(actions[0].id, "menu_syntax:capture.cancel");
                assert_eq!(actions[0].title, "Cancel without saving");
                assert_eq!(actions[0].description, None);
                assert_eq!(actions[0].category, ActionCategory::ScriptContext);
                assert_eq!(actions[0].section.as_deref(), Some("Power Syntax"));
            }

            #[test]
            fn section_to_actions_skips_disabled_rows() {
                let section = PowerSyntaxActionSection {
                    title: "Power Syntax".to_string(),
                    mode: SectionMode::Replace,
                    actions: vec![
                        MenuSyntaxAction {
                            id: "capture.save_and_copy_id".to_string(),
                            label: "Save and copy URL/ID to clipboard".to_string(),
                            kind: MenuSyntaxActionKind::SaveAndCopyId,
                            enabled: false,
                        },
                        MenuSyntaxAction {
                            id: "capture.cancel".to_string(),
                            label: "Cancel without saving".to_string(),
                            kind: MenuSyntaxActionKind::Cancel,
                            enabled: true,
                        },
                    ],
                };

                let action_ids: Vec<String> = power_syntax_section_to_actions(&section)
                    .into_iter()
                    .map(|action| action.id)
                    .collect();

                assert_eq!(action_ids, vec!["menu_syntax:capture.cancel".to_string()]);
            }

            #[test]
            fn section_to_actions_preserves_enabled_row_order() {
                let section = PowerSyntaxActionSection {
                    title: "Power Syntax".to_string(),
                    mode: SectionMode::Prepend,
                    actions: vec![
                        MenuSyntaxAction {
                            id: "refine.save_named_search".to_string(),
                            label: "Save filter as named search".to_string(),
                            kind: MenuSyntaxActionKind::SaveFilterAsNamedSearch,
                            enabled: true,
                        },
                        MenuSyntaxAction {
                            id: "refine.add_pinned".to_string(),
                            label: "Add to launcher pinned filters".to_string(),
                            kind: MenuSyntaxActionKind::AddToPinnedFilters,
                            enabled: true,
                        },
                    ],
                };

                let action_ids: Vec<String> = power_syntax_section_to_actions(&section)
                    .into_iter()
                    .map(|action| action.id)
                    .collect();

                assert_eq!(
                    action_ids,
                    vec![
                        "menu_syntax:refine.save_named_search".to_string(),
                        "menu_syntax:refine.add_pinned".to_string()
                    ]
                );
            }
        }
    }
}
