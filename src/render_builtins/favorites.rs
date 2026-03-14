impl ScriptListApp {
    /// Render the favorites browse list with search/filter.
    fn render_favorites_browse(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;

        // Load favorites and resolve to script/scriptlet names
        let favorites = script_kit_gpui::favorites::load_favorites()
            .unwrap_or_default();

        let resolved: Vec<(String, String)> = favorites
            .script_ids
            .iter()
            .map(|id| {
                let display_name = self
                    .scripts
                    .iter()
                    .find(|s| s.name == *id)
                    .map(|s| s.name.clone())
                    .or_else(|| {
                        self.scriptlets
                            .iter()
                            .find(|sl| sl.name == *id)
                            .map(|sl| sl.name.clone())
                    })
                    .unwrap_or_else(|| id.clone());
                let description = self
                    .scripts
                    .iter()
                    .find(|s| s.name == *id)
                    .and_then(|s| s.description.clone())
                    .or_else(|| {
                        self.scriptlets
                            .iter()
                            .find(|sl| sl.name == *id)
                            .and_then(|sl| sl.description.clone())
                    })
                    .unwrap_or_default();
                (id.clone(), format!("{}\x00{}", display_name, description))
            })
            .collect();

        let filter_lower = filter.to_lowercase();
        let filtered: Vec<(usize, &str, &str, &str)> = resolved
            .iter()
            .enumerate()
            .filter_map(|(idx, (id, packed))| {
                let (name, desc) = packed.split_once('\x00').unwrap_or((packed, ""));
                if filter.is_empty()
                    || name.to_lowercase().contains(&filter_lower)
                    || desc.to_lowercase().contains(&filter_lower)
                {
                    Some((idx, id.as_str(), name, desc))
                } else {
                    None
                }
            })
            .collect();

        let count = filtered.len();
        let list_colors = ListItemColors::from_theme(&self.theme);
        let entity = cx.entity().downgrade();

        let list_items: Vec<AnyElement> = filtered
            .iter()
            .enumerate()
            .map(|(display_idx, (_original_idx, fav_id, name, description))| {
                let is_selected = display_idx == selected_index;
                let fav_id_owned = fav_id.to_string();
                let entity_clone = entity.clone();

                div()
                    .id(display_idx)
                    .cursor_pointer()
                    .on_click(move |_event, window, cx| {
                        if let Some(app) = entity_clone.upgrade() {
                            app.update(cx, |this, cx| {
                                this.run_favorite(&fav_id_owned, window, cx);
                            });
                        }
                    })
                    .child(
                        ListItem::new(name.to_string(), list_colors)
                            .description_opt(if description.is_empty() {
                                None
                            } else {
                                Some(description.to_string())
                            })
                            .selected(is_selected)
                            .with_accent_bar(is_selected),
                    )
                    .into_any_element()
            })
            .collect();

        let list_element: AnyElement = if count == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No favorites yet \u{00b7} Star scripts from the actions menu (Cmd+K)"
                } else {
                    "No favorites match your filter"
                })
                .into_any_element()
        } else {
            div()
                .w_full()
                .flex()
                .flex_col()
                .min_h(px(0.))
                .children(list_items)
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div().flex_1().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(28.))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    .child(
                        div()
                            .text_size(px(design_typography.font_size_sm))
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} favorites", count)),
                    ),
            )
            .child(div().w_full().h(px(1.)).bg(rgb(self.theme.colors.ui.border)))
            .child(div().flex_1().w_full().min_h(px(0.)).child(list_element))
            .child(
                div()
                    .w_full()
                    .h(px(1.))
                    .bg(rgb(self.theme.colors.ui.border)),
            )
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_sm))
                    .text_size(px(design_typography.font_size_xs))
                    .text_color(rgb(text_muted))
                    .child(
                        "Enter: run \u{00b7} U: move up \u{00b7} J: move down \u{00b7} D: remove \u{00b7} Esc: back",
                    ),
            )
            .into_any_element()
    }

    /// Run a favorite script by its ID (script name).
    fn run_favorite(&mut self, id: &str, _window: &mut Window, cx: &mut Context<Self>) {
        // Try scripts first
        if let Some(script) = self.scripts.iter().find(|s| s.name == *id) {
            let path = script.path.to_string_lossy().to_string();
            tracing::info!(
                favorite_id = %id,
                path = %path,
                action = "run_favorite",
                "Running favorite script"
            );
            self.execute_script_by_path(&path, cx);
            return;
        }

        // Try scriptlets
        if let Some(scriptlet) = self.scriptlets.iter().find(|sl| sl.name == *id) {
            let scriptlet_clone = scriptlet.clone();
            tracing::info!(
                favorite_id = %id,
                action = "run_favorite_scriptlet",
                "Running favorite scriptlet"
            );
            self.execute_scriptlet(&scriptlet_clone, cx);
            return;
        }

        tracing::warn!(
            favorite_id = %id,
            action = "favorite_not_found",
            "Favorite script not found in scripts or scriptlets"
        );
        self.show_error_toast(format!("Script '{}' not found", id), cx);
    }

    /// Handle keyboard input for the favorites browse view.
    #[allow(dead_code)] // Called from startup_new_actions.rs interceptor
    pub(crate) fn handle_favorites_browse_key(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppView::FavoritesBrowseView {
            ref filter,
            ref mut selected_index,
        } = self.current_view
        {
            if crate::ui_foundation::is_key_enter(key) {
                let favorites = script_kit_gpui::favorites::load_favorites()
                    .unwrap_or_default();
                let filter_lower = filter.to_lowercase();
                let filtered: Vec<&String> = if filter.is_empty() {
                    favorites.script_ids.iter().collect()
                } else {
                    favorites
                        .script_ids
                        .iter()
                        .filter(|id| id.to_lowercase().contains(&filter_lower))
                        .collect()
                };
                if let Some(id) = filtered.get(*selected_index) {
                    let id = (*id).clone();
                    self.run_favorite(&id, window, cx);
                }
            } else if crate::ui_foundation::is_key_escape(key) {
                self.go_back_or_close(window, cx);
            } else if key.eq_ignore_ascii_case("d") && self.filter_text.is_empty() {
                // Remove selected favorite
                let favorites = script_kit_gpui::favorites::load_favorites()
                    .unwrap_or_default();
                let filtered: Vec<&String> = favorites.script_ids.iter().collect();
                if let Some(id) = filtered.get(*selected_index) {
                    let id_owned = (*id).clone();
                    tracing::info!(
                        favorite_id = %id_owned,
                        action = "favorite_remove",
                        "Removing favorite"
                    );
                    match script_kit_gpui::favorites::remove_favorite(&id_owned) {
                        Ok(updated) => {
                            if *selected_index >= updated.script_ids.len()
                                && !updated.script_ids.is_empty()
                            {
                                *selected_index = updated.script_ids.len() - 1;
                            }
                            self.show_hud(
                                format!("Removed '{}'", id_owned),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                action = "favorite_remove_failed",
                                "Failed to remove favorite"
                            );
                            self.show_error_toast(
                                format!("Failed to remove favorite: {}", e),
                                cx,
                            );
                        }
                    }
                    cx.notify();
                }
            } else if key.eq_ignore_ascii_case("u") && self.filter_text.is_empty() {
                // Move selected favorite up
                let favorites = script_kit_gpui::favorites::load_favorites()
                    .unwrap_or_default();
                if let Some(id) = favorites.script_ids.get(*selected_index) {
                    let id_owned = id.clone();
                    if *selected_index > 0 {
                        tracing::info!(
                            favorite_id = %id_owned,
                            action = "favorite_move_up",
                            "Moving favorite up"
                        );
                        if let Err(e) = script_kit_gpui::favorites::move_favorite_up(&id_owned) {
                            tracing::error!(
                                error = %e,
                                action = "favorite_move_up_failed",
                                "Failed to move favorite up"
                            );
                        } else {
                            *selected_index -= 1;
                        }
                        cx.notify();
                    }
                }
            } else if key.eq_ignore_ascii_case("j") && self.filter_text.is_empty() {
                // Move selected favorite down
                let favorites = script_kit_gpui::favorites::load_favorites()
                    .unwrap_or_default();
                if let Some(id) = favorites.script_ids.get(*selected_index) {
                    let id_owned = id.clone();
                    if *selected_index + 1 < favorites.script_ids.len() {
                        tracing::info!(
                            favorite_id = %id_owned,
                            action = "favorite_move_down",
                            "Moving favorite down"
                        );
                        if let Err(e) = script_kit_gpui::favorites::move_favorite_down(&id_owned) {
                            tracing::error!(
                                error = %e,
                                action = "favorite_move_down_failed",
                                "Failed to move favorite down"
                            );
                        } else {
                            *selected_index += 1;
                        }
                        cx.notify();
                    }
                }
            }
        }
    }
}
