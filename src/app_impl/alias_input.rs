use super::*;

const ALIAS_PLACEHOLDER: &str = "e.g. ch, clip, todo";
const ALIAS_HINT: &str = "Type the alias and a space in the launcher.";

fn alias_problem(text: &str) -> Option<SharedString> {
    super::alias_validation::alias_problem(text).map(SharedString::from)
}

impl ScriptListApp {
    /// Ask for a command's alias in the shared confirm modal: a field with
    /// the current alias, ↵ Save, Remove when one exists, Esc Cancel.
    pub(crate) fn show_alias_input(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "ALIAS",
            &format!(
                "Showing alias input for '{}' (id: {})",
                command_name, command_id
            ),
        );

        let existing_alias = if let Some(sources) = self.main_services.owned_sources() {
            sources
                .alias_overrides
                .get(&command_id)
                .cloned()
                .unwrap_or_default()
        } else {
            crate::aliases::load_alias_overrides()
                .ok()
                .and_then(|overrides| {
                    overrides.get(&command_id).cloned().or_else(|| {
                        self.get_selected_result()
                            .and_then(|selected| selected.command_preference_identity())
                            .filter(|identity| identity.exact_id == command_id)
                            .and_then(|identity| overrides.get(&identity.legacy_id).cloned())
                    })
                })
                .unwrap_or_default()
        };
        let has_alias = !existing_alias.is_empty();
        let field = crate::confirm::ConfirmTextField::shared(
            existing_alias,
            ALIAS_PLACEHOLDER,
            std::rc::Rc::new(alias_problem),
        );
        let open = std::rc::Rc::new(());
        let still_open = std::rc::Rc::downgrade(&open);
        self.alias_input_state = Some(AliasInputState {
            command_id,
            command_name: command_name.clone(),
            field: field.clone(),
            has_alias,
            _open: open,
        });
        self.clear_actions_popup_state();
        self.mark_main_data_changed();
        self.mark_main_presentation_changed();

        let options = crate::confirm::ParentActionDialogOptions {
            title: format!("Alias for \u{201c}{command_name}\u{201d}").into(),
            body: ALIAS_HINT.into(),
            primary_text: "Save".into(),
            secondary_text: has_alias.then(|| "Remove".into()),
            dismiss_text: "Cancel".into(),
            width: px(crate::confirm::PARENT_MODAL_WIDTH_PX),
        };
        let app = cx.entity().downgrade();
        let (submit_app, remove_app, dismiss_app) = (app.clone(), app.clone(), app);
        self.open_main_window_modal(
            move |window, cx| {
                crate::confirm::open_parent_input_dialog(
                    window,
                    cx,
                    options,
                    field,
                    // Closing the editor from elsewhere closes the modal.
                    move || still_open.upgrade().is_some(),
                    move |alias, _, cx| {
                        let _ = submit_app.update(cx, |app, cx| {
                            app.save_alias_with_text(alias, cx);
                        });
                    },
                    move |_, cx| {
                        let _ = remove_app.update(cx, |app, cx| {
                            app.save_alias_with_text(String::new(), cx);
                        });
                    },
                    move |_, cx| {
                        let _ = dismiss_app.update(cx, |app, cx| app.close_alias_input(cx));
                    },
                );
            },
            cx,
        );
        cx.notify();
    }

    /// Close the alias editor (and its modal) and return focus to the main
    /// filter input.
    pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
        if self.alias_input_state.take().is_some() {
            logging::log(
                "ALIAS",
                "Closing alias input, returning focus to main filter",
            );
            self.mark_main_data_changed();
            self.mark_main_presentation_changed();
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Replace the alias text in the open editor.
    pub(crate) fn update_alias_text(&mut self, text: String, cx: &mut Context<Self>) {
        if let Some(state) = &self.alias_input_state {
            state.field.borrow_mut().state.set_text(text);
            crate::confirm::refresh_confirm_popup(cx);
            self.mark_main_data_changed();
            cx.notify();
        }
    }

    /// Save `alias` for the command being edited (an empty alias removes
    /// it) and close the editor, reporting the outcome.
    pub(crate) fn save_alias_with_text(&mut self, alias: String, cx: &mut Context<Self>) {
        let Some(state) = self.alias_input_state.as_ref() else {
            return;
        };
        let command_id = state.command_id.clone();
        let command_name = state.command_name.clone();
        let alias_text = alias.trim().to_string();
        if !alias_text.is_empty() {
            if let Some(problem) = alias_problem(&alias_text) {
                self.show_error_toast(problem.to_string(), cx);
                self.close_alias_input(cx);
                return;
            }
        }
        if let Some(scope) = crate::runtime_policy::owned_evaluation() {
            if let Err(error) = scope.require_owned_path(&crate::aliases::default_aliases_path()) {
                self.show_error_toast(format!("Alias was not saved: {error}"), cx);
                self.close_alias_input(cx);
                return;
            }
        }
        let mut save_error = None;
        self.commit_main_menu_results_refresh("alias-override", None, cx, |app, _cx| {
            let result = if alias_text.is_empty() {
                crate::aliases::remove_alias_override(&command_id)
            } else {
                crate::aliases::save_alias_override(&command_id, &alias_text)
            };
            if let Err(error) = result {
                save_error = Some(error);
                return false;
            }
            if let MainServices::OwnedFixtures(sources) = &mut app.main_services {
                let overrides = &mut Arc::make_mut(sources).alias_overrides;
                if alias_text.is_empty() {
                    overrides.remove(&command_id);
                } else {
                    overrides.insert(command_id.clone(), alias_text.clone());
                }
            }
            app.rebuild_registries();
            app.mark_main_data_changed();
            true
        });
        self.close_alias_input(cx);
        if let Some(error) = save_error {
            self.show_error_toast(format!("Failed to save alias: {error}"), cx);
            return;
        }
        let message = if alias_text.is_empty() {
            "Alias removed".to_string()
        } else {
            format!("Alias set: {} → {}", alias_text, command_name)
        };
        if self.main_services.owned_sources().is_some() {
            self.toast_manager.push(
                components::toast::Toast::success(message, &self.theme)
                    .duration_ms(Some(TOAST_SUCCESS_MS)),
            );
            crate::runtime_policy::record_completed_fixture_effect();
        } else {
            self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
        }
    }
}
