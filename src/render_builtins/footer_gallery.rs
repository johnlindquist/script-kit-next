// Footer Gallery built-in view renderer

#[derive(Clone, Debug)]
pub struct FooterVariation {
    pub name: &'static str,
    pub font_family: &'static str,
    pub enter_shortcut: &'static str,
    pub actions_shortcut: &'static str,
    pub return_glyph_nudge_y: Option<f32>,
}

pub static FOOTER_VARIATIONS: &[FooterVariation] = &[
    FooterVariation {
        name: "1. Return glyph nudge 0px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(0.0),
    },
    FooterVariation {
        name: "2. Return glyph nudge 2px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(2.0),
    },
    FooterVariation {
        name: "3. Return glyph nudge 4px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(4.0),
    },
    FooterVariation {
        name: "4. Return glyph nudge 6px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(6.0),
    },
    FooterVariation {
        name: "5. Return glyph nudge 8px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(8.0),
    },
    FooterVariation {
        name: "6. Return glyph nudge 10px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(10.0),
    },
    FooterVariation {
        name: "7. Return glyph nudge 12px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(12.0),
    },
    FooterVariation {
        name: "8. Return glyph nudge 16px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(16.0),
    },
    FooterVariation {
        name: "9. Return glyph nudge 24px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(24.0),
    },
    FooterVariation {
        name: "10. Return glyph nudge 100px",
        font_family: "SF Mono",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(100.0),
    },
    FooterVariation {
        name: "11. Menlo return glyph nudge 4px",
        font_family: "Menlo",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(4.0),
    },
    FooterVariation {
        name: "12. Monaco return glyph nudge 4px",
        font_family: "Monaco",
        enter_shortcut: "↵",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: Some(4.0),
    },
    FooterVariation {
        name: "13. SF Mono (↩ / ⌘K)",
        font_family: "SF Mono",
        enter_shortcut: "↩",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: None,
    },
    FooterVariation {
        name: "14. SF Mono (⏎ / ⌘K)",
        font_family: "SF Mono",
        enter_shortcut: "⏎",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: None,
    },
    FooterVariation {
        name: "15. SF Mono (Enter / ⌘K)",
        font_family: "SF Mono",
        enter_shortcut: "Enter",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: None,
    },
    FooterVariation {
        name: "16. SF Mono (Return / ⌘K)",
        font_family: "SF Mono",
        enter_shortcut: "Return",
        actions_shortcut: "⌘K",
        return_glyph_nudge_y: None,
    },
];

pub(crate) fn footer_gallery_filtered_len(filter: &str) -> usize {
    if filter.is_empty() {
        FOOTER_VARIATIONS.len()
    } else {
        let filter_lower = filter.to_lowercase();
        FOOTER_VARIATIONS
            .iter()
            .filter(|v| {
                v.name.to_lowercase().contains(&filter_lower)
                    || v.font_family.to_lowercase().contains(&filter_lower)
            })
            .count()
    }
}

