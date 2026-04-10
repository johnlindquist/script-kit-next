//! ACP chat view.
//!
//! Renders an ACP conversation thread with markdown-rendered messages,
//! role-aware cards, empty/streaming/error states, and inline permission
//! approval cards. Wraps an `AcpThread` entity for the Tab AI surface.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use gpui::{
    div, list, prelude::*, px, rgb, rgba, Animation, AnimationExt, App, Context, ElementId, Entity,
    FocusHandle, Focusable, FontWeight, IntoElement, ListAlignment, ListState, ParentElement,
    Render, SharedString, Task, WeakEntity, Window,
};

use gpui_component::scroll::ScrollableElement;

use crate::components::text_input::{
    render_text_input_cursor_selection, TextHighlightRange, TextInputRenderConfig,
};
use crate::prompts::markdown::render_markdown_with_scope;
use crate::theme::{self, PromptColors};

use super::history_popup::{
    history_popup_key_intent, AcpHistoryPopupKeyIntent, HISTORY_POPUP_PAGE_JUMP,
    HISTORY_POPUP_SEARCH_LIMIT,
};
use super::thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadMessage, AcpThreadMessageRole, AcpThreadStatus,
};
use super::{AcpApprovalOption, AcpApprovalPreview, AcpApprovalPreviewKind, AcpApprovalRequest};

use crate::ai::message_parts::AiContextPart;
use crate::ai::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, ContextPickerTrigger, SlashCommandPayload,
};
use crate::ai::window::context_picker::{
    build_picker_items, build_slash_picker_items, build_slash_picker_items_with_descriptions,
    build_slash_picker_items_with_payloads, slash_picker_empty_row, slash_picker_loading_row,
    slash_picker_no_match_row,
};

/// Active @-mention session state for the ACP inline context picker.
#[derive(Debug, Clone)]
pub(crate) struct AcpMentionSession {
    /// Which trigger character opened this session (`@` or `/`).
    trigger: ContextPickerTrigger,
    /// Character range of the trigger+query in the input text.
    trigger_range: std::ops::Range<usize>,
    /// Currently highlighted row index.
    pub(crate) selected_index: usize,
    /// Ranked picker items for the current query.
    pub(crate) items: Vec<ContextPickerItem>,
}

#[derive(Debug, Clone, Copy)]
struct AcpMentionPopupParentWindow {
    handle: gpui::AnyWindowHandle,
    bounds: gpui::Bounds<gpui::Pixels>,
    display_id: Option<gpui::DisplayId>,
}

/// Click handler type for collapsible block toggle.
type ToggleHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;
/// Footer action callbacks use `&mut App` (not `Context<AcpChatView>`) so they can be
/// invoked without holding the AcpChatView borrow — toggle_actions needs to read the
/// entity, which panics if called from inside its own update.
type AcpFooterActionHandler = std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>;
/// Portal open callback — receives the portal kind so the host can open the
/// appropriate built-in view (file search, clipboard history, etc.).
/// Takes `&mut App` (not `&mut Window`) because the handler opens a new view
/// via entity update, and this callback is invoked from contexts where
/// `Window` is not available (e.g. `accept_mention_selection_impl`).
type AcpPortalHandler = std::sync::Arc<
    dyn Fn(crate::ai::window::context_picker::types::PortalKind, &mut App) + 'static,
>;

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

// ── Source-aware slash command model ──────────────────────────────────

/// The origin of a slash command entry discovered during skill enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlashCommandSource {
    /// A built-in Claude Code command (e.g. `/compact`, `/clear`).
    Default,
    /// A skill owned by a discovered plugin.
    PluginSkill(crate::plugins::PluginSkill),
    /// A user-level Claude Code skill from `~/.scriptkit/.claude/skills/`.
    ClaudeCodeSkill {
        skill_id: String,
        skill_path: std::path::PathBuf,
    },
}

impl SlashCommandSource {
    fn owner_label(&self) -> String {
        match self {
            Self::Default => "Built-in".to_string(),
            Self::PluginSkill(skill) => {
                if skill.plugin_title.is_empty() {
                    skill.plugin_id.clone()
                } else {
                    skill.plugin_title.clone()
                }
            }
            Self::ClaudeCodeSkill { .. } => "Claude Code".to_string(),
        }
    }
}

/// A discovered slash command entry with source identity and description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SlashCommandEntry {
    /// The bare slash name (e.g. `"compact"`, `"review"`).
    pub name: String,
    /// Human-readable description for the picker.
    pub description: String,
    /// Where this entry came from.
    pub source: SlashCommandSource,
}

impl SlashCommandEntry {
    pub(crate) fn default_command(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            source: SlashCommandSource::Default,
        }
    }

    fn plugin_skill(skill: &crate::plugins::PluginSkill) -> Self {
        let plugin_title = if skill.plugin_title.is_empty() {
            skill.plugin_id.clone()
        } else {
            skill.plugin_title.clone()
        };

        let raw_desc = if skill.description.is_empty() {
            format!("Plugin: {}", plugin_title)
        } else {
            format!("{} \u{2014} {}", plugin_title, skill.description)
        };

        let desc_chars: Vec<char> = raw_desc.chars().collect();
        let description = if desc_chars.len() > 80 {
            let truncated: String = desc_chars.into_iter().take(77).collect();
            format!("{truncated}\u{2026}")
        } else {
            raw_desc
        };

        Self {
            name: skill.skill_id.clone(),
            description,
            source: SlashCommandSource::PluginSkill(skill.clone()),
        }
    }

    fn claude_code_skill(
        name: String,
        description: String,
        skill_path: std::path::PathBuf,
    ) -> Self {
        Self {
            name: name.clone(),
            description,
            source: SlashCommandSource::ClaudeCodeSkill {
                skill_id: name,
                skill_path,
            },
        }
    }

    /// A key that uniquely identifies this entry across sources.
    pub(crate) fn qualified_key(&self) -> String {
        match &self.source {
            SlashCommandSource::Default => format!("default:{}", self.name),
            SlashCommandSource::PluginSkill(skill) => {
                format!("{}:{}", skill.plugin_id, skill.skill_id)
            }
            SlashCommandSource::ClaudeCodeSkill { skill_id, .. } => {
                format!("claude:{skill_id}")
            }
        }
    }

    /// Convert to a `SlashCommandPayload` for the context picker item kind.
    pub(crate) fn to_payload(&self) -> SlashCommandPayload {
        match &self.source {
            SlashCommandSource::Default => SlashCommandPayload::Default {
                name: self.name.clone(),
            },
            SlashCommandSource::PluginSkill(skill) => {
                SlashCommandPayload::PluginSkill(skill.clone())
            }
            SlashCommandSource::ClaudeCodeSkill {
                skill_id,
                skill_path,
            } => SlashCommandPayload::ClaudeCodeSkill {
                skill_id: skill_id.clone(),
                skill_path: skill_path.clone(),
            },
        }
    }
}

/// Build the staged prompt text for a local skill being accepted from
/// the ACP slash picker or main-menu skill launch.  Both entry paths
/// must produce the same deterministic payload so that the ACP agent
/// receives identical context regardless of how the user invoked the skill.
pub(crate) fn build_staged_skill_prompt(
    skill_title: &str,
    owner_label: &str,
    skill_path: &std::path::Path,
) -> String {
    let skill_content = std::fs::read_to_string(skill_path).unwrap_or_default();
    let owner_phrase = if owner_label == "Claude Code" {
        format!("from {owner_label}")
    } else {
        format!("from plugin \"{owner_label}\"")
    };
    if skill_content.is_empty() {
        format!("Use the skill \"{skill_title}\" {owner_phrase} for this session.")
    } else {
        format!(
            "Use the attached skill \"{skill_title}\" {owner_phrase} for this session.\n\n<skill path=\"{}\">\n{}\n</skill>",
            skill_path.display(),
            skill_content
        )
    }
}

/// Session mode for the ACP chat view.
#[derive(Clone)]
pub(crate) enum AcpChatSession {
    /// Live conversation with an ACP agent thread.
    Live(Entity<AcpThread>),
    /// Inline setup card — no launchable agent exists.
    Setup(Box<super::setup_state::AcpInlineSetupState>),
}

/// Explicit relaunch payload queued when setup retry is requested.
///
/// Carries the selected agent id and capability requirements from the
/// setup card so the next ACP open path can consume them ahead of
/// fallback preference loading.
#[derive(Debug, Clone)]
pub(crate) struct AcpRetryRequest {
    pub preferred_agent_id: Option<String>,
    pub launch_requirements: super::preflight::AcpLaunchRequirements,
}

impl AcpRetryRequest {
    pub(crate) fn from_setup_state(setup: &super::setup_state::AcpInlineSetupState) -> Self {
        Self {
            preferred_agent_id: setup
                .selected_agent
                .as_ref()
                .map(|agent| agent.id.to_string()),
            launch_requirements: setup.launch_requirements,
        }
    }
}

/// Explicit resume payload queued when a history item is selected for
/// re-opening. The ACP open path can consume this to load a saved
/// conversation by `session_id` instead of using clipboard text or
/// markdown export.
#[derive(Debug, Clone)]
pub(crate) struct AcpHistoryResumeRequest {
    pub session_id: String,
}

/// Structured state for the inline ACP history popup.
///
/// Replaces the old `Option<(usize, String, Vec<AcpHistoryEntry>)>` tuple
/// so ranked search metadata (`AcpHistorySearchHit`) is preserved through
/// render instead of being discarded before the popup sees it.
#[derive(Debug, Clone)]
pub(crate) struct AcpHistoryMenuState {
    pub(crate) selected_index: usize,
    pub(crate) query: String,
    pub(crate) hits: Vec<super::history::AcpHistorySearchHit>,
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
    /// Ranked history popup state. None = hidden.
    pub(crate) history_menu: Option<AcpHistoryMenuState>,
    /// Most recent timestamp when the history popup was explicitly dismissed.
    history_closed_at: Option<Instant>,
    /// Whether the + attachment menu popup is open.
    attach_menu_open: bool,
    /// Whether the model selector dropdown is open.
    model_selector_open: bool,
    /// Focused row within the model selector popup.
    model_selector_selected_index: usize,
    /// Cmd+F search: (query, current_match_index). None = search hidden.
    pub(crate) search_state: Option<(String, usize)>,
    /// Cached slash commands discovered at creation, with source identity.
    cached_slash_commands: Vec<SlashCommandEntry>,
    /// Handle to the deferred slash command discovery task.
    _slash_discovery_task: Task<()>,
    /// Active @-mention picker session (None = picker hidden).
    pub(crate) mention_session: Option<AcpMentionSession>,
    /// Cached parent window metadata for the detached picker popup.
    mention_popup_parent_window: Option<AcpMentionPopupParentWindow>,
    /// Canonical inline tokens that currently own their attached context part.
    ///
    /// This preserves non-inline chip attachments during mention sync while
    /// still letting deleted inline mentions remove the parts they created.
    inline_owned_context_tokens: HashSet<String>,
    /// Session-local alias registry mapping typed `@type:name` display tokens
    /// to full `AiContextPart` values for resolution and sync.
    typed_mention_aliases:
        std::collections::HashMap<String, crate::ai::message_parts::AiContextPart>,
    /// Agent picker overlay state for setup mode (None = hidden).
    setup_agent_picker: Option<AcpSetupAgentPickerState>,
    /// Most recently accepted picker item (for telemetry/testing).
    last_accepted_item: Option<crate::protocol::AcpAcceptedItem>,
    /// Bounded test probe ring buffer for agentic verification.
    test_probe: AcpTestProbe,
    /// Queued retry payload from setup card — consumed by the ACP open path.
    pending_retry_request: Option<AcpRetryRequest>,
    /// Queued history resume request — consumed by the ACP open path
    /// to load a saved conversation by session_id.
    pending_history_resume: Option<AcpHistoryResumeRequest>,
    /// Host-owned footer callback for toggling the actions popup.
    on_toggle_actions: Option<AcpFooterActionHandler>,
    /// Host-owned footer callback for closing the ACP surface.
    on_close_requested: Option<AcpFooterActionHandler>,
    /// Host-owned callback for opening the dedicated history command surface.
    on_open_history_command: Option<AcpFooterActionHandler>,
    /// Host-owned callback for opening a full built-in view as an attachment portal.
    on_open_portal: Option<AcpPortalHandler>,
    /// Stashed query text from the `@history` trigger, consumed once by the
    /// portal opener to prefilter the history popup.
    pending_history_portal_query: Option<String>,
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
    /// Most recent synthesised interaction trace (key-route + optional accept).
    pub(crate) last_interaction_trace: Option<crate::protocol::AcpLastInteractionTrace>,
}

