use super::*;

impl ScriptListApp {
    pub(crate) fn cycle_design(&mut self, cx: &mut Context<Self>) {
        let old_design = self.current_design;
        let new_design = old_design.next();
        let all_designs = DesignVariant::all();
        let old_idx = all_designs
            .iter()
            .position(|&v| v == old_design)
            .unwrap_or(0);
        let new_idx = all_designs
            .iter()
            .position(|&v| v == new_design)
            .unwrap_or(0);

        logging::log(
            "DESIGN",
            &format!(
                "Cycling design: {} ({}) -> {} ({}) [total: {}]",
                old_design.name(),
                old_idx,
                new_design.name(),
                new_idx,
                all_designs.len()
            ),
        );
        logging::log(
            "DESIGN",
            &format!(
                "Design '{}': {}",
                new_design.name(),
                new_design.description()
            ),
        );

        self.current_design = new_design;
        logging::log(
            "DESIGN",
            &format!("self.current_design is now: {:?}", self.current_design),
        );
        cx.notify();
    }

    pub(crate) fn update_theme(&mut self, cx: &mut Context<Self>) {
        let base_theme = theme::load_theme();

        // Preserve opacity offset in light mode, reset in dark mode
        if base_theme.is_dark_mode() {
            self.light_opacity_offset = 0.0;
            self.theme = std::sync::Arc::new(base_theme);
        } else if self.light_opacity_offset != 0.0 {
            // Apply the opacity offset if set
            self.theme =
                std::sync::Arc::new(base_theme.with_opacity_offset(self.light_opacity_offset));
        } else {
            self.theme = std::sync::Arc::new(base_theme);
        }

        logging::log("APP", "Theme reloaded based on system appearance");

        // Propagate theme to open ActionsDialog (if any) for hot-reload support
        if let Some(ref dialog) = self.actions_dialog {
            let theme_arc = std::sync::Arc::clone(&self.theme);
            dialog.update(cx, |d, _| {
                d.update_theme(theme_arc);
            });
            logging::log("APP", "Theme propagated to ActionsDialog");
        }

        cx.notify();
    }

    pub(crate) fn update_config(&mut self, cx: &mut Context<Self>) {
        self.config = config::load_config();
        clipboard_history::set_max_text_content_len(
            self.config.get_clipboard_history_max_text_length(),
        );
        // Hot-reload hotkeys from updated config
        hotkeys::update_hotkeys(&self.config);
        logging::log(
            "APP",
            &format!("Config reloaded: padding={:?}", self.config.get_padding()),
        );
        cx.notify();
    }

    /// Adjust the light theme opacity by a delta amount
    ///
    /// Use Cmd+Shift+[ to decrease and Cmd+Shift+] to increase.
    /// The offset is clamped to the range -0.5 to +0.5.
    pub(crate) fn adjust_light_opacity(&mut self, delta: f32, cx: &mut Context<Self>) {
        // Only adjust if we're in light mode
        let base_theme = theme::load_theme();
        if base_theme.is_dark_mode() {
            logging::log("APP", "Opacity adjustment only works in light mode");
            return;
        }

        // Adjust the offset
        self.light_opacity_offset = (self.light_opacity_offset + delta).clamp(-0.5, 0.5);

        // Create new theme with adjusted opacity
        let adjusted_theme = base_theme.with_opacity_offset(self.light_opacity_offset);
        self.theme = std::sync::Arc::new(adjusted_theme);

        let new_opacity = self.theme.get_opacity().main;
        logging::log(
            "APP",
            &format!(
                "Light opacity adjusted: offset={:.2}, main={:.2}",
                self.light_opacity_offset, new_opacity
            ),
        );

        // Show toast with current opacity level
        let percent = (new_opacity * 100.0).round() as i32;
        self.toast_manager.push(components::toast::Toast::info(
            format!("Opacity: {}%", percent),
            &self.theme,
        ));

        cx.notify();
    }

