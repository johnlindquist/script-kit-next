/// Permissions wizard: guided grant flow for the macOS permissions Script Kit
/// uses (model lives in `crate::permissions_wizard`).
///
/// Rows re-detect TCC status on every render, and a 2s poll keeps the surface
/// live while the user flips to System Settings and back — TCC publishes no
/// grant notifications, so polling is the reliable pattern here.
///
/// Grant flow per row (Enter/click): fire the one-shot native OS prompt via
/// `permissions_wizard::request_permission`; if the grant doesn't land
/// immediately, fall back to the Permiso assistant (Accessibility / Screen
/// Recording) or the matching System Settings pane, and let the poll flip the
/// card to Granted when the user returns.
fn permissions_wizard_rows() -> Vec<(
    crate::permissions_wizard::PermissionKind,
    crate::platform::permiso_detect::PermissionStatus,
)> {
    crate::permissions_wizard::PermissionKind::all()
        .iter()
        .map(|&kind| (kind, crate::permissions_wizard::detect_permission(kind)))
        .collect()
}

fn permissions_wizard_requirement_label(
    kind: crate::permissions_wizard::PermissionKind,
) -> &'static str {
    use crate::permissions_wizard::PermissionRequirement;
    match kind.requirement() {
        PermissionRequirement::Required => "Required",
        PermissionRequirement::Recommended => "Recommended",
        PermissionRequirement::Optional => "Optional",
    }
}

impl ScriptListApp {
    /// Apply the permission onboarding decision at startup.
    ///
    /// Fresh installs (or installs that never completed onboarding) with
    /// missing required permissions get the full wizard and the launcher is
    /// shown; installs that already completed onboarding get a reminder toast
    /// plus an OS notification (the launcher is usually hidden at startup, so
    /// the notification is the part the user actually sees).
    pub(crate) fn apply_permission_startup_intent(
        &mut self,
        is_fresh_install: bool,
        cx: &mut Context<Self>,
    ) {
        match crate::permissions_wizard::startup_intent(is_fresh_install) {
            crate::permissions_wizard::PermissionStartupIntent::OpenFullWizard => {
                tracing::info!(
                    event = "permission_onboarding_full_wizard",
                    is_fresh_install,
                    "Opening permissions wizard at startup"
                );
                self.open_permissions_wizard(cx);
                script_kit_gpui::request_show_main_window();
            }
            crate::permissions_wizard::PermissionStartupIntent::ShowReminder { missing } => {
                let names = missing
                    .iter()
                    .map(|kind| kind.name())
                    .collect::<Vec<_>>()
                    .join(", ");
                tracing::info!(
                    event = "permission_onboarding_reminder",
                    missing = %names,
                    "Missing required permissions at startup"
                );
                self.toast_manager.push(
                    components::toast::Toast::warning(
                        format!(
                            "Missing required permissions: {names}. Run \u{201c}Set Up Permissions\u{201d} to grant them."
                        ),
                        &self.theme,
                    )
                    .duration_ms(Some(TOAST_WARNING_MS)),
                );
                let _ = notify_rust::Notification::new()
                    .summary("Script Kit needs permissions")
                    .body(&format!(
                        "{names} not granted. Open Script Kit and run \u{201c}Set Up Permissions\u{201d}."
                    ))
                    .show();
                cx.notify();
            }
            crate::permissions_wizard::PermissionStartupIntent::None => {}
        }
    }

