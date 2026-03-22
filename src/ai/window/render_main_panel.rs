use super::*;
use crate::theme::opacity::{
    OPACITY_BORDER, OPACITY_DISABLED, OPACITY_SELECTED, OPACITY_TEXT_MUTED,
};

fn ai_main_panel_can_submit(
    input_value: &str,
    has_pending_image: bool,
    has_pending_context_parts: bool,
) -> bool {
    ai_window_can_submit_message(input_value, has_pending_image, has_pending_context_parts)
}

/// Check whether the unified context bar should be visible.
///
/// Shown whenever there is pre-submit preflight data **or** a post-submit receipt.
fn should_show_context_bar(
    preflight: &context_preflight::ContextPreflightState,
    last_receipt: &Option<crate::ai::message_parts::ContextResolutionReceipt>,
) -> bool {
    preflight.status != context_preflight::ContextPreflightStatus::Idle || last_receipt.is_some()
}

/// Build the human-readable summary line for the context bar.
///
/// Uses preflight state when available (pre-submit), falls back to
/// post-submit receipt data. This ensures the same data shape drives
/// both preview and post-send display.
fn build_context_bar_summary(
    preflight: &context_preflight::ContextPreflightState,
    last_receipt: &Option<crate::ai::message_parts::ContextResolutionReceipt>,
    last_prepared: &Option<crate::ai::message_parts::PreparedMessageReceipt>,
) -> (String, usize, usize, usize, usize) {
    // Returns (summary_text, resolved, failures, duplicates, approx_tokens)
    if preflight.status != context_preflight::ContextPreflightStatus::Idle {
        // Pre-submit: use preflight state
        (
            String::new(), // built by caller
            preflight.resolved,
            preflight.failures,
            preflight.duplicates_removed,
            preflight.approx_tokens,
        )
    } else if let Some(receipt) = last_receipt {
        // Post-submit: use last receipt
        let duplicates = last_prepared
            .as_ref()
            .and_then(|r| r.assembly.as_ref())
            .map(|a| a.duplicates_removed)
            .unwrap_or(0);
        let approx_tokens = context_preflight::estimate_tokens_from_text(&receipt.prompt_prefix);
        (
            String::new(),
            receipt.resolved,
            receipt.failures.len(),
            duplicates,
            approx_tokens,
        )
    } else {
        (String::new(), 0, 0, 0, 0)
    }
}

/// Format a context count, token estimate, dedup and failure count into
/// a compact summary string like `Context 3 · ~1.8k tokens · 1 deduped`.
fn format_context_summary(
    resolved: usize,
    failures: usize,
    duplicates: usize,
    approx_tokens: usize,
    is_loading: bool,
) -> String {
    let mut parts = Vec::with_capacity(4);

    if is_loading {
        parts.push("Context resolving\u{2026}".to_string());
    } else {
        parts.push(format!("Context {resolved}"));
    }

    if approx_tokens > 0 {
        if approx_tokens >= 1000 {
            let k = approx_tokens as f64 / 1000.0;
            parts.push(format!("~{k:.1}k tokens"));
        } else {
            parts.push(format!("~{approx_tokens} tokens"));
        }
    }

    if duplicates > 0 {
        parts.push(format!("{duplicates} deduped"));
    }

    if failures > 0 {
        parts.push(format!("{failures} failed"));
    }

    parts.join(" \u{00b7} ")
}

impl AiApp {
    pub(super) fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Build input area at bottom:
        // Row 1: single composer surface containing the input text field
        // Row 2: model picker + word count on left, submit/actions on right

        // Check if we have a pending image or context parts to show
        let has_pending_image = self.pending_image.is_some();
        let has_pending_context_parts = !self.pending_context_parts.is_empty();
        let is_editing = self.editing_message_id.is_some();
        let input_value = self.input_state.read(cx).value().to_string();
        let input_is_empty =
            !ai_main_panel_can_submit(&input_value, has_pending_image, has_pending_context_parts);
        let input_word_count = if input_value.trim().is_empty() {
            0
        } else {
            input_value.split_whitespace().count()
        };
        let action_button_colors =
            crate::components::ButtonColors::from_theme(&crate::theme::get_cached_theme());
        let entity = cx.entity();

        let show_bar = should_show_context_bar(&self.context_preflight, &self.last_context_receipt);