use crate::protocol::ACP_TEST_PROBE_MAX_EVENTS;

#[derive(Clone)]
struct AcpKeyRouteTelemetryArgs {
    route: crate::protocol::AcpKeyRoute,
    cursor_before: usize,
    cursor_after: usize,
    caused_submit: bool,
    consumed: bool,
    permission_active: bool,
}

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

    fn was_history_recently_closed(&self) -> bool {
        const HISTORY_CLOSE_DEBOUNCE: Duration = Duration::from_millis(300);
        self.history_closed_at
            .map(|t| t.elapsed() < HISTORY_CLOSE_DEBOUNCE)
            .unwrap_or(false)
    }

    fn mark_history_popup_closed(&mut self, cx: &mut Context<Self>) {
        self.history_menu = None;
        self.history_closed_at = Some(Instant::now());
        cx.notify();
    }

    pub(crate) fn dismiss_history_popup(&mut self, cx: &mut Context<Self>) {
        if self.history_menu.is_none() {
            return;
        }

        self.mark_history_popup_closed(cx);
        self.sync_history_popup_window_from_cached_parent(cx);
    }

    pub(crate) fn dismiss_history_popup_from_window(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        if self.history_menu.is_none() {
            return;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_popup_closed",
            reason,
            "Closed ACP history popup from detached window lifecycle"
        );
        self.mark_history_popup_closed(cx);
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
            ContextPickerItemKind::Portal(_) | ContextPickerItemKind::Inert => item.id.to_string(),
        }
    }

    fn cache_popup_parent_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let parent = AcpMentionPopupParentWindow {
            handle: window.window_handle(),
            bounds: window.bounds(),
            display_id: window.display(cx).map(|display| display.id()),
        };
        self.mention_popup_parent_window = Some(parent);
    }

    fn sync_acp_popup_windows_from_cached_parent(&mut self, cx: &mut Context<Self>) {
        self.sync_mention_popup_window_from_cached_parent(cx);
        self.sync_model_selector_popup_window_from_cached_parent(cx);
        self.sync_history_popup_window_from_cached_parent(cx);
    }

    fn selected_model_popup_index(&self, cx: &App) -> usize {
        let model_count = self.live_thread().read(cx).available_models().len();
        crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
            self.model_selector_selected_index,
            model_count,
        )
    }

    fn reset_model_selector_selection(&mut self, cx: &App) {
        let thread = self.live_thread().read(cx);
        let selected_id = thread.selected_model_id();
        let next_index = thread
            .available_models()
            .iter()
            .position(|model| Some(model.id.as_str()) == selected_id)
            .unwrap_or(0);
        self.model_selector_selected_index =
            crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
                next_index,
                thread.available_models().len(),
            );
    }

    fn mention_popup_snapshot(
        &self,
        cx: &App,
    ) -> Option<(
        crate::ai::acp::picker_popup::AcpMentionPopupSnapshot,
        f32,
        f32,
    )> {
        let session = self.mention_session.as_ref()?.clone();
        let parent = self.mention_popup_parent_window?;
        let input_text = self.live_thread().read(cx).input.text().to_string();
        let window_width = parent.bounds.size.width.as_f32();
        let (left, top, width) =
            self.mention_picker_anchor_for_session(&session, &input_text, window_width);

        Some((
            crate::ai::acp::picker_popup::AcpMentionPopupSnapshot {
                trigger: session.trigger,
                selected_index: session.selected_index,
                items: session.items,
                width,
            },
            left,
            top,
        ))
    }

    pub(crate) fn set_on_toggle_actions(
        &mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) {
        self.on_toggle_actions = Some(std::sync::Arc::new(callback));
    }

    pub(crate) fn set_on_close_requested(
        &mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) {
        self.on_close_requested = Some(std::sync::Arc::new(callback));
    }

    pub(crate) fn set_on_open_portal(
        &mut self,
        callback: impl Fn(crate::ai::window::context_picker::types::PortalKind, &mut App) + 'static,
    ) {
        self.on_open_portal = Some(std::sync::Arc::new(callback));
    }

    /// Register an inline mention token as owned so the mention sync system
    /// will remove the corresponding context part when the token is deleted.
    pub(crate) fn register_inline_owned_token(&mut self, token: String) {
        self.inline_owned_context_tokens.insert(token);
    }

    /// Register a typed mention alias so the parser can resolve `@type:name`
    /// tokens back to full `AiContextPart` values.
    pub(crate) fn register_typed_alias(
        &mut self,
        token: String,
        part: crate::ai::message_parts::AiContextPart,
    ) {
        self.typed_mention_aliases.insert(token, part);
    }

    /// Read-only access to the typed mention alias registry.
    pub(crate) fn typed_aliases(
        &self,
    ) -> &std::collections::HashMap<String, crate::ai::message_parts::AiContextPart> {
        &self.typed_mention_aliases
    }

    /// Expand typed display tokens in the input text back to full paths/URIs
    /// before sending to the AI. Replaces `@rs:demo` with `@file:"/full/path.rs"` etc.
    fn expand_typed_tokens_for_submit(&self, cx: &mut Context<Self>) {
        if self.typed_mention_aliases.is_empty() {
            return;
        }
        let text = self.live_thread().read(cx).input.text().to_string();
        if text.is_empty() {
            return;
        }

        let mentions = crate::ai::context_mentions::parse_inline_context_mentions_with_aliases(
            &text,
            &self.typed_mention_aliases,
        );
        if mentions.is_empty() {
            return;
        }

        // Build the expanded text by replacing typed tokens with full source paths.
        // Process mentions in reverse order to preserve character indices.
        let mut expanded = text.clone();
        for mention in mentions.iter().rev() {
            let full_ref = match &mention.part {
                crate::ai::message_parts::AiContextPart::FilePath { path, .. } => {
                    crate::ai::context_mentions::format_inline_file_token(path)
                }
                crate::ai::message_parts::AiContextPart::FocusedTarget {
                    target, label, ..
                } => {
                    // File/directory targets expand to full @file:path
                    if let Some(path) = target
                        .metadata
                        .as_ref()
                        .and_then(|m| m.get("path"))
                        .and_then(|v| v.as_str())
                    {
                        crate::ai::context_mentions::format_inline_file_token(path)
                    } else {
                        crate::ai::context_mentions::part_to_inline_token(&mention.part)
                            .unwrap_or_else(|| format!("@cmd:{label}"))
                    }
                }
                _ => continue,
            };
            let byte_start = expanded
                .char_indices()
                .nth(mention.range.start)
                .map(|(b, _)| b)
                .unwrap_or(0);
            let byte_end = expanded
                .char_indices()
                .nth(mention.range.end)
                .map(|(b, _)| b)
                .unwrap_or(expanded.len());
            expanded.replace_range(byte_start..byte_end, &full_ref);
        }

        if expanded != text {
            self.live_thread().update(cx, |thread, _cx| {
                thread.input.set_text(expanded);
            });
        }
    }

    /// Submit the current input, expanding typed display tokens to full paths first.
    fn submit_with_expanded_tokens(&mut self, cx: &mut Context<Self>) {
        self.expand_typed_tokens_for_submit(cx);
        let _ = self
            .live_thread()
            .update(cx, |thread, cx| thread.submit_input(cx));
    }

    /// Invoke a footer callback outside the AcpChatView borrow by spawning an
    /// immediate async task. The host callbacks (toggle_actions, close, etc.)
    /// may need to entity.read() this view, which panics if we're inside update.
    fn spawn_footer_callback(
        callback: AcpFooterActionHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let window_handle = window.window_handle();
        cx.spawn(async move |_this, cx| {
            let _ = window_handle.update(cx, |_root, window, cx| {
                callback(window, cx);
            });
        })
        .detach();
    }

    fn trigger_toggle_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(callback) = self.on_toggle_actions.clone() {
            // toggle_actions needs entity.read(cx) on AcpChatView, which panics
            // if called from within AcpChatView's own update. Spawn an immediate
            // async task to fully release the entity borrow first.
            Self::spawn_footer_callback(callback, window, cx);
        } else {
            tracing::warn!(
                target: "script_kit::acp",
                event = "acp_footer_toggle_actions_no_callback",
                "ACP footer actions click dropped because no host callback was installed"
            );
        }
    }

    fn trigger_close_requested(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(callback) = self.on_close_requested.clone() {
            Self::spawn_footer_callback(callback, window, cx);
        } else {
            tracing::warn!(
                target: "script_kit::acp",
                event = "acp_footer_close_no_callback",
                "ACP footer close click dropped because no host callback was installed"
            );
        }
    }

    pub(crate) fn set_on_open_history_command(
        &mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) {
        self.on_open_history_command = Some(std::sync::Arc::new(callback));
    }

    /// Prepare the embedded ACP view to be hidden behind another main-panel
    /// surface while keeping its live thread/session intact for reuse.
    pub(crate) fn prepare_for_host_hide(&mut self, cx: &mut Context<Self>) {
        self.attach_menu_open = false;
        self.model_selector_open = false;
        self.permission_options_open = false;
        self.mention_session = None;
        self.history_menu = None;
        self.setup_agent_picker = None;
        self.pending_history_portal_query = None;
        self.sync_acp_popup_windows_from_cached_parent(cx);
        cx.notify();
    }

    fn trigger_open_history_command(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(callback) = self.on_open_history_command.clone() {
            Self::spawn_footer_callback(callback, window, cx);
        } else {
            tracing::info!(
                target: "script_kit::acp",
                event = "acp_history_command_no_callback",
                "Cmd+P history command request dropped — no host callback installed"
            );
        }
    }

    pub(super) fn sync_mention_popup_window_from_cached_parent(&mut self, cx: &mut Context<Self>) {
        let Some(parent) = self.mention_popup_parent_window else {
            crate::ai::acp::picker_popup::close_mention_popup_window(cx);
            return;
        };

        let source_view = cx.entity().downgrade();
        if let Some((snapshot, left, top)) = self.mention_popup_snapshot(cx) {
            let _ = crate::ai::acp::picker_popup::sync_mention_popup_window(
                cx,
                crate::ai::acp::picker_popup::AcpMentionPopupRequest {
                    parent_window_handle: parent.handle,
                    parent_bounds: parent.bounds,
                    display_id: parent.display_id,
                    source_view,
                    snapshot,
                    left,
                    top,
                },
            );
        } else {
            crate::ai::acp::picker_popup::close_mention_popup_window(cx);
        }
    }

    /// Convert recent history entries into neutral hits (score 0, Title field).
    fn recent_history_hits() -> Vec<super::history::AcpHistorySearchHit> {
        super::history::load_history()
            .into_iter()
            .map(|entry| super::history::AcpHistorySearchHit {
                entry,
                score: 0,
                matched_field: super::history::AcpHistorySearchField::Title,
            })
            .collect()
    }

    fn history_popup_snapshot(
        &self,
    ) -> Option<crate::ai::acp::history_popup::AcpHistoryPopupSnapshot> {
        let menu = self.history_menu.as_ref()?;
        let entries = menu
            .hits
            .iter()
            .cloned()
            .map(crate::ai::acp::history_popup::AcpHistoryPopupEntry::from_hit)
            .collect::<Vec<_>>();
        let selected_index = if entries.is_empty() {
            0
        } else {
            menu.selected_index.min(entries.len().saturating_sub(1))
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_popup_snapshot_built",
            query = %menu.query,
            hit_count = menu.hits.len(),
            visible_count = entries.len(),
            selected_index,
        );

        Some(crate::ai::acp::history_popup::AcpHistoryPopupSnapshot {
            title: if menu.query.trim().is_empty() {
                SharedString::from("Recent Conversations (⌘P)")
            } else {
                SharedString::from(format!("History matches \u{201c}{}\u{201d}", menu.query))
            },
            query: SharedString::from(menu.query.clone()),
            selected_index,
            entries,
        })
    }

    pub(super) fn sync_history_popup_window_from_cached_parent(&mut self, cx: &mut Context<Self>) {
        let Some(parent) = self.mention_popup_parent_window else {
            crate::ai::acp::history_popup::close_history_popup_window(cx);
            return;
        };

        let source_view = cx.entity().downgrade();
        if let Some(snapshot) = self.history_popup_snapshot() {
            if let Err(error) = crate::ai::acp::history_popup::sync_history_popup_window(
                cx,
                crate::ai::acp::history_popup::AcpHistoryPopupRequest {
                    parent_window_handle: parent.handle,
                    parent_bounds: parent.bounds,
                    display_id: parent.display_id,
                    source_view,
                    snapshot,
                },
            ) {
                tracing::error!(error = %error, "acp_history_popup_sync_failed");
            }
        } else {
            crate::ai::acp::history_popup::close_history_popup_window(cx);
        }
    }

    fn model_selector_popup_snapshot(
        &self,
        cx: &App,
    ) -> Option<crate::ai::acp::model_selector_popup::AcpModelSelectorPopupSnapshot> {
        if !self.model_selector_open {
            return None;
        }

        let thread = self.live_thread().read(cx);
        let selected_id = thread.selected_model_id().map(str::to_string);
        let selected_index =
            crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
                self.model_selector_selected_index,
                thread.available_models().len(),
            );
        let entries = thread
            .available_models()
            .iter()
            .map(|model| {
                let display = model
                    .display_name
                    .clone()
                    .unwrap_or_else(|| model.id.clone());
                crate::ai::acp::model_selector_popup::AcpModelSelectorPopupEntry {
                    id: model.id.clone(),
                    display: SharedString::from(display),
                    is_active: selected_id.as_deref() == Some(model.id.as_str()),
                }
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return None;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_model_selector_popup_snapshot_built",
            entry_count = entries.len(),
            selected_index,
            "Built ACP model selector popup snapshot"
        );

        Some(
            crate::ai::acp::model_selector_popup::AcpModelSelectorPopupSnapshot {
                selected_index,
                entries,
            },
        )
    }

    pub(super) fn sync_model_selector_popup_window_from_cached_parent(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let Some(parent) = self.mention_popup_parent_window else {
            crate::ai::acp::model_selector_popup::close_model_selector_popup_window(cx);
            return;
        };

        let source_view = cx.entity().downgrade();
        if let Some(snapshot) = self.model_selector_popup_snapshot(cx) {
            if let Err(error) =
                crate::ai::acp::model_selector_popup::sync_model_selector_popup_window(
                    cx,
                    crate::ai::acp::model_selector_popup::AcpModelSelectorPopupRequest {
                        parent_window_handle: parent.handle,
                        parent_bounds: parent.bounds,
                        display_id: parent.display_id,
                        source_view,
                        snapshot,
                    },
                )
            {
                tracing::error!(error = %error, "acp_model_selector_popup_sync_failed");
            }
        } else {
            crate::ai::acp::model_selector_popup::close_model_selector_popup_window(cx);
        }
    }

    pub(crate) fn select_model_from_popup(&mut self, model_id: &str, cx: &mut Context<Self>) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_model_selector_selected",
            model_id,
            "Selected ACP model from inline dropdown"
        );
        self.live_thread().update(cx, |thread, cx| {
            thread.select_model(model_id, cx);
        });
        self.reset_model_selector_selection(cx);
        self.model_selector_open = false;
        self.sync_model_selector_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    pub(crate) fn dismiss_model_selector_popup(&mut self, cx: &mut Context<Self>) {
        if !self.model_selector_open {
            return;
        }

        self.model_selector_open = false;
        self.sync_model_selector_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    pub(crate) fn move_model_selector_selection(&mut self, direction: i32, cx: &mut Context<Self>) {
        let model_count = self.live_thread().read(cx).available_models().len();
        if model_count == 0 {
            return;
        }

        let selected_index = self.selected_model_popup_index(cx);
        self.model_selector_selected_index = if direction < 0 {
            crate::components::inline_dropdown::inline_dropdown_select_prev(
                selected_index,
                model_count,
            )
        } else {
            crate::components::inline_dropdown::inline_dropdown_select_next(
                selected_index,
                model_count,
            )
        };

        let visible = crate::components::inline_dropdown::inline_dropdown_visible_range(
            self.model_selector_selected_index,
            model_count,
            crate::ai::acp::popup_window::DENSE_PICKER_MAX_VISIBLE_ROWS,
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_model_selector_selection_moved",
            direction,
            selected_index = self.model_selector_selected_index,
            model_count,
            visible_start = visible.start,
            visible_end = visible.end,
            "Moved ACP model selector selection"
        );

        self.sync_model_selector_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    pub(crate) fn confirm_model_selector_selection(&mut self, cx: &mut Context<Self>) {
        let selected_index = self.selected_model_popup_index(cx);
        let model_id = self
            .live_thread()
            .read(cx)
            .available_models()
            .get(selected_index)
            .map(|model| model.id.clone());

        if let Some(model_id) = model_id {
            self.select_model_from_popup(&model_id, cx);
        } else {
            self.dismiss_model_selector_popup(cx);
        }
    }

    pub(crate) fn select_history_from_popup(
        &mut self,
        entry: &super::history::AcpHistoryEntry,
        cx: &mut Context<Self>,
    ) {
        self.history_menu = None;
        self.sync_history_popup_window_from_cached_parent(cx);
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
        cx.notify();
    }

    fn set_history_popup_query(&mut self, query: String, cx: &mut Context<Self>) {
        let hits = super::history::search_history(&query, HISTORY_POPUP_SEARCH_LIMIT);
        self.history_closed_at = None;
        self.history_menu = Some(AcpHistoryMenuState {
            selected_index: 0,
            query,
            hits,
        });
        self.sync_history_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    fn navigate_history_popup_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let Some(menu) = self.history_menu.as_mut() else {
            return;
        };
        if menu.hits.is_empty() {
            return;
        }

        let len = menu.hits.len();
        let current = menu.selected_index;
        menu.selected_index = if delta < 0 {
            current.saturating_sub((-delta) as usize)
        } else {
            (current + delta as usize).min(len.saturating_sub(1))
        };
        self.history_closed_at = None;
        self.sync_history_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    fn jump_history_popup_selection(&mut self, end: bool, cx: &mut Context<Self>) {
        let Some(menu) = self.history_menu.as_mut() else {
            return;
        };
        if menu.hits.is_empty() {
            return;
        }

        menu.selected_index = if end {
            menu.hits.len().saturating_sub(1)
        } else {
            0
        };
        self.history_closed_at = None;
        self.sync_history_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    fn page_history_popup_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let Some(menu) = self.history_menu.as_mut() else {
            return;
        };
        if menu.hits.is_empty() {
            return;
        }

        let len = menu.hits.len();
        menu.selected_index = if delta < 0 {
            menu.selected_index.saturating_sub(HISTORY_POPUP_PAGE_JUMP)
        } else {
            (menu.selected_index + HISTORY_POPUP_PAGE_JUMP).min(len.saturating_sub(1))
        };
        self.history_closed_at = None;
        self.sync_history_popup_window_from_cached_parent(cx);
        cx.notify();
    }

    fn execute_history_popup_selection(
        &mut self,
        modifiers: &gpui::Modifiers,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self
            .history_menu
            .as_ref()
            .and_then(|menu| menu.hits.get(menu.selected_index))
            .map(|hit| hit.entry.clone())
        else {
            return;
        };

        self.history_menu = None;
        self.history_closed_at = None;
        self.sync_history_popup_window_from_cached_parent(cx);

        if modifiers.platform {
            self.select_history_from_popup(&entry, cx);
            return;
        }

        let mode = if modifiers.shift {
            super::history_attachment::AcpHistoryAttachMode::Transcript
        } else {
            super::history_attachment::AcpHistoryAttachMode::Summary
        };

        if let Err(error) = self.attach_history_session(&entry.session_id, mode, cx) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_history_popup_attach_failed",
                session_id = %entry.session_id,
                mode = ?mode,
                error = %error,
            );
        }
        cx.notify();
    }

    /// Attach a prior conversation as a context chip via the existing file attachment path.
    pub(crate) fn attach_history_session(
        &mut self,
        session_id: &str,
        mode: super::history_attachment::AcpHistoryAttachMode,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let (path, label) = super::history_attachment::write_history_attachment(session_id, mode)?;
        let display_path = path.to_string_lossy().to_string();

        self.live_thread().update(cx, |thread, cx| {
            thread.add_context_part(
                AiContextPart::FilePath {
                    path: display_path.clone(),
                    label: label.clone(),
                },
                cx,
            );
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_attachment_added",
            session_id = %session_id,
            mode = ?mode,
            path = %display_path,
            label = %label,
        );

        cx.notify();
        Ok(())
    }

    /// Take the pending history portal query (consumed once by the portal opener).
    pub(crate) fn take_pending_history_portal_query(&mut self) -> Option<String> {
        self.pending_history_portal_query.take()
    }

    /// Open the history popup pre-seeded with search hits from the portal.
    pub(crate) fn open_history_portal_with_entries(
        &mut self,
        query: String,
        hits: Vec<super::history::AcpHistorySearchHit>,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_portal_opened",
            query = %query,
            hit_count = hits.len(),
        );
        self.attach_menu_open = false;
        self.model_selector_open = false;
        self.mention_session = None;
        self.history_closed_at = None;
        self.history_menu = Some(AcpHistoryMenuState {
            selected_index: 0,
            query,
            hits,
        });
        self.sync_acp_popup_windows_from_cached_parent(cx);
        cx.notify();
    }

    pub(crate) fn sync_history_popup_state_from_window(
        &mut self,
        query: String,
        hits: Vec<super::history::AcpHistorySearchHit>,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) {
        if self.history_menu.is_none() {
            return;
        }

        let clamped_selected_index = if hits.is_empty() {
            0
        } else {
            selected_index.min(hits.len().saturating_sub(1))
        };

        self.history_closed_at = None;
        self.history_menu = Some(AcpHistoryMenuState {
            selected_index: clamped_selected_index,
            query,
            hits,
        });
        cx.notify();
    }

    pub(crate) fn sync_history_popup_selection_from_window(
        &mut self,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(menu) = self.history_menu.as_mut() else {
            return;
        };

        menu.selected_index = if menu.hits.is_empty() {
            0
        } else {
            selected_index.min(menu.hits.len().saturating_sub(1))
        };
        self.history_closed_at = None;
        cx.notify();
    }

    pub(crate) fn open_history_popup_from_host(
        &mut self,
        parent_handle: gpui::AnyWindowHandle,
        parent_bounds: gpui::Bounds<gpui::Pixels>,
        display_id: Option<gpui::DisplayId>,
        cx: &mut Context<Self>,
    ) {
        self.mention_popup_parent_window = Some(AcpMentionPopupParentWindow {
            handle: parent_handle,
            bounds: parent_bounds,
            display_id,
        });

        if self.history_menu.is_none() {
            let hits = Self::recent_history_hits();
            if hits.is_empty() {
                self.sync_history_popup_window_from_cached_parent(cx);
                cx.notify();
                return;
            }

            self.attach_menu_open = false;
            self.model_selector_open = false;
            self.mention_session = None;
            self.history_closed_at = None;
            self.history_menu = Some(AcpHistoryMenuState {
                selected_index: 0,
                query: String::new(),
                hits,
            });
        }

        self.sync_acp_popup_windows_from_cached_parent(cx);
        cx.notify();
    }

    fn toggle_history_popup_from_cached_parent(&mut self, cx: &mut Context<Self>) {
        if self.history_menu.is_some() {
            self.dismiss_history_popup(cx);
            return;
        }

        if self.was_history_recently_closed() {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_history_popup_toggle_suppressed_recent_close",
                "Suppressed ACP history popup reopen because it was just closed"
            );
            return;
        } else {
            let hits = Self::recent_history_hits();
            if !hits.is_empty() {
                self.attach_menu_open = false;
                self.model_selector_open = false;
                self.mention_session = None;
                self.history_closed_at = None;
                self.history_menu = Some(AcpHistoryMenuState {
                    selected_index: 0,
                    query: String::new(),
                    hits,
                });
            }
        }
        self.sync_acp_popup_windows_from_cached_parent(cx);
        cx.notify();
    }

    pub(crate) fn toggle_history_popup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cache_popup_parent_window(window, cx);
        self.toggle_history_popup_from_cached_parent(cx);
    }

    pub(crate) fn dismiss_escape_popup(&mut self, cx: &mut Context<Self>) -> bool {
        if self.model_selector_open {
            self.dismiss_model_selector_popup(cx);
            return true;
        }

        if self.mention_session.is_some() {
            self.mention_session = None;
            self.sync_mention_popup_window_from_cached_parent(cx);
            cx.notify();
            return true;
        }

        if self.history_menu.is_some() {
            self.dismiss_history_popup(cx);
            return true;
        }

        false
    }

    pub(crate) fn has_escape_dismissible_popup(&self) -> bool {
        self.model_selector_open || self.mention_session.is_some() || self.history_menu.is_some()
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

        // Build setup snapshot from either session mode.
        let setup_snapshot = self.build_setup_protocol_snapshot(cx);

        if self.is_setup_mode() {
            let snapshot = AcpStateSnapshot {
                status: "setup".to_string(),
                setup: setup_snapshot,
                ..Default::default()
            };

            if let Some(ref setup) = snapshot.setup {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_snapshot_built",
                    reason_code = %setup.reason_code,
                    primary_action = ?setup.primary_action,
                    compatible_count = setup.compatible_agent_ids.len(),
                    agent_picker_open = setup.agent_picker_open,
                );
            }

            return snapshot;
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

        // If the live thread has a runtime recovery setup card, include it.
        let live_setup = if thread.setup_state().is_some() {
            setup_snapshot
        } else {
            None
        };

        let context_ready = thread.context_bootstrap_state() != AcpContextBootstrapState::Preparing;

        AcpStateSnapshot {
            schema_version: ACP_STATE_SCHEMA_VERSION,
            resolved_target: None, // Populated by the caller (prompt handler) based on target resolution.
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
            setup: live_setup,
            warnings: Vec::new(),
        }
    }

    /// Build a protocol-layer setup snapshot from the current session state.
    fn build_setup_protocol_snapshot(&self, cx: &App) -> Option<crate::protocol::AcpSetupSnapshot> {
        let (agent_picker_open, agent_picker_selected_id) =
            if let Some(ref picker) = self.setup_agent_picker {
                let selected_id = picker
                    .items
                    .get(picker.selected_index)
                    .map(|entry| entry.id.to_string());
                (true, selected_id)
            } else {
                (false, None)
            };

        match &self.session {
            AcpChatSession::Setup(setup) => {
                Some(setup.to_protocol_snapshot(agent_picker_open, agent_picker_selected_id))
            }
            AcpChatSession::Live(thread) => {
                let t = thread.read(cx);
                t.setup_state()
                    .map(|s| s.to_protocol_snapshot(agent_picker_open, agent_picker_selected_id))
            }
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
                    view.refresh_mention_session(cx);
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
            history_closed_at: None,
            attach_menu_open: false,
            model_selector_open: false,
            model_selector_selected_index: 0,
            search_state: None,
            cached_slash_commands: Vec::new(),
            _slash_discovery_task: slash_task,
            mention_session: None,
            mention_popup_parent_window: None,
            inline_owned_context_tokens: HashSet::new(),
            typed_mention_aliases: std::collections::HashMap::new(),
            setup_agent_picker: None,
            last_accepted_item: None,
            test_probe: AcpTestProbe::default(),
            pending_retry_request: None,
            pending_history_resume: None,
            on_toggle_actions: None,
            on_close_requested: None,
            on_open_history_command: None,
            on_open_portal: None,
            pending_history_portal_query: None,
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
            history_closed_at: None,
            attach_menu_open: false,
            model_selector_open: false,
            model_selector_selected_index: 0,
            search_state: None,
            cached_slash_commands: Vec::new(),
            _slash_discovery_task: noop_slash,
            mention_session: None,
            mention_popup_parent_window: None,
            inline_owned_context_tokens: HashSet::new(),
            typed_mention_aliases: std::collections::HashMap::new(),
            setup_agent_picker: None,
            last_accepted_item: None,
            test_probe: AcpTestProbe::default(),
            pending_retry_request: None,
            pending_history_resume: None,
            on_toggle_actions: None,
            on_close_requested: None,
            on_open_history_command: None,
            on_open_portal: None,
            pending_history_portal_query: None,
        }
    }

    /// Scan plugin skill directories for slash command candidates, combine with
    /// built-in Claude Code commands. Returns typed `SlashCommandEntry` entries
    /// with full source identity.
    ///
    /// Uses `discover_plugin_skills()` so skill enumeration is routed through
    /// plugin ownership instead of hand-scanning `kit/*/skills/`.
    fn discover_slash_commands() -> Vec<SlashCommandEntry> {
        let mut commands: Vec<SlashCommandEntry> = Self::DEFAULT_SLASH_COMMANDS
            .iter()
            .map(|s| SlashCommandEntry::default_command(s))
            .collect();

        let mut seen: std::collections::HashSet<String> =
            commands.iter().map(|e| e.qualified_key()).collect();

        // Seed collision tracker with default slash names so plugin/Claude
        // collisions against built-ins are detected.
        let default_names: std::collections::HashSet<String> =
            commands.iter().map(|e| e.name.clone()).collect();
        let mut owners_by_slash: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for entry in &commands {
            owners_by_slash
                .entry(entry.name.clone())
                .or_default()
                .push(entry.source.owner_label());
        }

        // Track plugin slash names for Claude-vs-plugin collision detection.
        let mut plugin_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Ok(index) = crate::plugins::discover_plugins() {
            if let Ok(skills) = crate::plugins::discover_plugin_skills(&index) {
                for skill in &skills {
                    let entry = SlashCommandEntry::plugin_skill(skill);
                    let owner = entry.source.owner_label();

                    plugin_names.insert(entry.name.clone());
                    owners_by_slash
                        .entry(entry.name.clone())
                        .or_default()
                        .push(owner);

                    if default_names.contains(&entry.name) {
                        tracing::warn!(
                            plugin_id = %skill.plugin_id,
                            skill_id = %skill.skill_id,
                            slash_name = %entry.name,
                            "acp_slash_plugin_collides_with_default"
                        );
                    }

                    if seen.insert(entry.qualified_key()) {
                        tracing::info!(
                            plugin_id = %skill.plugin_id,
                            skill_id = %skill.skill_id,
                            "acp_slash_skill_cataloged"
                        );
                        commands.push(entry);
                    }
                }
            }
        }

        // Also scan .claude/skills for user-level Claude Code skills
        let kit_path = crate::setup::get_kit_path();
        let claude_skills_dir = kit_path.join(".claude").join("skills");
        if let Ok(entries) = std::fs::read_dir(&claude_skills_dir) {
            for entry in entries.flatten() {
                let skill_md = entry.path().join("SKILL.md");
                if !skill_md.exists() {
                    continue;
                }
                let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                    continue;
                };

                let desc = std::fs::read_to_string(&skill_md)
                    .ok()
                    .and_then(|content| parse_skill_description(&content))
                    .unwrap_or_default();

                let slash_entry =
                    SlashCommandEntry::claude_code_skill(name.clone(), desc, skill_md);

                owners_by_slash
                    .entry(name.clone())
                    .or_default()
                    .push("Claude Code".to_string());

                if plugin_names.contains(&name) {
                    tracing::warn!(
                        skill_id = %name,
                        "acp_slash_claude_collides_with_plugin"
                    );
                }
                if default_names.contains(&name) {
                    tracing::warn!(
                        skill_id = %name,
                        "acp_slash_claude_collides_with_default"
                    );
                }

                if seen.insert(slash_entry.qualified_key()) {
                    commands.push(slash_entry);
                }
            }
        }

        // Final cross-source collision pass: warn when multiple distinct
        // owners share the same bare slash name.
        for (slash_name, owners) in &owners_by_slash {
            if owners.len() > 1 {
                tracing::warn!(
                    slash_name = %slash_name,
                    owners = ?owners,
                    "acp_slash_skill_name_collision"
                );
            }
        }

        tracing::info!(count = commands.len(), "acp_slash_entries_discovered");
        commands
    }

    /// Resolve cached slash commands against the agent-reported available
    /// commands. Plugin and Claude skills are always included regardless
    /// of provider advertisement; only default commands are gated.
    fn resolved_slash_commands(&self, available_commands: &[String]) -> Vec<SlashCommandEntry> {
        if available_commands.is_empty() {
            return self.cached_slash_commands.clone();
        }

        let available_set: std::collections::HashSet<&str> =
            available_commands.iter().map(|s| s.as_str()).collect();

        let mut result = Vec::new();

        for entry in &self.cached_slash_commands {
            match &entry.source {
                // Default commands are only included if the provider advertises them.
                SlashCommandSource::Default if available_set.contains(entry.name.as_str()) => {
                    result.push(entry.clone());
                }
                // Plugin and Claude skills are always included.
                SlashCommandSource::PluginSkill(_) | SlashCommandSource::ClaudeCodeSkill { .. } => {
                    result.push(entry.clone());
                }
                _ => {}
            }
        }

        // Include agent-reported commands that aren't in our cache
        for cmd in available_commands {
            let already_present = result.iter().any(|entry| {
                matches!(entry.source, SlashCommandSource::Default) && entry.name == *cmd
            });
            if !already_present {
                result.push(SlashCommandEntry::default_command(cmd));
            }
        }

        result
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
            let permission_active = self.live_thread().read(cx).pending_permission.is_some();
            self.emit_key_route_telemetry(
                "tab",
                AcpKeyRouteTelemetryArgs {
                    route: crate::protocol::AcpKeyRoute::Picker,
                    cursor_before,
                    cursor_after,
                    caused_submit: false,
                    consumed: true,
                    permission_active,
                },
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

    fn open_picker_trigger(&mut self, trigger: &str, cx: &mut Context<Self>) {
        self.attach_menu_open = false;
        self.model_selector_open = false;
        self.history_menu = None;
        self.set_input(trigger.to_string(), cx);
        self.refresh_mention_session(cx);
    }

    pub(crate) fn open_slash_picker(&mut self, cx: &mut Context<Self>) {
        self.open_picker_trigger("/", cx);
    }

    pub(crate) fn open_mention_picker(&mut self, cx: &mut Context<Self>) {
        self.open_picker_trigger("@", cx);
    }

    pub(crate) fn open_slash_picker_in_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cache_popup_parent_window(window, cx);
        self.open_slash_picker(cx);
    }

    pub(crate) fn open_mention_picker_in_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cache_popup_parent_window(window, cx);
        self.open_mention_picker(cx);
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
    #[allow(dead_code)]
    fn render_pending_context_chips(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use crate::ai::context_mentions::visible_context_chip_indices;

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

        let chip_indices = visible_context_chip_indices(&input_text, &parts);
        let chip_parts: Vec<(usize, &AiContextPart)> = chip_indices
            .into_iter()
            .filter_map(|ix| parts.get(ix).map(|part| (ix, part)))
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

        let mut container = div()
            .id("acp-pending-context-chips")
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(6.0))
            .px(px(12.0))
            .pb(px(6.0));

        for (chip_idx, &(remove_idx, part)) in chip_parts.iter().enumerate() {
            let label = SharedString::from(part.label().to_string());
            let remove_id =
                ElementId::Name(SharedString::from(format!("acp-ctx-remove-{chip_idx}")));

            let chip = div()
                .id(ElementId::Name(SharedString::from(format!(
                    "acp-ctx-chip-{chip_idx}"
                ))))
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
                                .id(remove_id)
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

            container = container.child(chip);
        }

        container.into_any_element()
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
                    // Active model label
                    .child({
                        let model_display = self
                            .live_thread()
                            .read(cx)
                            .selected_model_display()
                            .to_string();
                        div()
                            .id("acp-model-display")
                            .flex()
                            .items_center()
                            .text_xs()
                            .text_color(rgba(hint_text_rgba))
                            .child(model_display)
                    }),
            )
            // ── Right: clickable hint strip (matches main menu behavior) ──
            .child(crate::components::render_hint_icons_clickable(
                vec![
                    crate::components::ClickableHint::new(
                        "↩ Send",
                        cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
                            this.submit_with_expanded_tokens(cx);
                        }),
                    ),
                    crate::components::ClickableHint::new(
                        "⌘P History",
                        cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                            tracing::info!(
                                target: "script_kit::tab_ai",
                                event = "acp_toolbar_history_clicked",
                            );
                            this.trigger_open_history_command(window, cx);
                        }),
                    ),
                    crate::components::ClickableHint::new(
                        "⌘K Actions",
                        cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                            this.trigger_toggle_actions(window, cx);
                        }),
                    ),
                    crate::components::ClickableHint::new(
                        "⌘W Close",
                        cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
                            this.trigger_close_requested(_window, cx);
                        }),
                    ),
                ],
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
                    this.submit_with_expanded_tokens(cx);
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
    pub(super) fn refresh_mention_session(&mut self, cx: &mut Context<Self>) {
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
                        if self.cached_slash_commands.is_empty() {
                            // Async discovery hasn't completed yet — show
                            // intentional loading row instead of blank list.
                            vec![slash_picker_loading_row()]
                        } else {
                            let entries = if available_commands.is_empty() {
                                self.cached_slash_commands.clone()
                            } else {
                                self.resolved_slash_commands(&available_commands)
                            };
                            if entries.is_empty() {
                                // Discovery completed but catalog is empty
                                // (no defaults, no plugins, no Claude skills).
                                vec![slash_picker_empty_row()]
                            } else {
                                let payloads: Vec<(SlashCommandPayload, String)> = entries
                                    .iter()
                                    .map(|e| (e.to_payload(), e.description.clone()))
                                    .collect();
                                let mut items = build_slash_picker_items_with_payloads(
                                    &query,
                                    payloads.iter().map(|(p, d)| (p, d.as_str())),
                                );
                                if items.is_empty() {
                                    // Non-empty catalog filtered to zero by
                                    // query — distinct from empty catalog.
                                    items.push(slash_picker_no_match_row());
                                }
                                items
                            }
                        }
                    }
                };

                let selected_index =
                    crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
                        previous_index,
                        items.len(),
                    );
                let visible = Self::mention_visible_range_for(selected_index, items.len());
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_mention_picker_refreshed",
                    layout = "inline_dropdown",
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
            self.attach_menu_open = false;
            self.model_selector_open = false;
            self.history_menu = None;
        }

        self.mention_session = next_session;
        self.log_mention_visible_range("refresh");
        self.sync_acp_popup_windows_from_cached_parent(cx);
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

    /// Apply a hint chip token by inserting it at the cursor (or replacing
    /// the active trigger) and running it through the normal picker acceptance
    /// path. Preserves surrounding composer text.
    pub(super) fn apply_picker_hint_token(&mut self, token: &str, cx: &mut Context<Self>) {
        let (text, cursor) = {
            let thread = self.live_thread().read(cx);
            (thread.input.text().to_string(), thread.input.cursor())
        };

        let (next_text, next_cursor) =
            Self::replace_active_trigger_or_insert_at_cursor(&text, cursor, token);

        self.live_thread().update(cx, |thread, cx| {
            thread.input.set_text(next_text);
            thread.input.set_cursor(next_cursor);
            cx.notify();
        });
        self.refresh_mention_session(cx);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_picker_hint_applied",
            token,
            has_session = self.mention_session.is_some(),
            cursor_after = next_cursor,
        );
        if self.mention_session.is_some() {
            self.accept_mention_selection_impl(false, cx);
        } else {
            self.sync_inline_mentions(cx);
            self.mention_session = None;
            self.sync_mention_popup_window_from_cached_parent(cx);
            cx.notify();
        }
    }

    pub(super) fn insert_picker_hint_prefix(&mut self, prefix: &str, cx: &mut Context<Self>) {
        let (text, cursor) = {
            let thread = self.live_thread().read(cx);
            (thread.input.text().to_string(), thread.input.cursor())
        };

        let (next_text, next_cursor) =
            Self::replace_active_trigger_or_insert_at_cursor(&text, cursor, prefix);

        self.live_thread().update(cx, |thread, cx| {
            thread.input.set_text(next_text);
            thread.input.set_cursor(next_cursor);
            cx.notify();
        });
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_picker_hint_prefix_inserted",
            prefix,
            cursor_after = next_cursor,
        );
        self.refresh_mention_session(cx);
        self.sync_inline_mentions(cx);
        self.sync_mention_popup_window_from_cached_parent(cx);
    }

    /// Accept the currently selected picker row.
    ///
    /// Both Enter and Tab autocomplete the focused picker row. Literal slash
    /// commands are inserted into the composer; slash-picked context items
    /// attach a pending context part and remove the typed `/query` token.
    pub(crate) fn accept_mention_selection(&mut self, cx: &mut Context<Self>) {
        self.accept_mention_selection_impl(false, cx);
    }

    pub(crate) fn select_mention_index(&mut self, index: usize) {
        if let Some(session) = self.mention_session.as_mut() {
            if !session.items.is_empty() {
                session.selected_index = index.min(session.items.len().saturating_sub(1));
            }
        }
    }

    /// Insert `replacement` at the cursor, replacing the active trigger range
    /// if one is found. Preserves surrounding text and returns the updated
    /// text plus the new cursor position.
    fn replace_active_trigger_or_insert_at_cursor(
        text: &str,
        cursor: usize,
        replacement: &str,
    ) -> (String, usize) {
        let content = replacement.trim();
        let wants_trailing_space = replacement.chars().last().is_some_and(char::is_whitespace);

        match Self::find_active_trigger(text, cursor) {
            Some((_trigger, trigger_range, _query)) => {
                let mut inserted = content.to_string();
                if wants_trailing_space {
                    inserted.push(' ');
                }
                let cursor_after = trigger_range.start + inserted.chars().count();
                let next_text = Self::replace_text_in_char_range(text, trigger_range, &inserted);
                (next_text, cursor_after)
            }
            None => {
                let prev = cursor.checked_sub(1).and_then(|ix| text.chars().nth(ix));
                let next = text.chars().nth(cursor);
                let mut formatted = String::new();
                if prev.is_some_and(|ch| !ch.is_whitespace()) {
                    formatted.push(' ');
                }
                formatted.push_str(content);
                if wants_trailing_space || next.is_some_and(|ch| !ch.is_whitespace()) {
                    formatted.push(' ');
                }
                let cursor_after = cursor + formatted.trim_end().chars().count();
                let next_text = Self::replace_text_in_char_range(text, cursor..cursor, &formatted);
                (next_text, cursor_after)
            }
        }
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

        // Inert items (loading spinner, empty state) are non-actionable.
        if matches!(item.kind, ContextPickerItemKind::Inert) {
            tracing::debug!(item_id = %item.id, "acp_picker_inert_item_ignored");
            // Restore session so the picker stays open.
            self.mention_session = Some(session);
            return;
        }

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

        // ── Slash command acceptance: default inserts text, skills stage content ──
        if session.trigger == ContextPickerTrigger::Slash {
            if let ContextPickerItemKind::SlashCommand(ref payload) = item.kind {
                match payload {
                    SlashCommandPayload::Default { name } => {
                        // Default commands insert literal `/command ` text.
                        let current_text = self.live_thread().read(cx).input.text().to_string();
                        let command_text = format!("/{name} ");
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
                            slash_name = %name,
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
                    }
                    SlashCommandPayload::PluginSkill(skill) => {
                        // Plugin skills stage local <skill> content.
                        let owner = if skill.plugin_title.is_empty() {
                            &skill.plugin_id
                        } else {
                            &skill.plugin_title
                        };
                        let staged = build_staged_skill_prompt(&skill.title, owner, &skill.path);
                        tracing::info!(
                            plugin_id = %skill.plugin_id,
                            skill_id = %skill.skill_id,
                            "acp_slash_skill_selected"
                        );
                        self.live_thread().update(cx, |thread, cx| {
                            thread.input.set_text(staged);
                            thread.input.set_cursor(0);
                            cx.notify();
                        });
                    }
                    SlashCommandPayload::ClaudeCodeSkill {
                        skill_id,
                        skill_path,
                    } => {
                        // Claude Code skills stage local <skill> content.
                        let staged = build_staged_skill_prompt(skill_id, "Claude Code", skill_path);
                        tracing::info!(
                            skill_id = %skill_id,
                            path = %skill_path.display(),
                            "acp_slash_claude_skill_selected"
                        );
                        self.live_thread().update(cx, |thread, cx| {
                            thread.input.set_text(staged);
                            thread.input.set_cursor(0);
                            cx.notify();
                        });
                    }
                }
                self.sync_mention_popup_window_from_cached_parent(cx);
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
            ContextPickerItemKind::SlashCommand(_) | ContextPickerItemKind::Inert => return,
            ContextPickerItemKind::Portal(portal_kind) => {
                // For AcpHistory portals, stash the remaining input text as the
                // prefilter query before clearing the trigger text.
                if *portal_kind == crate::ai::window::context_picker::types::PortalKind::AcpHistory
                {
                    let current_text = self.live_thread().read(cx).input.text().to_string();
                    let remaining = Self::replace_text_in_char_range(
                        &current_text,
                        session.trigger_range.clone(),
                        "",
                    );
                    let query = remaining.trim().to_string();
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_history_portal_query_staged",
                        query = %query,
                    );
                    self.pending_history_portal_query = Some(query);
                }

                // Remove the trigger text (@file, @clip, etc.) from the input
                // before opening the portal so it doesn't linger on return.
                let current_text = self.live_thread().read(cx).input.text().to_string();
                let cleaned = Self::replace_text_in_char_range(
                    &current_text,
                    session.trigger_range.clone(),
                    "",
                );
                let cleaned_cursor = session.trigger_range.start;
                self.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text(cleaned);
                    thread.input.set_cursor(cleaned_cursor);
                    cx.notify();
                });

                // Dismiss the mention popup and invoke the host portal callback.
                // Deferred via App::defer to release the AcpChatView entity borrow.
                self.sync_mention_popup_window_from_cached_parent(cx);
                if let Some(callback) = self.on_open_portal.clone() {
                    let kind = *portal_kind;
                    cx.defer(move |cx| {
                        callback(kind, cx);
                    });
                }
                cx.notify();
                return;
            }
        };

        let current_text = self.live_thread().read(cx).input.text().to_string();

        // Decide ownership *before* mutating the thread — the check reads
        // the current pending_context_parts to see if the part was already
        // attached from a non-inline source (slash, chip, setup).
        let should_claim_inline_ownership = if allow_inline_sync {
            self.should_claim_inline_mention_ownership(&part, cx)
        } else {
            false
        };

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

        // Register typed alias for non-builtin parts so the parser can
        // resolve typed @type:name tokens back to the full AiContextPart.
        if matches!(
            item.kind,
            ContextPickerItemKind::File(_) | ContextPickerItemKind::Folder(_)
        ) {
            if let Some(token) = part_to_inline_token(&part) {
                self.typed_mention_aliases.insert(token, part.clone());
            }
        }

        if allow_inline_sync {
            if let Some(token) = part_to_inline_token(&part) {
                if should_claim_inline_ownership {
                    self.inline_owned_context_tokens.insert(token.clone());
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_inline_mention_ownership_claimed",
                        token = %token,
                        item_id = %item.id,
                        item_label = %item.label,
                    );
                } else {
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_inline_mention_ownership_preserved_existing_attachment",
                        token = %token,
                        item_id = %item.id,
                        item_label = %item.label,
                    );
                }
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
        self.sync_mention_popup_window_from_cached_parent(cx);
    }

    /// Check whether accepting a picker item should claim inline ownership
    /// of the resulting token.  Delegates to the shared helper in
    /// `context_mentions::should_claim_inline_mention_ownership`.
    fn should_claim_inline_mention_ownership(
        &self,
        part: &crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) -> bool {
        crate::ai::context_mentions::should_claim_inline_mention_ownership(
            part,
            self.live_thread().read(cx).pending_context_parts(),
            &self.inline_owned_context_tokens,
        )
    }

    /// Return highlight ranges for inline `@mentions` that are **actually
    /// attached** as pending context parts. Unattached lookalike tokens are
    /// not highlighted.
    fn attached_inline_mention_highlight_ranges(
        text: &str,
        attached_parts: &[AiContextPart],
        accent_color: u32,
        aliases: &std::collections::HashMap<String, AiContextPart>,
    ) -> Vec<TextHighlightRange> {
        use crate::ai::context_mentions::{
            parse_inline_context_mentions_with_aliases, part_to_inline_token,
        };

        let attached_tokens: HashSet<String> = attached_parts
            .iter()
            .filter_map(part_to_inline_token)
            .collect();

        parse_inline_context_mentions_with_aliases(text, aliases)
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
        let text = self.live_thread().read(cx).input.text().to_string();
        let attached_parts = self.live_thread().read(cx).pending_context_parts().to_vec();

        let plan = crate::ai::context_mentions::build_inline_mention_sync_plan_with_aliases(
            &text,
            &attached_parts,
            &self.inline_owned_context_tokens,
            &self.typed_mention_aliases,
        );

        self.live_thread().update(cx, |thread, cx| {
            for ix in plan.stale_indices.iter().rev().copied() {
                thread.remove_context_part(ix, cx);
            }
            for part in &plan.added_parts {
                thread.add_context_part(part.clone(), cx);
            }
        });

        self.inline_owned_context_tokens
            .retain(|token| plan.desired_tokens.contains(token));
        self.inline_owned_context_tokens
            .extend(plan.added_tokens.iter().cloned());

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_inline_mentions_synced",
            desired_count = plan.desired_parts.len(),
            added_count = plan.added_parts.len(),
            removed_count = plan.stale_indices.len(),
            token_count = self.inline_owned_context_tokens.len(),
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

    /// Approximate glyph width used for popup anchoring and visible-window math.
    const ACP_INPUT_APPROX_CHAR_WIDTH: f32 = 8.5;

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

    fn measure_acp_input_prefix_width(prefix: &str) -> f32 {
        if prefix.is_empty() {
            return 0.0;
        }

        prefix.chars().count() as f32 * Self::ACP_INPUT_APPROX_CHAR_WIDTH
    }

    /// Returns the maximum text wrapping width for the ACP composer.
    fn composer_wrap_width_for_window(window_width: f32) -> f32 {
        (window_width - (Self::ACP_INPUT_PADDING_X * 2.0)).max(1.0)
    }

    /// Returns the ACP composer cursor position `(x, y)` after rendering `text`,
    /// accounting for explicit newlines and simple visual wrapping.
    fn measure_acp_input_cursor_position(text: &str, window_width: f32) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, 0.0);
        }
        let wrap_width = Self::composer_wrap_width_for_window(window_width);
        let logical_lines: Vec<&str> = text.split('\n').collect();
        let mut visual_row = 0usize;
        let mut cursor_x = 0.0f32;
        for (ix, logical_line) in logical_lines.iter().enumerate() {
            let width = Self::measure_acp_input_prefix_width(logical_line);
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
        window_width: f32,
    ) -> (f32, f32, f32) {
        let picker_width = Self::mention_picker_width_for_window(window_width);
        let trigger_start_byte = Self::char_to_byte_offset(input_text, session.trigger_range.start);
        let prefix = &input_text[..trigger_start_byte];
        let trigger_text = match session.trigger {
            ContextPickerTrigger::Mention => "@",
            ContextPickerTrigger::Slash => "/",
        };
        let trigger_width = Self::measure_acp_input_prefix_width(trigger_text);
        let (after_trigger_x, after_trigger_y) = Self::measure_acp_input_cursor_position(
            &format!("{prefix}{trigger_text}"),
            window_width,
        );
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
        crate::components::inline_dropdown::inline_dropdown_visible_range(
            selected_index,
            item_count,
            Self::MENTION_PICKER_MAX_VISIBLE,
        )
    }

    /// Compute the visible range of items around the selected index.
    fn mention_visible_range(session: &AcpMentionSession) -> std::ops::Range<usize> {
        Self::mention_visible_range_for(session.selected_index, session.items.len())
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
                "Add or edit an ACP agent in ~/.scriptkit/acp/agents.json, then press Tab to retry"
                    .to_string()
            }
            super::setup_state::AcpSetupAction::SelectAgent => {
                "Press Enter to select a different agent".to_string()
            }
        };

        let secondary_hint: Option<String> = state.secondary_action.map(|action| match action {
            super::setup_state::AcpSetupAction::SelectAgent => "Enter: select agent".to_string(),
            super::setup_state::AcpSetupAction::Retry => "Tab: retry".to_string(),
            super::setup_state::AcpSetupAction::OpenCatalog => "Add agent".to_string(),
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
            .when_some(self.render_setup_agent_picker_inline(state), |d, picker| {
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

    fn setup_agent_install_label(state: super::catalog::AcpAgentInstallState) -> &'static str {
        match state {
            super::catalog::AcpAgentInstallState::Ready => "ready",
            super::catalog::AcpAgentInstallState::NeedsInstall => "install",
            super::catalog::AcpAgentInstallState::Unsupported => "unsupported",
        }
    }

    fn setup_agent_auth_label(state: super::catalog::AcpAgentAuthState) -> &'static str {
        match state {
            super::catalog::AcpAgentAuthState::Unknown => "auth?",
            super::catalog::AcpAgentAuthState::Authenticated => "authed",
            super::catalog::AcpAgentAuthState::NeedsAuthentication => "login",
        }
    }

    fn setup_agent_config_label(state: super::catalog::AcpAgentConfigState) -> &'static str {
        match state {
            super::catalog::AcpAgentConfigState::Valid => "config-ok",
            super::catalog::AcpAgentConfigState::Missing => "config-missing",
            super::catalog::AcpAgentConfigState::Invalid => "config-invalid",
        }
    }

    fn setup_agent_capability_label(
        entry: &super::catalog::AcpAgentCatalogEntry,
        requirements: super::preflight::AcpLaunchRequirements,
    ) -> Option<&'static str> {
        if !requirements.needs_embedded_context && !requirements.needs_image {
            return None;
        }
        if entry.satisfies_requirements(requirements) {
            Some("compatible")
        } else if requirements.needs_image {
            Some("image-mismatch")
        } else {
            Some("context-mismatch")
        }
    }

    fn format_setup_agent_picker_status(
        entry: &super::catalog::AcpAgentCatalogEntry,
        requirements: super::preflight::AcpLaunchRequirements,
    ) -> String {
        let mut parts = vec![
            format!("{:?}", entry.source),
            Self::setup_agent_install_label(entry.install_state).to_string(),
            Self::setup_agent_auth_label(entry.auth_state).to_string(),
            Self::setup_agent_config_label(entry.config_state).to_string(),
        ];
        if let Some(capability) = Self::setup_agent_capability_label(entry, requirements) {
            parts.push(capability.to_string());
        }
        if entry.last_session_ok {
            parts.push("last-ok".to_string());
        }
        parts.join(" \u{00b7} ")
    }

    /// Render the setup agent picker inline (non-mut version for use in render).
    fn render_setup_agent_picker_inline(
        &self,
        setup: &super::setup_state::AcpInlineSetupState,
    ) -> Option<gpui::AnyElement> {
        let picker = self.setup_agent_picker.as_ref()?;
        let theme = theme::get_cached_theme();

        let rows: Vec<gpui::AnyElement> = picker
            .items
            .iter()
            .enumerate()
            .map(|(ix, item)| {
                let is_selected = ix == picker.selected_index;
                let status_text =
                    Self::format_setup_agent_picker_status(item, setup.launch_requirements);
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

    /// Take the pending retry request, if any. Used by the ACP open path
    /// to consume an explicit relaunch payload ahead of fallback preference.
    pub(crate) fn take_retry_request(&mut self) -> Option<AcpRetryRequest> {
        self.pending_retry_request.take()
    }

    pub(crate) fn has_retry_request(&self) -> bool {
        self.pending_retry_request.is_some()
    }

    /// Stage a history resume request so the next ACP open path loads
    /// the saved conversation instead of starting fresh.
    pub(crate) fn stage_history_resume(&mut self, session_id: String, cx: &mut Context<Self>) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_history_resume_staged",
            session_id = %session_id,
        );
        self.pending_history_resume = Some(AcpHistoryResumeRequest { session_id });
        cx.notify();
    }

    /// Take the pending history resume request, if any. Used by the ACP
    /// open path to load a saved conversation by session_id.
    pub(crate) fn take_history_resume(&mut self) -> Option<AcpHistoryResumeRequest> {
        self.pending_history_resume.take()
    }

    /// Resume a conversation from history by session_id.
    ///
    /// Loads the saved conversation messages into the live thread.
    /// Returns `true` if the conversation was loaded, `false` if the
    /// saved file was not found (falls back to setting input text).
    pub(crate) fn resume_from_history(&mut self, session_id: &str, cx: &mut Context<Self>) -> bool {
        if let Some(conv) = super::history::load_conversation(session_id) {
            self.live_thread().update(cx, |thread, cx| {
                thread.load_saved_messages(&conv.messages, cx);
            });
            self.collapsed_ids.clear();
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_history_item_resumed",
                session_id = %session_id,
                message_count = conv.messages.len(),
            );
            cx.notify();
            true
        } else {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_history_resume_fallback",
                session_id = %session_id,
            );
            false
        }
    }

    /// Derive current launch requirements from whichever session mode is active.
    fn current_retry_launch_requirements(
        &self,
        cx: &mut Context<Self>,
    ) -> super::preflight::AcpLaunchRequirements {
        match &self.session {
            AcpChatSession::Setup(setup) => setup.launch_requirements,
            AcpChatSession::Live(thread) => thread.read(cx).current_setup_requirements(),
        }
    }

    /// Stage a retry request for an action-surface agent switch.
    ///
    /// Preserves the active session's capability requirements so the next
    /// ACP open path can consume them instead of re-deriving from scratch.
    pub(crate) fn stage_agent_switch_retry(
        &mut self,
        next_agent_id: String,
        cx: &mut Context<Self>,
    ) {
        let launch_requirements = self.current_retry_launch_requirements(cx);
        self.pending_retry_request = Some(AcpRetryRequest {
            preferred_agent_id: Some(next_agent_id.clone()),
            launch_requirements,
        });
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_switch_agent_retry_payload_staged",
            agent_id = %next_agent_id,
            needs_embedded_context = launch_requirements.needs_embedded_context,
            needs_image = launch_requirements.needs_image,
        );
        cx.notify();
    }

    /// Queue an explicit relaunch payload from the current setup state.
    /// Called on retry so the next ACP open path reuses the selected agent
    /// and capability requirements instead of re-deriving them.
    fn queue_setup_retry_request(&mut self, cx: &mut Context<Self>) {
        let Some(setup) = self.read_active_setup_state(cx) else {
            return;
        };
        let request = AcpRetryRequest::from_setup_state(&setup);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_retry_payload_queued",
            preferred_agent_id = ?request.preferred_agent_id,
            needs_embedded_context = request.launch_requirements.needs_embedded_context,
            needs_image = request.launch_requirements.needs_image,
        );
        self.pending_retry_request = Some(request);
        cx.propagate();
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

        let compatible_count = setup
            .catalog_entries
            .iter()
            .filter(|entry| entry.satisfies_requirements(setup.launch_requirements))
            .count();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_agent_picker_opened",
            item_count = self
                .setup_agent_picker
                .as_ref()
                .map(|p| p.items.len())
                .unwrap_or(0),
            selected_index,
            compatible_count,
            needs_embedded_context = setup.launch_requirements.needs_embedded_context,
            needs_image = setup.launch_requirements.needs_image,
        );
        cx.notify();
    }

    /// Confirm the currently highlighted agent in the setup picker,
    /// persist it synchronously as the preferred agent, re-resolve the
    /// setup card, and close the picker. Auto-retry only when both the
    /// synchronous persistence succeeded and the agent is ready.
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

        // Skip the blocking disk write when the user confirms the already-selected agent.
        let already_selected = current_setup
            .selected_agent
            .as_ref()
            .is_some_and(|selected| selected.id == agent.id);

        let persist_result: Result<(), anyhow::Error> = if already_selected {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_setup_agent_persist_skipped_same_selection",
                agent_id = %agent.id,
            );
            Ok(())
        } else {
            crate::ai::acp::persist_preferred_acp_agent_id_sync(Some(agent.id.to_string()))
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_agent_persist_before_retry",
            agent_id = %agent.id,
            persisted = persist_result.is_ok(),
            already_selected,
        );

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

        // Update the live thread's selected agent only when the selection changed.
        if !already_selected {
            if let AcpChatSession::Live(thread) = &self.session {
                let next_agent_for_thread = next_setup.selected_agent.clone();
                thread.update(cx, |thread, cx| {
                    thread.replace_selected_agent(next_agent_for_thread, cx);
                });
            }
        }

        let should_auto_retry = resolution.is_ready() && persist_result.is_ok();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_setup_agent_confirmed_for_runtime_recovery",
            agent_id = %agent.id,
            display_name = %agent.display_name,
            blocker = ?resolution.blocker,
            needs_embedded_context = current_setup.launch_requirements.needs_embedded_context,
            needs_image = current_setup.launch_requirements.needs_image,
            catalog_count = current_setup.catalog_entries.len(),
            auto_retry = should_auto_retry,
            already_selected,
        );

        self.replace_active_setup_state(next_setup, cx);

        if should_auto_retry {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_setup_agent_ready_retrying",
                agent_id = %agent.id,
                needs_embedded_context = current_setup.launch_requirements.needs_embedded_context,
                needs_image = current_setup.launch_requirements.needs_image,
                already_selected,
            );
            self.queue_setup_retry_request(cx);
        }
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
                self.queue_setup_retry_request(cx);
            }
            super::setup_state::AcpSetupAction::OpenCatalog => {
                match crate::ai::acp::open_acp_agents_catalog_in_editor() {
                    Ok(path) => {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_setup_open_catalog_requested",
                            path = %path.display(),
                        );
                    }
                    Err(error) => {
                        tracing::warn!(
                            target: "script_kit::tab_ai",
                            event = "acp_setup_open_catalog_failed",
                            error = %error,
                        );
                    }
                }
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

    // ── Automation setup action dispatch ─���───────────────────

    /// Perform a setup action from the automation protocol.
    ///
    /// Returns `Ok(())` on success, or an error message if the action
    /// cannot be performed in the current state.
    pub(crate) fn perform_setup_automation_action(
        &mut self,
        action: crate::protocol::AcpSetupActionKind,
        agent_id: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        use crate::protocol::AcpSetupActionKind;

        match action {
            AcpSetupActionKind::OpenAgentPicker => {
                if !self.has_active_setup(cx) {
                    return Err("no active setup card".to_string());
                }
                self.open_setup_agent_picker(cx);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_action_completed",
                    action = "openAgentPicker",
                    success = true,
                );
                Ok(())
            }
            AcpSetupActionKind::CloseAgentPicker => {
                self.setup_agent_picker = None;
                cx.notify();
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_action_completed",
                    action = "closeAgentPicker",
                    success = true,
                );
                Ok(())
            }
            AcpSetupActionKind::SelectAgent => {
                let target_id =
                    agent_id.ok_or_else(|| "selectAgent requires an agentId field".to_string())?;
                if !self.has_active_setup(cx) {
                    return Err("no active setup card".to_string());
                }
                // Open the picker if not already open, select the target agent,
                // then confirm — replicating the user flow deterministically.
                self.open_setup_agent_picker(cx);
                if let Some(ref mut picker) = self.setup_agent_picker {
                    if let Some(idx) = picker
                        .items
                        .iter()
                        .position(|entry| entry.id.as_ref() == target_id)
                    {
                        picker.selected_index = idx;
                    } else {
                        self.setup_agent_picker = None;
                        return Err(format!("agent '{}' not found in catalog", target_id));
                    }
                }
                self.confirm_setup_agent_picker(cx);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_action_completed",
                    action = "selectAgent",
                    success = true,
                    selected_agent_id = target_id,
                );
                Ok(())
            }
            AcpSetupActionKind::Retry
            | AcpSetupActionKind::Install
            | AcpSetupActionKind::Authenticate
            | AcpSetupActionKind::OpenCatalog => {
                if !self.has_active_setup(cx) {
                    return Err("no active setup card".to_string());
                }
                let internal = super::setup_state::AcpSetupAction::from_protocol_kind(action);
                self.handle_setup_action(internal, cx);
                let action_name = format!("{:?}", action);
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_setup_action_completed",
                    action = %action_name,
                    success = true,
                );
                Ok(())
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
        self.test_probe.last_interaction_trace = None;
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
            last_interaction_trace: self.test_probe.last_interaction_trace.clone(),
            state: self.collect_acp_state_snapshot(cx),
            warnings: Vec::new(),
        }
    }

    // ── Telemetry emission ───────────────────────────────────

    /// Emit structured ACP key-routing telemetry.
    ///
    /// Logged on `script_kit::acp_telemetry` target. Contains no user content —
    /// only the key name, route, indices, and booleans.
    fn emit_key_route_telemetry(&mut self, key: &str, telemetry_args: AcpKeyRouteTelemetryArgs) {
        let picker_open = self.mention_session.is_some();
        let telemetry = crate::protocol::AcpKeyRouteTelemetry {
            key: key.to_string(),
            route: telemetry_args.route.clone(),
            picker_open,
            permission_active: telemetry_args.permission_active,
            cursor_before: telemetry_args.cursor_before,
            cursor_after: telemetry_args.cursor_after,
            caused_submit: telemetry_args.caused_submit,
            consumed: telemetry_args.consumed,
        };
        // Build the interaction trace (no accept info yet — augmented by picker accept if it follows).
        let trace = crate::protocol::AcpLastInteractionTrace {
            key: key.to_string(),
            route: format!("{:?}", telemetry_args.route).to_lowercase(),
            picker_open_before: picker_open,
            accepted_via_key: None,
            accepted_label: None,
            cursor_before: telemetry_args.cursor_before,
            cursor_after: telemetry_args.cursor_after,
            caused_submit: telemetry_args.caused_submit,
        };
        self.test_probe.last_interaction_trace = Some(trace);

        // Record into test probe ring buffer.
        self.record_key_route(telemetry.clone());
        let telemetry_json = serde_json::to_string(&telemetry).unwrap_or_default();
        tracing::info!(
            target: "script_kit::acp_telemetry",
            event = "acp_key_routed",
            key = %key,
            route = ?telemetry_args.route,
            picker_open,
            permission_active = telemetry_args.permission_active,
            cursor_before = telemetry_args.cursor_before,
            cursor_after = telemetry_args.cursor_after,
            caused_submit = telemetry_args.caused_submit,
            consumed = telemetry_args.consumed,
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
        // Augment the last interaction trace with acceptance info.
        if let Some(ref mut trace) = self.test_probe.last_interaction_trace {
            trace.accepted_via_key = Some(accepted_via_key.to_string());
            trace.accepted_label = Some(item_label.to_string());
            trace.cursor_after = cursor_after;
            trace.caused_submit = caused_submit;
        }

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

        // Emit a single consolidated interaction trace log event.
        if let Some(ref trace) = self.test_probe.last_interaction_trace {
            tracing::info!(
                target: "script_kit::acp_telemetry",
                event = "acp_interaction_trace",
                trace.key = %trace.key,
                trace.route = %trace.route,
                trace.picker_open_before = trace.picker_open_before,
                trace.accepted_via_key = ?trace.accepted_via_key,
                trace.accepted_label = ?trace.accepted_label,
                trace.cursor_before = trace.cursor_before,
                trace.cursor_after = trace.cursor_after,
                trace.caused_submit = trace.caused_submit,
            );
        }
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

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

        if self.model_selector_open {
            if crate::ui_foundation::is_key_up(key) {
                self.move_model_selector_selection(-1, cx);
                cx.stop_propagation();
                return;
            }

            if crate::ui_foundation::is_key_down(key) {
                self.move_model_selector_selection(1, cx);
                cx.stop_propagation();
                return;
            }

            if crate::ui_foundation::is_key_enter(key) || crate::ui_foundation::is_key_tab(key) {
                self.confirm_model_selector_selection(cx);
                cx.stop_propagation();
                return;
            }
        }

        // ── Model selector dismiss on Escape ───────────────────
        if crate::ui_foundation::is_key_escape(key) && self.dismiss_escape_popup(cx) {
            cx.stop_propagation();
            return;
        }
        // Close model selector on any non-modifier key
        if self.model_selector_open {
            self.dismiss_model_selector_popup(cx);
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

        if self.history_menu.is_some() {
            match history_popup_key_intent(key, modifiers) {
                Some(AcpHistoryPopupKeyIntent::MoveUp) => {
                    self.navigate_history_popup_selection(-1, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::MoveDown) => {
                    self.navigate_history_popup_selection(1, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::MoveHome) => {
                    self.jump_history_popup_selection(false, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::MoveEnd) => {
                    self.jump_history_popup_selection(true, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::MovePageUp) => {
                    self.page_history_popup_selection(-1, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::MovePageDown) => {
                    self.page_history_popup_selection(1, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::ExecuteSelected) => {
                    self.execute_history_popup_selection(modifiers, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::Close) => {
                    self.dismiss_history_popup(cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::Backspace) => {
                    let next_query = self
                        .history_menu
                        .as_ref()
                        .map(|menu| {
                            let mut query = menu.query.clone();
                            query.pop();
                            query
                        })
                        .unwrap_or_default();
                    self.set_history_popup_query(next_query, cx);
                    cx.stop_propagation();
                    return;
                }
                Some(AcpHistoryPopupKeyIntent::TypeChar(ch)) => {
                    let next_query = self
                        .history_menu
                        .as_ref()
                        .map(|menu| format!("{}{}", menu.query, ch))
                        .unwrap_or_else(|| ch.to_string());
                    self.set_history_popup_query(next_query, cx);
                    cx.stop_propagation();
                    return;
                }
                None => {}
            }
        }

        // ── Cmd+K → open actions dialog ──────
        if modifiers.platform && crate::ui_foundation::is_key_k(key) {
            let detached_window_open = crate::ai::acp::chat_window::is_chat_window_open();
            let is_detached_host = crate::ai::acp::chat_window::is_chat_window(window);
            tracing::debug!(
                target: "script_kit::keyboard",
                event = "acp_cmd_k_route",
                detached_window_open,
                is_detached_host,
                host = if is_detached_host { "detached" } else { "embedded" },
                route = if is_detached_host { "detached_local" } else { "propagate_to_main_window" },
            );
            if is_detached_host {
                // Detached window: open actions popup directly
                tracing::info!(
                    target: "script_kit::keyboard",
                    event = "detached_actions_shortcut_pressed",
                );
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

        // ── Cmd+P → open dedicated history command surface ──────────
        if modifiers.platform && key.eq_ignore_ascii_case("p") {
            tracing::info!(event = "acp_history_shortcut_routed_to_command");
            self.trigger_open_history_command(window, cx);
            cx.stop_propagation();
            return;
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
                self.sync_mention_popup_window_from_cached_parent(cx);
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
                self.sync_mention_popup_window_from_cached_parent(cx);
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
                let permission_active = self.live_thread().read(cx).pending_permission.is_some();
                self.emit_key_route_telemetry(
                    key,
                    AcpKeyRouteTelemetryArgs {
                        route: crate::protocol::AcpKeyRoute::Picker,
                        cursor_before,
                        cursor_after,
                        caused_submit: false,
                        consumed: true,
                        permission_active,
                    },
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
                self.sync_mention_popup_window_from_cached_parent(cx);
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
            let permission_active = self.live_thread().read(cx).pending_permission.is_some();
            self.mention_session = None;
            self.sync_mention_popup_window_from_cached_parent(cx);
            self.submit_with_expanded_tokens(cx);
            self.emit_key_route_telemetry(
                key,
                AcpKeyRouteTelemetryArgs {
                    route: crate::protocol::AcpKeyRoute::Composer,
                    cursor_before,
                    cursor_after: 0,
                    caused_submit: true,
                    consumed: true,
                    permission_active,
                },
            );
            cx.stop_propagation();
            return;
        }

        // ── Token-atomic inline mention deletion ──────────────
        // When backspace/delete lands inside, at the trailing edge, or at
        // the leading edge of an inline @mention token, remove the whole
        // token plus one trailing space (when present) instead of deleting
        // a single character.
        if crate::ui_foundation::is_key_backspace(key) || crate::ui_foundation::is_key_delete(key) {
            let current_text = self.live_thread().read(cx).input.text().to_string();
            let cursor = self.live_thread().read(cx).input.cursor();

            if let Some((next_text, next_cursor)) =
                crate::ai::context_mentions::remove_inline_mention_at_cursor(
                    &current_text,
                    cursor,
                    crate::ui_foundation::is_key_delete(key),
                )
            {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_inline_mention_deleted_atomically",
                    key = %key,
                    cursor,
                    next_cursor,
                );

                self.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text(next_text);
                    thread.input.set_cursor(next_cursor);
                    cx.notify();
                });
                self.refresh_mention_session(cx);
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
        let history_popup_open = self.history_menu.is_some();
        let colors = Self::prompt_colors();
        let theme = theme::get_cached_theme();
        let mention_accent = theme::get_cached_theme().colors.accent.selected;
        let mention_highlights = Self::attached_inline_mention_highlight_ranges(
            &input_text,
            &attached_parts,
            mention_accent,
            &self.typed_mention_aliases,
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
                    this.cache_popup_parent_window(window, cx);

                    // Cmd+W in detached window: close the window directly.
                    // In the main panel, Cmd+W is handled by the interceptor.
                    let is_detached_host = crate::ai::acp::chat_window::is_chat_window(window);
                    if modifiers.platform && key.eq_ignore_ascii_case("w")
                        && is_detached_host
                    {
                        tracing::info!(
                            target: "script_kit::keyboard",
                            event = "detached_acp_cmd_w_close_requested",
                        );
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

                    this.handle_key_down(event, window, cx);
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
                            .text_size(px(Self::ACP_INPUT_FONT_SIZE))
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
                                    multiline: true,
                                    cursor_visible,
                                    cursor_color: theme.colors.accent.selected,
                                    text_color: theme.colors.text.primary,
                                    selection_color: theme.colors.accent.selected,
                                    selection_text_color: theme.colors.text.primary,
                                    cursor_height: 18.0,
                                    cursor_width: 2.0,
                                    container_height: Some(22.0),
                                    highlight_ranges: &mention_highlights,
                                    ..TextInputRenderConfig::default_for_prompt(&input_text)
                                })
                                .into_any_element()
                            }),
                    ),
            )
            // Context chips removed — all attachments are now inline @type:name tokens.
            // .child(self.render_pending_context_chips(cx))
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
            .when(history_popup_open, |d| {
                d.child(
                    div()
                        .id("acp-history-popup-backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom(px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT))
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.dismiss_history_popup(cx);
                            cx.stop_propagation();
                        })),
                )
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
    fn hint_prefix_replacement_preserves_deliberate_trailing_space() {
        let (updated, cursor) =
            AcpChatView::replace_active_trigger_or_insert_at_cursor("/he", 3, "/help ");
        assert_eq!(updated, "/help ");
        assert_eq!(
            cursor, 6,
            "cursor should land after the preserved trailing space"
        );
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
            description: SharedString::from("/tmp/secrets.txt"),
            meta: SharedString::from("@file:/tmp/secrets.txt"),
            kind: ContextPickerItemKind::File(std::path::PathBuf::from("/tmp/secrets.txt")),
            score: 100,
            label_highlight_indices: Vec::new(),
            meta_highlight_indices: Vec::new(),
        };
        let folder_item = ContextPickerItem {
            id: SharedString::from("folder:/Users/john/Documents"),
            label: SharedString::from("Documents"),
            description: SharedString::from("/Users/john/Documents"),
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
