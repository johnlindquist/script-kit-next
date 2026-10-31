/// The front window as the line above the input names it: its title, beside the app's own icon.
/// A title that ends with the app's own name ("README.md — Zed") drops it, since the line already
/// says "in Zed".
pub(crate) fn front_window_subject(app: &crate::frontmost_app_tracker::TrackedApp) -> sk_one_window::Subject {
    let window_title = app
        .window_title
        .as_deref()
        .map(str::trim)
        .map(|title| without_app_suffix(title, &app.name))
        .filter(|title| !title.is_empty() && *title != app.name);
    sk_one_window::Subject {
        kind: sk_one_window::SubjectKind::Window,
        label: "Front window".into(),
        title: window_title.unwrap_or(&app.name).to_string(),
        sub: if window_title.is_some() {
            app.name.clone()
        } else {
            String::new()
        },
        icon: app_icon_name(&app.bundle_id),
        row_key: Some(format!("front-pid:{}",app.pid)),
        ..Default::default()
    }
}

/// `title` without a trailing " — App", " – App" or " - App" naming `app`.
fn without_app_suffix<'a>(title: &'a str, app: &str) -> &'a str {
    if app.is_empty() {
        return title;
    }
    for dash in [" \u{2014} ", " \u{2013} ", " - "] {
        if let Some(rest) = title
            .strip_suffix(app)
            .and_then(|rest| rest.strip_suffix(dash))
        {
            return rest.trim_end();
        }
    }
    title
}

/// The line's icon for an app: its own icon when the launcher knows its bundle.
fn app_icon_name(bundle_id: &str) -> String {
    let bundle_id = bundle_id.trim();
    if bundle_id.is_empty() {
        "window".to_string()
    } else {
        format!(
            "{}{bundle_id}",
            crate::one_window::slot_line::APP_ICON_PREFIX
        )
    }
}