        let input_area = div()
            .id("ai-input-area")
            .flex()
            .flex_col()
            .w_full()
            // NO .bg() - let vibrancy show through from root
            .border_t_1()
            .border_color(cx.theme().border.opacity(OPACITY_DISABLED))
            .px(MSG_PX)
            .py(S4)
            .gap(S3)
            // Handle image file drops
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Unified context bar (pre-submit preflight or post-submit receipt)
            .when(show_bar, |d| d.child(self.render_context_bar(cx)))
            // Editing indicator (shown above input when editing a message)
            .when(is_editing, |d| d.child(self.render_editing_indicator(cx)))
            // Context picker overlay (shown above input when @ trigger is active)
            .when(self.is_context_picker_open(), |d| {
                d.child(self.render_context_picker(cx))
            })
            // Pending context part chips (shown above input when parts are attached)
            .when(has_pending_context_parts, |d| {
                d.child(self.render_pending_context_chips(cx))
            })
            // Pending image preview (shown above input when image is attached)
            .when(has_pending_image, |d| {
                d.child(self.render_pending_image_preview(cx))
            })
            // Composer row: one surface for text input
            .child(
                div()
                    .id("ai-composer")
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    .min_h(COMPOSER_H)
                    .px(S3)
                    .py(S2)
                    .gap(S2)
                    .rounded(R_LG)
                    .border_1()
                    .border_color(cx.theme().border.opacity(OPACITY_SELECTED))
                    .bg(cx.theme().muted.opacity(OPACITY_DISABLED))
                    .child(self.render_input_with_cursor(cx)),
            )
            // Bottom row: Model picker left, submit right
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .overflow_hidden()
                    // Left side: Model picker + word count
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .overflow_hidden()
                            .child(self.render_model_picker(cx))
                            .when(input_word_count > 0, |d| {
                                let label: SharedString =
                                    format!("~{} words", input_word_count).into();
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_TEXT_MUTED),
                                        )
                                        .child(label),
                                )
                            }),
                    )
                    // Right side: Submit/Stop + Actions
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .flex_shrink_0()
                            // Actions ⌘K
                            .child({
                                let actions_entity = entity.clone();
                                crate::components::Button::new("Actions", action_button_colors)
                                    .id("ai-actions-btn")
                                    .variant(crate::components::ButtonVariant::Ghost)
                                    .shortcut("⌘K")
                                    .on_click(Box::new(move |_, window, cx| {
                                        actions_entity.update(cx, |this, cx| {
                                            this.show_command_bar(window, cx);
                                        });
                                    }))
                            })
                            // Submit or Stop button
                            .child(if self.is_streaming {
                                let stop_entity = entity.clone();
                                crate::components::Button::new("Stop", action_button_colors)
                                    .id("stop-btn")
                                    .variant(crate::components::ButtonVariant::Ghost)
                                    .shortcut("Esc")
                                    .on_click(Box::new(move |_, _window, cx| {
                                        stop_entity.update(cx, |this, cx| {
                                            this.stop_streaming(cx);
                                        });
                                    }))
                                    .into_any_element()
                            } else {
                                let submit_button =
                                    crate::components::Button::new("Submit", action_button_colors)
                                        .id("submit-btn")
                                        .variant(crate::components::ButtonVariant::Ghost)
                                        .shortcut("↵")
                                        .disabled(input_is_empty);
                                if input_is_empty {
                                    submit_button.into_any_element()
                                } else {
                                    let submit_entity = entity.clone();
                                    submit_button
                                        .on_click(Box::new(move |_, window, cx| {
                                            submit_entity.update(cx, |this, cx| {
                                                this.submit_message(window, cx);
                                            });
                                        }))
                                        .into_any_element()
                                }
                            }),
                    ),
            );

        // Determine what to show in the content area
        let has_messages = !self.current_messages.is_empty() || self.is_streaming;

        // Build main layout
        // Structure: content area (flex_1, scrollable) -> input area (fixed)
        div()
            .id("ai-main-panel")
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            // Handle image file drops anywhere on the main panel
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Content area - fills remaining space above the input area.
            // min_h_0 is critical: without it a flex child won't shrink below its
            // content size, preventing overflow/scroll from working.
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(if has_messages {
                        self.render_messages(cx).into_any_element()
                    } else {
                        self.render_welcome(cx).into_any_element()
                    }),
            )
            // Input area (fixed height, always visible at bottom)
            .child(input_area)
    }

    /// Render the unified context bar: a compact summary line that opens
    /// a drawer with per-part provenance rows. Works both pre-submit
    /// (from preflight state) and post-submit (from resolution receipt).
    fn render_context_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_loading =
            self.context_preflight.status == context_preflight::ContextPreflightStatus::Loading;

        let (_, resolved, failures, duplicates, approx_tokens) = build_context_bar_summary(
            &self.context_preflight,
            &self.last_context_receipt,
            &self.last_prepared_message_receipt,
        );

        let summary_text: SharedString =
            format_context_summary(resolved, failures, duplicates, approx_tokens, is_loading)
                .into();

        let has_failures = failures > 0;
        let (bg_color, text_color) = if has_failures {
            (
                cx.theme().danger.opacity(OPACITY_DISABLED),
                cx.theme().danger,
            )
        } else {
            (
                cx.theme().accent.opacity(OPACITY_DISABLED),
                cx.theme().accent,
            )
        };

        let chevron_icon = if self.show_context_drawer {
            LocalIconName::ChevronDown
        } else {
            LocalIconName::ChevronRight
        };

        let shortcut_label: SharedString = "\u{2325}\u{2318}I".into();

        div()
            .id("context-bar")
            .flex()
            .flex_col()
            .gap(S2)
            // Summary line — clickable to toggle the drawer
            .child(
                div()
                    .id("context-bar-summary")
                    .flex()
                    .items_center()
                    .gap(S2)
                    .px(S3)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(bg_color)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_context_drawer(cx);
                    }))
                    .child(
                        svg()
                            .external_path(chevron_icon.external_path())
                            .size(ICON_XS)
                            .text_color(text_color),
                    )
                    .child(div().text_xs().text_color(text_color).child(summary_text))
                    .child(
                        div()
                            .ml_auto()
                            .text_xs()
                            .text_color(text_color.opacity(OPACITY_TEXT_MUTED))
                            .child(shortcut_label),
                    ),
            )
            // Drawer — per-part rows with provenance, then optional raw JSON
            .when(self.show_context_drawer, |container| {
                container.child(self.render_context_drawer(cx))
            })
            .into_any_element()
    }

    /// Render the context drawer: human-readable per-part rows with
    /// provenance and status indicators, followed by an optional raw
    /// JSON inspector toggle.
    fn render_context_drawer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let fg = theme.foreground;
        let muted_fg = theme.muted_foreground;

        let mut drawer = div()
            .id("context-drawer")
            .flex()
            .flex_col()
            .gap(S1)
            .px(S3)
            .py(S2)
            .rounded(R_MD)
            .bg(theme.muted.opacity(OPACITY_DISABLED))
            .max_h(px(300.0))
            .overflow_y_scroll();

        // Build rows from outcomes if we have a receipt (preflight or post-submit)
        let receipt = self
            .context_preflight
            .receipt
            .as_ref()
            .or(self.last_prepared_message_receipt.as_ref());

        if let Some(prepared) = receipt {
            for (idx, outcome) in prepared.outcomes.iter().enumerate() {
                let row_id = SharedString::from(format!("drawer-row-{idx}"));
                let label: SharedString = outcome.label.clone().into();
                let source: SharedString = outcome.source.clone().into();

                let (status_label, status_color) = match outcome.kind {
                    crate::ai::message_parts::ContextPartPreparationOutcomeKind::FullContent => {
                        ("resolved", theme.accent)
                    }
                    crate::ai::message_parts::ContextPartPreparationOutcomeKind::MetadataOnly => {
                        ("truncated", theme.warning)
                    }
                    crate::ai::message_parts::ContextPartPreparationOutcomeKind::Failed => {
                        ("failed", theme.danger)
                    }
                };
                let status_text: SharedString = status_label.into();

                let mut row = div()
                    .id(row_id)
                    .flex()
                    .items_center()
                    .gap(S2)
                    .py(S1)
                    // Label
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .max_w(px(140.0))
                            .child(label),
                    )
                    // Source (URI or path)
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(muted_fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(source),
                    )
                    // Status pill
                    .child(
                        div()
                            .text_xs()
                            .px(S2)
                            .rounded(R_SM)
                            .bg(status_color.opacity(OPACITY_DISABLED))
                            .text_color(status_color)
                            .flex_shrink_0()
                            .child(status_text),
                    );

                // Detail (truncation reason, error message)
                if let Some(detail) = &outcome.detail {
                    let detail_text: SharedString = detail.clone().into();
                    row = row.child(
                        div()
                            .text_xs()
                            .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
                            .overflow_hidden()
                            .text_ellipsis()
                            .max_w(px(200.0))
                            .child(detail_text),
                    );
                }

                drawer = drawer.child(row);
            }

            // Show failures from the resolution receipt that may not appear in outcomes
            for failure in &prepared.context.failures {
                let fail_label: SharedString = failure.label.clone().into();
                let fail_error: SharedString = failure.error.clone().into();
                drawer = drawer.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(S2)
                        .py(S1)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.danger)
                                .child(fail_label),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(muted_fg)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(fail_error),
                        )
                        .child(
                            div()
                                .text_xs()
                                .px(S2)
                                .rounded(R_SM)
                                .bg(theme.danger.opacity(OPACITY_DISABLED))
                                .text_color(theme.danger)
                                .flex_shrink_0()
                                .child(SharedString::from("failed")),
                        ),
                );
            }
        } else if self.last_context_receipt.is_some() {
            // Fallback: show from the resolution receipt when no prepared receipt exists
            if let Some(receipt) = &self.last_context_receipt {
                let summary_line: SharedString = format!(
                    "{} resolved, {} failed",
                    receipt.resolved,
                    receipt.failures.len()
                )
                .into();
                drawer = drawer.child(div().text_xs().text_color(muted_fg).child(summary_line));
            }
        }

        // Prompt Compiler toggle — reuses existing ⌥⌘I inspector behavior
        drawer = drawer.child(
            div()
                .id("drawer-json-toggle")
                .flex()
                .items_center()
                .gap(S1)
                .pt(S2)
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.toggle_context_inspector(cx);
                }))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_fg)
                        .child(SharedString::from(if self.show_context_inspector {
                            "Hide compiled prompt"
                        } else {
                            "Show compiled prompt"
                        })),
                ),
        );

        // Prompt Compiler pane — human-readable view of the compiled outbound message
        if self.show_context_inspector {
            drawer = drawer.child(self.render_prompt_compiler_pane(cx));
        }

        drawer
    }

    /// Toggle the context drawer open/closed.
    pub(super) fn toggle_context_drawer(&mut self, cx: &mut Context<Self>) {
        self.show_context_drawer = !self.show_context_drawer;
        tracing::info!(
            target: "ai",
            visible = self.show_context_drawer,
            "ai_context_drawer_toggled"
        );
        cx.notify();
    }

    /// Select the best available receipt for the prompt compiler pane.
    ///
    /// Prefers `context_preflight.receipt` (pre-send) and falls back to
    /// `last_prepared_message_receipt` (post-send).
    fn active_prompt_compiler_receipt(
        &self,
    ) -> Option<&crate::ai::message_parts::PreparedMessageReceipt> {
        self.context_preflight
            .receipt
            .as_ref()
            .or(self.last_prepared_message_receipt.as_ref())
    }

    /// Render the prompt compiler pane: a human-readable, keyboard-first
    /// view of the compiled outbound message. Replaces raw JSON inspector.
    fn render_prompt_compiler_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let fg = theme.foreground;
        let muted_fg = theme.muted_foreground;

        let receipt = self.active_prompt_compiler_receipt();
        if let Some(receipt) = receipt {
            let preview =
                crate::ai::window::prompt_compiler::PromptCompilerPreview::from_receipt(receipt);

            // Summary line
            let summary: SharedString = format!(
                "{} / {} resolved \u{00b7} {} failed \u{00b7} {} deduped \u{00b7} ~{} tokens (approx)",
                preview.resolved,
                preview.attempted,
                preview.failures,
                preview.duplicates_removed,
                preview.approx_tokens,
            )
            .into();

            let raw_text: SharedString = preview.raw_content.clone().into();
            let final_text: SharedString = preview.final_user_content.clone().into();

            let decision_label: SharedString = match preview.decision {
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Ready => {
                    "Ready".into()
                }
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Partial => {
                    "Partial".into()
                }
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Blocked => {
                    "Blocked".into()
                }
            };
            let decision_color = match preview.decision {
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Ready => theme.accent,
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Partial => {
                    theme.warning
                }
                crate::ai::window::prompt_compiler::PromptCompilerDecision::Blocked => {
                    theme.danger
                }
            };

            let mut pane = div()
                .id("prompt-compiler")
                .flex()
                .flex_col()
                .gap(S2)
                .pt(S2)
                .max_h(px(320.0))
                .overflow_y_scroll()
                // Decision + summary
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(S2)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .px(S2)
                                .rounded(R_SM)
                                .bg(decision_color.opacity(OPACITY_DISABLED))
                                .text_color(decision_color)
                                .child(decision_label),
                        )
                        .child(div().text_xs().text_color(muted_fg).child(summary)),
                )
                // Authored text section
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(muted_fg)
                        .child(SharedString::from("Authored text")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(fg)
                        .px(S2)
                        .py(S1)
                        .rounded(R_SM)
                        .bg(theme.muted.opacity(OPACITY_DISABLED))
                        .child(raw_text),
                )
                // Exact outbound message section
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(muted_fg)
                        .child(SharedString::from("Exact outbound message")),
                )
                .child(
                    div()
                        .id("compiler-outbound")
                        .text_xs()
                        .text_color(fg)
                        .px(S2)
                        .py(S1)
                        .rounded(R_SM)
                        .bg(theme.muted.opacity(OPACITY_DISABLED))
                        .max_h(px(160.0))
                        .overflow_y_scroll()
                        .child(final_text),
                );

            // Semantic rows for context parts
            if !preview.rows.is_empty() {
                pane = pane.child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(muted_fg)
                        .pt(S1)
                        .child(SharedString::from("Context parts")),
                );

                for (idx, row) in preview.rows.iter().enumerate() {
                    let row_id = SharedString::from(format!("compiler-row-{idx}"));
                    let label: SharedString = row.label.clone().into();
                    let source: SharedString = row.source.clone().into();

                    let (kind_label, kind_color) = match row.kind {
                        crate::ai::window::prompt_compiler::PromptCompilerRowKind::FullContent => {
                            ("resolved", theme.accent)
                        }
                        crate::ai::window::prompt_compiler::PromptCompilerRowKind::MetadataOnly => {
                            ("metadata-only", theme.warning)
                        }
                        crate::ai::window::prompt_compiler::PromptCompilerRowKind::Failed => {
                            ("failed", theme.danger)
                        }
                        crate::ai::window::prompt_compiler::PromptCompilerRowKind::DuplicateDropped => {
                            ("deduped", muted_fg)
                        }
                        crate::ai::window::prompt_compiler::PromptCompilerRowKind::UnresolvedPart => {
                            ("unresolved", theme.danger)
                        }
                    };
                    let kind_text: SharedString = kind_label.into();

                    let mut row_el = div()
                        .id(row_id)
                        .flex()
                        .items_center()
                        .gap(S2)
                        .py(S1)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(fg)
                                .overflow_hidden()
                                .text_ellipsis()
                                .max_w(px(140.0))
                                .child(label),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(muted_fg)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(source),
                        )
                        .child(
                            div()
                                .text_xs()
                                .px(S2)
                                .rounded(R_SM)
                                .bg(kind_color.opacity(OPACITY_DISABLED))
                                .text_color(kind_color)
                                .flex_shrink_0()
                                .child(kind_text),
                        );

                    if let Some(detail) = &row.detail {
                        let detail_text: SharedString = detail.clone().into();
                        row_el = row_el.child(
                            div()
                                .text_xs()
                                .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
                                .overflow_hidden()
                                .text_ellipsis()
                                .max_w(px(200.0))
                                .child(detail_text),
                        );
                    }

                    pane = pane.child(row_el);
                }
            }

            pane.into_any_element()
        } else {
            div()
                .id("prompt-compiler")
                .pt(S2)
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_fg)
                        .child(SharedString::from("No compiled prompt available.")),
                )
                .into_any_element()
        }
    }

    /// Render chips representing pending context parts above the composer.
    ///
    /// Each ResourceUri chip includes an expand/collapse chevron that toggles
    /// an inline preview panel showing the source URI, profile, and payload
    /// summary. FilePath chips show only the close button (no preview needed).
    fn render_pending_context_chips(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;
        let preview_idx = self.context_preview_index;

        let chips: Vec<_> = self
            .pending_context_parts
            .iter()
            .enumerate()
            .map(|(idx, part)| {
                let label: SharedString = part.label().to_string().into();
                let is_resource = matches!(
                    part,
                    crate::ai::message_parts::AiContextPart::ResourceUri { .. }
                );
                let is_previewed = preview_idx == Some(idx);
                let icon_name = if is_resource {
                    LocalIconName::Code
                } else {
                    LocalIconName::File
                };

                // Chip border highlights when its preview is open
                let chip_border = if is_previewed {
                    accent.opacity(OPACITY_SELECTED)
                } else {
                    accent.opacity(OPACITY_BORDER)
                };

                let mut chip = div()
                    .id(SharedString::from(format!("ctx-part-{}", idx)))
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(accent.opacity(OPACITY_DISABLED))
                    .border_1()
                    .border_color(chip_border)
                    .child(
                        svg()
                            .external_path(icon_name.external_path())
                            .size(ICON_XS)
                            .text_color(accent),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .overflow_hidden()
                            .text_ellipsis()
                            .max_w(px(160.0))
                            .child(label),
                    );

                // Preview toggle (chevron) — only for ResourceUri chips
                if is_resource {
                    let chevron_icon = if is_previewed {
                        LocalIconName::ChevronDown
                    } else {
                        LocalIconName::ChevronRight
                    };
                    chip = chip.child(
                        div()
                            .id(SharedString::from(format!("ctx-preview-{}", idx)))
                            .cursor_pointer()
                            .hover(|el| el.text_color(accent))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_context_preview(idx, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(chevron_icon.external_path())
                                    .size(ICON_XS)
                                    .text_color(muted_fg),
                            ),
                    );
                }

                // Close button
                chip = chip.child(
                    div()
                        .id(SharedString::from(format!("ctx-remove-{}", idx)))
                        .cursor_pointer()
                        .hover(|el| el.text_color(cx.theme().danger))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            // Centralize preview index maintenance in remove_context_part().
                            this.remove_context_part(idx, cx);
                        }))
                        .child(
                            svg()
                                .external_path(LocalIconName::Close.external_path())
                                .size(ICON_XS)
                                .text_color(muted_fg),
                        ),
                );

                chip
            })
            .collect();

        let mut container = div()
            .id("pending-context-chips")
            .flex()
            .flex_col()
            .gap(S2)
            .child(div().flex().flex_row().flex_wrap().gap(S2).children(chips));

        // Inline preview panel — shown below chips when a ResourceUri is expanded
        if let Some((_, preview)) = self.active_context_preview() {
            container = container.child(self.render_context_preview_panel(&preview, cx));
        }

        container
    }

    /// Render the inline preview panel for an expanded context chip.
    fn render_context_preview_panel(
        &self,
        preview: &context_preview::ContextPreviewInfo,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let accent = theme.accent;
        let muted_fg = theme.muted_foreground;
        let fg = theme.foreground;

        let profile_label: SharedString = match preview.profile {
            context_preview::ContextPreviewProfile::Minimal => "Profile: minimal".into(),
            context_preview::ContextPreviewProfile::Full => "Profile: full".into(),
            context_preview::ContextPreviewProfile::Custom => "Profile: custom".into(),
            context_preview::ContextPreviewProfile::FilePath => "Type: file".into(),
        };

        // Visual distinction: full profile gets accent bg, minimal gets muted
        let profile_bg = match preview.profile {
            context_preview::ContextPreviewProfile::Full => accent.opacity(OPACITY_DISABLED),
            _ => theme.muted.opacity(OPACITY_DISABLED),
        };

        let uri_label: SharedString = preview.source_uri.clone().into();
        let desc_label: SharedString = preview.description.clone().into();

        let mut panel = div()
            .id("context-preview-panel")
            .flex()
            .flex_col()
            .gap(S1)
            .px(S3)
            .py(S2)
            .rounded(R_MD)
            .border_1()
            .border_color(accent.opacity(OPACITY_BORDER))
            .bg(theme.background.opacity(OPACITY_SELECTED))
            // Profile badge
            .child(
                div().flex().items_center().gap(S2).child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .px(S2)
                        .py(S0)
                        .rounded(R_SM)
                        .bg(profile_bg)
                        .text_color(fg)
                        .child(profile_label),
                ),
            )
            // Source URI
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S1)
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_fg)
                            .child(SharedString::from("URI:")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(uri_label),
                    ),
            )
            // Description
            .child(div().text_xs().text_color(muted_fg).child(desc_label));

        // Diagnostics badge
        if preview.has_diagnostics {
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap(S1)
                    .child(
                        svg()
                            .external_path(LocalIconName::Warning.external_path())
                            .size(ICON_XS)
                            .text_color(theme.warning),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.warning)
                            .child(SharedString::from("Includes diagnostics")),
                    ),
            );
        }

        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_text_present() {
        assert!(ai_main_panel_can_submit("hello", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_pending_image_present_and_text_empty() {
        assert!(ai_main_panel_can_submit("", true, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_false_when_text_empty_and_no_pending_image() {
        assert!(!ai_main_panel_can_submit("", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_false_for_whitespace_without_image() {
        assert!(!ai_main_panel_can_submit("   ", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_context_parts_present() {
        assert!(ai_main_panel_can_submit("", false, true));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_context_parts_and_text_present() {
        assert!(ai_main_panel_can_submit("hello", false, true));
    }

    #[test]
    fn test_should_show_context_bar_idle_no_receipt() {
        let preflight = context_preflight::ContextPreflightState::default();
        assert!(!should_show_context_bar(&preflight, &None));
    }

    #[test]
    fn test_should_show_context_bar_loading_preflight() {
        let preflight = context_preflight::ContextPreflightState {
            status: context_preflight::ContextPreflightStatus::Loading,
            ..Default::default()
        };
        assert!(should_show_context_bar(&preflight, &None));
    }

    #[test]
    fn test_should_show_context_bar_ready_preflight() {
        let preflight = context_preflight::ContextPreflightState {
            status: context_preflight::ContextPreflightStatus::Ready,
            resolved: 2,
            ..Default::default()
        };
        assert!(should_show_context_bar(&preflight, &None));
    }

    #[test]
    fn test_should_show_context_bar_post_submit_receipt() {
        let preflight = context_preflight::ContextPreflightState::default();
        let receipt = Some(crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 1,
            resolved: 1,
            failures: vec![],
            prompt_prefix: "test".to_string(),
        });
        assert!(should_show_context_bar(&preflight, &receipt));
    }

    #[test]
    fn test_format_context_summary_basic() {
        let s = format_context_summary(3, 0, 0, 1800, false);
        assert!(s.contains("Context 3"));
        assert!(s.contains("~1.8k tokens"));
    }

    #[test]
    fn test_format_context_summary_with_dedup_and_failures() {
        let s = format_context_summary(2, 1, 1, 500, false);
        assert!(s.contains("Context 2"));
        assert!(s.contains("~500 tokens"));
        assert!(s.contains("1 deduped"));
        assert!(s.contains("1 failed"));
    }

    #[test]
    fn test_format_context_summary_loading() {
        let s = format_context_summary(0, 0, 0, 0, true);
        assert!(s.contains("resolving"));
    }

    #[test]
    fn test_format_context_summary_zero_tokens_omitted() {
        let s = format_context_summary(1, 0, 0, 0, false);
        assert!(s.contains("Context 1"));
        assert!(!s.contains("tokens"));
    }

    #[test]
    fn test_build_context_bar_summary_preflight_takes_precedence() {
        let preflight = context_preflight::ContextPreflightState {
            status: context_preflight::ContextPreflightStatus::Ready,
            resolved: 3,
            failures: 1,
            duplicates_removed: 2,
            approx_tokens: 750,
            ..Default::default()
        };
        let (_, resolved, failures, duplicates, tokens) =
            build_context_bar_summary(&preflight, &None, &None);
        assert_eq!(resolved, 3);
        assert_eq!(failures, 1);
        assert_eq!(duplicates, 2);
        assert_eq!(tokens, 750);
    }

    #[test]
    fn test_build_context_bar_summary_falls_back_to_receipt() {
        let preflight = context_preflight::ContextPreflightState::default();
        let receipt = Some(crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 2,
            resolved: 2,
            failures: vec![],
            prompt_prefix: "abcdefgh".to_string(), // 8 chars → 2 tokens
        });
        let (_, resolved, failures, _, tokens) =
            build_context_bar_summary(&preflight, &receipt, &None);
        assert_eq!(resolved, 2);
        assert_eq!(failures, 0);
        assert_eq!(tokens, 2);
    }
}