    /// Request focus for a specific target. Focus will be applied once on the
    /// next render when window access is available, then cleared.
    ///
    /// This avoids the "perpetually enforce focus in render()" anti-pattern.
    /// Use this instead of directly calling window.focus() from non-render code.
    #[allow(dead_code)] // Public API for external callers without direct pending_focus access
    pub fn request_focus(&mut self, target: FocusTarget, cx: &mut Context<Self>) {
        self.pending_focus = Some(target);
        cx.notify();
    }

    // === FocusCoordinator Integration Methods ===
    // These methods provide a unified focus management API using the new FocusCoordinator.
    // They exist alongside the old system for gradual migration.

    /// Request focus using the new coordinator system.
    ///
    /// This sets both the coordinator's pending request AND syncs to the old system
    /// for backward compatibility during migration.
    #[allow(dead_code)]
    pub fn focus_via_coordinator(
        &mut self,
        request: focus_coordinator::FocusRequest,
        cx: &mut Context<Self>,
    ) {
        self.focus_coordinator.request(request);
        // Sync to old system for backward compatibility
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Push an overlay (like actions dialog) with automatic restore on pop.
    ///
    /// Saves current focus state and requests focus to the overlay.
    /// Call `pop_focus_overlay()` when the overlay closes to restore.
    pub fn push_focus_overlay(
        &mut self,
        overlay_request: focus_coordinator::FocusRequest,
        cx: &mut Context<Self>,
    ) {
        self.focus_coordinator.push_overlay(overlay_request);
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Pop an overlay and restore previous focus state.
    ///
    /// Called when an overlay (actions dialog, shortcut recorder, etc.) closes.
    /// Restores focus to whatever was focused before the overlay opened.
    pub fn pop_focus_overlay(&mut self, cx: &mut Context<Self>) {
        self.focus_coordinator.pop_overlay();
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Clear all overlays and return to main filter focus.
    ///
    /// Useful for "escape all" or error recovery scenarios.
    #[allow(dead_code)]
    pub fn clear_focus_overlays(&mut self, cx: &mut Context<Self>) {
        self.focus_coordinator.clear_overlays();
        // Sync to old system
        self.sync_coordinator_to_legacy();
        cx.notify();
    }

    /// Get the current cursor owner from the coordinator.
    #[allow(dead_code)]
    pub fn current_cursor_owner(&self) -> focus_coordinator::CursorOwner {
        self.focus_coordinator.cursor_owner()
    }

    /// Sync coordinator state to legacy focused_input/pending_focus fields.
    ///
    /// This bridges the new and old systems during migration.
    pub(crate) fn sync_coordinator_to_legacy(&mut self) {
        // Sync cursor owner to focused_input
        self.focused_input = match self.focus_coordinator.cursor_owner() {
            focus_coordinator::CursorOwner::MainFilter => FocusedInput::MainFilter,
            focus_coordinator::CursorOwner::ActionsSearch => FocusedInput::ActionsSearch,
            focus_coordinator::CursorOwner::ArgPrompt => FocusedInput::ArgPrompt,
            focus_coordinator::CursorOwner::ChatPrompt => FocusedInput::None, // ChatPrompt not in old enum
            focus_coordinator::CursorOwner::None => FocusedInput::None,
        };

        // Sync pending target to pending_focus
        if let Some(request) = self.focus_coordinator.peek_pending() {
            self.pending_focus = Some(match request.target {
                focus_coordinator::FocusTarget::MainFilter => FocusTarget::MainFilter,
                focus_coordinator::FocusTarget::ActionsDialog => FocusTarget::ActionsDialog,
                focus_coordinator::FocusTarget::ArgPrompt => FocusTarget::AppRoot, // ArgPrompt uses AppRoot
                focus_coordinator::FocusTarget::PathPrompt => FocusTarget::PathPrompt,
                focus_coordinator::FocusTarget::FormPrompt => FocusTarget::FormPrompt,
                focus_coordinator::FocusTarget::EditorPrompt => FocusTarget::EditorPrompt,
                focus_coordinator::FocusTarget::SelectPrompt => FocusTarget::SelectPrompt,
                focus_coordinator::FocusTarget::EnvPrompt => FocusTarget::EnvPrompt,
                focus_coordinator::FocusTarget::DropPrompt => FocusTarget::DropPrompt,
                focus_coordinator::FocusTarget::TemplatePrompt => FocusTarget::TemplatePrompt,
                focus_coordinator::FocusTarget::TermPrompt => FocusTarget::TermPrompt,
                focus_coordinator::FocusTarget::ChatPrompt => FocusTarget::ChatPrompt,
                focus_coordinator::FocusTarget::DivPrompt => FocusTarget::AppRoot, // DivPrompt uses AppRoot
                focus_coordinator::FocusTarget::ScratchPad => FocusTarget::EditorPrompt,
                focus_coordinator::FocusTarget::QuickTerminal => FocusTarget::TermPrompt,
            });
        }
    }

    /// Apply pending focus if set. Called at the start of render() when window
    /// is focused. This applies focus exactly once, then clears pending_focus.
    ///
    /// Returns true if focus was applied (for logging/debugging).
    pub(crate) fn apply_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(target) = self.pending_focus.take() else {
            return false;
        };

        // Also consume the coordinator's pending request to keep current_cursor_owner
        // in sync. This is critical for push_overlay/pop_overlay's infer_current_request()
        // to know what was focused before the overlay opened.
        self.focus_coordinator.take_pending();

        logging::log("FOCUS", &format!("Applying pending focus: {:?}", target));

        match target {
            FocusTarget::MainFilter => {
                let input_state = self.gpui_input_state.clone();
                input_state.update(cx, |state, cx| {
                    state.focus(window, cx);
                });
                self.focused_input = FocusedInput::MainFilter;
            }
            FocusTarget::ActionsDialog => {
                if let Some(ref dialog) = self.actions_dialog {
                    let fh = dialog.read(cx).focus_handle.clone();
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::ActionsSearch;
                }
            }
            FocusTarget::EditorPrompt => {
                let entity = match &self.current_view {
                    AppView::EditorPrompt { entity, .. } => Some(entity),
                    AppView::ScratchPadView { entity, .. } => Some(entity),
                    _ => None,
                };
                if let Some(entity) = entity {
                    entity.update(cx, |editor, cx| {
                        editor.focus(window, cx);
                    });
                    // EditorPrompt has its own cursor management
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::PathPrompt => {
                if let AppView::PathPrompt { focus_handle, .. } = &self.current_view {
                    let fh = focus_handle.clone();
                    window.focus(&fh, cx);
                    // PathPrompt has its own cursor management
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::FormPrompt => {
                if let AppView::FormPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    // FormPrompt has its own focus handling
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::SelectPrompt => {
                if let AppView::SelectPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::EnvPrompt => {
                if let AppView::EnvPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::DropPrompt => {
                if let AppView::DropPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::TemplatePrompt => {
                if let AppView::TemplatePrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::TermPrompt => {
                let entity = match &self.current_view {
                    AppView::TermPrompt { entity, .. } => Some(entity),
                    AppView::QuickTerminalView { entity, .. } => Some(entity),
                    _ => None,
                };
                if let Some(entity) = entity {
                    let fh = entity.read(cx).focus_handle.clone();
                    window.focus(&fh, cx);
                    // Terminal handles its own cursor
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::ChatPrompt => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::NamingPrompt => {
                if let AppView::NamingPrompt { entity, .. } = &self.current_view {
                    let fh = entity.read(cx).focus_handle(cx);
                    window.focus(&fh, cx);
                    self.focused_input = FocusedInput::None;
                }
            }
            FocusTarget::AppRoot => {
                window.focus(&self.focus_handle, cx);
                // Don't reset focused_input here - the caller already set it appropriately.
                // For example, ArgPrompt sets focused_input = ArgPrompt before setting
                // pending_focus = AppRoot, and we want to preserve that so the cursor blinks.
            }
        }

        true
    }

}
