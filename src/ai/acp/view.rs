//! ACP chat view.
//!
//! Renders an ACP conversation thread with markdown-rendered messages,
//! role-aware cards, empty/streaming/error states, and permission approval
//! overlay. Wraps an `AcpThread` entity for the Tab AI surface.

use std::collections::HashSet;
use std::time::Duration;

use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, ParentElement, Render, ScrollHandle, SharedString, Task, Window,
};

use crate::components::text_input::{render_text_input_cursor_selection, TextInputRenderConfig};
use crate::prompts::markdown::render_markdown_with_scope;
use crate::theme::{self, PromptColors};

use super::thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadMessage, AcpThreadMessageRole, AcpThreadStatus,
};
use super::{AcpApprovalOption, AcpApprovalPreview, AcpApprovalPreviewKind, AcpApprovalRequest};

/// Click handler type for collapsible block toggle.
type ToggleHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;

/// Parse the `description` field from YAML frontmatter in a SKILL.md file.
fn parse_skill_description(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let end = content[3..].find("---")?;
    let frontmatter = &content[3..3 + end];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("description:") {
            let desc = rest.trim().trim_matches('"').trim_matches('\'');
            // Truncate long descriptions for the menu
            if desc.len() > 80 {
                return Some(format!("{}\u{2026}", &desc[..77]));
            }
            return Some(desc.to_string());
        }
    }
    None
}

/// GPUI view entity wrapping an `AcpThread` for the Tab AI surface.
pub(crate) struct AcpChatView {
    pub(crate) thread: Entity<AcpThread>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    /// Index of the currently highlighted permission option in the overlay.
    permission_index: usize,
    /// Message IDs that are currently collapsed (thinking/tool blocks).
    collapsed_ids: HashSet<u64>,
    /// Track message count for auto-scroll detection.
    last_message_count: usize,
    /// Cursor blink state.
    cursor_visible: bool,
    /// Handle to the cursor blink task.
    _blink_task: Task<()>,
    /// Slash command menu: selected index (None = menu hidden).
    slash_menu_index: Option<usize>,
    /// Cached slash commands (name, description) discovered at creation.
    cached_slash_commands: Vec<(String, String)>,
}