impl ScriptListApp {
    /// Refresh the passive "selected text" sniff that powers the header hint
    /// chip. Runs `get_selected_text_ax_only` on the background executor — it
    /// never posts keystrokes and never reads or writes the pasteboard, so it
    /// is safe to run speculatively on every show. Clears the previous hint
    /// synchronously so a stale selection from the last show can never flash.
    pub(crate) fn refresh_shown_selection_hint(&mut self, cx: &mut Context<Self>) {
        self.shown_selection_hint_token = self.shown_selection_hint_token.wrapping_add(1);
        let token = self.shown_selection_hint_token;
        if self.shown_selection_hint_text.take().is_some() {
            cx.notify();
        }
        // Target the app the user came FROM by pid: once our panel shows, the
        // system-wide focused element can already point at Script Kit itself.
        let source_app = crate::frontmost_app_tracker::get_last_real_app();
        let source_pid = source_app.as_ref().map(|app| app.pid);
        self.begin_subject_tabs(source_app.as_ref().map(|a| crate::one_window::subject_tabs::window_source(front_window_subject(a), crate::frontmost_app_tracker::observed_window_age_ms())),token,cx);
        if let Some(app) = &source_app {
            self.focus_front_window(app, cx);
            // Tab and same-app window switches change the title without an
            // activation, so the tracker's cached title may be stale.
            let shown = app.clone();
            cx.spawn(async move |this, cx| {
                let fresh = cx
                    .background_executor()
                    .spawn(async { crate::frontmost_app_tracker::refresh_front_window_title() })
                    .await;
                let _ = this.update(cx, |view, cx| {
                    let Some(fresh) = fresh else {
                        return;
                    };
                    if view.shown_selection_hint_token != token
                        || fresh.pid != shown.pid
                        || fresh.window_title == shown.window_title
                    {
                        return;
                    }
                    if view.shown_selection_hint_text.is_none() {view.focus_front_window(&fresh, cx);} else {view.change_subjects(sk_one_window::window::subject_tabs::TabsEvent::Observe(crate::one_window::subject_tabs::window_source(front_window_subject(&fresh),crate::frontmost_app_tracker::observed_window_age_ms())),cx);}
                    cx.notify();
                });
            })
            .detach();
        }
        let source_icon = source_app
            .as_ref()
            .map(|app| app_icon_name(&app.bundle_id))
            .unwrap_or_else(|| "selection".to_string());
        let source_name = source_app.map(|app| app.name);
        cx.spawn(async move |this, cx| {
            // Passive AX-only reads: selection first, then the focused
            // field's whole text ("draft") so the style rows can preview what
            // a rewrite would capture even when nothing is selected.
            let result = cx
                .background_executor()
                .spawn(async move {
                    match crate::platform::accessibility::focused_text::selected_text_for_app_ax_only(
                        source_pid,
                    ) {
                        Ok(Some(text)) => Ok((Some(text), false)),
                        Ok(None) | Err(_) => {
                            crate::platform::accessibility::focused_text::focused_text_for_app_ax_only(
                                source_pid,
                            )
                            .map(|draft| (draft, true))
                        }
                    }
                })
                .await;
            let error = result.as_ref().err().map(|e| e.to_string());
            let (selection, is_draft) = match result {
                Ok((text, is_draft)) => (text.filter(|t| !t.trim().is_empty()), is_draft),
                Err(_) => (None, false),
            };
            let _ = this.update(cx, |app, cx| {
                if app.shown_selection_hint_token != token {
                    return;
                }
                tracing::info!(
                    target: "script_kit::selection_hint",
                    event = "selection_hint_sniff_complete",
                    has_selection = selection.is_some(),
                    is_draft,
                    chars = selection.as_deref().map(|s| s.chars().count()).unwrap_or(0),
                    source_app = source_name.as_deref().unwrap_or("unknown"),
                    source_pid = source_pid.unwrap_or(-1),
                    error = error.as_deref().unwrap_or(""),
                );
                // The header chip stays selection-only (a chip for every
                // focused text field would be noise); the preview cache gets
                // both so style rows and the submit-freeze agree.
                app.shown_selection_hint_text = if is_draft { None } else { selection.clone() };
                if let Some(text) = app.shown_selection_hint_text.as_deref() {
                    let snippet =
                        crate::components::main_view_chrome::selection_hint_snippet(text, 48);
                    let selected = sk_one_window::Subject {
                        kind: sk_one_window::SubjectKind::Selection,
                        label: "Selected".into(),
                        title: format!("\u{201c}{snippet}\u{201d}"),
                        sub: source_name
                            .as_deref()
                            .map(|name| format!("selected in {name}"))
                            .unwrap_or_default(),
                        icon: source_icon.clone(),
                        ..Default::default()
                    };
                    let source=crate::one_window::subject_tabs::selection_source(selected.clone(),text);
                    app.focus_slot_line(selected,cx);
                    app.change_subjects(sk_one_window::window::subject_tabs::TabsEvent::Observe(source),cx);
                }
                app.spine_live_preview_cache.seed_selection_preview(
                    selection,
                    is_draft,
                    source_name.clone(),
                );
                cx.notify();
            });
        })
        .detach();
    }

    /// What "this" means in the line above the input.
    fn focus_slot_line(&mut self, subject: sk_one_window::Subject, cx: &mut Context<Self>) {
        self.one_window_handle(sk_one_window::Event::Focused(subject), cx);
    }

    /// The line names `app`'s front window as "this".
    pub(crate) fn focus_front_window(
        &mut self,
        app: &crate::frontmost_app_tracker::TrackedApp,
        cx: &mut Context<Self>,
    ) {
        let subject=front_window_subject(app);
        if self.one_window.tabs_enabled() { self.change_subjects(sk_one_window::window::subject_tabs::TabsEvent::Observe(crate::one_window::subject_tabs::window_source(subject.clone(),crate::frontmost_app_tracker::observed_window_age_ms())),cx); }
        self.focus_slot_line(subject, cx);
    }

