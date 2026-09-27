use super::*;

const ALIAS_PLACEHOLDER: &str = "e.g. ch, clip, todo";
const ALIAS_HINT: &str = "Type the alias and a space in the launcher.";
const ALIAS_VIEW: &str = "alias";

fn alias_problem(text: &str) -> Option<SharedString> {
    super::alias_validation::alias_problem(text).map(SharedString::from)
}

impl ScriptListApp {
    /// Ask for a command's alias in a view under the same input: the input holds the current
    /// alias, ↵ saves what it says (nothing removes it), Esc goes back.
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
        self.alias_input_state = Some(AliasInputState {
            command_id,
            command_name: command_name.clone(),
            has_alias,
        });
        self.clear_actions_popup_state();
        self.mark_main_data_changed();
        self.mark_main_presentation_changed();
        self.one_window_handle(
            sk_one_window::Event::View(sk_one_window::ViewOp::Open(sk_one_window::ViewSpec {
                name: ALIAS_VIEW.to_string(),
                label: format!("Alias for \u{201c}{command_name}\u{201d}"),
                enter: Some(sk_one_window::EnterTarget::new(ALIAS_VIEW, "Save alias")),
                takes_input: true,
                text: Some(existing_alias),
                leave_text: Some(String::new()),
            })),
            cx,
        );
        cx.notify();
    }

    /// Whether the alias view is the one on top.
    pub(crate) fn alias_view_on_top(&self) -> bool {
        self.one_window
            .views()
            .top()
            .is_some_and(|top| top.name == ALIAS_VIEW)
    }

    /// The alias view's list: what the words in the input would become.
    pub(crate) fn alias_view_rows(&self) -> Vec<sk_one_window::Row> {
        let Some(state) = &self.alias_input_state else {
            return Vec::new();
        };
        let text = self.one_window.text().trim();
        let (title, sub) = match alias_problem(text) {
            Some(problem) => (problem.to_string(), ALIAS_PLACEHOLDER.to_string()),
            None if text.is_empty() && state.has_alias => {
                ("Remove the alias".to_string(), ALIAS_HINT.to_string())
            }
            None if text.is_empty() => (ALIAS_PLACEHOLDER.to_string(), ALIAS_HINT.to_string()),
            None => (
                format!("{text} → {}", state.command_name),
                ALIAS_HINT.to_string(),
            ),
        };
        vec![sk_one_window::Row::new(ALIAS_VIEW, title)
            .with_sub(sub)
            .with_icon("pencil")
            .with_head(format!("Alias for \u{201c}{}\u{201d}", state.command_name))]
    }

    /// Enter in the alias view saves the input's words as the alias; empty words remove it.
    pub(crate) fn alias_view_key(&mut self, key: sk_one_window::Key, cx: &mut Context<Self>) {
        if key == sk_one_window::Key::Enter {
            let alias = self.one_window.text().to_string();
            self.save_alias_with_text(alias, cx);
        }
    }

    /// Close the alias editor and return focus to the main filter input.
    pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
        if self.alias_input_state.take().is_some() {
            logging::log(
                "ALIAS",
                "Closing alias input, returning focus to main filter",
            );
            if self.alias_view_on_top() {
                self.one_window_handle(sk_one_window::Event::View(sk_one_window::ViewOp::Back), cx);
            }
            self.mark_main_data_changed();
            self.mark_main_presentation_changed();
            self.pending_focus = Some(FocusTarget::MainFilter);
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
                self.notice_failed(problem.to_string(), cx);
                self.close_alias_input(cx);
                return;
            }
        }
        if let Some(scope) = crate::runtime_policy::owned_evaluation() {
            if let Err(error) = scope.require_owned_path(&crate::aliases::default_aliases_path()) {
                self.notice_failed(format!("Alias was not saved: {error}"), cx);
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
            self.notice_failed(format!("Failed to save alias: {error}"), cx);
            return;
        }
        let message = if alias_text.is_empty() {
            "Alias removed".to_string()
        } else {
            format!("Alias set: {} → {}", alias_text, command_name)
        };
        if self.main_services.owned_sources().is_some() {
            self.one_window_notice(crate::line_notice::DONE, message, cx);
            crate::runtime_policy::record_completed_fixture_effect();
        } else {
            self.one_window_notice(crate::line_notice::INFO, message, cx);
        }
    }
}