impl ScriptListApp {
    fn footer_gallery_visible_rows(filter: &str) -> Vec<FooterVariation> {
        if filter.is_empty() {
            FOOTER_VARIATIONS.to_vec()
        } else {
            let filter_lower = filter.to_lowercase();
            FOOTER_VARIATIONS
                .iter()
                .filter(|v| {
                    v.name.to_lowercase().contains(&filter_lower)
                        || v.font_family.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect()
        }
    }

    pub(crate) fn footer_gallery_visible_row_labels(filter: &str) -> Vec<String> {
        Self::footer_gallery_visible_rows(filter)
            .iter()
            .map(|v| v.name.to_string())
            .collect()
    }

    fn footer_gallery_count_label(filtered_len: usize) -> String {
        let suffix = if filtered_len == 1 { "" } else { "s" };
        format!("{} variation{}", filtered_len, suffix)
    }

    /// Render footer gallery showing 25 different versions of the footer
    fn render_footer_gallery(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for global styling
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();

        let color_resolver =
            crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design);
        let empty_text_color = color_resolver.empty_text_color();
        let empty_font_family = typography_resolver.primary_font().to_string();

        let text_primary = self.theme.colors.text.primary;

        // Build list of filtered variations
        let filtered_items = Self::footer_gallery_visible_rows(&filter);
        let filtered_len = filtered_items.len();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FooterGalleryView { selected_index, .. } = &mut this.current_view {
                    let current_filtered_len = filtered_len;

                    match key {
                        _ if is_key_up(key) => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.footer_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        _ if is_key_down(key) => {
                            if *selected_index < current_filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.footer_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        // Text editing is owned by the shared GPUI input and
                        // routed through InputEvent::Change in startup.rs.
                        _ => {}
                    }
                }
            },
        );

        let list_element: AnyElement = if filtered_len == 0 {
            crate::list_item::EmptyState::new(
                "No footer variations match your search",
                empty_text_color,
                &empty_font_family,
            )
            .icon(crate::designs::icon_variations::IconName::StarFilled)
            .into_element()
        } else {
            let items_for_closure = filtered_items.clone();
            let selected = selected_index;
            let theme_ref = self.theme.clone();
            let design_colors_clone = design_colors;

            uniform_list(
                "footer-gallery",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some(var) = items_for_closure.get(ix) {
                                let is_selected = ix == selected;

                                let label_color = if is_selected {
                                    rgb(design_colors_clone.accent)
                                } else {
                                    rgb(design_colors_clone.text_primary)
                                };

                                // Render the variation details
                                let header = div()
                                    .w_full()
                                    .px(px(16.0))
                                    .py(px(4.0))
                                    .flex()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(label_color)
                                            .child(var.name),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(design_colors_clone.text_muted))
                                            .child(format!("Font: {}", var.font_family)),
                                    );

                                // Instantiate the real PromptFooter component
                                let config = crate::components::prompt_footer::PromptFooterConfig::new()
                                    .primary_label("Run Script")
                                    .primary_shortcut(var.enter_shortcut)
                                    .secondary_label("Actions")
                                    .secondary_shortcut(var.actions_shortcut)
                                    .shortcut_font_family(var.font_family)
                                    .shortcut_return_glyph_nudge_y(var.return_glyph_nudge_y.unwrap_or(4.0))
                                    .show_logo(true)
                                    .show_primary(true)
                                    .show_secondary(true)
                                    .show_info_label(true)
                                    .info_label(format!("Preview {}", ix + 1));

                                let footer_colors = crate::components::prompt_footer::PromptFooterColors::from_theme(&theme_ref);
                                let footer_preview = crate::components::prompt_footer::PromptFooter::new(config, footer_colors);

                                let mut row_div = div()
                                    .id(ElementId::NamedInteger("footer-gallery-row".into(), ix as u64))
                                    .w_full()
                                    .h(px(80.0))
                                    .flex()
                                    .flex_col()
                                    .justify_between()
                                    .child(header)
                                    .child(footer_preview);

                                if is_selected {
                                    row_div = row_div.bg(rgba(
                                        (design_colors_clone.background_selected << 8) | 0x0f,
                                    )); // ~6% opacity
                                }

                                row_div.into_any_element()
                            } else {
                                div()
                                    .id(ElementId::NamedInteger("footer-gallery-empty".into(), ix as u64))
                                    .h(px(80.0))
                                    .into_any_element()
                            }
                        })
                        .collect()
                },
            )
            .w_full()
            .h_full()
            .track_scroll(&self.footer_gallery_scroll_handle)
            .into_any_element()
        };

        let footer_hints: Vec<SharedString> = vec!["Esc Back".into()];
        crate::components::emit_surface_prompt_hint_audit(
            "footer_gallery",
            &footer_hints,
            "footer_gallery_footer",
        );

        let footer = div()
            .id("footer-gallery-footer-tooltip")
            .child(crate::components::render_simple_hint_strip(
                footer_hints,
                None,
            ))
            .into_any_element();
        let footer = self.main_window_footer_slot(footer);

        let content = div()
            .flex_1()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .child(list_element);

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;

        crate::components::main_view_chrome::render_main_view_chrome(
            crate::components::main_view_chrome::render_main_view_shell()
                .key_context("footer_gallery")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(vec![
                    self.render_builtin_main_input_count_label(Self::footer_gallery_count_label(
                        filtered_len,
                    )),
                ]),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main: content.into_any_element(),
                footer,
                overlays: Vec::new(),
            },
        )
    }
}