    pub(crate) fn open_permissions_wizard(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::PermissionsWizardView { .. }) {
            self.current_view = AppView::PermissionsWizardView { selected_index: 0 };
            self.spawn_permissions_wizard_poll(cx);
        }
        cx.notify();
    }

    /// Re-render the wizard every 2s while it is the active view so cards flip
    /// to Granted after the user toggles Script Kit in System Settings.
    /// The loop exits as soon as the view changes, so reopening the wizard
    /// (which spawns a fresh loop) never accumulates pollers.
    fn spawn_permissions_wizard_poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(2))
                    .await;

                let mut keep_polling = false;
                let _ = cx.update(|cx| {
                    let _ = this.update(cx, |app, cx| {
                        keep_polling =
                            matches!(app.current_view, AppView::PermissionsWizardView { .. });
                        if keep_polling && script_kit_gpui::is_main_window_visible() {
                            cx.notify();
                        }
                    });
                });

                if !keep_polling {
                    break;
                }
            }
        })
        .detach();
    }

    fn execute_permissions_wizard_grant(
        &mut self,
        kind: crate::permissions_wizard::PermissionKind,
        cx: &mut Context<Self>,
    ) {
        use crate::platform::permiso_detect::PermissionStatus;

        if crate::permissions_wizard::detect_permission(kind) == PermissionStatus::Authorized {
            self.show_hud(
                format!("{} already granted", kind.name()),
                Some(HUD_SHORT_MS),
                cx,
            );
            cx.notify();
            return;
        }

        if crate::permissions_wizard::request_permission(kind) == PermissionStatus::Authorized {
            self.show_hud(
                format!("{} granted", kind.name()),
                Some(HUD_SHORT_MS),
                cx,
            );
            cx.notify();
            return;
        }

        let opened = match kind {
            crate::permissions_wizard::PermissionKind::Accessibility => {
                platform::permiso::PermisoAssistant::present_retained(
                    platform::permiso::PermisoPanel::Accessibility,
                )
                .is_ok()
            }
            crate::permissions_wizard::PermissionKind::ScreenRecording => {
                platform::permiso::PermisoAssistant::present_retained(
                    platform::permiso::PermisoPanel::ScreenRecording,
                )
                .is_ok()
            }
            _ => crate::permissions_wizard::open_permission_settings(kind).is_ok(),
        };

        if opened {
            self.toast_manager.push(
                components::toast::Toast::info(
                    format!(
                        "Enable \u{201c}{}\u{201d} for Script Kit in System Settings — this list updates automatically",
                        kind.name()
                    ),
                    &self.theme,
                )
                .duration_ms(Some(TOAST_INFO_MS)),
            );
        } else {
            self.show_error_toast(
                format!("Couldn't open System Settings for {}", kind.name()),
                cx,
            );
        }
        cx.notify();
    }

    /// Render the permissions wizard using the same contracted shell as other
    /// built-in views, with a static title in place of the filter input.
    fn render_permissions_wizard(
        &mut self,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list("permissions_wizard", true),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let chrome = theme::AppChromeColors::from_theme(&self.theme);
        let info_palette = crate::components::info_palette(&self.theme);
        let success_color = gpui::rgb(self.theme.colors.ui.success);

        let rows = permissions_wizard_rows();
        let granted_count = rows
            .iter()
            .filter(|(_, status)| {
                *status == crate::platform::permiso_detect::PermissionStatus::Authorized
            })
            .count();
        let row_count = rows.len();
        let all_required_granted =
            crate::permissions_wizard::PermissionSnapshot::current().all_required_granted();
        let list_colors = ListItemColors::from_theme(&self.theme);

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if is_key_escape(key) {
                    crate::permissions_wizard::mark_onboarding_completed();
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    crate::permissions_wizard::mark_onboarding_completed();
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                let current_selected = if let AppView::PermissionsWizardView { selected_index } =
                    &this.current_view
                {
                    *selected_index
                } else {
                    return;
                };

                let row_count = crate::permissions_wizard::PermissionKind::all().len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::PermissionsWizardView { selected_index } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_down(key) {
                    if current_selected < row_count.saturating_sub(1) {
                        if let AppView::PermissionsWizardView { selected_index } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_enter(key) {
                    if let Some(&kind) =
                        crate::permissions_wizard::PermissionKind::all().get(current_selected)
                    {
                        this.execute_permissions_wizard_grant(kind, cx);
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        let entity = cx.entity().downgrade();
        let hovered = self.hovered_index;

        let list_items: Vec<AnyElement> = rows
            .iter()
            .enumerate()
            .map(|(ix, (kind, status))| {
                let kind = *kind;
                let granted = *status
                    == crate::platform::permiso_detect::PermissionStatus::Authorized;
                let is_selected = ix == selected_index;
                let is_hovered = hovered == Some(ix);
                let entity_click = entity.clone();
                let entity_hover = entity.clone();

                let badge: AnyElement = if granted {
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(success_color)
                        .child("✓ Granted")
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(info_palette.hint)
                        .child(permissions_wizard_requirement_label(kind))
                        .into_any_element()
                };

                div()
                    .id(ix)
                    .cursor_pointer()
                    .on_click(move |event, window, cx| {
                        if let Some(app) = entity_click.upgrade() {
                            app.update(cx, |this, cx| {
                                let was_selected =
                                    if let AppView::PermissionsWizardView { selected_index } =
                                        &mut this.current_view
                                    {
                                        let was_selected = *selected_index == ix;
                                        *selected_index = ix;
                                        was_selected
                                    } else {
                                        false
                                    };
                                let click_count = event.click_count();
                                if crate::ui_foundation::should_submit_selected_row_click(
                                    was_selected,
                                    click_count,
                                ) {
                                    this.execute_permissions_wizard_grant(kind, cx);
                                } else {
                                    cx.notify();
                                }
                                let _ = window;
                            });
                        }
                        cx.stop_propagation();
                    })
                    .on_hover({
                        let entity_h = entity_hover;
                        move |is_hovered: &bool, _window: &mut Window, cx: &mut gpui::App| {
                            if let Some(app) = entity_h.upgrade() {
                                app.update(cx, |this, cx| {
                                    if *is_hovered {
                                        this.input_mode = InputMode::Mouse;
                                        if this.hovered_index != Some(ix) {
                                            this.hovered_index = Some(ix);
                                            cx.notify();
                                        }
                                    } else if this.hovered_index == Some(ix) {
                                        this.hovered_index = None;
                                        cx.notify();
                                    }
                                });
                            }
                        }
                    })
                    .child(
                        ListItem::new(kind.name().to_string(), list_colors)
                            .icon_kind_opt(crate::list_item::IconKind::from_icon_hint(kind.icon()))
                            .description_opt(Some(kind.subtitle().to_string()))
                            .trailing_accessory(badge)
                            .selected(is_selected)
                            .hovered(is_hovered)
                            .with_accent_bar(is_selected),
                    )
                    .into_any_element()
            })
            .collect();

        let intro = div()
            .w_full()
            .px(px(design_spacing.padding_md))
            .pb(px(design_spacing.padding_xs))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(crate::components::INFO_TYPE_SCALE.body.size))
                    .line_height(px(crate::components::INFO_TYPE_SCALE.body.line))
                    .text_color(info_palette.body)
                    .child(
                        "Script Kit uses these macOS permissions to read selected text, \
                         paste into other apps, run shortcuts, and capture context. \
                         Press ↵ on a row to grant it — cards update automatically.",
                    ),
            );

        let footer_note: Option<AnyElement> = if all_required_granted {
            Some(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_md))
                    .pt(px(design_spacing.padding_xs))
                    .text_xs()
                    .text_color(success_color)
                    .child("All required permissions granted — press Esc to finish.")
                    .into_any_element(),
            )
        } else {
            None
        };

        let content = div()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .flex()
            .flex_col()
            .child(intro)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .min_h(px(0.))
                    .children(list_items),
            )
            .children(footer_note);

        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            vec![
                gpui::SharedString::from("↵ Grant"),
                gpui::SharedString::from("Esc Done"),
            ],
            None,
        ));

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;

        let header_title = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(chrome.text_primary_hex))
                    .child("Set Up Permissions"),
            )
            .child(self.render_builtin_main_input_count_label(format!(
                "{granted_count} of {row_count} granted"
            )))
            .into_any_element();

        crate::components::main_view_chrome::render_main_view_chrome(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(rgb(chrome.text_primary_hex))
                .font_family(self.theme_font_family())
                .key_context("permissions_wizard")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: crate::components::main_view_chrome::MainViewHeaderChrome {
                    context: None,
                    input: header_title,
                    padding_x: shell.header_padding_x,
                    padding_y: shell.header_padding_y,
                    gap: shell.header_gap,
                },
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
