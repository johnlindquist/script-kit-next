impl ScriptListApp {
    /// Render the quicklinks browse list with search/filter.
    fn render_quicklinks_browse(
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

        let all_links = script_kit_gpui::quicklinks::load_quicklinks();
        let filter_lower = filter.to_lowercase();
        let filtered: Vec<_> = if filter.is_empty() {
            all_links.iter().enumerate().collect()
        } else {
            all_links
                .iter()
                .enumerate()
                .filter(|(_, link)| {
                    link.name.to_lowercase().contains(&filter_lower)
                        || link.url_template.to_lowercase().contains(&filter_lower)
                })
                .collect()
        };

        let count = filtered.len();
        let list_colors = ListItemColors::from_theme(&self.theme);
        let entity = cx.entity().downgrade();

        let list_items: Vec<AnyElement> = filtered
            .iter()
            .enumerate()
            .map(|(display_idx, (_original_idx, link))| {
                let is_selected = display_idx == selected_index;
                let link_id = link.id.clone();
                let entity_clone = entity.clone();

                let has_query = script_kit_gpui::quicklinks::has_query_placeholder(&link.url_template);
                let badge = if has_query { " ({query})" } else { "" };
                let description = format!("{}{}", link.url_template, badge);

                div()
                    .id(display_idx)
                    .cursor_pointer()
                    .on_click(move |_event, window, cx| {
                        if let Some(app) = entity_clone.upgrade() {
                            app.update(cx, |this, cx| {
                                this.open_quicklink(&link_id, window, cx);
                            });
                        }
                    })
                    .child(
                        ListItem::new(link.name.clone(), list_colors)
                            .description_opt(Some(description))
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
                    "No quicklinks yet \u{00b7} Press N to create one"
                } else {
                    "No quicklinks match your filter"
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
                            .child(format!("{} quicklinks", count)),
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
                        "Enter: open \u{00b7} N: new quicklink \u{00b7} E: edit \u{00b7} D: delete \u{00b7} Esc: back",
                    ),
            )
            .into_any_element()
    }

    /// Render the quicklinks create/edit form.
    fn render_quicklinks_edit(
        &mut self,
        editing_id: Option<String>,
        name: String,
        url_template: String,
        active_field: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;
        let accent = self.theme.colors.accent.selected;
        let border = self.theme.colors.ui.border;

        let entity = cx.entity().downgrade();

        let title = if editing_id.is_some() {
            "Edit Quicklink"
        } else {
            "New Quicklink"
        };

        let fields: Vec<(&str, &str, &str, usize)> = vec![
            ("Name", "My Search Engine", &name, 0),
            (
                "URL Template",
                "https://example.com/search?q={query}",
                &url_template,
                1,
            ),
        ];

        let field_elements: Vec<AnyElement> = fields
            .iter()
            .map(|(label, placeholder, value, idx)| {
                let is_active = *idx == active_field;
                let label_color = if is_active { accent } else { text_dimmed };
                let border_color = if is_active { accent } else { border };
                let entity_for_click = entity.clone();
                let field_idx = *idx;

                div()
                    .w_full()
                    .mb(px(design_spacing.padding_md))
                    .child(
                        div()
                            .text_size(px(design_typography.font_size_sm))
                            .text_color(rgb(label_color))
                            .mb(px(4.))
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("ql-field-{}", idx)))
                            .w_full()
                            .px(px(design_spacing.padding_md))
                            .py(px(design_spacing.padding_sm))
                            .border_1()
                            .border_color(rgb(border_color))
                            .rounded(px(design_visual.radius_md))
                            .text_size(px(design_typography.font_size_md))
                            .text_color(rgb(text_primary))
                            .cursor_pointer()
                            .on_click(move |_event, _window, cx| {
                                if let Some(app) = entity_for_click.upgrade() {
                                    app.update(cx, |this, cx| {
                                        if let AppView::QuicklinksEditView {
                                            ref mut active_field,
                                            ..
                                        } = this.current_view
                                        {
                                            *active_field = field_idx;
                                            cx.notify();
                                        }
                                    });
                                }
                            })
                            .child(if value.is_empty() {
                                div()
                                    .text_color(rgb(text_muted))
                                    .child(placeholder.to_string())
                                    .into_any_element()
                            } else {
                                div().child(value.to_string()).into_any_element()
                            }),
                    )
                    .into_any_element()
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .text_size(px(design_typography.font_size_xl))
                    .child(title.to_string()),
            )
            .child(div().w_full().h(px(1.)).bg(rgb(border)))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .children(field_elements)
                    .child(
                        div()
                            .text_size(px(design_typography.font_size_xs))
                            .text_color(rgb(text_muted))
                            .child(
                                "Type to edit \u{00b7} Tab: next field \u{00b7} Enter: save \u{00b7} Esc: cancel \u{00b7} Use {query} in URL for search",
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Open a quicklink by its ID (expands URL template, prompts for query if needed).
    fn open_quicklink(&mut self, link_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let links = script_kit_gpui::quicklinks::load_quicklinks();
        let Some(link) = links.iter().find(|l| l.id == link_id) else {
            tracing::warn!(link_id = %link_id, action = "open_quicklink_not_found", "Quicklink not found");
            self.show_error_toast("Quicklink not found", cx);
            return;
        };

        if script_kit_gpui::quicklinks::has_query_placeholder(&link.url_template) {
            // Use filter text as default query
            let query = self.filter_text.trim().to_string();
            if query.is_empty() {
                // Transition to a mini prompt for query input would be ideal,
                // but for now just open the raw template without expansion
                tracing::info!(link_id = %link_id, name = %link.name, action = "open_quicklink_no_query", "Opening quicklink without query");
            }
            let expanded = script_kit_gpui::quicklinks::expand_url(&link.url_template, &query);
            self.open_quicklink_url(&link.name, &expanded, window, cx);
        } else {
            self.open_quicklink_url(&link.name, &link.url_template, window, cx);
        }
    }

    /// Open a URL and show feedback.
    fn open_quicklink_url(
        &mut self,
        name: &str,
        url: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match open::that(url) {
            Ok(()) => {
                tracing::info!(name = %name, url = %url, action = "quicklink_opened", "Opened quicklink");
                self.show_hud(format!("Opened {}", name), Some(HUD_SHORT_MS), cx);
                self.close_and_reset_window(cx);
            }
            Err(e) => {
                tracing::error!(name = %name, url = %url, error = %e, action = "quicklink_open_failed", "Failed to open quicklink");
                self.show_error_toast(format!("Failed to open: {}", e), cx);
            }
        }
    }

    /// Handle keyboard input for the quicklinks browse view.
    #[allow(dead_code)] // Called from startup_new_actions.rs interceptor
    pub(crate) fn handle_quicklinks_browse_key(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppView::QuicklinksBrowseView {
            ref filter,
            ref mut selected_index,
        } = self.current_view
        {
            if crate::ui_foundation::is_key_enter(key) {
                let links = script_kit_gpui::quicklinks::load_quicklinks();
                let filter_lower = filter.to_lowercase();
                let filtered: Vec<_> = if filter.is_empty() {
                    links.iter().collect()
                } else {
                    links
                        .iter()
                        .filter(|l| {
                            l.name.to_lowercase().contains(&filter_lower)
                                || l.url_template.to_lowercase().contains(&filter_lower)
                        })
                        .collect()
                };
                if let Some(link) = filtered.get(*selected_index) {
                    let id = link.id.clone();
                    self.open_quicklink(&id, window, cx);
                }
            } else if crate::ui_foundation::is_key_escape(key) {
                self.go_back_or_close(window, cx);
            } else if key.eq_ignore_ascii_case("n") && !self.filter_text.is_empty() {
                // 'n' only triggers new when filter is empty (otherwise it's typing)
                // This is handled below
            } else if key.eq_ignore_ascii_case("n") {
                // Create new quicklink
                tracing::info!(action = "quicklinks_create_start", "Starting new quicklink creation");
                self.filter_text.clear();
                self.pending_filter_sync = true;
                self.current_view = AppView::QuicklinksEditView {
                    editing_id: None,
                    name: String::new(),
                    url_template: String::new(),
                    active_field: 0,
                };
                cx.notify();
            } else if key.eq_ignore_ascii_case("e") && self.filter_text.is_empty() {
                // Edit selected quicklink
                let links = script_kit_gpui::quicklinks::load_quicklinks();
                let filtered: Vec<_> = links.iter().collect();
                if let Some(link) = filtered.get(*selected_index) {
                    tracing::info!(link_id = %link.id, action = "quicklinks_edit_start", "Editing quicklink");
                    self.filter_text.clear();
                    self.pending_filter_sync = true;
                    self.current_view = AppView::QuicklinksEditView {
                        editing_id: Some(link.id.clone()),
                        name: link.name.clone(),
                        url_template: link.url_template.clone(),
                        active_field: 0,
                    };
                    cx.notify();
                }
            } else if key.eq_ignore_ascii_case("d") && self.filter_text.is_empty() {
                // Delete selected quicklink
                let links = script_kit_gpui::quicklinks::load_quicklinks();
                let filtered: Vec<_> = links.iter().collect();
                if let Some(link) = filtered.get(*selected_index) {
                    let link_name = link.name.clone();
                    let link_id = link.id.clone();
                    tracing::info!(link_id = %link_id, name = %link_name, action = "quicklinks_delete", "Deleting quicklink");
                    script_kit_gpui::quicklinks::delete_quicklink(&link_id);
                    // Adjust selection after delete
                    let new_links = script_kit_gpui::quicklinks::load_quicklinks();
                    if *selected_index >= new_links.len() && !new_links.is_empty() {
                        *selected_index = new_links.len() - 1;
                    }
                    self.show_hud(
                        format!("Deleted '{}'", link_name),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                    cx.notify();
                }
            }
        }
    }

    /// Handle keyboard input for the quicklinks edit/create form.
    #[allow(dead_code)] // Called from startup_new_actions.rs interceptor
    pub(crate) fn handle_quicklinks_edit_key(
        &mut self,
        key: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppView::QuicklinksEditView {
            ref editing_id,
            ref mut name,
            ref mut url_template,
            ref mut active_field,
        } = self.current_view
        {
            let active = *active_field;
            if crate::ui_foundation::is_key_tab(key) {
                *active_field = (active + 1) % 2;
                cx.notify();
            } else if crate::ui_foundation::is_key_enter(key) {
                let name_val = name.trim().to_string();
                let url_val = url_template.trim().to_string();

                // Validate
                if name_val.is_empty() {
                    self.show_error_toast("Name cannot be empty", cx);
                    return;
                }
                if url_val.is_empty() {
                    self.show_error_toast("URL cannot be empty", cx);
                    return;
                }
                if !script_kit_gpui::quicklinks::is_valid_url_template(&url_val) {
                    self.show_error_toast(
                        "URL must start with http://, https://, or contain {query}",
                        cx,
                    );
                    return;
                }

                if let Some(id) = editing_id {
                    // Update existing
                    let success =
                        script_kit_gpui::quicklinks::update_quicklink(id, &name_val, &url_val);
                    if success {
                        tracing::info!(link_id = %id, name = %name_val, action = "quicklink_updated", "Quicklink updated");
                        self.show_hud(
                            format!("Updated '{}'", name_val),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                    } else {
                        tracing::error!(link_id = %id, action = "quicklink_update_failed", "Failed to update quicklink");
                        self.show_error_toast("Failed to update quicklink", cx);
                    }
                } else {
                    // Create new
                    let link =
                        script_kit_gpui::quicklinks::create_quicklink(&name_val, &url_val);
                    let mut all = script_kit_gpui::quicklinks::load_quicklinks();
                    all.push(link);
                    script_kit_gpui::quicklinks::save_quicklinks(&all);
                    tracing::info!(name = %name_val, url = %url_val, action = "quicklink_created", "Quicklink created");
                    self.show_hud(
                        format!("Created '{}'", name_val),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                }
                // Return to browse view
                self.filter_text.clear();
                self.pending_filter_sync = true;
                self.current_view = AppView::QuicklinksBrowseView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.pending_focus = Some(FocusTarget::MainFilter);
                cx.notify();
            } else if crate::ui_foundation::is_key_escape(key) {
                // Cancel - return to browse view
                self.filter_text.clear();
                self.pending_filter_sync = true;
                self.current_view = AppView::QuicklinksBrowseView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.pending_focus = Some(FocusTarget::MainFilter);
                cx.notify();
            } else if crate::ui_foundation::is_key_backspace(key) {
                match active {
                    0 => {
                        name.pop();
                    }
                    _ => {
                        url_template.pop();
                    }
                }
                cx.notify();
            } else if key.len() == 1 {
                match active {
                    0 => name.push_str(key),
                    _ => url_template.push_str(key),
                }
                cx.notify();
            }
        }
    }

}