    /// Header hint chip shown only while the main menu is up AND the passive
    /// sniff found selected text in the app the user came from.
    pub(crate) fn selection_hint_chip(
        &self,
    ) -> Option<crate::components::main_view_chrome::SemanticChipSpec> {
        if !matches!(self.current_view, AppView::ScriptList) {
            return None;
        }
        let fixture_text = (std::env::var("SCRIPT_KIT_TEST_STATUS").ok().as_deref() == Some("1")
            && std::env::var("SCRIPT_KIT_TEST_CONTEXT_CHIP_FIXTURE")
                .ok()
                .as_deref()
                == Some("selection"))
        .then_some("Fixture selected text");
        let text = self.shown_selection_hint_text.as_deref().or(fixture_text)?;
        Some(
            crate::components::main_view_chrome::SemanticChipSpec::context_attachment(
                crate::components::main_view_chrome::MAIN_VIEW_CONTEXT_SELECTION_BUTTON_ID,
                format!(
                    "Selected: \u{201c}{}\u{201d}",
                    crate::components::main_view_chrome::selection_hint_snippet(text, 24)
                ),
                false,
            ),
        )
    }

    /// What Tab actually does on the current surface — the single source of
    /// truth for the header Tab chip. MUST mirror the Tab interceptor's
    /// branch order in `startup.rs` (menu-syntax pickers → empty-input cwd
    /// pick → directory-browse completion → Quick AI) so the chip never
    /// advertises an action Tab won't take.
    pub(crate) fn main_header_tab_chip_action(
        &self,
    ) -> crate::components::main_view_chrome::MainViewTabChipAction {
        use crate::components::main_view_chrome::MainViewTabChipAction;

        if !matches!(self.current_view, AppView::ScriptList) || self.show_actions_popup {
            return MainViewTabChipAction::Inactive;
        }
        // Menu-syntax pickers claim Tab first and show their own hints.
        if self.menu_syntax_object_selector_owns_main_keyboard()
            || self.menu_syntax_trigger_picker_owns_main_keyboard()
        {
            return MainViewTabChipAction::Inactive;
        }
        if self.filter_text.trim().is_empty() {
            if self.spine_enabled {
                return MainViewTabChipAction::ChangeCwd;
            }
            return MainViewTabChipAction::Inactive;
        }
        // Directory-browse queries keep Tab for path completion.
        if crate::file_search::looks_like_root_directory_browse_query(&self.filter_text) {
            return MainViewTabChipAction::Inactive;
        }
        MainViewTabChipAction::QuickAi
    }

    /// Whether Shift+Tab opens the profile (agent/model) picker on the
    /// current surface — mirrors `try_open_profile_search_from_script_list_shift_tab`.
    fn main_header_shift_tab_key_active(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && self.spine_enabled
            && !self.show_actions_popup
    }

