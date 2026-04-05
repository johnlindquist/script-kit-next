//! ACP chat view.
//!
//! Renders an ACP conversation thread with markdown-rendered messages,
//! role-aware cards, empty/streaming/error states, and inline permission
//! approval cards. Wraps an `AcpThread` entity for the Tab AI surface.

use std::collections::HashSet;
use std::time::Duration;

use gpui::{
    div, list, prelude::*, px, rgb, rgba, Animation, AnimationExt, App, Context, Entity,
    FocusHandle, Focusable, FontWeight, IntoElement, ListAlignment, ListState, ParentElement,
    Render, SharedString, Task, WeakEntity, Window,
};

use gpui_component::scroll::ScrollableElement;

use crate::components::text_input::{
    render_text_input_cursor_selection, TextHighlightRange, TextInputRenderConfig,
    TextInputRenderIndicator,
};
use crate::prompts::markdown::render_markdown_with_scope;
use crate::theme::{self, PromptColors};

use super::thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadMessage, AcpThreadMessageRole, AcpThreadStatus,
};
use super::{AcpApprovalOption, AcpApprovalPreview, AcpApprovalPreviewKind, AcpApprovalRequest};

use crate::ai::message_parts::AiContextPart;
use crate::ai::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger,
};
use crate::ai::window::context_picker::{build_picker_items, build_slash_picker_items};

/// Active @-mention session state for the ACP inline context picker.
#[derive(Debug, Clone)]
struct AcpMentionSession {
    /// Which trigger character opened this session (`@` or `/`).
    trigger: ContextPickerTrigger,
    /// Character range of the trigger+query in the input text.
    trigger_range: std::ops::Range<usize>,
    /// Currently highlighted row index.
    selected_index: usize,
    /// Ranked picker items for the current query.
    items: Vec<ContextPickerItem>,
}

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

/// Session mode for the ACP chat view.
#[derive(Clone)]
pub(crate) enum AcpChatSession {
    /// Live conversation with an ACP agent thread.
    Live(Entity<AcpThread>),
    /// Inline setup card — no launchable agent exists.
    Setup(Box<super::setup_state::AcpInlineSetupState>),
}

/// GPUI view entity wrapping an `AcpThread` for the Tab AI surface.
pub(crate) struct AcpChatView {
    /// The ACP session — either a live thread or inline setup state.
    pub(crate) session: AcpChatSession,
    focus_handle: FocusHandle,
    /// Virtualized variable-height message list state.
    pub(crate) list_state: ListState,
    /// Index of the currently highlighted permission option in the inline card.
    permission_index: usize,
    /// Whether the inline permission options list is expanded.
    permission_options_open: bool,
    /// Message IDs that are currently collapsed (thinking/tool blocks).
    pub(crate) collapsed_ids: HashSet<u64>,
    /// Track message count for list splice updates.
    last_message_count: usize,
    /// Cursor blink state.
    cursor_visible: bool,
    /// Handle to the cursor blink task.
    _blink_task: Task<()>,
    /// History picker: (selected_index, filter_text, all_entries). None = hidden.
    pub(crate) history_menu: Option<(usize, String, Vec<super::history::AcpHistoryEntry>)>,
    /// Whether the + attachment menu popup is open.
    attach_menu_open: bool,
    /// Whether the model selector dropdown is open.
    model_selector_open: bool,
    /// Cmd+F search: (query, current_match_index). None = search hidden.
    pub(crate) search_state: Option<(String, usize)>,
    /// Cached slash commands (name, description) discovered at creation.
    cached_slash_commands: Vec<(String, String)>,
    /// Handle to the deferred slash command discovery task.
    _slash_discovery_task: Task<()>,
    /// Active @-mention picker session (None = picker hidden).
    mention_session: Option<AcpMentionSession>,
    /// Canonical inline tokens that currently own their attached context part.
    ///
    /// This preserves non-inline chip attachments during mention sync while
    /// still letting deleted inline mentions remove the parts they created.
    inline_owned_context_tokens: HashSet<String>,
    /// Agent picker overlay state for setup mode (None = hidden).
    setup_agent_picker: Option<AcpSetupAgentPickerState>,
    /// Most recently accepted picker item (for telemetry/testing).
    last_accepted_item: Option<crate::protocol::AcpAcceptedItem>,
    /// Bounded test probe ring buffer for agentic verification.
    test_probe: AcpTestProbe,
}

/// Bounded ring buffer for ACP test probe events.
///
/// Agents can reset, record, and snapshot this to verify native picker
/// acceptance without scraping logs. Storage is cheap and bounded.
#[derive(Clone, Debug, Default)]
pub(crate) struct AcpTestProbe {
    /// Monotonically increasing event counter.
    pub(crate) event_seq: u64,
    /// Recent key-route events (bounded by `MAX_EVENTS`).
    pub(crate) key_routes: std::collections::VecDeque<crate::protocol::AcpKeyRouteTelemetry>,
    /// Recent picker-acceptance events (bounded by `MAX_EVENTS`).
    pub(crate) accepted_items:
        std::collections::VecDeque<crate::protocol::AcpPickerItemAcceptedTelemetry>,
    /// Most recent input-layout telemetry.
    pub(crate) input_layout: Option<crate::protocol::AcpInputLayoutTelemetry>,
}

use crate::protocol::ACP_TEST_PROBE_MAX_EVENTS;

/// State for the setup-mode agent selection picker.
#[derive(Debug, Clone)]
struct AcpSetupAgentPickerState {
    items: Vec<super::catalog::AcpAgentCatalogEntry>,
    selected_index: usize,
}

impl AcpChatView {
    fn composer_is_active(
        window_active: bool,
        view_focused: bool,
        actions_window_open: bool,
    ) -> bool {
        window_active && view_focused && !actions_window_open
    }

    fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
        text.char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(text.len())
    }

    fn telemetry_item_id(item: &ContextPickerItem) -> String {
        match &item.kind {
            ContextPickerItemKind::BuiltIn(_) | ContextPickerItemKind::SlashCommand(_) => {
                item.id.to_string()
            }
            ContextPickerItemKind::File(_) => format!("file:{}", item.label),
            ContextPickerItemKind::Folder(_) => format!("folder:{}", item.label),
        }
    }

    /// Access the live thread entity, if in live mode.
    pub(crate) fn thread(&self) -> Option<Entity<AcpThread>> {
        match &self.session {
            AcpChatSession::Live(t) => Some(t.clone()),
            AcpChatSession::Setup(_) => None,
        }
    }

    /// Whether this view is in setup mode (no live thread).
    pub(crate) fn is_setup_mode(&self) -> bool {
        matches!(self.session, AcpChatSession::Setup(_))
    }

    /// Internal accessor returning a reference to the live thread entity.
    ///
    /// Only called from code paths guarded by `render()` and `handle_key_down()`
    /// early-returns in setup mode.
    #[inline]
    pub(crate) fn live_thread(&self) -> &Entity<AcpThread> {
        match &self.session {
            AcpChatSession::Live(t) => t,
            AcpChatSession::Setup(_) => unreachable!("live_thread called in setup mode"),
        }
    }

    /// Build a machine-readable ACP state snapshot for agentic testing.
    ///
    /// Returns cursor, picker, accepted item, thread status, layout metrics,
    /// and context readiness — everything an agent needs to verify ACP
    /// interactions without screenshots.
    pub(crate) fn collect_acp_state_snapshot(&self, cx: &App) -> crate::protocol::AcpStateSnapshot {
        use crate::protocol::{
            AcpInputLayoutMetrics, AcpPickerState as PickerState, AcpStateSnapshot,
            ACP_STATE_SCHEMA_VERSION,
        };

        if self.is_setup_mode() {
            return AcpStateSnapshot {
                status: "setup".to_string(),
                ..Default::default()
            };
        }

        let thread = self.live_thread().read(cx);
        let status_str = match thread.status {
            AcpThreadStatus::Idle => "idle",
            AcpThreadStatus::Streaming => "streaming",
            AcpThreadStatus::WaitingForPermission => "waitingForPermission",
            AcpThreadStatus::Error => "error",
        };

        let input_text = thread.input.text().to_string();
        let cursor_index = thread.input.cursor();
        let selection = thread.input.selection();
        let has_selection = !selection.is_empty();
        let selection_range = if has_selection {
            let (start, end) = selection.range();
            Some([start, end])
        } else {
            None
        };

        let picker = self.mention_session.as_ref().map(|session| {
            let selected_label = session
                .items
                .get(session.selected_index)
                .map(|item| item.label.to_string());
            let trigger = match session.trigger {
                ContextPickerTrigger::Mention => "@",
                ContextPickerTrigger::Slash => "/",
            };
            PickerState {
                open: true,
                trigger: trigger.to_string(),
                item_count: session.items.len(),
                selected_index: session.selected_index,
                selected_label,
            }
        });

        let char_count = input_text.chars().count();
        let (visible_start, visible_end) = thread.input.visible_window_range(60);
        let cursor_in_window = cursor_index.saturating_sub(visible_start);

        let context_ready = thread.context_bootstrap_state() != AcpContextBootstrapState::Preparing;

        AcpStateSnapshot {
            schema_version: ACP_STATE_SCHEMA_VERSION,
            status: status_str.to_string(),
            input_text,
            cursor_index,
            has_selection,
            selection_range,
            message_count: thread.messages.len(),
            picker,
            last_accepted_item: self.last_accepted_item.clone(),
            context_chip_count: thread.pending_context_parts().len(),
            context_ready,
            has_pending_permission: thread.pending_permission.is_some(),
            input_layout: Some(AcpInputLayoutMetrics {
                char_count,
                visible_start,
                visible_end,
                cursor_in_window,
            }),
        }
    }

    pub(crate) fn new(thread: Entity<AcpThread>, cx: &mut Context<Self>) -> Self {
        // Virtualized list: bottom-aligned (chat), 200px overdraw for smooth scroll.
        let list_state = ListState::new(0, ListAlignment::Bottom, px(200.0));
        list_state.set_follow_tail(true);

        // Auto-scroll when thread state changes (new messages, streaming updates).
        cx.observe(&thread, |this: &mut Self, thread, cx| {
            let thread_ref = thread.read(cx);
            let count = thread_ref.messages.len();
            let is_streaming = matches!(thread_ref.status, AcpThreadStatus::Streaming);

            // Splice new messages into the list state.
            if count != this.last_message_count {
                let old_count = this.last_message_count;
                this.last_message_count = count;
                let delta = count.saturating_sub(old_count);
                if count > old_count && delta <= 3 {
                    // Small append (typical streaming: 1-2 new messages at a time).
                    this.list_state
                        .splice(old_count..old_count, count - old_count);
                } else {
                    // Full replacement (conversation load, clear, or large batch).
                    this.list_state.reset(count);
                    this.list_state.set_follow_tail(true);
                }
            }

            // Re-engage follow-tail when streaming starts so content stays at bottom.
            if is_streaming {
                this.list_state.set_follow_tail(true);
            }

            // Update the unified picker (@ mentions + / commands) on any input change.
            this.refresh_mention_session(cx);
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

        // Defer slash command discovery (filesystem I/O) to after the first
        // render frame so the view switch is not blocked by skill enumeration.
        let slash_task = cx.spawn(async move |this, cx| {
            // Yield to let the initial render happen first.
            cx.background_executor()
                .timer(Duration::from_millis(1))
                .await;
            let commands = Self::discover_slash_commands();
            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    view.cached_slash_commands = commands;
                    cx.notify();
                })
            });
        });

        Self {
            session: AcpChatSession::Live(thread),
            focus_handle: cx.focus_handle(),
            list_state,
            permission_index: 0,
            permission_options_open: false,
            collapsed_ids: HashSet::new(),
            last_message_count: 0,
            cursor_visible: true,
            _blink_task: blink_task,
            history_menu: None,
            attach_menu_open: false,
            model_selector_open: false,
            search_state: None,
            cached_slash_commands: Vec::new(),
            _slash_discovery_task: slash_task,
            mention_session: None,
            inline_owned_context_tokens: HashSet::new(),
            setup_agent_picker: None,
            last_accepted_item: None,
            test_probe: AcpTestProbe::default(),
        }
    }

    /// Create an `AcpChatView` in **setup mode** — no live thread, just an
    /// inline setup card describing the blocker and available recovery actions.
    pub(crate) fn new_setup(
        state: super::setup_state::AcpInlineSetupState,
        cx: &mut Context<Self>,
    ) -> Self {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_surface_rendered",
            title = %state.title,
        );
        let list_state = ListState::new(0, ListAlignment::Bottom, px(200.0));
        let noop_blink = cx.spawn(async move |_this, _cx| {});
        let noop_slash = cx.spawn(async move |_this, _cx| {});
        Self {
            session: AcpChatSession::Setup(Box::new(state)),
            focus_handle: cx.focus_handle(),
            list_state,
            permission_index: 0,
            permission_options_open: false,
            collapsed_ids: HashSet::new(),
            last_message_count: 0,
            cursor_visible: false,
            _blink_task: noop_blink,
            history_menu: None,
            attach_menu_open: false,
            model_selector_open: false,
            search_state: None,
            cached_slash_commands: Vec::new(),
            _slash_discovery_task: noop_slash,
            mention_session: None,
            inline_owned_context_tokens: HashSet::new(),
            setup_agent_picker: None,
            last_accepted_item: None,
            test_probe: AcpTestProbe::default(),
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

    /// Consume Tab / Shift+Tab. When a permission card is active,
    /// cycle the highlighted option; otherwise just swallow the key so
    /// the global interceptors do not re-open a fresh ACP chat.
    pub(crate) fn handle_tab_key(&mut self, has_shift: bool, cx: &mut Context<Self>) -> bool {
        if self.is_setup_mode() {
            cx.notify();
            return true;
        }

        let option_count = self
            .live_thread()
            .read(cx)
            .pending_permission
            .as_ref()
            .map(|r| r.options.len())
            .unwrap_or(0);

        if option_count > 0 {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, has_shift);
            self.permission_options_open = option_count > 1;
            cx.notify();
            return true;
        }

        // Plain Tab accepts the focused picker item (same as Enter but without submit).
        if !has_shift && self.mention_session.is_some() {
            let pre_accept_item = self.mention_session.as_ref().and_then(|s| {
                s.items.get(s.selected_index).map(|item| {
                    let trigger_str = match s.trigger {
                        crate::ai::window::context_picker::types::ContextPickerTrigger::Mention => {
                            "@"
                        }
                        crate::ai::window::context_picker::types::ContextPickerTrigger::Slash => {
                            "/"
                        }
                    };
                    (
                        trigger_str.to_string(),
                        item.label.to_string(),
                        Self::telemetry_item_id(item),
                    )
                })
            });
            let cursor_before = self.live_thread().read(cx).input.cursor();
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_picker_tab_accept",
                selected_index = self
                    .mention_session
                    .as_ref()
                    .map(|s| s.selected_index)
                    .unwrap_or(0),
            );
            self.accept_mention_selection_impl(false, cx);
            let cursor_after = self.live_thread().read(cx).input.cursor();
            self.emit_key_route_telemetry(
                "tab",
                crate::protocol::AcpKeyRoute::Picker,
                cursor_before,
                cursor_after,
                false,
                true,
            );
            if let Some((trigger, label, id)) = pre_accept_item {
                self.emit_picker_accepted_telemetry(
                    &trigger,
                    &label,
                    &id,
                    "tab",
                    cursor_after,
                    false,
                );
            }
            if let Some(ref layout) = self.collect_acp_state_snapshot(cx).input_layout {
                self.emit_input_layout_telemetry(layout);
            }
            return true;
        }

        cx.notify();
        true
    }

    fn approve_permission(&mut self, option_id: Option<String>, cx: &mut Context<Self>) {
        self.permission_index = 0;
        self.permission_options_open = false;
        self.live_thread().update(cx, |thread, cx| {
            thread.approve_pending_permission(option_id, cx);
        });
    }

    fn permission_request_tool_call_id(request: &AcpApprovalRequest) -> Option<&str> {
        let tool_call_id = request.preview.as_ref()?.tool_call_id.trim();
        if tool_call_id.is_empty() {
            None
        } else {
            Some(tool_call_id)
        }
    }

    fn permission_request_matches_message(
        msg: &AcpThreadMessage,
        request: &AcpApprovalRequest,
    ) -> bool {
        msg.tool_call_id
            .as_deref()
            .zip(Self::permission_request_tool_call_id(request))
            .is_some_and(|(msg_id, request_id)| msg_id == request_id)
    }

    fn selected_permission_option<'a>(
        &self,
        request: &'a AcpApprovalRequest,
    ) -> Option<(usize, &'a AcpApprovalOption)> {
        let index = self.normalized_permission_index(request.options.len());
        request.options.get(index).map(|option| (index, option))
    }

    fn first_allow_once_option(
        request: &AcpApprovalRequest,
    ) -> Option<(usize, &AcpApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| !option.is_reject() && !option.is_persistent_allow())
    }

    fn first_allow_option(request: &AcpApprovalRequest) -> Option<(usize, &AcpApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| !option.is_reject())
    }

    fn first_reject_option(request: &AcpApprovalRequest) -> Option<(usize, &AcpApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| option.is_reject())
    }

    fn preferred_allow_option<'a>(
        &self,
        request: &'a AcpApprovalRequest,
    ) -> Option<(usize, &'a AcpApprovalOption)> {
        match self.selected_permission_option(request) {
            Some((index, option)) if !option.is_reject() => Some((index, option)),
            _ => {
                Self::first_allow_once_option(request).or_else(|| Self::first_allow_option(request))
            }
        }
    }

    fn approve_preferred_allow_option(
        &mut self,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some((index, option)) = self.preferred_allow_option(request) {
            self.permission_index = index;
            self.approve_permission(Some(option.option_id.clone()), cx);
            true
        } else {
            false
        }
    }

    fn approve_reject_option(
        &mut self,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some((index, option)) = Self::first_reject_option(request) {
            self.permission_index = index;
            self.approve_permission(Some(option.option_id.clone()), cx);
            true
        } else {
            self.approve_permission(None, cx);
            true
        }
    }

    fn toggle_permission_options(
        &mut self,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if request.options.len() <= 1 {
            return false;
        }

        if !self.permission_options_open {
            if let Some((index, _)) = self.preferred_allow_option(request) {
                self.permission_index = index;
            }
        }

        self.permission_options_open = !self.permission_options_open;
        cx.notify();
        true
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

    /// Handle key events when an inline permission card is active.
    /// Returns `true` if the key was consumed.
    fn handle_permission_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        request: &AcpApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        let option_count = request.options.len();
        self.permission_index = self.normalized_permission_index(option_count);

        if modifiers.platform
            && !modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("y")
        {
            return self.approve_preferred_allow_option(request, cx);
        }

        if modifiers.platform
            && modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("a")
        {
            self.toggle_permission_options(request, cx);
            return true;
        }

        if modifiers.platform
            && modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("z")
        {
            return self.approve_reject_option(request, cx);
        }

        if crate::ui_foundation::is_key_up(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, true);
            self.permission_options_open = option_count > 1;
            cx.notify();
            return true;
        }

        if crate::ui_foundation::is_key_down(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, false);
            self.permission_options_open = option_count > 1;
            cx.notify();
            return true;
        }

        // J/K navigation (vim-style, unmodified only)
        match key {
            "j" | "J" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, false);
                self.permission_options_open = option_count > 1;
                cx.notify();
                return true;
            }
            "k" | "K" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, true);
                self.permission_options_open = option_count > 1;
                cx.notify();
                return true;
            }
            _ => {}
        }

        if crate::ui_foundation::is_key_escape(key) && self.permission_options_open {
            self.permission_options_open = false;
            cx.notify();
            return true;
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
                let _ = self.approve_preferred_allow_option(request, cx);
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
        self.live_thread()
            .update(cx, |thread, cx| thread.set_input(value, cx));
    }

    // ── Rendering helpers ─────────────────────────────────────────

    fn prompt_colors() -> PromptColors {
        PromptColors::from_theme(&theme::get_cached_theme())
    }

    /// Render context chips below the composer input, but only for parts
    /// that are NOT already represented by an inline `@mention` token.
    ///
    /// Accent left-bar design: a 2px gold bar on the left edge with
    /// a ghost-opacity chip containing the label and a × dismiss button.
    fn render_pending_context_chips(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use crate::ai::context_mentions::{parse_inline_context_mentions, part_to_inline_token};

        let (parts, input_text) = {
            let thread = self.live_thread().read(cx);
            (
                thread.pending_context_parts().to_vec(),
                thread.input.text().to_string(),
            )
        };

        if parts.is_empty() {
            return div()
                .id("acp-pending-context-chips-empty")
                .into_any_element();
        }

        // Tokens present inline — suppress chips for these.
        let inline_tokens: HashSet<String> = parse_inline_context_mentions(&input_text)
            .into_iter()
            .map(|m| m.token)
            .collect();

        // Filter to parts that have no inline token representation.
        let chip_parts: Vec<(usize, &AiContextPart)> = parts
            .iter()
            .enumerate()
            .filter(|(_, part)| match part_to_inline_token(part) {
                Some(token) => !inline_tokens.contains(&token),
                None => true,
            })
            .collect();

        if chip_parts.is_empty() {
            return div()
                .id("acp-pending-context-chips-empty")
                .into_any_element();
        }

        let theme = theme::get_cached_theme();
        let accent = theme.colors.accent.selected;
        let border = theme.colors.ui.border;
        let dimmed = theme.colors.text.dimmed;
        let muted_text = theme.colors.text.muted;
        let primary_text = theme.colors.text.primary;

        // Show only the first non-inline part as a chip.
        let (remove_idx, part) = chip_parts[0];
        let label = SharedString::from(part.label().to_string());

        let chip = div()
            .id("acp-ctx-chip")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(5.0))
            // Gold left accent bar
            .child(
                div()
                    .w(px(2.0))
                    .h(px(14.0))
                    .rounded(px(1.0))
                    .bg(rgb(accent)),
            )
            // Label + dismiss in ghost container
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(4.0))
                    .py(px(2.0))
                    .rounded(px(3.0))
                    .bg(rgba((border << 8) | 0x0A))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(dimmed))
                            .overflow_hidden()
                            .text_ellipsis()
                            .max_w(px(280.0))
                            .child(label),
                    )
                    .child(
                        div()
                            .id("acp-ctx-remove-0")
                            .cursor_pointer()
                            .text_xs()
                            .text_color(rgba((muted_text << 8) | 0x60))
                            .px(px(4.0))
                            .py(px(1.0))
                            .rounded(px(999.0))
                            .hover(|el| {
                                el.text_color(rgb(primary_text))
                                    .bg(rgba((border << 8) | 0x18))
                                    .rounded(px(999.0))
                            })
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.live_thread().update(cx, |thread, cx| {
                                    thread.remove_context_part(remove_idx, cx);
                                });
                            }))
                            .child("\u{00d7}"),
                    ),
            );

        div()
            .id("acp-pending-context-chips")
            .px(px(12.0))
            .pb(px(6.0))
            .child(chip)
            .into_any_element()
    }

    /// Render a bootstrap note row below the context chips.
    ///
    /// Shows a status note during Ask Anything capture (e.g. "Capturing
    /// desktop context…" while preparing, "Ask Anything ready" once done).
    /// Hidden when there is no note or when the note is empty.
    fn render_context_bootstrap_note(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let (state, note) = {
            let thread = self.live_thread().read(cx);
            (
                thread.context_bootstrap_state(),
                thread.context_bootstrap_note().map(|v| v.to_string()),
            )
        };

        let Some(note) = note.filter(|v| !v.trim().is_empty()) else {
            return div()
                .id("acp-context-bootstrap-note-empty")
                .into_any_element();
        };

        let theme = theme::get_cached_theme();
        let accent = theme.colors.accent.selected;
        let border = theme.colors.ui.border;

        let (fg_color, bg, outline) = match state {
            AcpContextBootstrapState::Preparing => {
                (accent, (accent << 8) | 0x10, (accent << 8) | 0x24)
            }
            AcpContextBootstrapState::Ready => (
                theme.colors.text.muted,
                (border << 8) | 0x10,
                (border << 8) | 0x24,
            ),
            AcpContextBootstrapState::Failed => (
                theme.colors.text.primary,
                (border << 8) | 0x14,
                (border << 8) | 0x28,
            ),
        };

        div()
            .id("acp-context-bootstrap-note")
            .px(px(12.0))
            .pb(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(6.0))
                    .bg(rgba(bg))
                    .border_1()
                    .border_color(rgba(outline))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(fg_color))
                            .child(SharedString::from(note)),
                    ),
            )
            .into_any_element()
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
        let (label, status_hint) = if is_tool {
            // Tool body format: "{title}\n{status}\n{content}"
            let mut lines = msg.body.lines();
            let title = lines
                .next()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() < 80)
                .unwrap_or_else(|| "Tool".to_string());
            let status = lines
                .next()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() < 40);
            (title, status)
        } else {
            ("Thinking".to_string(), None)
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
            .when_some(status_hint.clone(), |d, status| {
                d.child(div().text_xs().opacity(0.35).child(status))
            })
            .when(
                is_collapsed && line_count > 1 && status_hint.is_none(),
                |d| {
                    d.child(
                        div()
                            .text_xs()
                            .opacity(0.35)
                            .child(format!("{line_count} lines")),
                    )
                },
            );

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
            .py(px(8.0))
            .rounded(px(8.0))
            .bg(rgba(0xEF444410))
            .border_l_2()
            .border_color(rgba(0xEF444480))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .pb(px(4.0))
                    .child(div().text_xs().opacity(0.75).child("\u{26A0}"))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .opacity(0.75)
                            .child("Error"),
                    ),
            )
            .child(render_markdown_with_scope(&msg.body, colors, Some(&scope_id)).w_full())
            .child(
                div().pt(px(4.0)).text_xs().opacity(0.40).child(
                    "Try sending your message again or use \u{2318}N for a new conversation",
                ),
            )
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
        let theme = theme::get_cached_theme();

        div()
            .pt(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.48)
                    .child(title),
            )
            .child(
                div()
                    .mt(px(4.0))
                    .max_h(px(120.0))
                    .overflow_y_hidden()
                    .border_l_2()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x18))
                    .bg(rgba((theme.colors.text.primary << 8) | 0x04))
                    .pl(px(10.0))
                    .pr(px(8.0))
                    .py(px(6.0))
                    .text_xs()
                    .opacity(0.76)
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
            .pt(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .px(px(7.0))
                            .py(px(3.0))
                            .rounded(px(999.0))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(badge_border)
                            .text_xs()
                            .opacity(0.68)
                            .child(preview.kind.badge_label()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .opacity(0.92)
                            .child(preview.tool_title.clone()),
                    ),
            )
            .when_some(preview.subject.clone(), |d, subject| {
                d.child(div().pt(px(4.0)).text_sm().opacity(0.68).child(subject))
            })
            .into_any_element()
    }

    fn render_permission_option_row(
        option: &AcpApprovalOption,
        index: usize,
        is_selected: bool,
        view: WeakEntity<AcpChatView>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let option_id = option.option_id.clone();

        let (accent, bg, hover_bg, caption) = if option.is_reject() {
            (
                rgba(0xEF4444AA),
                if is_selected {
                    rgba(0xEF444418)
                } else {
                    rgba(0xEF444406)
                },
                rgba(0xEF444410),
                "Deny this request",
            )
        } else if option.is_persistent_allow() {
            (
                rgb(theme.colors.accent.selected),
                if is_selected {
                    rgba((theme.colors.accent.selected << 8) | 0x18)
                } else {
                    rgba((theme.colors.accent.selected << 8) | 0x06)
                },
                rgba((theme.colors.accent.selected << 8) | 0x10),
                "Remember this choice",
            )
        } else {
            (
                rgb(theme.colors.accent.selected),
                if is_selected {
                    rgba((theme.colors.accent.selected << 8) | 0x12)
                } else {
                    rgba((theme.colors.text.primary << 8) | 0x04)
                },
                rgba((theme.colors.text.primary << 8) | 0x08),
                "Approve once",
            )
        };

        div()
            .id(SharedString::from(format!("perm-opt-{index}")))
            .mt(px(4.0))
            .pl(px(10.0))
            .pr(px(6.0))
            .py(px(6.0))
            .border_l_2()
            .border_color(if is_selected {
                accent
            } else {
                rgba(0x00000000)
            })
            .cursor_pointer()
            .bg(bg)
            .hover(move |d| d.bg(hover_bg))
            .on_click(move |_event, _window, cx| {
                if let Some(entity) = view.upgrade() {
                    entity.update(cx, |this, cx| {
                        this.permission_index = index;
                        this.approve_permission(Some(option_id.clone()), cx);
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(option.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .opacity(0.34)
                            .child(format!("{}", index + 1)),
                    ),
            )
            .child(div().pt(px(2.0)).text_xs().opacity(0.42).child(caption))
            .into_any_element()
    }

    fn render_permission_inline_card(
        request: &AcpApprovalRequest,
        selected_index: usize,
        options_open: bool,
        view: WeakEntity<AcpChatView>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let preview = request.preview.clone();
        let selected_index = selected_index.min(request.options.len().saturating_sub(1));
        let show_options_button = request.options.len() > 2
            || request
                .options
                .iter()
                .any(|option| option.is_persistent_allow());
        let selected_option_label = request
            .options
            .get(selected_index)
            .map(|option| option.name.clone())
            .unwrap_or_else(|| "Options".to_string());
        let shortcut_hint = if show_options_button {
            "\u{2318}Y Allow \u{00b7} \u{2318}\u{2325}A Options \u{00b7} \u{2318}\u{2325}Z Deny"
        } else {
            "\u{2318}Y Allow \u{00b7} \u{2318}\u{2325}Z Deny \u{00b7} Esc Cancel"
        };

        let accent = preview
            .as_ref()
            .map(|preview| match preview.kind {
                AcpApprovalPreviewKind::Read => rgba((theme.colors.ui.border << 8) | 0x44),
                AcpApprovalPreviewKind::Write => rgb(theme.colors.accent.selected),
                AcpApprovalPreviewKind::Execute => rgba(0xF59E0BCC),
                AcpApprovalPreviewKind::Generic => rgba((theme.colors.ui.border << 8) | 0x32),
            })
            .unwrap_or_else(|| rgb(theme.colors.accent.selected));

        let allow_request = request.clone();
        let allow_view = view.clone();
        let deny_request = request.clone();
        let deny_view = view.clone();
        let options_request = request.clone();
        let options_view = view.clone();

        div()
            .id("acp-inline-permission-card")
            .w_full()
            .mt(px(6.0))
            .ml(px(12.0))
            .pl(px(10.0))
            .pr(px(8.0))
            .py(px(8.0))
            .border_l_2()
            .border_color(accent)
            .bg(rgba((theme.colors.text.primary << 8) | 0x04))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.48)
                    .child(request.title.clone()),
            )
            .when_some(preview.clone(), |d, preview| {
                d.child(Self::render_permission_header(&preview))
                    .when_some(preview.summary, |d, summary| {
                        d.child(div().pt(px(6.0)).text_sm().opacity(0.72).child(summary))
                    })
                    .when_some(preview.input_preview, |d, input| {
                        d.child(Self::render_permission_section("Input", input))
                    })
                    .when_some(preview.output_preview, |d, output| {
                        d.child(Self::render_permission_section("Output", output))
                    })
            })
            .when(preview.is_none(), |d| {
                d.child(
                    div()
                        .pt(px(6.0))
                        .text_sm()
                        .opacity(0.72)
                        .child(request.body.clone()),
                )
            })
            .child(
                div()
                    .pt(px(8.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("acp-inline-permission-allow")
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .cursor_pointer()
                                    .border_l_2()
                                    .border_color(rgb(theme.colors.accent.selected))
                                    .bg(rgba((theme.colors.accent.selected << 8) | 0x12))
                                    .hover(|d| {
                                        d.bg(rgba((theme.colors.accent.selected << 8) | 0x1C))
                                    })
                                    .on_click(move |_event, _window, cx| {
                                        if let Some(entity) = allow_view.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                let _ = this.approve_preferred_allow_option(
                                                    &allow_request,
                                                    cx,
                                                );
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child("Allow"),
                                            )
                                            .child(
                                                div().text_xs().opacity(0.42).child("\u{2318}Y"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .id("acp-inline-permission-deny")
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .cursor_pointer()
                                    .border_l_2()
                                    .border_color(rgba(0xEF4444AA))
                                    .bg(rgba(0xEF444408))
                                    .hover(|d| d.bg(rgba(0xEF444414)))
                                    .on_click(move |_event, _window, cx| {
                                        if let Some(entity) = deny_view.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                let _ =
                                                    this.approve_reject_option(&deny_request, cx);
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child("Deny"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .opacity(0.42)
                                                    .child("\u{2318}\u{2325}Z"),
                                            ),
                                    ),
                            ),
                    )
                    .when(show_options_button, |d| {
                        d.child(
                            div()
                                .id("acp-inline-permission-options")
                                .px(px(10.0))
                                .py(px(6.0))
                                .cursor_pointer()
                                .border_l_2()
                                .border_color(if options_open {
                                    rgb(theme.colors.accent.selected)
                                } else {
                                    rgba(0x00000000)
                                })
                                .bg(rgba((theme.colors.text.primary << 8) | 0x06))
                                .hover(|this| {
                                    this.bg(rgba((theme.colors.text.primary << 8) | 0x0C))
                                })
                                .on_click(move |_event, _window, cx| {
                                    if let Some(entity) = options_view.upgrade() {
                                        entity.update(cx, |this, cx| {
                                            let _ = this
                                                .toggle_permission_options(&options_request, cx);
                                        });
                                    }
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(selected_option_label.clone()),
                                        )
                                        .child(div().text_xs().opacity(0.42).child(
                                            if options_open {
                                                "\u{2318}\u{2325}A \u{25BE}"
                                            } else {
                                                "\u{2318}\u{2325}A \u{25B8}"
                                            },
                                        )),
                                ),
                        )
                    }),
            )
            .when(options_open && request.options.len() > 1, |d| {
                d.child(
                    div()
                        .pt(px(6.0))
                        .children(request.options.iter().enumerate().map(|(i, option)| {
                            Self::render_permission_option_row(
                                option,
                                i,
                                i == selected_index,
                                view.clone(),
                            )
                        })),
                )
            })
            .child(
                div()
                    .pt(px(8.0))
                    .text_xs()
                    .opacity(0.42)
                    .child(shortcut_hint),
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

    fn render_attach_menu(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .w_full()
            .px(px(8.0))
            .pb(px(4.0))
            .child(
                div()
                    .w_full()
                    .rounded(px(8.0))
                    .bg(rgb(theme.colors.background.search_box))
                    .border_1()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x40))
                    .py(px(4.0))
                    .child(
                        div()
                            .id("attach-paste")
                            .w_full()
                            .px(px(10.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|d| d.bg(rgba((theme.colors.text.primary << 8) | 0x0C)))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                if let Some(clipboard) = cx.read_from_clipboard() {
                                    if let Some(text) = clipboard.text() {
                                        if !text.is_empty() {
                                            this.live_thread().update(cx, |thread, cx| {
                                                thread.input.insert_str(&text);
                                                cx.notify();
                                            });
                                            this.cursor_visible = true;
                                        }
                                    }
                                }
                                this.attach_menu_open = false;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(div().text_sm().child("Paste Clipboard"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .opacity(0.45)
                                            .child("Insert clipboard text at cursor"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("attach-screenshot")
                            .w_full()
                            .px(px(10.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|d| d.bg(rgba((theme.colors.text.primary << 8) | 0x0C)))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                // Insert a hint about the screenshot path
                                this.live_thread().update(cx, |thread, cx| {
                                    thread.input.insert_str("What's on my screen? ");
                                    cx.notify();
                                });
                                this.attach_menu_open = false;
                                this.cursor_visible = true;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(div().text_sm().child("Ask About Screen"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .opacity(0.45)
                                            .child("Screenshot is in context"),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_model_selector(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let thread = self.live_thread().read(cx);
        let models = thread.available_models().to_vec();
        let selected_id = thread.selected_model_id().map(|s| s.to_string());

        div()
            .absolute()
            .bottom(px(
                crate::window_resize::mini_layout::HINT_STRIP_HEIGHT + 4.0
            ))
            .left(px(8.0))
            .w(px(200.0))
            .rounded(px(8.0))
            .bg(rgb(theme.colors.background.main))
            .border_1()
            .border_color(rgba((theme.colors.ui.border << 8) | 0x40))
            .py(px(4.0))
            .shadow_md()
            .children(models.into_iter().enumerate().map(|(idx, model)| {
                let model_id = model.id.clone();
                let display = model
                    .display_name
                    .clone()
                    .unwrap_or_else(|| model.id.clone());
                let is_selected = selected_id.as_deref() == Some(model_id.as_str());
                let accent = theme.colors.accent.selected;
                let text_primary = theme.colors.text.primary;

                div()
                    .id(SharedString::from(format!("model-{idx}")))
                    .w_full()
                    .px(px(10.0))
                    .py(px(5.0))
                    .cursor_pointer()
                    .rounded(px(4.0))
                    .mx(px(4.0))
                    .hover(|d| d.bg(rgba((text_primary << 8) | 0x0C)))
                    .when(is_selected, |d| d.bg(rgba((accent << 8) | 0x10)))
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.live_thread().update(cx, |thread, cx| {
                            thread.select_model(&model_id, cx);
                        });
                        this.model_selector_open = false;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .when(is_selected, |d| d.text_color(rgb(accent)))
                                    .child(display),
                            )
                            .when(is_selected, |d| {
                                d.child(div().text_xs().text_color(rgb(accent)).child("\u{2713}"))
                            }),
                    )
            }))
            .into_any_element()
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let is_streaming = matches!(
            self.live_thread().read(cx).status,
            AcpThreadStatus::Streaming
        );

        // Hint strip opacity: match main menu's OPACITY_TEXT_MUTED (0.65)
        let hint_text_hex = theme.colors.text.primary;
        let hint_opacity_byte = (crate::theme::opacity::OPACITY_TEXT_MUTED * 255.0).round() as u32;
        let hint_text_rgba = (hint_text_hex << 8) | hint_opacity_byte;

        div()
            .w_full()
            .h(px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT))
            .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
            .py(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_Y))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            // Subtle top border to separate hint strip from content
            .border_t(px(1.0))
            .border_color(rgba((theme.colors.text.primary << 8) | 0x10))
            // ── Left: streaming dot + model selector ─────
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .when(is_streaming, |d| {
                        let accent = rgb(theme.colors.accent.selected);
                        let pulse_duration = Duration::from_millis(1200);
                        d.child(
                            div()
                                .id("acp-streaming-dot")
                                .size(px(6.0))
                                .rounded_full()
                                .bg(accent)
                                .with_animation(
                                    "acp-streaming-dot-pulse",
                                    Animation::new(pulse_duration).repeat(),
                                    move |el, delta| {
                                        let sine = (delta * std::f32::consts::PI * 2.0).sin();
                                        let a = 0.5 + 0.5 * sine;
                                        el.bg(gpui::Rgba {
                                            r: accent.r,
                                            g: accent.g,
                                            b: accent.b,
                                            a,
                                        })
                                    },
                                ),
                        )
                    })
                    // Model selector button
                    .child({
                        let model_display = self
                            .live_thread()
                            .read(cx)
                            .selected_model_display()
                            .to_string();
                        let is_open = self.model_selector_open;
                        let chevron = if is_open { "\u{25B4}" } else { "\u{25BE}" }; // ▴ / ▾
                        div()
                            .id("acp-model-btn")
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .cursor_pointer()
                            .text_xs()
                            .text_color(if is_open {
                                rgb(theme.colors.accent.selected)
                            } else {
                                rgba(hint_text_rgba)
                            })
                            .hover(|d| d.text_color(rgb(theme.colors.text.primary)))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.model_selector_open = !this.model_selector_open;
                                // Close other menus
                                this.attach_menu_open = false;
                                this.mention_session = None;
                                this.history_menu = None;
                                cx.notify();
                            }))
                            .child(model_display)
                            .child(chevron)
                    }),
            )
            // ── Right: hint strip (matches main menu format) ─────
            .child(crate::components::render_hint_icons(
                &["↩ Send", "⌘K Actions", "⌘W Close"],
                hint_text_rgba,
            ))
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
                    let _ = this
                        .live_thread()
                        .update(cx, |thread, cx| thread.submit_input(cx));
                }));
        } else if is_streaming {
            btn = btn
                .cursor_pointer()
                .on_click(cx.listener(|this, _event, _window, cx| {
                    this.live_thread()
                        .update(cx, |thread, cx| thread.cancel_streaming(cx));
                }));
        }

        btn.child(icon_char).into_any_element()
    }

    // ── @-mention picker ──────────────────────────────────────────

    /// Maximum visible rows in the mention picker.
    pub(super) const MENTION_PICKER_MAX_VISIBLE: usize = 8;

    /// Detect an active `@query` from the input text and cursor position.
    ///
    /// Returns the character range of `@query` and the query string, or `None`
    /// if the cursor is not inside a valid mention trigger.
    /// Find an active trigger (`@` or `/`) before the cursor.
    ///
    /// Returns `(trigger, char_range, query_text)` when the cursor is
    /// immediately after an in-progress `@query` or `/query`.
    fn find_active_trigger(
        text: &str,
        cursor: usize,
    ) -> Option<(ContextPickerTrigger, std::ops::Range<usize>, String)> {
        crate::ai::window::context_picker::extract_context_picker_query_before_cursor(text, cursor)
            .map(|m| (m.trigger, m.char_range, m.query))
    }

    /// Re-derive the mention session from current input state.
    ///
    /// Called after every input mutation and cursor movement.
    fn refresh_mention_session(&mut self, cx: &mut Context<Self>) {
        let (text, cursor, available_commands) = {
            let thread = self.live_thread().read(cx);
            (
                thread.input.text().to_string(),
                thread.input.cursor(),
                thread.available_commands().to_vec(),
            )
        };

        let previous_index = self
            .mention_session
            .as_ref()
            .map(|s| s.selected_index)
            .unwrap_or(0);

        let next_session = match Self::find_active_trigger(&text, cursor) {
            Some((trigger, trigger_range, query)) => {
                let items = match trigger {
                    ContextPickerTrigger::Mention => build_picker_items(trigger, &query),
                    ContextPickerTrigger::Slash => {
                        if available_commands.is_empty() {
                            build_slash_picker_items(
                                &query,
                                self.cached_slash_commands
                                    .iter()
                                    .map(|(name, _)| name.as_str()),
                            )
                        } else {
                            build_slash_picker_items(
                                &query,
                                available_commands.iter().map(String::as_str),
                            )
                        }
                    }
                };

                let selected_index = if items.is_empty() {
                    0
                } else {
                    previous_index.min(items.len().saturating_sub(1))
                };
                let visible = Self::mention_visible_range_for(selected_index, items.len());
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_mention_picker_refreshed",
                    layout = "dense_monoline_shared",
                    ?trigger,
                    query = %query,
                    item_count = items.len(),
                    selected_index,
                    live_command_count = available_commands.len(),
                    anchor_char = trigger_range.start,
                    visible_start = visible.start,
                    visible_end = visible.end,
                );
                Some(AcpMentionSession {
                    trigger,
                    trigger_range,
                    selected_index,
                    items,
                })
            }
            None => None,
        };

        if next_session.is_some() {
            self.last_accepted_item = None;
        }

        self.mention_session = next_session;
        self.log_mention_visible_range("refresh");
        cx.notify();
    }

    /// Log the visible window range for observability.
    fn log_mention_visible_range(&self, reason: &'static str) {
        let Some(session) = self.mention_session.as_ref() else {
            return;
        };
        let visible = Self::mention_visible_range(session);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_mention_visible_range",
            reason,
            selected_index = session.selected_index,
            item_count = session.items.len(),
            visible_start = visible.start,
            visible_end = visible.end,
        );
    }

    /// Apply a hint chip token by writing it into the composer and running
    /// it through the normal picker acceptance path.
    fn apply_picker_hint_token(&mut self, token: &str, cx: &mut Context<Self>) {
        self.live_thread().update(cx, |thread, cx| {
            thread.input.set_text(token.to_string());
            cx.notify();
        });
        self.refresh_mention_session(cx);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_picker_hint_applied",
            token,
            has_session = self.mention_session.is_some(),
        );
        if self.mention_session.is_some() {
            self.accept_mention_selection_impl(false, cx);
        } else {
            self.sync_inline_mentions(cx);
            self.mention_session = None;
            cx.notify();
        }
    }

    /// Accept the currently selected picker row.
    ///
    /// Both Enter and Tab autocomplete the focused picker row. Literal slash
    /// commands are inserted into the composer; slash-picked context items
    /// attach a pending context part and remove the typed `/query` token.
    fn accept_mention_selection(&mut self, cx: &mut Context<Self>) {
        self.accept_mention_selection_impl(false, cx);
    }

    /// Replace a char-range in the given text with `replacement`.
    fn replace_text_in_char_range(
        text: &str,
        char_range: std::ops::Range<usize>,
        replacement: &str,
    ) -> String {
        let start_byte = Self::char_to_byte_offset(text, char_range.start);
        let end_byte = Self::char_to_byte_offset(text, char_range.end);
        let mut out =
            String::with_capacity(text.len() - (end_byte - start_byte) + replacement.len());
        out.push_str(&text[..start_byte]);
        out.push_str(replacement);
        out.push_str(&text[end_byte..]);
        out
    }

    /// Return the caret position immediately after replacing `char_range`
    /// with `replacement`.
    fn caret_after_replacement(char_range: &std::ops::Range<usize>, replacement: &str) -> usize {
        char_range.start + replacement.chars().count()
    }

    /// Accept the currently selected picker row, optionally submitting literal
    /// slash commands after insertion.
    ///
    /// `submit` only applies to literal slash commands such as `/compact`.
    /// Context attachments picked from slash mode never auto-submit.
    fn accept_mention_selection_impl(&mut self, submit: bool, cx: &mut Context<Self>) {
        use crate::ai::context_mentions::part_to_inline_token;

        let session = match self.mention_session.take() {
            Some(s) => s,
            None => return,
        };
        let item = match session.items.get(session.selected_index).cloned() {
            Some(i) => i,
            None => return,
        };

        let trigger_str = match session.trigger {
            ContextPickerTrigger::Mention => "@",
            ContextPickerTrigger::Slash => "/",
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_picker_item_accepted",
            trigger = ?session.trigger,
            submit,
            item_id = %item.id,
            item_label = %item.label,
        );

        // Record accepted item for telemetry / getAcpState queries.
        // cursor_after is set to 0 here and updated after insertion below.
        self.last_accepted_item = Some(crate::protocol::AcpAcceptedItem {
            label: item.label.to_string(),
            id: item.id.to_string(),
            trigger: trigger_str.to_string(),
            cursor_after: 0, // Updated after insertion.
        });

        // ── Literal ACP slash commands stay in the composer as `/command ` text ──
        if session.trigger == ContextPickerTrigger::Slash {
            if let ContextPickerItemKind::SlashCommand(command) = &item.kind {
                let current_text = self.live_thread().read(cx).input.text().to_string();
                let command_text = format!("/{command} ");
                let next_text = Self::replace_text_in_char_range(
                    &current_text,
                    session.trigger_range.clone(),
                    &command_text,
                );
                let next_cursor =
                    Self::caret_after_replacement(&session.trigger_range, &command_text);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_picker_literal_slash_inserted",
                    command = %command,
                    submit,
                );
                if let Some(ref mut accepted) = self.last_accepted_item {
                    accepted.cursor_after = next_cursor;
                }
                self.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text(next_text);
                    thread.input.set_cursor(next_cursor);
                    if submit {
                        let _ = thread.submit_input(cx);
                    } else {
                        cx.notify();
                    }
                });
                cx.notify();
                return;
            }
        }

        // ── Build context part; decide if inline-mention sync applies ──
        let (part, inline_text, allow_inline_sync) = match &item.kind {
            ContextPickerItemKind::BuiltIn(kind) => (
                kind.part(),
                kind.spec().mention.unwrap_or("@snapshot").to_string(),
                session.trigger == ContextPickerTrigger::Mention,
            ),
            ContextPickerItemKind::File(path) | ContextPickerItemKind::Folder(path) => {
                let path_text = path.to_string_lossy().to_string();
                let file_part = AiContextPart::FilePath {
                    path: path_text.clone(),
                    label: item.label.to_string(),
                };
                let inline_text = crate::ai::context_mentions::part_to_inline_token(&file_part)
                    .unwrap_or_else(|| format!("@file:{path_text}"));
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_inline_file_token_inserted",
                    path = %path_text,
                    inline_text = %inline_text,
                );
                (
                    file_part,
                    inline_text,
                    session.trigger == ContextPickerTrigger::Mention,
                )
            }
            ContextPickerItemKind::SlashCommand(_) => return,
        };

        let current_text = self.live_thread().read(cx).input.text().to_string();

        // For @-mention triggers: replace trigger+query with the inline
        // mention text and run inline sync.
        // Slash mode is command-only, so built-in context items should not
        // normally reach this path from `/`.
        let replacement = if allow_inline_sync {
            format!("{inline_text} ")
        } else {
            String::new()
        };
        let next_cursor = Self::caret_after_replacement(&session.trigger_range, &replacement);

        if let Some(ref mut accepted) = self.last_accepted_item {
            accepted.cursor_after = next_cursor;
        }

        let next_text = Self::replace_text_in_char_range(
            &current_text,
            session.trigger_range.clone(),
            &replacement,
        );

        self.live_thread().update(cx, |thread, cx| {
            thread.input.set_text(next_text);
            thread.input.set_cursor(next_cursor);
            thread.add_context_part(part.clone(), cx);
            cx.notify();
        });

        if allow_inline_sync {
            if let Some(token) = part_to_inline_token(&part) {
                self.inline_owned_context_tokens.insert(token);
            }
            self.sync_inline_mentions(cx);
        } else {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_picker_context_attached_from_slash",
                item_id = %item.id,
                item_label = %item.label,
                source = %part.source(),
            );
            cx.notify();
        }
    }

    /// Return highlight ranges for inline `@mentions` that are **actually
    /// attached** as pending context parts. Unattached lookalike tokens are
    /// not highlighted.
    fn attached_inline_mention_highlight_ranges(
        text: &str,
        attached_parts: &[AiContextPart],
        accent_color: u32,
    ) -> Vec<TextHighlightRange> {
        use crate::ai::context_mentions::{parse_inline_context_mentions, part_to_inline_token};

        let attached_tokens: HashSet<String> = attached_parts
            .iter()
            .filter_map(part_to_inline_token)
            .collect();

        parse_inline_context_mentions(text)
            .into_iter()
            .filter(|mention| attached_tokens.contains(&mention.canonical_token))
            .map(|mention| TextHighlightRange {
                start: mention.range.start,
                end: mention.range.end,
                color: accent_color,
            })
            .collect()
    }

    /// Synchronise `pending_context_parts` from the live inline `@mention`
    /// tokens. Removes stale parts whose token was deleted from the input
    /// and adds new parts for freshly typed tokens.
    fn sync_inline_mentions(&mut self, cx: &mut Context<Self>) {
        use crate::ai::context_mentions::{parse_inline_context_mentions, part_to_inline_token};

        let text = self.live_thread().read(cx).input.text().to_string();
        let parsed = parse_inline_context_mentions(&text);

        // Use canonical tokens for dedup and ownership tracking.
        let desired_tokens: HashSet<String> =
            parsed.iter().map(|m| m.canonical_token.clone()).collect();
        let mut new_inline_owned_tokens = HashSet::new();
        let mut removed_tokens = Vec::new();

        self.live_thread().update(cx, |thread, cx| {
            // Remove stale inline parts (iterate in reverse to keep indices stable).
            let stale_indices: Vec<usize> = thread
                .pending_context_parts()
                .iter()
                .enumerate()
                .filter_map(|(ix, part)| {
                    let token = part_to_inline_token(part)?;
                    (self.inline_owned_context_tokens.contains(&token)
                        && !desired_tokens.contains(&token))
                    .then_some(ix)
                })
                .collect();
            for ix in stale_indices.into_iter().rev() {
                if let Some(token) = thread
                    .pending_context_parts()
                    .get(ix)
                    .and_then(part_to_inline_token)
                {
                    removed_tokens.push(token);
                }
                thread.remove_context_part(ix, cx);
            }

            // Add new parts that aren't already attached.
            let existing_tokens: HashSet<String> = thread
                .pending_context_parts()
                .iter()
                .filter_map(part_to_inline_token)
                .collect();
            for mention in &parsed {
                if !existing_tokens.contains(&mention.canonical_token) {
                    thread.add_context_part(mention.part.clone(), cx);
                    new_inline_owned_tokens.insert(mention.canonical_token.clone());
                }
            }
        });

        let added_tokens: Vec<String> = new_inline_owned_tokens.iter().cloned().collect();

        self.inline_owned_context_tokens
            .retain(|token| desired_tokens.contains(token));
        self.inline_owned_context_tokens
            .extend(added_tokens.iter().cloned());

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_inline_mentions_synced",
            inline_count = parsed.len(),
            canonical_count = desired_tokens.len(),
            added_count = added_tokens.len(),
            removed_count = removed_tokens.len(),
            added_tokens = ?added_tokens,
            removed_tokens = ?removed_tokens,
            text_len = text.len(),
        );
    }

    /// Fixed gold tint for accepted inline `@mentions`.
    /// Matches the picker bar and fuzzy highlight accent.
    const ACP_MENTION_INLINE_GOLD: u32 = 0xFBBF24;

    /// Fixed picker dropdown width.
    const ACP_MENTION_PICKER_WIDTH: f32 = 320.0;

    /// Minimum usable picker width when the window is narrow.
    const ACP_MENTION_PICKER_MIN_WIDTH: f32 = 200.0;

    /// Horizontal padding used by the ACP composer input row.
    const ACP_INPUT_PADDING_X: f32 = 12.0;

    /// Keep the picker inset from the right edge so it never clips.
    const ACP_MENTION_PICKER_EDGE_GUTTER: f32 = 12.0;

    /// Top padding used by the ACP composer input row.
    const ACP_INPUT_PADDING_Y: f32 = 10.0;

    /// Effective visual line height of the ACP composer text.
    const ACP_INPUT_LINE_HEIGHT: f32 = 22.0;

    /// Gap between the active mention line and the picker.
    const ACP_MENTION_PICKER_OFFSET_Y: f32 = 4.0;

    /// Composer text size used for the inline ACP input.
    const ACP_INPUT_FONT_SIZE: f32 = 17.0;

    fn acp_input_max_visible_chars(&self, window: &Window) -> usize {
        const ACP_INPUT_MIN_VISIBLE_CHARS: usize = 18;
        const ACP_INPUT_MAX_VISIBLE_CHARS: usize = 96;
        const ACP_INPUT_APPROX_CHAR_WIDTH_PX: f32 = 8.5;
        const ACP_INPUT_HORIZONTAL_PADDING_PX: f32 = 48.0;

        let window_width = window.window_bounds().get_bounds().size.width.as_f32();
        let usable_width = (window_width - ACP_INPUT_HORIZONTAL_PADDING_PX).max(160.0);
        let visible_chars = (usable_width / ACP_INPUT_APPROX_CHAR_WIDTH_PX).floor() as usize;
        visible_chars.clamp(ACP_INPUT_MIN_VISIBLE_CHARS, ACP_INPUT_MAX_VISIBLE_CHARS)
    }

    fn mention_picker_width_for_window(window_width: f32) -> f32 {
        let max_width = (window_width - (Self::ACP_MENTION_PICKER_EDGE_GUTTER * 2.0))
            .min(Self::ACP_MENTION_PICKER_WIDTH);
        max_width.max(Self::ACP_MENTION_PICKER_MIN_WIDTH)
    }

    fn clamp_mention_picker_left(anchor_left: f32, picker_width: f32, window_width: f32) -> f32 {
        let min_left = Self::ACP_INPUT_PADDING_X;
        let max_left =
            (window_width - picker_width - Self::ACP_MENTION_PICKER_EDGE_GUTTER).max(min_left);
        anchor_left.clamp(min_left, max_left)
    }

    fn measure_acp_input_prefix_width(window: &Window, prefix: &str) -> f32 {
        if prefix.is_empty() {
            return 0.0;
        }

        let style = gpui::TextStyle {
            font_size: px(Self::ACP_INPUT_FONT_SIZE).into(),
            ..Default::default()
        };
        let run = style.to_run(prefix.len());
        window
            .text_system()
            .layout_line(prefix, px(Self::ACP_INPUT_FONT_SIZE), &[run], None)
            .width
            .as_f32()
    }

    /// Returns the maximum text wrapping width for the ACP composer.
    fn composer_wrap_width_for_window(window_width: f32) -> f32 {
        (window_width - (Self::ACP_INPUT_PADDING_X * 2.0)).max(1.0)
    }

    /// Returns the ACP composer cursor position `(x, y)` after rendering `text`,
    /// accounting for explicit newlines and simple visual wrapping.
    fn measure_acp_input_cursor_position(window: &Window, text: &str) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, 0.0);
        }
        let window_width = window.window_bounds().get_bounds().size.width.as_f32();
        let wrap_width = Self::composer_wrap_width_for_window(window_width);
        let logical_lines: Vec<&str> = text.split('\n').collect();
        let mut visual_row = 0usize;
        let mut cursor_x = 0.0f32;
        for (ix, logical_line) in logical_lines.iter().enumerate() {
            let width = Self::measure_acp_input_prefix_width(window, logical_line);
            let wraps = if logical_line.is_empty() {
                1usize
            } else {
                (width / wrap_width).floor() as usize + 1
            };
            if ix + 1 == logical_lines.len() {
                visual_row += wraps.saturating_sub(1);
                cursor_x = if logical_line.is_empty() {
                    0.0
                } else {
                    width % wrap_width
                };
            } else {
                visual_row += wraps;
            }
        }
        (cursor_x, visual_row as f32 * Self::ACP_INPUT_LINE_HEIGHT)
    }

    /// Returns `(left, top, width)` for the mention picker, anchored to the
    /// trigger character position in the ACP composer, including wrapping.
    fn mention_picker_anchor_for_session(
        &self,
        session: &AcpMentionSession,
        input_text: &str,
        window: &Window,
    ) -> (f32, f32, f32) {
        let window_width = window.window_bounds().get_bounds().size.width.as_f32();
        let picker_width = Self::mention_picker_width_for_window(window_width);
        let trigger_start_byte = Self::char_to_byte_offset(input_text, session.trigger_range.start);
        let prefix = &input_text[..trigger_start_byte];
        let trigger_text = match session.trigger {
            ContextPickerTrigger::Mention => "@",
            ContextPickerTrigger::Slash => "/",
        };
        let trigger_width = Self::measure_acp_input_prefix_width(window, trigger_text);
        let (after_trigger_x, after_trigger_y) =
            Self::measure_acp_input_cursor_position(window, &format!("{prefix}{trigger_text}"));
        let unclamped_left = Self::ACP_INPUT_PADDING_X + (after_trigger_x - trigger_width).max(0.0);
        let left = Self::clamp_mention_picker_left(unclamped_left, picker_width, window_width);
        let top = Self::ACP_INPUT_PADDING_Y
            + after_trigger_y
            + Self::ACP_INPUT_LINE_HEIGHT
            + Self::ACP_MENTION_PICKER_OFFSET_Y;
        (left, top, picker_width)
    }

    /// Compute the visible range of items around a selected index.
    pub(super) fn mention_visible_range_for(
        selected_index: usize,
        item_count: usize,
    ) -> std::ops::Range<usize> {
        let max_visible = Self::MENTION_PICKER_MAX_VISIBLE;
        if item_count <= max_visible {
            return 0..item_count;
        }
        let half = max_visible / 2;
        let mut start = selected_index.saturating_sub(half);
        let max_start = item_count.saturating_sub(max_visible);
        if start > max_start {
            start = max_start;
        }
        start..(start + max_visible).min(item_count)
    }

    /// Compute the visible range of items around the selected index.
    fn mention_visible_range(session: &AcpMentionSession) -> std::ops::Range<usize> {
        Self::mention_visible_range_for(session.selected_index, session.items.len())
    }

    /// Render the mention picker dropdown using the shared dense-monoline
    /// row contract (`context_picker_row`).
    fn render_mention_picker(
        &self,
        session: &AcpMentionSession,
        width: f32,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        use crate::ai::context_picker_row::render_dense_monoline_picker_row;

        let theme = theme::get_cached_theme();
        let visible = Self::mention_visible_range(session);
        let fg: gpui::Hsla = rgb(theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = rgb(theme.colors.text.muted).into();

        div()
            .id("acp-mention-picker")
            .w(px(width))
            // Vibrancy-friendly: near-transparent bg, no border/rounded
            .bg(fg.opacity(0.02))
            .py(px(2.0))
            .children(
                session
                    .items
                    .iter()
                    .enumerate()
                    .skip(visible.start)
                    .take(visible.len())
                    .map(|(idx, item)| {
                        let is_selected = idx == session.selected_index;
                        render_dense_monoline_picker_row(
                            SharedString::from(format!("acp-mention-row-{idx}")),
                            item.label.clone(),
                            item.meta.clone(),
                            &item.label_highlight_indices,
                            &item.meta_highlight_indices,
                            is_selected,
                            fg,
                            muted_fg,
                        )
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            if let Some(session) = this.mention_session.as_mut() {
                                session.selected_index = idx;
                            }
                            this.accept_mention_selection(cx);
                        }))
                        .into_any_element()
                    }),
            )
            .into_any_element()
    }

    /// Render empty state for the unified picker with clickable hint chips.
    /// Shows trigger-appropriate hints based on whether `@` or `/` was typed.
    fn render_mention_empty_state(&self, width: f32, cx: &mut Context<Self>) -> gpui::AnyElement {
        use crate::ai::context_picker_row::{GHOST, HINT, MUTED_OP};
        use crate::list_item::FONT_MONO;

        let cached_theme = theme::get_cached_theme();
        let fg: gpui::Hsla = rgb(cached_theme.colors.text.primary).into();
        let muted_fg: gpui::Hsla = rgb(cached_theme.colors.text.muted).into();
        let trigger = self
            .mention_session
            .as_ref()
            .map(|s| s.trigger)
            .unwrap_or(ContextPickerTrigger::Mention);
        let is_slash = trigger == ContextPickerTrigger::Slash;
        let hints = crate::ai::window::context_picker::empty_state_hints(trigger);

        let mut chips: Vec<gpui::AnyElement> = Vec::new();
        for hint in hints {
            let hint_display = SharedString::from(hint.display);
            let hint_insertion = hint.insertion.to_string();
            let close_after_apply = !hint.insertion.ends_with(':');
            chips.push(
                div()
                    .id(SharedString::from(format!("mention-hint-{}", hint.display)))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .bg(fg.opacity(GHOST))
                    .hover(|el| el.bg(fg.opacity(0.08)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_mention_empty_hint_applied",
                            display = %hint_display,
                            insertion = %hint_insertion,
                        );
                        if close_after_apply {
                            this.apply_picker_hint_token(&hint_insertion, cx);
                        } else {
                            let live_t = this.live_thread().clone();
                            live_t.update(cx, |thread, cx| {
                                thread.input.set_text(hint_insertion.clone());
                                cx.notify();
                            });
                            this.refresh_mention_session(cx);
                            this.sync_inline_mentions(cx);
                        }
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(muted_fg.opacity(HINT))
                            .child(SharedString::from(hint.display)),
                    )
                    .into_any_element(),
            );
        }

        div()
            .id("acp-mention-empty-state")
            .w(px(width))
            .bg(fg.opacity(0.02))
            .py(px(4.0))
            .px(px(6.0))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(muted_fg.opacity(MUTED_OP))
                    .child(if is_slash {
                        "No matching commands"
                    } else {
                        "No matching context"
                    }),
            )
            .child(div().flex().items_center().gap(px(4.0)).children(chips))
            .into_any_element()
    }

    // ── Slash command helpers ─────────────────────────────────────

    /// Known Claude Code slash commands (used when the agent doesn't send
    /// an AvailableCommandsUpdate notification).
    const DEFAULT_SLASH_COMMANDS: &'static [&'static str] = &[
        "compact", "clear", "bug", "help", "init", "login", "logout", "status", "cost", "doctor",
        "review", "memory",
    ];

    // ── Key handling ──────────────────────────────────────────────

    /// Render the inline setup card for setup mode.
    fn render_setup_card(
        &self,
        state: &super::setup_state::AcpInlineSetupState,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let action_hint: String = match state.primary_action {
            super::setup_state::AcpSetupAction::Retry => "Press Tab to retry".to_string(),
            super::setup_state::AcpSetupAction::Install => {
                "Install the agent, then press Tab to retry".to_string()
            }
            super::setup_state::AcpSetupAction::Authenticate => {
                "Authenticate, then press Tab to retry".to_string()
            }
            super::setup_state::AcpSetupAction::OpenCatalog => {
                "Edit ~/.scriptkit/acp/agents.json, then press Tab to retry".to_string()
            }
            super::setup_state::AcpSetupAction::SelectAgent => {
                "Press Enter to select a different agent".to_string()
            }
        };

        let secondary_hint: Option<String> = state.secondary_action.map(|action| match action {
            super::setup_state::AcpSetupAction::SelectAgent => "Enter: select agent".to_string(),
            super::setup_state::AcpSetupAction::Retry => "Tab: retry".to_string(),
            super::setup_state::AcpSetupAction::OpenCatalog => "Edit agents.json".to_string(),
            _ => String::new(),
        });

        let agent_name: Option<String> = state
            .selected_agent
            .as_ref()
            .map(|a| format!("Selected: {}", a.display_name));

        div()
            .id("acp-inline-setup")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.0))
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(theme.colors.text.primary))
                    .child(state.title.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(theme.colors.text.muted))
                    .max_w(px(400.0))
                    .text_center()
                    .child(state.body.clone()),
            )
            .when_some(agent_name, |d, name| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(name),
                )
            })
            // Show the agent picker when open.
            .when_some(self.render_setup_agent_picker_inline(), |d, picker| {
                d.child(picker)
            })
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme.colors.text.muted))
                    .child(action_hint),
            )
            .when_some(secondary_hint.filter(|s| !s.is_empty()), |d, hint| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .opacity(0.6)
                        .child(hint),
                )
            })
            .into_any_element()
    }

    /// Render the setup agent picker inline (non-mut version for use in render).
    fn render_setup_agent_picker_inline(&self) -> Option<gpui::AnyElement> {
        let picker = self.setup_agent_picker.as_ref()?;
        let theme = theme::get_cached_theme();

        let rows: Vec<gpui::AnyElement> = picker
            .items
            .iter()
            .enumerate()
            .map(|(ix, item)| {
                let is_selected = ix == picker.selected_index;
                let status_text: String = format!(
                    "{:?} \u{00b7} {:?} \u{00b7} {:?}",
                    item.source, item.install_state, item.config_state,
                );
                div()
                    .id(gpui::ElementId::Name(
                        format!("acp-setup-agent-{ix}").into(),
                    ))
                    .w_full()
                    .px(px(10.0))
                    .py(px(5.0))
                    .when(is_selected, |d| {
                        d.bg(rgba((theme.colors.accent.selected << 8) | 0x14))
                            .border_l_2()
                            .border_color(rgb(theme.colors.accent.selected))
                    })
                    .when(!is_selected, |d| {
                        d.border_l_2().border_color(gpui::transparent_black())
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme.colors.text.primary))
                                    .child(item.display_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.muted))
                                    .child(status_text),
                            ),
                    )
                    .into_any_element()
            })
            .collect();

        Some(
            div()
                .id("acp-setup-agent-picker-container")
                .w_full()
                .max_w(px(400.0))
                .max_h(px(220.0))
                .overflow_y_scroll()
                .rounded(px(8.0))
                .bg(rgb(theme.colors.background.search_box))
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x40))
                .py(px(4.0))
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(4.0))
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child("Select an ACP agent"),
                )
                .children(rows)
                .child(
                    div()
                        .w_full()
                        .px(px(10.0))
                        .pt(px(6.0))
                        .pb(px(4.0))
                        .border_t_1()
                        .border_color(rgba((theme.colors.ui.border << 8) | 0x15))
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(
                            "\u{2191}\u{2193} navigate \u{00b7} Enter select \u{00b7} Esc close",
                        ),
                )
                .into_any_element(),
        )
    }

    /// Whether an active setup card is showing (initial or runtime recovery).
    fn has_active_setup(&self, cx: &mut Context<Self>) -> bool {
        match &self.session {
            AcpChatSession::Setup(_) => true,
            AcpChatSession::Live(thread) => thread.read(cx).setup_state().is_some(),
        }
    }

    /// Read the active setup state from either session mode.
    fn read_active_setup_state(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<super::setup_state::AcpInlineSetupState> {
        match &self.session {
            AcpChatSession::Setup(setup) => Some((**setup).clone()),
            AcpChatSession::Live(thread) => thread.read(cx).setup_state().cloned(),
        }
    }

    /// Replace the active setup state in whichever session mode is current.
    fn replace_active_setup_state(
        &mut self,
        next: super::setup_state::AcpInlineSetupState,
        cx: &mut Context<Self>,
    ) {
        match &mut self.session {
            AcpChatSession::Setup(setup) => {
                **setup = next;
                cx.notify();
            }
            AcpChatSession::Live(thread) => {
                thread.update(cx, |thread, cx| {
                    thread.replace_setup_state(next, cx);
                });
            }
        }
    }

    /// Open the agent selection picker overlay (works in both initial setup
    /// and runtime recovery).
    fn open_setup_agent_picker(&mut self, cx: &mut Context<Self>) {
        let Some(setup) = self.read_active_setup_state(cx) else {
            return;
        };
        if setup.catalog_entries.is_empty() {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_setup_agent_picker_empty_catalog",
            );
            return;
        }
        let selected_index = setup
            .selected_agent
            .as_ref()
            .and_then(|selected| {
                setup
                    .catalog_entries
                    .iter()
                    .position(|entry| entry.id == selected.id)
            })
            .unwrap_or(0);

        self.setup_agent_picker = Some(AcpSetupAgentPickerState {
            items: setup.catalog_entries.clone(),
            selected_index,
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_agent_picker_opened",
            item_count = self
                .setup_agent_picker
                .as_ref()
                .map(|p| p.items.len())
                .unwrap_or(0),
            selected_index,
        );
        cx.notify();
    }

    /// Confirm the currently highlighted agent in the setup picker,
    /// persist it as the preferred agent, re-resolve the setup card, and
    /// close the picker.
    fn confirm_setup_agent_picker(&mut self, cx: &mut Context<Self>) {
        let Some(picker) = self.setup_agent_picker.take() else {
            return;
        };
        let Some(agent) = picker.items.get(picker.selected_index).cloned() else {
            return;
        };
        let Some(current_setup) = self.read_active_setup_state(cx) else {
            return;
        };

        // Persist selection.
        crate::ai::acp::persist_preferred_acp_agent_id(Some(agent.id.to_string()));

        // Re-resolve against the catalog to rebuild card title/body/actions.
        let resolution = crate::ai::acp::resolve_acp_launch_with_requirements(
            &current_setup.catalog_entries,
            Some(agent.id.as_ref()),
            current_setup.launch_requirements,
        );
        let next_setup = crate::ai::acp::AcpInlineSetupState::from_resolution(
            &resolution,
            current_setup.launch_requirements,
        );

        // Update the live thread's selected agent so recovery state is
        // immediately truthful without waiting for a reopen.
        if let AcpChatSession::Live(thread) = &self.session {
            let next_agent_for_thread = next_setup.selected_agent.clone();
            thread.update(cx, |thread, cx| {
                thread.replace_selected_agent(next_agent_for_thread, cx);
            });
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_agent_confirmed_for_runtime_recovery",
            agent_id = %agent.id,
            display_name = %agent.display_name,
            blocker = ?resolution.blocker,
            needs_embedded_context = current_setup.launch_requirements.needs_embedded_context,
            needs_image = current_setup.launch_requirements.needs_image,
            catalog_count = current_setup.catalog_entries.len(),
        );

        self.replace_active_setup_state(next_setup, cx);
    }

    /// Handle a setup action triggered by the user.
    fn handle_setup_action(
        &mut self,
        action: super::setup_state::AcpSetupAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            super::setup_state::AcpSetupAction::SelectAgent => {
                self.open_setup_agent_picker(cx);
            }
            super::setup_state::AcpSetupAction::Retry => {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_retry_requested",
                );
                // Propagate so the parent (tab_ai_mode) can re-run preflight.
                cx.propagate();
            }
            super::setup_state::AcpSetupAction::OpenCatalog => {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_open_catalog_requested",
                );
            }
            super::setup_state::AcpSetupAction::Install
            | super::setup_state::AcpSetupAction::Authenticate => {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_external_action_requested",
                    action = ?action,
                );
            }
        }
    }

    // ── Test probe methods ────────────────────────────────────

    /// Reset the test probe, clearing all recorded events.
    pub(crate) fn reset_test_probe(&mut self) {
        self.test_probe.event_seq = 0;
        self.test_probe.key_routes.clear();
        self.test_probe.accepted_items.clear();
        self.test_probe.input_layout = None;
        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_test_probe_reset",
        );
    }

    /// Record a key-route event into the test probe ring buffer.
    pub(crate) fn record_key_route(&mut self, event: crate::protocol::AcpKeyRouteTelemetry) {
        self.test_probe.event_seq += 1;
        if self.test_probe.key_routes.len() >= ACP_TEST_PROBE_MAX_EVENTS {
            self.test_probe.key_routes.pop_front();
        }
        self.test_probe.key_routes.push_back(event);
    }

    /// Record a picker-acceptance event into the test probe ring buffer.
    pub(crate) fn record_picker_accept(
        &mut self,
        event: crate::protocol::AcpPickerItemAcceptedTelemetry,
    ) {
        self.test_probe.event_seq += 1;
        if self.test_probe.accepted_items.len() >= ACP_TEST_PROBE_MAX_EVENTS {
            self.test_probe.accepted_items.pop_front();
        }
        self.test_probe.accepted_items.push_back(event);
    }

    /// Record an input-layout event into the test probe.
    pub(crate) fn record_input_layout(&mut self, event: crate::protocol::AcpInputLayoutTelemetry) {
        self.test_probe.event_seq += 1;
        self.test_probe.input_layout = Some(event);
    }

    /// Build a bounded snapshot of the test probe for agent queries.
    pub(crate) fn test_probe_snapshot(
        &self,
        tail: usize,
        cx: &gpui::App,
    ) -> crate::protocol::AcpTestProbeSnapshot {
        use crate::protocol::ACP_TEST_PROBE_SCHEMA_VERSION;

        let key_routes: Vec<_> = self
            .test_probe
            .key_routes
            .iter()
            .rev()
            .take(tail)
            .rev()
            .cloned()
            .collect();
        let accepted_items: Vec<_> = self
            .test_probe
            .accepted_items
            .iter()
            .rev()
            .take(tail)
            .rev()
            .cloned()
            .collect();

        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_test_probe_snapshot_requested",
            tail = tail,
            event_seq = self.test_probe.event_seq,
        );

        crate::protocol::AcpTestProbeSnapshot {
            schema_version: ACP_TEST_PROBE_SCHEMA_VERSION,
            event_seq: self.test_probe.event_seq,
            key_routes,
            accepted_items,
            input_layout: self.test_probe.input_layout.clone(),
            state: self.collect_acp_state_snapshot(cx),
        }
    }

    // ── Telemetry emission ───────────────────────────────────

    /// Emit structured ACP key-routing telemetry.
    ///
    /// Logged on `script_kit::acp_telemetry` target. Contains no user content —
    /// only the key name, route, indices, and booleans.
    fn emit_key_route_telemetry(
        &mut self,
        key: &str,
        route: crate::protocol::AcpKeyRoute,
        cursor_before: usize,
        cursor_after: usize,
        caused_submit: bool,
        consumed: bool,
    ) {
        let picker_open = self.mention_session.is_some();
        // Note: telemetry-only field; we don't have cx here so we report
        // based on best local knowledge. The actual permission state is
        // checked in handle_key_down where cx is available.
        let permission_active = false;
        let telemetry = crate::protocol::AcpKeyRouteTelemetry {
            key: key.to_string(),
            route: route.clone(),
            picker_open,
            permission_active,
            cursor_before,
            cursor_after,
            caused_submit,
            consumed,
        };
        // Record into test probe ring buffer.
        self.record_key_route(telemetry.clone());
        let telemetry_json = serde_json::to_string(&telemetry).unwrap_or_default();
        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_key_routed",
            key = %key,
            route = ?route,
            picker_open,
            permission_active,
            cursor_before,
            cursor_after,
            caused_submit,
            consumed,
            telemetry_json = %telemetry_json,
        );
    }

    /// Emit structured picker-accepted telemetry after a mention/slash item is accepted.
    fn emit_picker_accepted_telemetry(
        &mut self,
        trigger: &str,
        item_label: &str,
        item_id: &str,
        accepted_via_key: &str,
        cursor_after: usize,
        caused_submit: bool,
    ) {
        let telemetry = crate::protocol::AcpPickerItemAcceptedTelemetry {
            trigger: trigger.to_string(),
            item_label: item_label.to_string(),
            item_id: item_id.to_string(),
            accepted_via_key: accepted_via_key.to_string(),
            cursor_after,
            caused_submit,
        };
        // Record into test probe ring buffer.
        self.record_picker_accept(telemetry.clone());
        let telemetry_json = serde_json::to_string(&telemetry).unwrap_or_default();
        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_picker_item_accepted",
            trigger = %trigger,
            item_label = %item_label,
            item_id = %item_id,
            accepted_via_key = %accepted_via_key,
            cursor_after,
            caused_submit,
            telemetry_json = %telemetry_json,
        );
    }

    /// Emit structured input-layout telemetry after a mutation that may shift the visible window.
    fn emit_input_layout_telemetry(&mut self, layout: &crate::protocol::AcpInputLayoutMetrics) {
        let telemetry = crate::protocol::AcpInputLayoutTelemetry {
            char_count: layout.char_count,
            visible_start: layout.visible_start,
            visible_end: layout.visible_end,
            cursor_in_window: layout.cursor_in_window,
        };
        // Record into test probe.
        self.record_input_layout(telemetry.clone());
        let telemetry_json = serde_json::to_string(&telemetry).unwrap_or_default();
        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_input_layout",
            char_count = layout.char_count,
            visible_start = layout.visible_start,
            visible_end = layout.visible_end,
            cursor_in_window = layout.cursor_in_window,
            telemetry_json = %telemetry_json,
        );
    }

    fn handle_key_down(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        // Setup mode (initial or runtime recovery): handle agent picker and setup actions.
        if self.has_active_setup(cx) {
            let key = event.keystroke.key.as_str();

            // Agent picker is open — intercept navigation keys.
            if let Some(ref mut picker) = self.setup_agent_picker {
                if crate::ui_foundation::is_key_up(key) {
                    if picker.selected_index > 0 {
                        picker.selected_index -= 1;
                    }
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
                if crate::ui_foundation::is_key_down(key) {
                    if picker.selected_index + 1 < picker.items.len() {
                        picker.selected_index += 1;
                    }
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
                if crate::ui_foundation::is_key_enter(key) {
                    self.confirm_setup_agent_picker(cx);
                    cx.stop_propagation();
                    return;
                }
                if crate::ui_foundation::is_key_escape(key) {
                    self.setup_agent_picker = None;
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
                // Other keys fall through.
                cx.propagate();
                return;
            }

            // No picker open — handle setup-level keys.
            if crate::ui_foundation::is_key_tab(key) {
                // Tab in setup = retry (re-run preflight with potentially new agent).
                self.handle_setup_action(super::setup_state::AcpSetupAction::Retry, cx);
                return;
            }
            if crate::ui_foundation::is_key_enter(key) {
                // Enter triggers the primary action (works for both Setup and Live recovery).
                let action = match self.read_active_setup_state(cx) {
                    Some(state) => state.primary_action,
                    None => return,
                };
                self.handle_setup_action(action, cx);
                cx.stop_propagation();
                return;
            }

            cx.propagate();
            return;
        }

        // Reset cursor blink on any key press.
        self.cursor_visible = true;

        // ── Inline approval intercept ────────────────────────────
        let pending_permission = self.live_thread().read(cx).pending_permission.clone();
        if let Some(ref request) = pending_permission {
            if self.handle_permission_key_down(event, request, cx) {
                cx.stop_propagation();
                return;
            }
            // Block composer typing while approval is pending, but still allow
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

        // ── Model selector dismiss on Escape ───────────────────
        if self.model_selector_open && crate::ui_foundation::is_key_escape(key) {
            self.model_selector_open = false;
            cx.notify();
            cx.stop_propagation();
            return;
        }
        // Close model selector on any non-modifier key
        if self.model_selector_open {
            self.model_selector_open = false;
            cx.notify();
        }

        // ── Attach menu dismiss on Escape ───────────────────────
        if self.attach_menu_open && crate::ui_foundation::is_key_escape(key) {
            self.attach_menu_open = false;
            cx.notify();
            cx.stop_propagation();
            return;
        }
        // Close attach menu on any non-modifier key
        if self.attach_menu_open {
            self.attach_menu_open = false;
            cx.notify();
        }

        // ── Cmd+F → toggle search ────────────────────────────
        if modifiers.platform && key.eq_ignore_ascii_case("f") {
            if self.search_state.is_some() {
                self.search_state = None;
            } else {
                self.search_state = Some((String::new(), 0));
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // ── Search intercept (when search bar is open) ──────
        let search_messages = if self.search_state.is_some() {
            Some(self.live_thread().read(cx).messages.clone())
        } else {
            None
        };
        if let Some((ref mut query, ref mut match_idx)) = self.search_state {
            if crate::ui_foundation::is_key_escape(key) {
                self.search_state = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_enter(key) {
                // Enter = next match, Shift+Enter = previous match.
                if !query.is_empty() {
                    let ql = query.to_lowercase();
                    if let Some(messages) = search_messages.as_ref() {
                        let match_indices: Vec<usize> = messages
                            .iter()
                            .enumerate()
                            .filter(|(_, m)| m.body.to_lowercase().contains(&ql))
                            .map(|(i, _)| i)
                            .collect();
                        if !match_indices.is_empty() {
                            let total = match_indices.len();
                            if modifiers.shift {
                                // Previous match (wrap backward).
                                *match_idx = (*match_idx + total - 1) % total;
                            } else {
                                // Next match (wrap forward).
                                *match_idx = (*match_idx + 1) % total;
                            }
                            self.list_state
                                .scroll_to_reveal_item(match_indices[*match_idx]);
                        }
                    }
                }
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_backspace(key) {
                query.pop();
                *match_idx = 0;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if let Some(ch) = event.keystroke.key_char.as_deref() {
                if !ch.is_empty() && !modifiers.platform && !modifiers.control {
                    query.push_str(ch);
                    *match_idx = 0;
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        // ── Cmd+K → open actions dialog ──────
        if modifiers.platform && crate::ui_foundation::is_key_k(key) {
            if crate::ai::acp::chat_window::is_chat_window_open() {
                // Detached window: open actions popup directly
                tracing::info!(event = "detached_actions_shortcut_pressed");
                crate::ai::acp::chat_window::toggle_detached_actions(cx);
                cx.stop_propagation();
            } else {
                // Main panel: propagate to parent interceptor
                cx.propagate();
            }
            return;
        }

        // ── Cmd+. → cancel streaming (standard macOS cancel) ──────
        if modifiers.platform && key == "." {
            let is_streaming = matches!(
                self.live_thread().read(cx).status,
                AcpThreadStatus::Streaming
            );
            if is_streaming {
                self.live_thread()
                    .update(cx, |thread, cx| thread.cancel_streaming(cx));
            }
            cx.stop_propagation();
            return;
        }

        // ── Cmd+Up/Down → jump between user turns ──────────────
        if modifiers.platform && crate::ui_foundation::is_key_up(key) {
            let messages = &self.live_thread().read(cx).messages;
            let current_top = self.list_state.logical_scroll_top().item_ix;
            // Find the user message before the current scroll position
            if let Some(target) = messages[..current_top.saturating_sub(1)]
                .iter()
                .rposition(|m| matches!(m.role, AcpThreadMessageRole::User))
            {
                self.list_state.scroll_to_reveal_item(target);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && crate::ui_foundation::is_key_down(key) {
            let messages = &self.live_thread().read(cx).messages;
            let current_top = self.list_state.logical_scroll_top().item_ix;
            // Find the user message after the current scroll position
            let search_start = (current_top + 1).min(messages.len());
            if let Some(offset) = messages[search_start..]
                .iter()
                .position(|m| matches!(m.role, AcpThreadMessageRole::User))
            {
                self.list_state.scroll_to_reveal_item(search_start + offset);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        // ── Cmd+/ → toggle slash command picker ──────────────────
        if modifiers.platform && key == "/" {
            if self
                .mention_session
                .as_ref()
                .is_some_and(|s| s.trigger == ContextPickerTrigger::Slash)
            {
                // Close picker and clear the "/" prefix
                self.mention_session = None;
                self.live_thread().update(cx, |thread, cx| {
                    let text = thread.input.text().to_string();
                    if text.starts_with('/') {
                        thread.input.set_text(String::new());
                    }
                    cx.notify();
                });
            } else {
                // Open picker by inserting "/" into input
                self.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text("/".to_string());
                    cx.notify();
                });
                self.refresh_mention_session(cx);
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // ── Cmd+Shift+C → copy last response to clipboard ──────
        if modifiers.platform && modifiers.shift && key.eq_ignore_ascii_case("c") {
            let last = self
                .live_thread()
                .read(cx)
                .messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, super::thread::AcpThreadMessageRole::Assistant))
                .map(|m| m.body.to_string());
            if let Some(text) = last {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            }
            cx.stop_propagation();
            return;
        }

        // ── Cmd+N / Cmd+L → new conversation (clear messages, keep session) ──
        if modifiers.platform && (key.eq_ignore_ascii_case("n") || key.eq_ignore_ascii_case("l")) {
            self.live_thread().update(cx, |thread, cx| {
                thread.clear_messages(cx);
            });
            self.collapsed_ids.clear();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // ── Cmd+P → toggle conversation history picker ──────────
        if modifiers.platform && key.eq_ignore_ascii_case("p") {
            if self.history_menu.is_some() {
                self.history_menu = None;
            } else {
                let entries = super::history::load_history();
                if !entries.is_empty() {
                    self.history_menu = Some((0, String::new(), entries));
                }
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // ── History picker intercept ─────────────────────────────
        if let Some((ref mut idx, ref mut filter, ref entries)) = self.history_menu {
            // Filter entries by search text
            let filtered: Vec<_> = if filter.is_empty() {
                entries.iter().collect()
            } else {
                let q = filter.to_lowercase();
                entries
                    .iter()
                    .filter(|e| e.first_message.to_lowercase().contains(&q))
                    .collect()
            };
            let count = filtered.len();

            if crate::ui_foundation::is_key_up(key) {
                *idx = idx.saturating_sub(1);
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_down(key) {
                *idx = (*idx + 1).min(count.saturating_sub(1));
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_enter(key) {
                let selected = filtered.get(*idx).cloned().cloned();
                self.history_menu = None;
                if let Some(entry) = selected {
                    // Try to load full conversation; fall back to inserting first message
                    if let Some(conv) = super::history::load_conversation(&entry.session_id) {
                        self.live_thread().update(cx, |thread, cx| {
                            thread.load_saved_messages(&conv.messages, cx);
                        });
                        self.collapsed_ids.clear();
                    } else {
                        self.live_thread().update(cx, |thread, cx| {
                            thread.input.set_text(entry.first_message.clone());
                            cx.notify();
                        });
                    }
                }
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_escape(key) {
                self.history_menu = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_backspace(key) {
                filter.pop();
                *idx = 0;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            // Typed characters filter the list
            if let Some(ch) = event.keystroke.key_char.as_deref() {
                if !ch.is_empty() && !modifiers.platform && !modifiers.control {
                    filter.push_str(ch);
                    *idx = 0;
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        // ── Unified picker intercept (@ mentions + / commands) ────
        if self.mention_session.is_some() {
            if crate::ui_foundation::is_key_up(key) {
                if let Some(session) = self.mention_session.as_mut() {
                    if !session.items.is_empty() {
                        session.selected_index = if session.selected_index == 0 {
                            session.items.len() - 1
                        } else {
                            session.selected_index - 1
                        };
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_mention_selection_changed",
                            direction = "prev",
                            selected_index = session.selected_index,
                            item_count = session.items.len(),
                        );
                    }
                }
                self.log_mention_visible_range("keyboard_prev");
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_down(key) {
                if let Some(session) = self.mention_session.as_mut() {
                    if !session.items.is_empty() {
                        session.selected_index = (session.selected_index + 1) % session.items.len();
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_mention_selection_changed",
                            direction = "next",
                            selected_index = session.selected_index,
                            item_count = session.items.len(),
                        );
                    }
                }
                self.log_mention_visible_range("keyboard_next");
                cx.notify();
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_enter(key) || crate::ui_foundation::is_key_tab(key) {
                // Both Enter and Tab autocomplete the focused picker item.
                // Submitting the ACP message still requires a later Enter after
                // the picker closes.
                let accepted_via_key = if crate::ui_foundation::is_key_tab(key) {
                    "tab"
                } else {
                    "enter"
                };
                let pre_accept_item = self.mention_session.as_ref().and_then(|s| {
                    s.items.get(s.selected_index).map(|item| {
                        let trigger_str = match s.trigger {
                            crate::ai::window::context_picker::types::ContextPickerTrigger::Mention => "@",
                            crate::ai::window::context_picker::types::ContextPickerTrigger::Slash => "/",
                        };
                        (
                            trigger_str.to_string(),
                            item.label.to_string(),
                            Self::telemetry_item_id(item),
                        )
                    })
                });
                let cursor_before = self.live_thread().read(cx).input.cursor();
                self.accept_mention_selection_impl(false, cx);
                let cursor_after = self.live_thread().read(cx).input.cursor();
                self.emit_key_route_telemetry(
                    key,
                    crate::protocol::AcpKeyRoute::Picker,
                    cursor_before,
                    cursor_after,
                    false,
                    true,
                );
                if let Some((trigger, label, id)) = pre_accept_item {
                    self.emit_picker_accepted_telemetry(
                        &trigger,
                        &label,
                        &id,
                        accepted_via_key,
                        cursor_after,
                        false,
                    );
                }
                if let Some(ref layout) = self.collect_acp_state_snapshot(cx).input_layout {
                    self.emit_input_layout_telemetry(layout);
                }
                cx.stop_propagation();
                return;
            }
            if crate::ui_foundation::is_key_escape(key) {
                self.mention_session = None;
                cx.notify();
                cx.stop_propagation();
                return;
            }
            // Other keys fall through to normal input handling,
            // which will update the query text and refresh the session.
        }

        // Shift+Enter inserts a newline.
        if crate::ui_foundation::is_key_enter(key) && modifiers.shift {
            self.live_thread().update(cx, |thread, cx| {
                thread.input.insert_char('\n');
                cx.notify();
            });
            cx.stop_propagation();
            return;
        }

        // Escape with no open dialogs: let it propagate to the main window
        // interceptor, which will return to the main menu.
        if crate::ui_foundation::is_key_escape(key) {
            cx.propagate();
            return;
        }

        // Enter submits.
        if crate::ui_foundation::is_key_enter(key) && !modifiers.shift {
            let cursor_before = self.live_thread().read(cx).input.cursor();
            self.mention_session = None;
            let _ = self
                .live_thread()
                .update(cx, |thread, cx| thread.submit_input(cx));
            self.emit_key_route_telemetry(
                key,
                crate::protocol::AcpKeyRoute::Composer,
                cursor_before,
                0,
                true,
                true,
            );
            cx.stop_propagation();
            return;
        }

        // ── Token-atomic inline mention deletion ──────────────
        // When backspace/delete lands inside or at the trailing edge of an
        // inline @mention token, remove the whole token plus one trailing
        // space (when present) instead of deleting a single character.
        if crate::ui_foundation::is_key_backspace(key) || key == "delete" {
            let current_text = self.live_thread().read(cx).input.text().to_string();
            let cursor = self.live_thread().read(cx).input.cursor();

            if let Some(range) =
                crate::ai::context_mentions::mention_range_at_cursor(&current_text, cursor)
            {
                let chars: Vec<char> = current_text.chars().collect();
                let mut end_char = range.end;
                // Consume one trailing space when present.
                if chars.get(end_char) == Some(&' ') {
                    end_char += 1;
                }

                let start_byte = Self::char_to_byte_offset(&current_text, range.start);
                let end_byte = Self::char_to_byte_offset(&current_text, end_char);

                let mut next_text =
                    String::with_capacity(current_text.len() - (end_byte - start_byte));
                next_text.push_str(&current_text[..start_byte]);
                next_text.push_str(&current_text[end_byte..]);

                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_inline_mention_deleted_atomically",
                    cursor,
                    token_range_start = range.start,
                    token_range_end = range.end,
                    next_cursor = range.start,
                );

                self.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text(next_text);
                    thread.input.set_cursor(range.start);
                    cx.notify();
                });
                self.sync_inline_mentions(cx);
                cx.notify();
                cx.stop_propagation();
                return;
            }
        }

        // Delegate all other keys to TextInputState::handle_key().
        // handle_key requires T: Render, so we extract input, mutate it here,
        // then write it back.
        let key_char = event.keystroke.key_char.as_deref();
        let mut input_snapshot = self.live_thread().read(cx).input.clone();
        let handled = input_snapshot.handle_key(
            key,
            key_char,
            modifiers.platform,
            modifiers.alt,
            modifiers.shift,
            cx,
        );

        if handled {
            self.live_thread().update(cx, |thread, cx| {
                thread.input = input_snapshot;
                cx.notify();
            });
            self.refresh_mention_session(cx);
            self.sync_inline_mentions(cx);
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Setup mode: render the inline setup card instead of the chat.
        if let AcpChatSession::Setup(ref state) = self.session {
            return self.render_setup_card(state.as_ref()).into_any_element();
        }

        // Runtime setup recovery: if the live thread received a SetupRequired
        // event, show the setup card instead of the errored chat transcript.
        {
            let thread_ref = self.live_thread().read(cx);
            if let Some(setup) = thread_ref.setup_state().cloned() {
                return self.render_setup_card(&setup).into_any_element();
            }
        }

        let thread = self.live_thread().read(cx);
        let status = thread.status;
        let is_empty = thread.messages.is_empty();
        let input_text = thread.input.text().to_string();
        let input_cursor = thread.input.cursor();
        let input_selection = thread.input.selection();
        let composer_active = Self::composer_is_active(
            window.is_window_active(),
            self.focus_handle.is_focused(window),
            crate::actions::is_actions_window_open(),
        );
        let cursor_visible = self.cursor_visible && composer_active;
        let pending_permission = thread.pending_permission.clone();
        let plan_entries = thread.active_plan_entries().to_vec();
        let attached_parts = thread.pending_context_parts().to_vec();
        let messages: Vec<AcpThreadMessage> = thread.messages.clone();
        let colors = Self::prompt_colors();
        let theme = theme::get_cached_theme();
        let input_has_newline = input_text.contains('\n');
        let max_visible_chars = self.acp_input_max_visible_chars(window);
        let (window_start, window_end) = thread.input.visible_window_range(max_visible_chars);
        let is_window_truncated_left = !input_has_newline && window_start > 0;
        let is_window_truncated_right =
            !input_has_newline && window_end < input_text.chars().count();
        let mention_highlights = Self::attached_inline_mention_highlight_ranges(
            &input_text,
            &attached_parts,
            Self::ACP_MENTION_INLINE_GOLD,
        );
        let pending_permission_has_message_target = pending_permission
            .as_ref()
            .and_then(Self::permission_request_tool_call_id)
            .is_some_and(|tool_call_id| {
                messages
                    .iter()
                    .any(|msg| msg.tool_call_id.as_deref() == Some(tool_call_id))
            });
        let view_entity: WeakEntity<AcpChatView> = cx.entity().downgrade();

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let modifiers = &event.keystroke.modifiers;

                    // Cmd+W in detached window: close the window directly.
                    // In the main panel, Cmd+W is handled by the interceptor.
                    if modifiers.platform && key.eq_ignore_ascii_case("w")
                        && crate::ai::acp::chat_window::is_chat_window_open()
                    {
                        let wb = window.window_bounds();
                        crate::window_state::save_window_from_gpui(
                            crate::window_state::WindowRole::AcpChat,
                            wb,
                        );
                        crate::ai::acp::chat_window::clear_chat_window_handle();
                        window.remove_window();
                        cx.stop_propagation();
                        return;
                    }

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
                            .flex_col()
                            .justify_center()
                            .min_h(px(22.0))
                            // Empirical: px(17) here renders identically to px(16) in
                            // the main menu input.  The 1px offset is a GPUI layout quirk —
                            // both paths target the same visual size (design_typography.font_size_lg).
                            .text_size(px(17.0))
                            .line_height(px(22.0))
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
                                            .child(if is_empty {
                                                "Ask anything\u{2026}"
                                            } else {
                                                "Follow up\u{2026}"
                                            }),
                                    )
                                    .into_any_element()
                            } else {
                                render_text_input_cursor_selection(TextInputRenderConfig {
                                    cursor: input_cursor,
                                    selection: Some(input_selection),
                                    window: (!input_has_newline).then_some((window_start, window_end)),
                                    multiline: input_has_newline,
                                    cursor_visible,
                                    cursor_color: theme.colors.accent.selected,
                                    text_color: theme.colors.text.primary,
                                    selection_color: theme.colors.accent.selected,
                                    selection_text_color: theme.colors.text.primary,
                                    cursor_height: 18.0,
                                    cursor_width: 2.0,
                                    container_height: Some(22.0),
                                    overflow_x_hidden: !input_has_newline,
                                    leading_indicator: is_window_truncated_left.then_some(
                                        TextInputRenderIndicator {
                                            text: "...",
                                            color: theme.colors.text.muted,
                                        },
                                    ),
                                    trailing_indicator: is_window_truncated_right.then_some(
                                        TextInputRenderIndicator {
                                            text: "...",
                                            color: theme.colors.text.muted,
                                        },
                                    ),
                                    highlight_ranges: &mention_highlights,
                                    ..TextInputRenderConfig::default_for_prompt(&input_text)
                                })
                                .into_any_element()
                            }),
                    ),
            )
            // ── Context chips (focused target / Ask Anything) ────
            .child(self.render_pending_context_chips(cx))
            .child(self.render_context_bootstrap_note(cx))
            // ── Search bar (Cmd+F) ─────────────────────────
            .when_some(self.search_state.clone(), |d, (query, current_idx)| {
                let match_count = if query.is_empty() {
                    0
                } else {
                    let q = query.to_lowercase();
                    messages.iter().filter(|m| m.body.to_lowercase().contains(&q)).count()
                };
                let display_idx = if match_count > 0 {
                    (current_idx % match_count) + 1
                } else {
                    0
                };
                d.child(
                    div()
                        .w_full()
                        .px(px(12.0))
                        .py(px(4.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_xs()
                                .opacity(0.50)
                                .child("\u{1F50D}"),
                        )
                        .child(
                            div()
                                .flex_grow()
                                .text_sm()
                                .child(if query.is_empty() {
                                    "Search conversation\u{2026}".to_string()
                                } else {
                                    query.clone()
                                }),
                        )
                        .when(!query.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .opacity(0.45)
                                    .child(if match_count > 0 {
                                        format!("{display_idx}/{match_count}")
                                    } else {
                                        "0 matches".to_string()
                                    }),
                            )
                        })
                        .when(match_count > 1, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .opacity(0.30)
                                    .child("\u{21A9} next \u{00b7} \u{21E7}\u{21A9} prev"),
                            )
                        })
                        .child(
                            div().text_xs().opacity(0.25).child("esc \u{00d7}"),
                        ),
                )
            })
            // ── Unified picker (@ mentions + / commands) ─────────
            .when_some(self.mention_session.clone(), |d, session| {
                let (picker_left, picker_top, picker_width) =
                    self.mention_picker_anchor_for_session(&session, &input_text, window);
                let picker = if session.items.is_empty() {
                    self.render_mention_empty_state(picker_width, cx)
                } else {
                    self.render_mention_picker(&session, picker_width, cx)
                };
                d.child(
                    div()
                        .id("acp-mention-picker-layer")
                        .absolute()
                        .left(px(picker_left))
                        .top(px(picker_top))
                        .w(px(picker_width))
                        .child(picker),
                )
            })
            // ── History picker (below input, replaces message list) ──
            .when_some(
                self.history_menu
                    .as_ref()
                    .map(|(idx, filter, entries)| (*idx, filter.clone(), entries.clone())),
                |d, (idx, filter, all_entries)| {
                    let theme = theme::get_cached_theme();
                    // Apply filter
                    let entries: Vec<_> = if filter.is_empty() {
                        all_entries
                    } else {
                        let q = filter.to_lowercase();
                        all_entries
                            .into_iter()
                            .filter(|e| e.first_message.to_lowercase().contains(&q))
                            .collect()
                    };
                    d.child(
                        div().w_full().px(px(8.0)).child(
                            div()
                                .id("acp-history-picker")
                                .w_full()
                                .max_h(px(300.0))
                                .overflow_y_scroll()
                                .rounded(px(8.0))
                                .bg(rgb(theme.colors.background.search_box))
                                .border_1()
                                .border_color(rgba((theme.colors.ui.border << 8) | 0x40))
                                .py(px(4.0))
                                .child(
                                    div()
                                        .px(px(10.0))
                                        .py(px(4.0))
                                        .text_xs()
                                        .opacity(0.45)
                                        .child(if filter.is_empty() {
                                            "Recent Conversations (\u{2318}P)".to_string()
                                        } else {
                                            format!("Search: {filter}")
                                        }),
                                )
                                .children(entries.iter().enumerate().map(|(i, entry)| {
                                    let is_selected = i == idx;
                                    let date = entry
                                        .timestamp
                                        .split('T')
                                        .next()
                                        .unwrap_or(&entry.timestamp);
                                    div()
                                        .w_full()
                                        .px(px(10.0))
                                        .py(px(5.0))
                                        .when(is_selected, |d| {
                                            d.bg(rgba((theme.colors.accent.selected << 8) | 0x14))
                                                .border_l_2()
                                                .border_color(rgb(theme.colors.accent.selected))
                                        })
                                        .when(!is_selected, |d| {
                                            d.border_l_2().border_color(gpui::transparent_black())
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap(px(1.0))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .child(entry.first_message.clone()),
                                                )
                                                .child(div().text_xs().opacity(0.40).child(
                                                    format!(
                                                        "{} messages \u{00b7} {}",
                                                        entry.message_count, date
                                                    ),
                                                )),
                                        )
                                }))
                                // Keyboard hint at bottom
                                .child(
                                    div()
                                        .w_full()
                                        .px(px(10.0))
                                        .pt(px(6.0))
                                        .pb(px(4.0))
                                        .border_t_1()
                                        .border_color(rgba(
                                            (theme.colors.ui.border << 8) | 0x15,
                                        ))
                                        .text_xs()
                                        .opacity(0.35)
                                        .child(
                                            "\u{2191}\u{2193} navigate \u{00b7} Enter load \u{00b7} Esc close \u{00b7} type to search",
                                        ),
                                ),
                        ),
                    )
                },
            )
            // ── Message list (middle, virtualized) ────────────
            .when(is_empty, |d| {
                d.child(
                    div()
                        .flex_grow()
                        .min_h(px(0.))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .opacity(0.30)
                                .text_xs()
                                .child("Type / for skills")
                                .child("\u{21E7}\u{21A9} for newlines")
                                .child("\u{2318}P history \u{00b7} \u{2318}K actions")
                                .child("\u{2318}N new \u{00b7} \u{2318}W close"),
                        ),
                )
            })
            .when(!is_empty, |d| {
                // Capture state for the list render callback.
                let messages_snapshot = messages.clone();
                let collapsed_ids = self.collapsed_ids.clone();
                let search_state = self.search_state.clone();
                let weak_view = view_entity.clone();
                let pending_permission_snap = pending_permission.clone();
                let permission_index_snap = self.permission_index;
                let permission_options_open_snap = self.permission_options_open;
                let colors_snap = colors;
                let theme_snap = theme::get_cached_theme();
                let _is_streaming = matches!(status, AcpThreadStatus::Streaming);

                d.child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h(px(0.))
                        .overflow_hidden()
                        .child(list(self.list_state.clone(), move |ix, _window, _cx| {
                        let msg = &messages_snapshot[ix];
                        let msg_id = msg.id;
                        let is_collapsible = matches!(
                            msg.role,
                            AcpThreadMessageRole::Thought | AcpThreadMessageRole::Tool
                        );
                        let is_collapsed =
                            is_collapsible && !collapsed_ids.contains(&msg_id);

                        let on_toggle: Option<ToggleHandler> = if is_collapsible {
                            let weak = weak_view.clone();
                            Some(Box::new(move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut App| {
                                if let Some(entity) = weak.upgrade() {
                                    entity.update(cx, |view, cx| {
                                        if view.collapsed_ids.contains(&msg_id) {
                                            view.collapsed_ids.remove(&msg_id);
                                        } else {
                                            view.collapsed_ids.insert(msg_id);
                                        }
                                        cx.notify();
                                    });
                                }
                            }))
                        } else {
                            None
                        };

                        let prev_was_user = ix > 0
                            && matches!(messages_snapshot[ix - 1].role, AcpThreadMessageRole::User);
                        let is_response_start = prev_was_user
                            && !matches!(msg.role, AcpThreadMessageRole::User);
                        let is_new_turn = ix > 0
                            && matches!(msg.role, AcpThreadMessageRole::User)
                            && !matches!(messages_snapshot[ix - 1].role, AcpThreadMessageRole::User);

                        // Search highlight
                        let (is_search_match, is_current_match) =
                            if let Some((ref q, current_idx)) = search_state {
                                if !q.is_empty()
                                    && msg.body.to_lowercase().contains(&q.to_lowercase())
                                {
                                    let ql = q.to_lowercase();
                                    let match_num = messages_snapshot[..=ix]
                                        .iter()
                                        .filter(|m| m.body.to_lowercase().contains(&ql))
                                        .count()
                                        - 1;
                                    let total = messages_snapshot
                                        .iter()
                                        .filter(|m| m.body.to_lowercase().contains(&ql))
                                        .count();
                                    let target =
                                        if total > 0 { current_idx % total } else { 0 };
                                    (true, match_num == target)
                                } else {
                                    (false, false)
                                }
                            } else {
                                (false, false)
                            };
                        let inline_permission = pending_permission_snap
                            .as_ref()
                            .filter(|request| Self::permission_request_matches_message(msg, request))
                            .cloned();

                        div()
                            .w_full()
                            .px(px(8.0))
                            .pb(px(4.0))
                            .when(is_response_start, |d| d.mt(px(4.0)))
                            .when(is_new_turn, |d| {
                                d.mt(px(8.0)).pt(px(8.0)).border_t_1().border_color(rgba(
                                    (theme_snap.colors.ui.border << 8) | 0x18,
                                ))
                            })
                            .when(is_search_match && !is_current_match, |d| {
                                d.bg(rgba((theme_snap.colors.accent.selected << 8) | 0x08))
                                    .rounded(px(4.0))
                            })
                            .when(is_current_match, |d| {
                                d.bg(rgba((theme_snap.colors.accent.selected << 8) | 0x18))
                                    .rounded(px(4.0))
                                    .border_l_2()
                                    .border_color(rgb(theme_snap.colors.accent.selected))
                            })
                            .child(Self::render_message(
                                msg,
                                &colors_snap,
                                is_collapsed,
                                on_toggle,
                            ))
                            .when_some(inline_permission, |d, request| {
                                d.child(Self::render_permission_inline_card(
                                    &request,
                                    permission_index_snap,
                                    permission_options_open_snap,
                                    weak_view.clone(),
                                ))
                            })
                            .into_any()
                    })
                    .size_full()
                    .with_sizing_behavior(gpui::ListSizingBehavior::Auto))
                        .vertical_scrollbar(&self.list_state),
                )
            })
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
            // ── Pending permission fallback (non-tool-linked) ──────
            .when_some(
                pending_permission
                    .clone()
                    .filter(|_| !pending_permission_has_message_target),
                |d, request| {
                    d.child(
                        div()
                            .w_full()
                            .px(px(8.0))
                            .pb(px(4.0))
                            .child(Self::render_permission_inline_card(
                                &request,
                                self.permission_index,
                                self.permission_options_open,
                                view_entity.clone(),
                            )),
                    )
                },
            )
            // ── Attach menu popup ──────────────────────────
            .when(self.attach_menu_open, |d| {
                d.child(self.render_attach_menu(cx))
            })
            // ── Model selector popup ──────────────────────────
            .when(self.model_selector_open, |d| {
                d.child(self.render_model_selector(cx))
            })
            // ── BOTTOM: Hint strip ─────────────────────
            .child(self.render_toolbar(cx))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::AcpChatView;
    use crate::ai::acp::permission_broker::{AcpApprovalPreview, AcpApprovalRequest};
    use crate::ai::acp::thread::{AcpThreadMessage, AcpThreadMessageRole};
    use crate::ai::window::context_picker::types::{ContextPickerItem, ContextPickerItemKind};
    use gpui::SharedString;

    #[test]
    fn mention_picker_width_respects_window_gutters() {
        let width = AcpChatView::mention_picker_width_for_window(240.0);
        assert_eq!(
            width, 216.0,
            "picker width should shrink to fit within the window gutters"
        );
    }

    #[test]
    fn mention_picker_width_caps_at_design_width() {
        let width = AcpChatView::mention_picker_width_for_window(1200.0);
        assert_eq!(
            width,
            AcpChatView::ACP_MENTION_PICKER_WIDTH,
            "wide windows should keep the canonical picker width"
        );
    }

    #[test]
    fn mention_picker_left_clamps_to_visible_right_edge() {
        let left = AcpChatView::clamp_mention_picker_left(640.0, 320.0, 800.0);
        assert_eq!(
            left, 468.0,
            "picker should shift left so its right edge stays onscreen"
        );
    }

    #[test]
    fn mention_picker_left_never_moves_past_left_padding() {
        let left = AcpChatView::clamp_mention_picker_left(-30.0, 320.0, 800.0);
        assert_eq!(
            left,
            AcpChatView::ACP_INPUT_PADDING_X,
            "picker should stay aligned to the input gutter when the anchor is too far left"
        );
    }

    #[test]
    fn caret_after_replacement_tracks_inserted_token_not_end_of_composer() {
        let range = 6..10;
        let replacement = "@snapshot ";
        assert_eq!(
            AcpChatView::caret_after_replacement(&range, replacement),
            16,
            "caret should land immediately after the accepted token"
        );
    }

    #[test]
    fn replace_text_in_char_range_preserves_surrounding_text() {
        let updated = AcpChatView::replace_text_in_char_range("hello @con", 6..10, "@snapshot ");
        assert_eq!(updated, "hello @snapshot ");
    }

    #[test]
    fn composer_is_active_requires_focus_and_no_actions_window() {
        assert!(AcpChatView::composer_is_active(true, true, false));
        assert!(!AcpChatView::composer_is_active(true, false, false));
        assert!(!AcpChatView::composer_is_active(false, true, false));
        assert!(!AcpChatView::composer_is_active(true, true, true));
    }

    #[test]
    fn permission_request_matches_tool_message_by_tool_call_id() {
        let (reply_tx, _reply_rx) = async_channel::bounded(1);
        let request = AcpApprovalRequest {
            id: 1,
            title: "ACP permission request".into(),
            body: String::new(),
            preview: Some(AcpApprovalPreview::new("write_text_file", "tc-123")),
            options: vec![],
            reply_tx,
        };
        let msg = AcpThreadMessage {
            id: 9,
            role: AcpThreadMessageRole::Tool,
            body: "Write file\nrunning".into(),
            tool_call_id: Some("tc-123".to_string()),
        };

        assert!(AcpChatView::permission_request_matches_message(
            &msg, &request
        ));
    }

    #[test]
    fn telemetry_item_id_redacts_local_paths() {
        let file_item = ContextPickerItem {
            id: SharedString::from("file:/tmp/secrets.txt"),
            label: SharedString::from("secrets.txt"),
            meta: SharedString::from("@file:/tmp/secrets.txt"),
            kind: ContextPickerItemKind::File(std::path::PathBuf::from("/tmp/secrets.txt")),
            score: 100,
            label_highlight_indices: Vec::new(),
            meta_highlight_indices: Vec::new(),
        };
        let folder_item = ContextPickerItem {
            id: SharedString::from("folder:/Users/john/Documents"),
            label: SharedString::from("Documents"),
            meta: SharedString::from("@file:/Users/john/Documents"),
            kind: ContextPickerItemKind::Folder(std::path::PathBuf::from("/Users/john/Documents")),
            score: 100,
            label_highlight_indices: Vec::new(),
            meta_highlight_indices: Vec::new(),
        };

        assert_eq!(
            AcpChatView::telemetry_item_id(&file_item),
            "file:secrets.txt"
        );
        assert_eq!(
            AcpChatView::telemetry_item_id(&folder_item),
            "folder:Documents"
        );
    }
}