impl AcpChatView {
    pub(crate) fn new(thread: Entity<AcpThread>, cx: &mut Context<Self>) -> Self {
        // Auto-scroll when thread state changes (new messages, streaming updates).
        cx.observe(&thread, |this: &mut Self, thread, cx| {
            let thread_ref = thread.read(cx);
            let count = thread_ref.messages.len();
            let is_streaming = matches!(thread_ref.status, AcpThreadStatus::Streaming);

            // Scroll to bottom on new messages or while streaming (content growing).
            if count != this.last_message_count || is_streaming {
                this.last_message_count = count;
                this.scroll_handle.scroll_to_bottom();
            }

            // Update slash command menu on any input change.
            this.update_slash_menu(cx);
            cx.notify();
        })
        .detach();

        // Cursor blink loop (530ms interval, same as ChatPrompt).
        let blink_task = cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(530))
                .await;
            if !crate::is_main_window_visible() {
                continue;
            }
            let result = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    view.cursor_visible = !view.cursor_visible;
                    cx.notify();
                })
            });
            if result.is_err() {
                break;
            }
        });

        Self {
            thread,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            permission_index: 0,
            collapsed_ids: HashSet::new(),
            last_message_count: 0,
            cursor_visible: true,
            _blink_task: blink_task,
            slash_menu_index: None,
            cached_slash_commands: Self::discover_slash_commands(),
        }
    }

    /// Scan ~/.scriptkit/skills/ for skill directories, combine with
    /// built-in Claude Code commands. Returns (name, description) tuples.
    fn discover_slash_commands() -> Vec<(String, String)> {
        let mut commands: Vec<(String, String)> = Self::DEFAULT_SLASH_COMMANDS
            .iter()
            .map(|s| (s.to_string(), String::new()))
            .collect();

        let mut seen: std::collections::HashSet<String> =
            commands.iter().map(|(name, _)| name.clone()).collect();

        // Scan both skills directories for SKILL.md entries.
        let dirs = [
            crate::setup::get_kit_path().join("skills"),
            crate::setup::get_kit_path().join(".claude").join("skills"),
        ];

        for dir in &dirs {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let skill_md = entry.path().join("SKILL.md");
                if !skill_md.exists() {
                    continue;
                }
                let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                if seen.contains(&name) {
                    continue;
                }

                // Parse description from YAML frontmatter
                let desc = std::fs::read_to_string(&skill_md)
                    .ok()
                    .and_then(|content| parse_skill_description(&content))
                    .unwrap_or_default();

                seen.insert(name.clone());
                commands.push((name, desc));
            }
        }

        commands
    }

    /// Consume Tab / Shift+Tab. When the permission overlay is open,
    /// cycle the highlighted option; otherwise just swallow the key so
    /// the global interceptors do not re-open a fresh ACP chat.
    pub(crate) fn handle_tab_key(&mut self, has_shift: bool, cx: &mut Context<Self>) -> bool {
        let option_count = self
            .thread
            .read(cx)
            .pending_permission
            .as_ref()
            .map(|r| r.options.len())
            .unwrap_or(0);

        if option_count > 0 {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, has_shift);
            cx.notify();
            return true;
        }

        cx.notify();
        true
    }

    fn approve_permission(&mut self, option_id: Option<String>, cx: &mut Context<Self>) {
        self.permission_index = 0;
        self.thread.update(cx, |thread, cx| {
            thread.approve_pending_permission(option_id, cx);
        });
    }

    fn normalized_permission_index(&self, option_count: usize) -> usize {
        if option_count == 0 {
            0
        } else {
            self.permission_index.min(option_count - 1)
        }
    }

    fn step_permission_index(current: usize, option_count: usize, reverse: bool) -> usize {
        if option_count == 0 {
            return 0;
        }

        if reverse {
            if current == 0 {
                option_count - 1
            } else {
                current - 1
            }
        } else {
            (current + 1) % option_count
        }
    }

    /// Handle key events when the permission overlay is displayed.
    /// Returns `true` if the key was consumed.
    fn handle_permission_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let option_count = request.options.len();
        self.permission_index = self.normalized_permission_index(option_count);

        if crate::ui_foundation::is_key_up(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, true);
            cx.notify();
            return true;
        }

        if crate::ui_foundation::is_key_down(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, false);
            cx.notify();
            return true;
        }

        // J/K navigation (vim-style, unmodified only)
        match key {
            "j" | "J" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, false);
                cx.notify();
                return true;
            }
            "k" | "K" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, true);
                cx.notify();
                return true;
            }
            _ => {}
        }

        if crate::ui_foundation::is_key_escape(key) {
            self.approve_permission(None, cx);
            return true;
        }

        if crate::ui_foundation::is_key_enter(key) {
            if let Some(option) = request
                .options
                .get(self.normalized_permission_index(option_count))
            {
                self.approve_permission(Some(option.option_id.clone()), cx);
            } else {
                self.approve_permission(None, cx);
            }
            return true;
        }

        // 1-9 instant pick
        if let Ok(digit) = key.parse::<usize>() {
            if digit >= 1 {
                let idx = digit - 1;
                if let Some(option) = request.options.get(idx) {
                    self.permission_index = idx;
                    self.approve_permission(Some(option.option_id.clone()), cx);
                    return true;
                }
            }
        }

        false
    }

    pub(crate) fn set_input(&mut self, value: String, cx: &mut Context<Self>) {
        self.thread
            .update(cx, |thread, cx| thread.set_input(value, cx));
    }

    // ── Rendering helpers ─────────────────────────────────────────

    fn prompt_colors() -> PromptColors {
        PromptColors::from_theme(&theme::get_cached_theme())
    }

    /// Render a message. Thinking and Tool messages are collapsible.
    fn render_message(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        is_collapsed: bool,
        on_toggle: Option<ToggleHandler>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        match msg.role {
            AcpThreadMessageRole::User => Self::render_user_message(msg, colors, &theme),
            AcpThreadMessageRole::Assistant => Self::render_assistant_message(msg, colors, &theme),
            AcpThreadMessageRole::Thought => {
                Self::render_collapsible_block(msg, colors, &theme, is_collapsed, on_toggle, false)
            }
            AcpThreadMessageRole::Tool => {
                Self::render_collapsible_block(msg, colors, &theme, is_collapsed, on_toggle, true)
            }
            AcpThreadMessageRole::Error => Self::render_error_message(msg, colors),
            AcpThreadMessageRole::System => Self::render_system_message(msg, colors, &theme),
        }
    }

    fn render_user_message(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.text.primary << 8) | 0x06))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_assistant_message(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        _theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        // Assistant messages: no card, no border — just markdown flowing
        div()
            .w_full()
            .px(px(12.0))
            .py(px(4.0))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    /// Thinking and Tool blocks: collapsible with header + optional gradient fade.
    fn render_collapsible_block(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
        is_collapsed: bool,
        on_toggle: Option<ToggleHandler>,
        is_tool: bool,
    ) -> gpui::AnyElement {
        let label = if is_tool {
            // Extract tool name from first line if present, else "Tool"
            msg.body
                .lines()
                .next()
                .and_then(|line| {
                    let trimmed = line.trim();
                    if trimmed.len() < 80 {
                        Some(trimmed.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Tool".to_string())
        } else {
            "Thinking".to_string()
        };

        let chevron = if is_collapsed {
            "\u{25B8}" // ▸
        } else {
            "\u{25BE}" // ▾
        };

        let line_count = msg.body.lines().count();
        let header_opacity = if is_tool { 0.55 } else { 0.50 };
        let left_border_color = if is_tool {
            rgba((theme.colors.accent.selected << 8) | 0x30)
        } else {
            rgba((theme.colors.text.primary << 8) | 0x18)
        };

        let scope_id = format!("acp-msg-{}", msg.id);

        let mut container = div()
            .w_full()
            .pl(px(12.0))
            .pr(px(12.0))
            .py(px(2.0))
            .border_l_2()
            .border_color(left_border_color);

        // Header row (always visible) — clickable toggle
        let header = div()
            .id(SharedString::from(format!("acp-toggle-{}", msg.id)))
            .flex()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .child(
                div()
                    .text_xs()
                    .opacity(header_opacity)
                    .child(chevron.to_string()),
            )
            .child(div().text_xs().opacity(header_opacity).child(label))
            .when(is_collapsed && line_count > 1, |d| {
                d.child(
                    div()
                        .text_xs()
                        .opacity(0.35)
                        .child(format!("{line_count} lines")),
                )
            });

        let header = if let Some(toggle) = on_toggle {
            header.on_click(toggle)
        } else {
            header
        };

        container = container.child(header);

        // Body (collapsed = hidden, expanded = shown with max-height + gradient)
        if !is_collapsed {
            let body = div()
                .pt(px(4.0))
                .max_h(px(200.0))
                .overflow_y_hidden()
                .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full());

            container = container.child(body);
        }

        container.into_any_element()
    }

    fn render_error_message(msg: &AcpThreadMessage, colors: &PromptColors) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(6.0))
            .border_l_2()
            .border_color(rgba(0xEF444480))
            .child(div().text_xs().opacity(0.65).pb(px(2.0)).child("Error"))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_system_message(
        msg: &AcpThreadMessage,
        colors: &PromptColors,
        theme: &crate::theme::Theme,
    ) -> gpui::AnyElement {
        let scope_id = format!("acp-msg-{}", msg.id);

        div()
            .w_full()
            .px(px(12.0))
            .py(px(4.0))
            .opacity(0.60)
            .border_l_2()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x30))
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .into_any_element()
    }

    fn render_permission_section(title: &'static str, text: String) -> gpui::AnyElement {
        div()
            .pt(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.64)
                    .child(title),
            )
            .child(
                div()
                    .mt(px(4.0))
                    .max_h(px(140.0))
                    .overflow_y_hidden()
                    .rounded(px(8.0))
                    .bg(rgba(0x00000018))
                    .px(px(10.0))
                    .py(px(8.0))
                    .text_xs()
                    .child(text),
            )
            .into_any_element()
    }

    fn render_permission_header(preview: &AcpApprovalPreview) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        let (badge_bg, badge_border) = match preview.kind {
            AcpApprovalPreviewKind::Read => (
                rgba((theme.colors.text.primary << 8) | 0x10),
                rgba((theme.colors.ui.border << 8) | 0x30),
            ),
            AcpApprovalPreviewKind::Write => (
                rgba((theme.colors.accent.selected << 8) | 0x16),
                rgba((theme.colors.accent.selected << 8) | 0x38),
            ),
            AcpApprovalPreviewKind::Execute => (rgba(0xF59E0B18), rgba(0xF59E0B50)),
            AcpApprovalPreviewKind::Generic => (
                rgba((theme.colors.text.primary << 8) | 0x08),
                rgba((theme.colors.ui.border << 8) | 0x24),
            ),
        };

        div()
            .pt(px(8.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(999.0))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(badge_border)
                            .text_xs()
                            .opacity(0.8)
                            .child(preview.kind.badge_label()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(preview.tool_title.clone()),
                    ),
            )
            .when_some(preview.subject.clone(), |d, subject| {
                d.child(div().pt(px(6.0)).text_sm().opacity(0.82).child(subject))
            })
            .child(
                div()
                    .pt(px(2.0))
                    .text_xs()
                    .opacity(0.52)
                    .child(format!("Tool call ID: {}", preview.tool_call_id)),
            )
            .into_any_element()
    }

    fn render_permission_option_row(
        option: &AcpApprovalOption,
        index: usize,
        is_selected: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let option_id = option.option_id.clone();

        let (bg, border, caption) = if option.is_reject() {
            (
                if is_selected {
                    rgba(0xEF444424)
                } else {
                    rgba(0xEF444412)
                },
                rgba(0xEF444460),
                "Cancel this request",
            )
        } else if option.is_persistent_allow() {
            (
                if is_selected {
                    rgba((theme.colors.accent.selected << 8) | 0x22)
                } else {
                    rgba((theme.colors.accent.selected << 8) | 0x12)
                },
                rgba((theme.colors.accent.selected << 8) | 0x48),
                "Remember this choice",
            )
        } else {
            (
                if is_selected {
                    rgba((theme.colors.accent.selected << 8) | 0x1C)
                } else {
                    rgba((theme.colors.text.primary << 8) | 0x08)
                },
                if is_selected {
                    rgba((theme.colors.accent.selected << 8) | 0x60)
                } else {
                    rgba((theme.colors.ui.border << 8) | 0x30)
                },
                "Allow once",
            )
        };

        div()
            .id(SharedString::from(format!("perm-opt-{index}")))
            .mt(px(8.0))
            .px(px(12.0))
            .py(px(10.0))
            .rounded(px(10.0))
            .cursor_pointer()
            .bg(bg)
            .border_1()
            .border_color(border)
            .hover(|d| d.bg(rgba((theme.colors.text.primary << 8) | 0x12)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.approve_permission(Some(option_id.clone()), cx);
            }))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("{} \u{00b7} {}", index + 1, option.name)),
            )
            .child(div().pt(px(2.0)).text_xs().opacity(0.58).child(caption))
            .child(
                div()
                    .pt(px(1.0))
                    .text_xs()
                    .opacity(0.44)
                    .child(option.kind.clone()),
            )
            .into_any_element()
    }

    fn render_permission_overlay(
        request: &AcpApprovalRequest,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let preview = request.preview.clone();
        let selected_index = selected_index.min(request.options.len().saturating_sub(1));

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(theme::modal_overlay_bg(&theme, 0x80))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(640.0))
                    .max_w_full()
                    .mx_4()
                    .p_4()
                    .rounded(px(14.0))
                    .bg(rgb(theme.colors.background.search_box))
                    .border_1()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x99))
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(request.title.clone()),
                    )
                    // ── Structured preview sections ──────────────
                    .when_some(preview.clone(), |d, preview| {
                        d.child(Self::render_permission_header(&preview))
                            .when_some(preview.summary, |d, summary| {
                                d.child(Self::render_permission_section("Summary", summary))
                            })
                            .when_some(preview.input_preview, |d, input| {
                                d.child(Self::render_permission_section("Input", input))
                            })
                            .when_some(preview.output_preview, |d, output| {
                                d.child(Self::render_permission_section("Output", output))
                            })
                            .when(!preview.option_summary.is_empty(), |d| {
                                d.child(
                                    div()
                                        .pt(px(8.0))
                                        .text_xs()
                                        .opacity(0.52)
                                        .child(format!(
                                            "Available options: {}",
                                            preview.option_summary.join(" \u{00b7} ")
                                        )),
                                )
                            })
                    })
                    // ── Fallback to body when no preview ─────────
                    .when(preview.is_none(), |d| {
                        d.child(
                            div()
                                .pt(px(8.0))
                                .pb(px(12.0))
                                .text_sm()
                                .opacity(0.76)
                                .child(request.body.clone()),
                        )
                    })
                    // ── Option list with semantic rows ───────────
                    .children(request.options.iter().enumerate().map(|(i, option)| {
                        Self::render_permission_option_row(option, i, i == selected_index, cx)
                    }))
                    // ── Keyboard hint strip ──────────────────────
                    .child(
                        div()
                            .pt(px(12.0))
                            .text_xs()
                            .opacity(0.56)
                            .child(
                                "Tab/\u{21e7}Tab or \u{2191}\u{2193} or J/K to move \u{00b7} 1\u{2013}9 to choose \u{00b7} Enter to confirm \u{00b7} Esc to cancel",
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_mode_badge(mode_id: &str) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(999.0))
            .bg(rgba((theme.colors.accent.selected << 8) | 0x14))
            .border_1()
            .border_color(rgba((theme.colors.accent.selected << 8) | 0x30))
            .text_xs()
            .opacity(0.78)
            .child(format!("Mode: {mode_id}"))
            .into_any_element()
    }

    fn render_commands_strip(commands: &[String]) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.text.primary << 8) | 0x06))
            .border_1()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x20))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.62)
                    .pb(px(3.0))
                    .child("Commands"),
            )
            .child(
                div()
                    .text_xs()
                    .opacity(0.58)
                    .child(commands.join(" \u{00b7} ")),
            )
            .into_any_element()
    }

    fn render_plan_strip(entries: &[String]) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba((theme.colors.accent.selected << 8) | 0x0C))
            .border_1()
            .border_color(rgba((theme.colors.accent.selected << 8) | 0x28))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.7)
                    .pb(px(4.0))
                    .child("Plan"),
            )
            .children(entries.iter().enumerate().map(|(i, entry)| {
                div()
                    .text_xs()
                    .opacity(0.65)
                    .py(px(1.0))
                    .child(format!("{}. {}", i + 1, entry))
            }))
            .into_any_element()
    }

    // ── Toolbar ───────────────────────────────────────────────────

    fn render_toolbar(
        &self,
        status: AcpThreadStatus,
        has_input_text: bool,
        mode_label: Option<&str>,
        display_name: &str,
        elapsed_secs: Option<u64>,
        context_state: AcpContextBootstrapState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let is_streaming = matches!(status, AcpThreadStatus::Streaming);
        let can_send = matches!(status, AcpThreadStatus::Idle) && has_input_text;
        let context_loading = matches!(context_state, AcpContextBootstrapState::Preparing);

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(4.0))
            .py(px(2.0))
            // ── Left: paste button + context indicator ─
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .id("acp-attach-btn")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(22.0))
                            .rounded(px(6.0))
                            .text_xs()
                            .opacity(0.50)
                            .hover(|s| s.opacity(0.85))
                            .child("+")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                if let Some(clipboard) = cx.read_from_clipboard() {
                                    if let Some(text) = clipboard.text() {
                                        if !text.is_empty() {
                                            this.thread.update(cx, |thread, cx| {
                                                thread.input.insert_str(&text);
                                                cx.notify();
                                            });
                                            this.cursor_visible = true;
                                            cx.notify();
                                        }
                                    }
                                }
                            })),
                    )
                    .when(context_loading, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .opacity(0.40)
                                .child("attaching context\u{2026}"),
                        )
                    })
                    .when(!context_loading, |d| {
                        d.child(div().text_xs().opacity(0.35).child("~/.scriptkit"))
                    }),
            )
            // ── Right: mode, model, send ─────────
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    // Mode pill (label + chevron)
                    .when_some(mode_label.map(str::to_string), |d, mode| {
                        d.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(2.0))
                                .text_xs()
                                .opacity(0.55)
                                .child(mode)
                                .child("\u{25BE}"),
                        )
                    })
                    // Elapsed time (shown after 2s of streaming)
                    .when_some(elapsed_secs.filter(|&s| s >= 2), |d, secs| {
                        d.child(div().text_xs().opacity(0.45).child(format!("{secs}s")))
                    })
                    // Model pill (label + chevron)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .text_xs()
                            .opacity(0.55)
                            .child(display_name.to_string())
                            .child("\u{25BE}"),
                    )
                    // Send / Stop button
                    .child(self.render_send_button(can_send, is_streaming, &theme, cx)),
            )
    }

    fn render_send_button(
        &self,
        can_send: bool,
        is_streaming: bool,
        theme: &crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let accent = theme.colors.accent.selected;
        let text_primary = theme.colors.text.primary;

        let (icon_char, bg, opacity) = if is_streaming {
            // Red stop square
            ("\u{25A0}", rgba(0xEF444460), 0.90_f32)
        } else if can_send {
            // Accent send arrow
            ("\u{2191}", rgba((accent << 8) | 0x30), 0.90)
        } else {
            // Muted disabled arrow
            ("\u{2191}", rgba((text_primary << 8) | 0x06), 0.30)
        };

        let mut btn = div()
            .id("acp-send-btn")
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.0))
            .rounded(px(6.0))
            .bg(bg)
            .text_sm()
            .opacity(opacity);

        if can_send {
            btn = btn
                .cursor_pointer()
                .on_click(cx.listener(|this, _event, _window, cx| {
                    let _ = this.thread.update(cx, |thread, cx| thread.submit_input(cx));
                }));
        } else if is_streaming {
            btn = btn
                .cursor_pointer()
                .on_click(cx.listener(|this, _event, _window, cx| {
                    this.thread
                        .update(cx, |thread, cx| thread.cancel_streaming(cx));
                }));
        }

        btn.child(icon_char).into_any_element()
    }

    // ── Slash command menu ─────────────────────────────────────────

    /// Known Claude Code slash commands (used when the agent doesn't send
    /// an AvailableCommandsUpdate notification).
    const DEFAULT_SLASH_COMMANDS: &'static [&'static str] = &[
        "compact", "clear", "bug", "help", "init", "login", "logout", "status", "cost", "doctor",
        "review", "memory",
    ];

    /// Return commands that match the current `/` prefix in the input.
    fn filtered_slash_commands(&self, cx: &Context<Self>) -> Vec<(String, String)> {
        let text = self.thread.read(cx).input.text().to_string();
        if !text.starts_with('/') {
            return Vec::new();
        }
        let query = &text[1..]; // strip the `/`

        // Use agent-provided commands if available, otherwise cached defaults + skills.
        let agent_commands = self.thread.read(cx).available_commands().to_vec();
        let commands: Vec<(String, String)> = if agent_commands.is_empty() {
            self.cached_slash_commands.clone()
        } else {
            agent_commands
                .into_iter()
                .map(|name| (name, String::new()))
                .collect()
        };

        if query.is_empty() {
            return commands;
        }
        let query_lower = query.to_lowercase();
        commands
            .into_iter()
            .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Update slash menu state after the input text changes.
    fn update_slash_menu(&mut self, cx: &Context<Self>) {
        let text = self.thread.read(cx).input.text().to_string();
        if text.starts_with('/') && !text.contains(' ') {
            // Show menu when typing /... (no space yet = still filtering)
            let filtered = self.filtered_slash_commands(cx);
            if !filtered.is_empty() {
                let idx = self.slash_menu_index.unwrap_or(0);
                self.slash_menu_index = Some(idx.min(filtered.len().saturating_sub(1)));
            } else {
                self.slash_menu_index = None;
            }
        } else {
            self.slash_menu_index = None;
        }
    }

    fn render_slash_menu(
        &self,
        commands: &[(String, String)],
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .id("acp-slash-menu")
            .w_full()
            .max_h(px(200.0))
            .overflow_y_scroll()
            .rounded(px(8.0))
            .bg(rgb(theme.colors.background.search_box))
            .border_1()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x40))
            .py(px(4.0))
            .children(commands.iter().enumerate().map(|(i, (name, desc))| {
                let is_selected = i == selected_index;
                let cmd_text = format!("/{name} ");
                div()
                    .id(SharedString::from(format!("slash-cmd-{i}")))
                    .w_full()
                    .px(px(10.0))
                    .py(px(4.0))
                    .cursor_pointer()
                    .when(is_selected, |d| {
                        d.bg(rgba((theme.colors.accent.selected << 8) | 0x1C))
                    })
                    .hover(|d| d.bg(rgba((theme.colors.text.primary << 8) | 0x0C)))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.thread.update(cx, |thread, cx| {
                            thread.input.set_text(cmd_text.clone());
                            cx.notify();
                        });
                        this.slash_menu_index = None;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().text_sm().child(format!("/{name}")))
                            .when(!desc.is_empty(), |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .opacity(0.45)
                                        .overflow_x_hidden()
                                        .child(desc.clone()),
                                )
                            }),
                    )
            }))
            .into_any_element()
    }

    // ── Key handling ──────────────────────────────────────────────

    fn handle_key_down(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        // Reset cursor blink on any key press.
        self.cursor_visible = true;

        // ── Permission overlay intercept ─────────────────────────
        let pending_permission = self.thread.read(cx).pending_permission.clone();
        if let Some(ref request) = pending_permission {
            if self.handle_permission_key_down(event, request, cx) {
                cx.stop_propagation();
                return;
            }
            // Block composer typing behind the modal, but still allow
            // platform/control/alt shortcuts to propagate.
            if !event.keystroke.modifiers.platform
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt
            {
                cx.stop_propagation();
                return;
            }
        }

        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        // ── Slash command menu intercept ─────────────────────────
        if self.slash_menu_index.is_some() {
            if crate::ui_foundation::is_key_up(key) {
                let idx = self.slash_menu_index.unwrap_or(0);
                self.slash_menu_index = Some(idx.saturating_sub(1));
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_down(key) {
                let idx = self.slash_menu_index.unwrap_or(0);
                let filtered = self.filtered_slash_commands(cx);
                self.slash_menu_index = Some((idx + 1).min(filtered.len().saturating_sub(1)));
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_enter(key) {
                let filtered = self.filtered_slash_commands(cx);
                let idx = self.slash_menu_index.unwrap_or(0);
                if let Some((name, _)) = filtered.get(idx) {
                    let cmd_text = format!("/{name} ");
                    self.thread.update(cx, |thread, cx| {
                        thread.input.set_text(cmd_text);
                        cx.notify();
                    });
                }
                self.slash_menu_index = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_escape(key) {
                self.slash_menu_index = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            // Other keys fall through to normal input handling,
            // which will update the filter text.
        }

        // Shift+Enter inserts a newline.
        if crate::ui_foundation::is_key_enter(key) && modifiers.shift {
            self.thread.update(cx, |thread, cx| {
                thread.input.insert_char('\n');
                cx.notify();
            });
            cx.stop_propagation();
            return;
        }

        // Enter submits.
        if crate::ui_foundation::is_key_enter(key) && !modifiers.shift {
            self.slash_menu_index = None;
            let _ = self.thread.update(cx, |thread, cx| thread.submit_input(cx));
            cx.stop_propagation();
            return;
        }

        // Delegate all other keys to TextInputState::handle_key().
        // handle_key requires T: Render, so we extract input, mutate it here,
        // then write it back.
        let key_char = event.keystroke.key_char.as_deref();
        let mut input_snapshot = self.thread.read(cx).input.clone();
        let handled = input_snapshot.handle_key(
            key,
            key_char,
            modifiers.platform,
            modifiers.alt,
            modifiers.shift,
            cx,
        );

        if handled {
            self.thread.update(cx, |thread, cx| {
                thread.input = input_snapshot;
                cx.notify();
            });
            self.update_slash_menu(cx);
            cx.stop_propagation();
        } else {
            cx.propagate();
        }
    }
}

impl Focusable for AcpChatView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AcpChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let thread = self.thread.read(cx);
        let status = thread.status;
        let is_empty = thread.messages.is_empty();
        let input_text = thread.input.text().to_string();
        let input_cursor = thread.input.cursor();
        let input_selection = thread.input.selection();
        let cursor_visible = self.cursor_visible;
        let pending_permission = thread.pending_permission.clone();
        let plan_entries = thread.active_plan_entries().to_vec();
        let mode_label = thread.active_mode_id().map(str::to_string);
        let display_name = thread.display_name().to_string();
        let elapsed_secs = thread.stream_elapsed_secs();
        let available_commands = thread.available_commands().to_vec();
        let context_state = thread.context_bootstrap_state();
        // queued_submit and context_note removed — no blocking empty state.
        let messages: Vec<AcpThreadMessage> = thread.messages.clone();
        let colors = Self::prompt_colors();
        let theme = theme::get_cached_theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    this.handle_key_down(event, cx);
                }),
            )
            // ── TOP: Input (exact match with main menu mini layout) ────
            // Uses same constants: HEADER_PADDING_X=12, HEADER_PADDING_Y=10,
            // input_height=22 (CURSOR_HEIGHT_LG+2*CURSOR_MARGIN_Y), font_size_lg=16
            .child(
                div()
                    .w_full()
                    .px(px(12.0))
                    .py(px(10.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(22.0))
                            .text_size(px(16.0))
                            .text_color(if input_text.is_empty() {
                                rgb(theme.colors.text.muted)
                            } else {
                                rgb(theme.colors.text.primary)
                            })
                            .child(if input_text.is_empty() {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .child(div().w(px(2.0)).h(px(18.0)).when(cursor_visible, |d| {
                                        d.bg(rgb(theme.colors.text.primary))
                                    }))
                                    .child(
                                        div()
                                            .ml(px(-2.0))
                                            .text_color(rgb(theme.colors.text.muted))
                                            .child("Ask Claude Code\u{2026}"),
                                    )
                                    .into_any_element()
                            } else {
                                render_text_input_cursor_selection(TextInputRenderConfig {
                                    cursor: input_cursor,
                                    selection: Some(input_selection),
                                    cursor_visible,
                                    cursor_color: theme.colors.accent.selected,
                                    text_color: theme.colors.text.primary,
                                    selection_color: theme.colors.accent.selected,
                                    selection_text_color: theme.colors.text.primary,
                                    cursor_height: 18.0,
                                    cursor_width: 2.0,
                                    container_height: Some(22.0),
                                    ..TextInputRenderConfig::default_for_prompt(&input_text)
                                })
                                .into_any_element()
                            }),
                    ),
            )
            // ── Slash command menu (below input) ─────────────
            .when_some(self.slash_menu_index, |d, idx| {
                let filtered = self.filtered_slash_commands(cx);
                if filtered.is_empty() {
                    d
                } else {
                    d.child(
                        div()
                            .w_full()
                            .px(px(8.0))
                            .child(self.render_slash_menu(&filtered, idx, cx)),
                    )
                }
            })
            // ── Message list (middle, scrollable) ────────────
            .child(
                div()
                    .id("acp-message-list")
                    .flex_grow()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .min_h(gpui::px(0.))
                    .when(!is_empty, |d| {
                        d.px(px(8.0)).py(px(8.0)).flex().flex_col().children(
                            messages.iter().enumerate().map(|(i, msg)| {
                                let msg_id = msg.id;
                                let is_collapsible = matches!(
                                    msg.role,
                                    AcpThreadMessageRole::Thought | AcpThreadMessageRole::Tool
                                );
                                let is_collapsed =
                                    if matches!(msg.role, AcpThreadMessageRole::Thought) {
                                        !self.collapsed_ids.contains(&msg_id)
                                    } else {
                                        is_collapsible && self.collapsed_ids.contains(&msg_id)
                                    };

                                let on_toggle: Option<ToggleHandler> = if is_collapsible {
                                    Some(Box::new(cx.listener(move |this, _event, _window, cx| {
                                        if this.collapsed_ids.contains(&msg_id) {
                                            this.collapsed_ids.remove(&msg_id);
                                        } else {
                                            this.collapsed_ids.insert(msg_id);
                                        }
                                        cx.notify();
                                    })))
                                } else {
                                    None
                                };

                                let prev_was_user = i > 0
                                    && matches!(messages[i - 1].role, AcpThreadMessageRole::User);
                                let is_response_start = prev_was_user
                                    && !matches!(msg.role, AcpThreadMessageRole::User);

                                div()
                                    .w_full()
                                    .pb(px(4.0))
                                    .when(is_response_start, |d| d.mt(px(4.0)))
                                    .child(Self::render_message(
                                        msg,
                                        &colors,
                                        is_collapsed,
                                        on_toggle,
                                    ))
                            }),
                        )
                    }),
            )
            // ── Plan strip ────────────────────────────────────
            .when(!plan_entries.is_empty(), |d| {
                d.child(
                    div()
                        .w_full()
                        .px(px(8.0))
                        .pb(px(4.0))
                        .child(Self::render_plan_strip(&plan_entries)),
                )
            })
            // ── BOTTOM: Toolbar with cwd ─────────────────────
            .child(self.render_toolbar(
                status,
                !input_text.is_empty(),
                mode_label.as_deref(),
                &display_name,
                elapsed_secs,
                context_state,
                cx,
            ))
            // ── Permission overlay ────────────────────────────
            .when_some(pending_permission, |d, request| {
                d.child(Self::render_permission_overlay(
                    &request,
                    self.permission_index,
                    cx,
                ))
            })
    }
}