    pub(crate) fn main_view_context_zone_spec(
        &self,
    ) -> crate::components::main_view_chrome::MainViewContextZoneSpec {
        use crate::components::main_view_chrome::{
            MainViewContextZoneSpec, MainViewTabChipAction, SemanticChipAction, SemanticChipSpec,
            MAIN_VIEW_AGENT_MODEL_UNAVAILABLE_LABEL, MAIN_VIEW_CONTEXT_CWD_BUTTON_ID,
            MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID, MAIN_VIEW_CONTEXT_QUICK_AI_BUTTON_ID,
            MAIN_VIEW_CWD_UNAVAILABLE_LABEL, MAIN_VIEW_QUICK_AI_CHIP_LABEL,
        };

        let fixture_unavailable = std::env::var("SCRIPT_KIT_TEST_STATUS").ok().as_deref()
            == Some("1")
            && std::env::var("SCRIPT_KIT_TEST_CONTEXT_CHIP_FIXTURE")
                .ok()
                .as_deref()
                == Some("unavailable");
        let mut cwd = (!fixture_unavailable)
            .then(|| self.global_footer_cwd_chip().map(|chip| chip.label))
            .flatten()
            .or_else(|| {
                (!fixture_unavailable)
                    .then(|| {
                        std::env::current_dir()
                            .ok()
                            .map(|cwd| crate::file_search::shorten_path(&cwd.to_string_lossy()))
                    })
                    .flatten()
            });
        let flow_session_identity = match &self.current_view {
            AppView::FlowSessionView { session_id } => self
                .conversations
                .flow_sessions
                .iter()
                .find(|(meta, _)| meta.id == *session_id)
                .map(|(meta, _)| {
                    crate::flows::session::FlowSessionIdentitySnapshot::from_meta(meta)
                }),
            _ => None,
        };
        if let Some(identity) = &flow_session_identity {
            cwd = Some(identity.cwd_display.clone());
        }
        let cwd_available = cwd.is_some();
        let cwd_label = cwd.unwrap_or_else(|| MAIN_VIEW_CWD_UNAVAILABLE_LABEL.to_string());

        // In a flow session the active agent IS the flow: the shared
        // Agent·Model chip carries "<flow> · <engine>" rather than the global
        // spine agent, which is not what this conversation talks to.
        let flow_session_label = flow_session_identity.as_ref().map(|identity| {
            let model = identity
                .model
                .as_deref()
                .map(|model| format!(" · {model}"))
                .unwrap_or_default();
            let rethread = if identity.needs_rethread {
                " · reconnecting"
            } else {
                ""
            };
            format!(
                "{} · {}{} · {}{}",
                identity.friendly_name,
                identity.engine,
                model,
                if identity.read_only {
                    "Archived"
                } else {
                    "Active"
                },
                rethread,
            )
        });
        let agent_model = (!fixture_unavailable)
            .then(|| flow_session_label.or_else(|| self.agent_model_footer_label()))
            .flatten();
        let agent_model_available = agent_model.is_some();
        let agent_model_label =
            agent_model.unwrap_or_else(|| MAIN_VIEW_AGENT_MODEL_UNAVAILABLE_LABEL.to_string());

        let leading_identity = if fixture_unavailable {
            SemanticChipSpec::disabled_identity(
                MAIN_VIEW_CONTEXT_CWD_BUTTON_ID,
                cwd_label,
                "No working directory is available",
            )
        } else {
            match self.main_header_tab_chip_action() {
                MainViewTabChipAction::QuickAi => SemanticChipSpec::enabled_identity(
                    MAIN_VIEW_CONTEXT_QUICK_AI_BUTTON_ID,
                    MAIN_VIEW_QUICK_AI_CHIP_LABEL,
                    SemanticChipAction::OpenSurface,
                    "⇥",
                ),
                MainViewTabChipAction::ChangeCwd if cwd_available => {
                    SemanticChipSpec::enabled_identity(
                        MAIN_VIEW_CONTEXT_CWD_BUTTON_ID,
                        cwd_label,
                        SemanticChipAction::OpenSelector,
                        "⇥",
                    )
                }
                MainViewTabChipAction::ChangeCwd | MainViewTabChipAction::Inactive => {
                    SemanticChipSpec::disabled_identity(
                        MAIN_VIEW_CONTEXT_CWD_BUTTON_ID,
                        cwd_label,
                        if cwd_available {
                            "Tab is owned by the current surface"
                        } else {
                            "No working directory is available"
                        },
                    )
                }
            }
        };
        let trailing_identity = if self.main_header_shift_tab_key_active() && agent_model_available
        {
            SemanticChipSpec::enabled_identity(
                MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID,
                agent_model_label,
                SemanticChipAction::OpenSelector,
                "⇧⇥",
            )
        } else {
            SemanticChipSpec::disabled_identity(
                MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID,
                agent_model_label,
                if agent_model_available {
                    "The profile selector is unavailable on this surface"
                } else {
                    "No agent model is available"
                },
            )
        };

        match MainViewContextZoneSpec::try_new(
            leading_identity.clone(),
            self.selection_hint_chip(),
            trailing_identity.clone(),
        ) {
            Ok(zone) => zone,
            Err(error) => {
                tracing::error!(error, "main_view.context_zone_rejected_invalid_attachment");
                MainViewContextZoneSpec {
                    leading_identity,
                    context_attachment: None,
                    trailing_identity,
                }
            }
        }
    }

    pub(crate) fn main_view_context_chip_has_action(
        &self,
        semantic_id: &str,
        action: crate::components::main_view_chrome::SemanticChipAction,
    ) -> bool {
        let zone = self.main_view_context_zone_spec();
        let enabled = [
            Some(&zone.leading_identity),
            zone.context_attachment.as_ref(),
            Some(&zone.trailing_identity),
        ]
        .into_iter()
        .flatten()
        .any(|chip| {
            chip.semantic_id.as_ref() == semantic_id
                && chip.enabled
                && chip.body_action == Some(action)
        });
        enabled
    }
}
