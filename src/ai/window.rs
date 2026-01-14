//! AI Chat Window
//!
//! A separate floating window for AI chat, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.
//!
//! # Architecture
//!
//! The window follows a Raycast-style layout:
//! - Left sidebar: Chat history list with search, grouped by date (Today, Yesterday, This Week, Older)
//! - Right main panel: Welcome state ("Ask Anything") or chat messages
//! - Bottom: Input area + model picker + submit button

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use gpui::{
    div, hsla, point, prelude::*, px, rgba, size, svg, App, BoxShadow, Context, Entity,
    ExternalPaths, FocusHandle, Focusable, IntoElement, KeyDownEvent, ParentElement, Render,
    ScrollHandle, SharedString, Styled, Subscription, Window, WindowBounds, WindowOptions,
};

// Import local IconName for SVG icons (has external_path() method)
use crate::designs::icon_variations::IconName as LocalIconName;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants},
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    theme::ActiveTheme,
    Icon, IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use tracing::{debug, info};

// Using the unified CommandBar component for AI command bar (Cmd+K)
// Opens in a separate vibrancy window for proper macOS blur effect
use super::config::ModelInfo;
use super::model::{Chat, ChatId, ChatSource, Message, MessageRole};
use super::providers::ProviderRegistry;
use super::storage;
use crate::actions::{get_ai_command_bar_actions, CommandBar, CommandBarConfig};
use crate::theme;

/// Events from the streaming thread
enum StreamingEvent {
    /// A chunk of text received
    Chunk(String),
    /// Streaming completed successfully
    Done,
    /// An error occurred
    Error(String),
}

/// A preset configuration for starting new chats
#[derive(Clone)]
struct AiPreset {
    /// Unique identifier
    id: &'static str,
    /// Display name
    name: &'static str,
    /// Description shown in dropdown
    description: &'static str,
    /// System prompt to use
    system_prompt: &'static str,
    /// Icon name
    icon: LocalIconName,
    /// Preferred model ID (if any)
    preferred_model: Option<&'static str>,
}

impl AiPreset {
    /// Get default presets
    fn default_presets() -> Vec<AiPreset> {
        vec![
            AiPreset {
                id: "general",
                name: "General Assistant",
                description: "Helpful AI assistant for any task",
                system_prompt: "You are a helpful AI assistant.",
                icon: LocalIconName::Star,
                preferred_model: None,
            },
            AiPreset {
                id: "coder",
                name: "Code Assistant",
                description: "Expert programmer and debugger",
                system_prompt: "You are an expert programmer. Write clean, efficient, well-documented code. Explain your reasoning.",
                icon: LocalIconName::Code,
                preferred_model: None,
            },
            AiPreset {
                id: "writer",
                name: "Writing Assistant",
                description: "Help with writing and editing",
                system_prompt: "You are a skilled writer and editor. Help improve writing clarity, grammar, and style.",
                icon: LocalIconName::FileCode,
                preferred_model: None,
            },
            AiPreset {
                id: "researcher",
                name: "Research Assistant",
                description: "Deep analysis and research",
                system_prompt: "You are a thorough researcher. Analyze topics deeply, cite sources when possible, and provide comprehensive answers.",
                icon: LocalIconName::MagnifyingGlass,
                preferred_model: None,
            },
            AiPreset {
                id: "creative",
                name: "Creative Partner",
                description: "Brainstorming and creative ideas",
                system_prompt: "You are a creative partner. Help brainstorm ideas, think outside the box, and explore possibilities.",
                icon: LocalIconName::BoltFilled,
                preferred_model: None,
            },
        ]
    }
}

/// Date group categories for sidebar organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateGroup {
    Today,
    Yesterday,
    ThisWeek,
    Older,
}

impl DateGroup {
    /// Get the display label for this group
    fn label(&self) -> &'static str {
        match self {
            DateGroup::Today => "Today",
            DateGroup::Yesterday => "Yesterday",
            DateGroup::ThisWeek => "This Week",
            DateGroup::Older => "Older",
        }
    }
}

/// Determine which date group a date belongs to
fn get_date_group(date: NaiveDate, today: NaiveDate) -> DateGroup {
    let days_ago = today.signed_duration_since(date).num_days();

    if days_ago == 0 {
        DateGroup::Today
    } else if days_ago == 1 {
        DateGroup::Yesterday
    } else if days_ago < 7
        && date.weekday().num_days_from_monday() < today.weekday().num_days_from_monday()
    {
        // Same week (and not earlier in a previous week)
        DateGroup::ThisWeek
    } else if days_ago < 7 {
        DateGroup::ThisWeek
    } else {
        DateGroup::Older
    }
}

/// Group chats by date categories
fn group_chats_by_date(chats: &[Chat]) -> Vec<(DateGroup, Vec<&Chat>)> {
    let today = Utc::now().date_naive();

    let mut today_chats: Vec<&Chat> = Vec::new();
    let mut yesterday_chats: Vec<&Chat> = Vec::new();
    let mut this_week_chats: Vec<&Chat> = Vec::new();
    let mut older_chats: Vec<&Chat> = Vec::new();

    for chat in chats {
        let chat_date = chat.updated_at.date_naive();
        match get_date_group(chat_date, today) {
            DateGroup::Today => today_chats.push(chat),
            DateGroup::Yesterday => yesterday_chats.push(chat),
            DateGroup::ThisWeek => this_week_chats.push(chat),
            DateGroup::Older => older_chats.push(chat),
        }
    }

    let mut groups = Vec::new();
    if !today_chats.is_empty() {
        groups.push((DateGroup::Today, today_chats));
    }
    if !yesterday_chats.is_empty() {
        groups.push((DateGroup::Yesterday, yesterday_chats));
    }
    if !this_week_chats.is_empty() {
        groups.push((DateGroup::ThisWeek, this_week_chats));
    }
    if !older_chats.is_empty() {
        groups.push((DateGroup::Older, older_chats));
    }

    groups
}

/// Generate a contextual mock AI response based on the user's message
/// Used for demo/testing when no AI providers are configured
fn generate_mock_response(user_message: &str) -> String {
    let msg_lower = user_message.to_lowercase();

    // Contextual responses based on common patterns
    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.starts_with("hey") {
        return "Hello! I'm Script Kit's AI assistant running in demo mode. Since no API key is configured, I'm providing mock responses. To enable real AI, set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY` in your environment.".to_string();
    }

    if msg_lower.contains("script") || msg_lower.contains("automation") {
        return "Script Kit is a powerful automation tool! Here are some things you can do:\n\n1. **Create scripts** - Write TypeScript/JavaScript to automate tasks\n2. **Use prompts** - `arg()`, `editor()`, `div()` for interactive UIs\n3. **Hotkeys** - Bind scripts to global keyboard shortcuts\n4. **Snippets** - Text expansion with dynamic content\n\nTry running a script with `Cmd+;` to see it in action!".to_string();
    }

    if msg_lower.contains("help") || msg_lower.contains("how") {
        return "I'm here to help! In demo mode, I can explain Script Kit concepts:\n\n• **Scripts** live in `~/.scriptkit/scripts/`\n• **SDK** provides `arg()`, `div()`, `editor()`, and more\n• **Hotkeys** are configured in script metadata\n• **This AI chat** works with Claude or GPT when you add an API key\n\nWhat would you like to know more about?".to_string();
    }

    if msg_lower.contains("code") || msg_lower.contains("example") {
        return "Here's a simple Script Kit example:\n\n```typescript\n// Name: Hello World\n// Shortcut: cmd+shift+h\n\nconst name = await arg(\"What's your name?\");\nawait div(`<h1>Hello, ${name}!</h1>`);\n```\n\nThis creates a script that:\n1. Asks for your name via a prompt\n2. Displays a greeting in an HTML view\n\nSave this to `~/.scriptkit/scripts/hello.ts` and run it!".to_string();
    }

    if msg_lower.contains("api") || msg_lower.contains("key") || msg_lower.contains("configure") {
        return "To enable real AI responses, configure an API key:\n\n**For Claude (Anthropic):**\n```bash\nexport SCRIPT_KIT_ANTHROPIC_API_KEY=\"sk-ant-...\"\n```\n\n**For GPT (OpenAI):**\n```bash\nexport SCRIPT_KIT_OPENAI_API_KEY=\"sk-...\"\n```\n\nAdd these to your `~/.zshrc` or `~/.scriptkit/.env` file, then restart Script Kit.".to_string();
    }

    // Default response for unrecognized queries
    format!(
        "I received your message: \"{}\"\n\n\
        I'm running in **demo mode** because no AI API key is configured. \
        My responses are pre-written examples.\n\n\
        To get real AI responses:\n\
        1. Get an API key from Anthropic or OpenAI\n\
        2. Set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY`\n\
        3. Restart Script Kit\n\n\
        Try asking about \"scripts\", \"help\", or \"code examples\" to see more demo responses!",
        user_message.chars().take(50).collect::<String>()
    )
}

/// Global handle to the AI window
static AI_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

/// Global flag to request input focus in the AI window.
/// This replaces the problematic AI_APP_ENTITY which caused memory leaks.
/// The flag is checked in AiApp::render() and cleared after use.
static AI_FOCUS_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Pending commands for the AI window (for testing via stdin).
/// These are processed in AiApp::render() to avoid needing a global entity reference.
static AI_PENDING_COMMANDS: std::sync::OnceLock<std::sync::Mutex<Vec<AiCommand>>> =
    std::sync::OnceLock::new();

/// Commands that can be sent to the AI window (for testing)
#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
enum AiCommand {
    SetSearch(String),
    SetInput {
        text: String,
        submit: bool,
    },
    /// Set input with an attached image (base64 encoded PNG)
    SetInputWithImage {
        text: String,
        image_base64: String,
        submit: bool,
    },
    /// Initialize the chat with pending messages from open_ai_window_with_chat
    InitializeWithPendingChat,
    /// Show the command bar overlay (Cmd+K menu)
    ShowCommandBar,
    /// Simulate a key press (for testing)
    SimulateKey {
        key: String,
        modifiers: Vec<String>,
    },
}

fn get_pending_commands() -> &'static std::sync::Mutex<Vec<AiCommand>> {
    AI_PENDING_COMMANDS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn push_ai_command(cmd: AiCommand) {
    if let Ok(mut cmds) = get_pending_commands().lock() {
        cmds.push(cmd);
    }
}

fn take_ai_commands() -> Vec<AiCommand> {
    get_pending_commands()
        .lock()
        .ok()
        .map(|mut cmds| std::mem::take(&mut *cmds))
        .unwrap_or_default()
}

// NOTE: AI_APP_ENTITY was removed to prevent memory leaks.
// The entity was being kept alive by this global reference and by theme watcher tasks,
// causing the AiApp to never be dropped even after the window closed.
// Instead, we use AI_FOCUS_REQUESTED (AtomicBool) which AiApp checks in render().

/// The main AI chat application view
pub struct AiApp {
    /// All chats (cached from storage)
    chats: Vec<Chat>,

    /// Currently selected chat ID
    selected_chat_id: Option<ChatId>,

    /// Cache of last message preview per chat (ChatId -> preview text)
    message_previews: std::collections::HashMap<ChatId, String>,

    /// Chat input state (using gpui-component's Input)
    input_state: Entity<InputState>,

    /// Search input state for sidebar
    search_state: Entity<InputState>,

    /// Current search query
    search_query: String,

    /// Whether the sidebar is collapsed
    sidebar_collapsed: bool,

    /// Provider registry with available AI providers
    provider_registry: ProviderRegistry,

    /// Available models from all providers
    available_models: Vec<ModelInfo>,

    /// Currently selected model for new chats
    selected_model: Option<ModelInfo>,

    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,

    // === Streaming State ===
    /// Whether we're currently streaming a response
    is_streaming: bool,

    /// Content accumulated during streaming
    streaming_content: String,

    /// The chat ID that is currently streaming (guards against chat-switch corruption)
    /// When user switches chats mid-stream, updates for this chat_id are ignored
    /// if selected_chat_id differs
    streaming_chat_id: Option<ChatId>,

    /// Generation counter for streaming sessions (guards against stale updates)
    /// Incremented each time streaming starts. Old streaming updates become no-ops.
    streaming_generation: u64,

    /// Messages for the currently selected chat (cached for display)
    current_messages: Vec<Message>,

    /// Scroll handle for the messages area (for auto-scrolling during streaming)
    messages_scroll_handle: ScrollHandle,

    /// Cached box shadows from theme (avoid reloading theme on every render)
    cached_box_shadows: Vec<BoxShadow>,

    /// Flag to request input focus on next render.
    /// This replaces the need for a global AI_APP_ENTITY reference.
    /// Set this flag via window.update() and AiApp will process it on render.
    needs_focus_input: bool,

    /// Flag to request main focus_handle focus on next render (for command bar keyboard routing).
    /// When true, the render function will focus the main focus_handle instead of the input,
    /// ensuring keyboard events route to the window's key handler for command bar navigation.
    needs_command_bar_focus: bool,

    /// Track last persisted bounds for debounced save on close paths
    /// (traffic light, Cmd+W) that don't go through close_ai_window
    last_persisted_bounds: Option<gpui::WindowBounds>,

    /// Last time we saved bounds (debounce to avoid too-frequent saves)
    last_bounds_save: std::time::Instant,

    /// Theme revision seen - used to detect theme changes and recompute cached values
    theme_rev_seen: u64,

    /// Pending image attachment (base64 encoded PNG) to include with next message
    pending_image: Option<String>,

    /// Timestamp when setup command was last copied (for showing "Copied!" feedback)
    setup_copied_at: Option<std::time::Instant>,

    /// Whether we're showing the API key input field (configure mode)
    showing_api_key_input: bool,

    /// API key input state (for configure flow)
    api_key_input_state: Entity<InputState>,

    // === Command Bar State ===
    /// Command bar component (Cmd+K) - uses the unified CommandBar wrapper
    command_bar: CommandBar,

    // === Model Picker State ===
    /// Whether the model picker dropdown is visible
    showing_model_picker: bool,

    /// Selected index in model picker
    model_picker_selected_index: usize,

    // === Presets State ===
    /// Whether the new chat dropdown (presets) is visible
    showing_presets_dropdown: bool,

    /// Available presets
    presets: Vec<AiPreset>,

    /// Selected preset index
    presets_selected_index: usize,

    // === Attachments State ===
    /// Whether the attachments picker is visible
    showing_attachments_picker: bool,

    /// List of pending attachments (file paths)
    pending_attachments: Vec<String>,
}

impl AiApp {
    /// Create a new AiApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_ai_db() {
            tracing::error!(error = %e, "Failed to initialize AI database");
        }

        // Load chats from storage
        let chats = storage::get_all_chats().unwrap_or_default();
        let selected_chat_id = chats.first().map(|c| c.id);

        // Load message previews for each chat
        let mut message_previews = std::collections::HashMap::new();
        for chat in &chats {
            if let Ok(messages) = storage::get_recent_messages(&chat.id, 1) {
                if let Some(last_msg) = messages.first() {
                    // Truncate preview to ~60 chars
                    let preview: String = last_msg.content.chars().take(60).collect();
                    let preview = if preview.len() < last_msg.content.len() {
                        format!("{}...", preview.trim())
                    } else {
                        preview
                    };
                    message_previews.insert(chat.id, preview);
                }
            }
        }

        // Initialize provider registry from environment
        let provider_registry = ProviderRegistry::from_environment();
        let available_models = provider_registry.get_all_models();

        // Select default model (prefer Claude Haiku 4.5, then 3.5 Haiku, then Sonnet, then GPT-4o)
        let selected_model = available_models
            .iter()
            .find(|m| m.id.contains("haiku-4-5"))
            .or_else(|| {
                available_models
                    .iter()
                    .find(|m| m.id.contains("claude-3-5-haiku"))
            })
            .or_else(|| {
                available_models
                    .iter()
                    .find(|m| m.id.contains("claude-3-5-sonnet"))
            })
            .or_else(|| available_models.iter().find(|m| m.id == "gpt-4o"))
            .or_else(|| available_models.first())
            .cloned();

        info!(
            providers = provider_registry.provider_ids().len(),
            models = available_models.len(),
            selected = selected_model
                .as_ref()
                .map(|m| m.display_name.as_str())
                .unwrap_or("none"),
            "AI providers initialized"
        );

        // Create input states
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Ask anything..."));

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search chats..."));

        let api_key_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter your Vercel API key...")
                .masked(true)
        });

        let focus_handle = cx.focus_handle();

        // Subscribe to input changes and Enter key
        let input_sub = cx.subscribe_in(&input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.on_input_change(cx),
                InputEvent::PressEnter { .. } => this.submit_message(window, cx),
                _ => {}
            }
        });

        // Subscribe to search changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        // Subscribe to API key input changes (Enter submits the key)
        let api_key_sub = cx.subscribe_in(&api_key_input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::PressEnter { .. }) {
                    this.submit_api_key(window, cx);
                }
            }
        });

        // Load messages for the selected chat
        let current_messages = selected_chat_id
            .and_then(|id| storage::get_chat_messages(&id).ok())
            .unwrap_or_default();

        info!(chat_count = chats.len(), "AI app initialized");

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

        Self {
            chats,
            selected_chat_id,
            message_previews,
            input_state,
            search_state,
            search_query: String::new(),
            sidebar_collapsed: false,
            provider_registry,
            available_models,
            selected_model,
            focus_handle,
            _subscriptions: vec![input_sub, search_sub, api_key_sub],
            // Streaming state
            is_streaming: false,
            streaming_content: String::new(),
            streaming_chat_id: None,
            streaming_generation: 0,
            current_messages,
            messages_scroll_handle: ScrollHandle::new(),
            cached_box_shadows,
            needs_focus_input: false,
            needs_command_bar_focus: false,
            last_persisted_bounds: None,
            last_bounds_save: std::time::Instant::now(),
            theme_rev_seen: crate::theme::service::theme_revision(),
            pending_image: None,
            setup_copied_at: None,
            showing_api_key_input: false,
            api_key_input_state,
            // Command bar state (uses the unified CommandBar component)
            command_bar: CommandBar::new(
                get_ai_command_bar_actions(),
                CommandBarConfig::ai_style(),
                std::sync::Arc::new(theme::load_theme()),
            ),
            // Model picker state
            showing_model_picker: false,
            model_picker_selected_index: 0,
            // Presets state
            showing_presets_dropdown: false,
            presets: AiPreset::default_presets(),
            presets_selected_index: 0,
            // Attachments state
            showing_attachments_picker: false,
            pending_attachments: Vec::new(),
        }
    }

    /// Debounce interval for bounds persistence (in milliseconds)
    const BOUNDS_DEBOUNCE_MS: u64 = 250;

    /// Update cached theme-derived values if theme revision has changed.
    ///
    /// This is called during render to detect theme hot-reloads and recompute
    /// values like box shadows that are derived from the theme.
    fn maybe_update_theme_cache(&mut self) {
        let current_rev = crate::theme::service::theme_revision();
        if self.theme_rev_seen != current_rev {
            self.theme_rev_seen = current_rev;
            self.cached_box_shadows = Self::compute_box_shadows();
        }
    }

    /// Persist window bounds if they've changed (debounced).
    ///
    /// This ensures bounds are saved even when the window is closed via traffic light
    /// (red close button) which doesn't go through our close handlers.
    fn maybe_persist_bounds(&mut self, window: &gpui::Window) {
        let wb = window.window_bounds();

        // Skip if bounds haven't changed
        if self.last_persisted_bounds.as_ref() == Some(&wb) {
            return;
        }

        // Debounce to avoid too-frequent saves
        if self.last_bounds_save.elapsed()
            < std::time::Duration::from_millis(Self::BOUNDS_DEBOUNCE_MS)
        {
            return;
        }

        // Save bounds
        self.last_persisted_bounds = Some(wb);
        self.last_bounds_save = std::time::Instant::now();
        crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Ai, wb);
    }

    /// Handle input changes
    fn on_input_change(&mut self, _cx: &mut Context<Self>) {
        // TODO: Handle input changes (e.g., streaming, auto-complete)
    }

    /// Handle paste event - check for clipboard images
    ///
    /// If clipboard contains an image, encode it as base64 and store as pending_image.
    /// If clipboard contains text, let the normal input handling process it.
    ///
    /// Returns true if an image was pasted (caller should not process text).
    fn handle_paste_for_image(&mut self, cx: &mut Context<Self>) -> bool {
        // Use arboard to read clipboard since it handles images
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                // Check for image first
                if let Ok(image_data) = clipboard.get_image() {
                    // Convert image to base64 PNG
                    match crate::clipboard_history::encode_image_as_png(&image_data) {
                        Ok(encoded) => {
                            // Strip the "png:" prefix since we store raw base64
                            let base64_data =
                                encoded.strip_prefix("png:").unwrap_or(&encoded).to_string();

                            let size_kb = base64_data.len() / 1024;
                            info!(
                                width = image_data.width,
                                height = image_data.height,
                                size_kb = size_kb,
                                "Image pasted from clipboard"
                            );

                            self.pending_image = Some(base64_data);
                            cx.notify();
                            return true;
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to encode pasted image");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to access clipboard");
            }
        }
        false
    }

    /// Remove the pending image attachment
    fn remove_pending_image(&mut self, cx: &mut Context<Self>) {
        if self.pending_image.is_some() {
            self.pending_image = None;
            info!("Pending image removed");
            cx.notify();
        }
    }

    /// Handle file drop - if it's an image, set it as pending image
    fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let paths = paths.paths();
        if paths.is_empty() {
            return;
        }

        // Only handle the first file for now
        let path = &paths[0];
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Check if it's an image file
        let is_image = matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"
        );

        if !is_image {
            info!("Dropped file is not an image: {:?}", path);
            return;
        }

        // Read and encode the file as base64
        match std::fs::read(path) {
            Ok(data) => {
                use base64::Engine;
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
                self.pending_image = Some(base64_data);
                info!("Image file dropped and attached: {:?}", path);
                cx.notify();
            }
            Err(e) => {
                info!("Failed to read dropped image file: {:?} - {}", path, e);
            }
        }
    }

    /// Render the pending image preview badge
    fn render_pending_image_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().items_center().gap_2().px_3().py_1().child(
            div()
                .id("pending-image-preview")
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(cx.theme().muted.opacity(0.3))
                .border_1()
                .border_color(cx.theme().accent.opacity(0.5))
                // Icon + label
                .child(
                    svg()
                        .external_path(LocalIconName::File.external_path())
                        .size(px(14.))
                        .text_color(cx.theme().accent),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground)
                        .child("PNG"),
                )
                // Remove button
                .child(
                    div()
                        .id("remove-image-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(16.))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(0.0, 0.7, 0.5, 0.2))) // Red hover
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.remove_pending_image(cx);
                            }),
                        )
                        .child(
                            svg()
                                .external_path(LocalIconName::Close.external_path())
                                .size(px(10.))
                                .text_color(cx.theme().muted_foreground),
                        ),
                ),
        )
    }

    /// Focus the main chat input
    /// Called when the window is opened to allow immediate typing
    pub fn focus_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            // Focus and ensure cursor is at the end of any existing text
            // For empty input, this puts cursor at position 0 with proper blinking
            let text_len = state.text().len();
            state.set_selection(text_len, text_len, window, cx);
        });
        info!("AI input focused for immediate typing");
    }

    /// Request focus on next render cycle.
    /// This is used when bringing an existing window to front - the caller
    /// sets this flag via window.update() and the flag is processed in render().
    /// This pattern avoids the need for a global Entity<AiApp> reference.
    pub fn request_focus(&mut self, cx: &mut Context<Self>) {
        self.needs_focus_input = true;
        cx.notify(); // Trigger re-render to process the flag
    }

    /// Handle model selection change
    ///
    /// Updates both the UI state and persists the model change to the current chat
    /// so that BYOK per-chat is maintained.
    fn on_model_change(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(model) = self.available_models.get(index) {
            info!(
                model_id = model.id,
                model_name = model.display_name,
                provider = model.provider,
                "Model selected"
            );
            self.selected_model = Some(model.clone());

            // Update the current chat's model in storage (BYOK per-chat)
            if let Some(chat_id) = self.selected_chat_id {
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
                    chat.model_id = model.id.clone();
                    chat.provider = model.provider.clone();
                    chat.touch(); // Update updated_at

                    // Persist to database
                    if let Err(e) = storage::update_chat(chat) {
                        tracing::error!(error = %e, chat_id = %chat_id, "Failed to persist model change to chat");
                    }
                }
            }

            cx.notify();
        }
    }

    /// Update a chat's timestamp and move it to the top of the list
    ///
    /// Called after message activity to keep the chat list sorted by recency.
    fn touch_and_reorder_chat(&mut self, chat_id: ChatId) {
        // Find the chat and update its timestamp
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            chat.touch(); // Updates updated_at to now

            // Persist the timestamp update to storage
            if let Err(e) = storage::update_chat(chat) {
                tracing::error!(error = %e, chat_id = %chat_id, "Failed to persist chat timestamp");
            }
        }

        // Reorder: move the active chat to the top
        if let Some(pos) = self.chats.iter().position(|c| c.id == chat_id) {
            if pos > 0 {
                let chat = self.chats.remove(pos);
                self.chats.insert(0, chat);
            }
        }
    }

    /// Handle search query changes - filters chats in real-time as user types
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        debug!(query = %query, "Search query changed");

        // If search is not empty, filter chats
        if !query.trim().is_empty() {
            // Use simple case-insensitive title matching for responsiveness
            // FTS search is available but can fail on special characters
            let query_lower = query.to_lowercase();
            let all_chats = storage::get_all_chats().unwrap_or_default();
            self.chats = all_chats
                .into_iter()
                .filter(|chat| chat.title.to_lowercase().contains(&query_lower))
                .collect();

            debug!(results = self.chats.len(), "Search filtered chats");

            // Always select first result when filtering
            if !self.chats.is_empty() {
                let first_id = self.chats[0].id;
                if self.selected_chat_id != Some(first_id) {
                    self.selected_chat_id = Some(first_id);
                    // Load messages for the selected chat
                    self.current_messages =
                        storage::get_chat_messages(&first_id).unwrap_or_default();
                }
            } else {
                self.selected_chat_id = None;
                self.current_messages = Vec::new();
            }
        } else {
            // Reload all chats when search is cleared
            self.chats = storage::get_all_chats().unwrap_or_default();
            // Keep current selection if it still exists, otherwise select first
            if let Some(id) = self.selected_chat_id {
                if !self.chats.iter().any(|c| c.id == id) {
                    self.selected_chat_id = self.chats.first().map(|c| c.id);
                    if let Some(new_id) = self.selected_chat_id {
                        self.current_messages =
                            storage::get_chat_messages(&new_id).unwrap_or_default();
                    }
                }
            }
        }

        cx.notify();
    }

    // === Command Bar Methods ===
    // These delegate to the CommandBar component which handles all window/state management.

    /// Show the command bar as a separate vibrancy window (Cmd+K)
    fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Open the command bar (CommandBar handles window creation internally)
        self.command_bar.open(window, cx);

        // CRITICAL: Focus main focus_handle so keyboard events route to us
        // The ActionsWindow is a visual-only popup - it does NOT take keyboard focus.
        // macOS popup windows often don't receive keyboard events properly.
        // This also unfocuses the Input component which would otherwise consume arrow keys.
        self.focus_handle.focus(window, cx);

        // Request command bar focus on next render for keyboard routing
        // This ensures the focus persists even if something else tries to steal it
        self.needs_command_bar_focus = true;

        // Log focus state for debugging - check both main handle AND input's focus state
        let main_focused = self.focus_handle.is_focused(window);
        let input_focused = self
            .input_state
            .read(cx)
            .focus_handle(cx)
            .is_focused(window);
        crate::logging::log(
            "AI",
            &format!(
                "show_command_bar: main_focus={} input_focus={} (input should be false for arrow keys to work)",
                main_focused, input_focused
            ),
        );

        cx.notify();
    }

    /// Hide the command bar (closes the vibrancy window)
    fn hide_command_bar(&mut self, cx: &mut Context<Self>) {
        self.command_bar.close(cx);
    }

    /// Handle character input in command bar
    fn command_bar_handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.command_bar.handle_char(ch, cx);
    }

    /// Handle backspace in command bar
    fn command_bar_handle_backspace(&mut self, cx: &mut Context<Self>) {
        self.command_bar.handle_backspace(cx);
    }

    /// Move selection up in command bar
    fn command_bar_select_prev(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_prev(cx);
    }

    /// Move selection down in command bar
    fn command_bar_select_next(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_next(cx);
    }

    /// Execute the selected command bar action
    fn execute_command_bar_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(action_id) = self.command_bar.execute_selected_action(cx) {
            self.execute_action(&action_id, window, cx);
        }
    }

    /// Execute an action by ID
    fn execute_action(&mut self, action_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        match action_id {
            "copy_response" => self.copy_last_response(cx),
            "copy_chat" => self.copy_entire_chat(cx),
            "copy_last_code" => self.copy_last_code_block(cx),
            "submit" => self.submit_message(window, cx),
            "new_chat" => {
                self.create_chat(window, cx);
            }
            "delete_chat" => {
                self.delete_selected_chat(cx);
            }
            "add_attachment" => {
                self.show_attachments_picker(window, cx);
            }
            "paste_image" => self.paste_image_from_clipboard(cx),
            "change_model" => {
                self.show_model_picker(window, cx);
            }
            _ => {
                tracing::warn!(action = action_id, "Unknown action");
            }
        }
    }

    /// Handle a simulated key press (for testing via stdin)
    fn handle_simulated_key(
        &mut self,
        key: &str,
        modifiers: &[String],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let has_cmd = modifiers
            .iter()
            .any(|m| m == "cmd" || m == "meta" || m == "command");
        let key_lower = key.to_lowercase();

        crate::logging::log(
            "AI",
            &format!(
                "SimulateKey: key='{}' modifiers={:?} command_bar_open={}",
                key_lower,
                modifiers,
                self.command_bar.is_open()
            ),
        );

        // Handle Cmd+K to toggle command bar
        if has_cmd && key_lower == "k" {
            crate::logging::log("AI", "SimulateKey: Cmd+K - toggling command bar");
            if self.command_bar.is_open() {
                self.hide_command_bar(cx);
            } else {
                self.hide_all_dropdowns(cx);
                self.show_command_bar(window, cx);
            }
            return;
        }

        // Handle command bar navigation when it's open
        if self.command_bar.is_open() {
            match key_lower.as_str() {
                "up" | "arrowup" => {
                    crate::logging::log("AI", "SimulateKey: Up in command bar");
                    self.command_bar_select_prev(cx);
                }
                "down" | "arrowdown" => {
                    crate::logging::log("AI", "SimulateKey: Down in command bar");
                    self.command_bar_select_next(cx);
                }
                "enter" => {
                    crate::logging::log("AI", "SimulateKey: Enter in command bar");
                    self.execute_command_bar_action(window, cx);
                }
                "escape" => {
                    crate::logging::log("AI", "SimulateKey: Escape - closing command bar");
                    self.hide_command_bar(cx);
                }
                "backspace" | "delete" => {
                    crate::logging::log("AI", "SimulateKey: Backspace in command bar");
                    self.command_bar_handle_backspace(cx);
                }
                _ => {
                    // Handle printable characters for search
                    if let Some(ch) = key_lower.chars().next() {
                        if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                            crate::logging::log(
                                "AI",
                                &format!("SimulateKey: Typing '{}' in command bar search", ch),
                            );
                            self.command_bar_handle_char(ch, cx);
                        }
                    }
                }
            }
            return;
        }

        // Handle model picker navigation
        if self.showing_model_picker {
            match key_lower.as_str() {
                "up" | "arrowup" => self.model_picker_select_prev(cx),
                "down" | "arrowdown" => self.model_picker_select_next(cx),
                "enter" => self.select_model_from_picker(cx),
                "escape" => self.hide_model_picker(cx),
                _ => {}
            }
            return;
        }

        // Handle presets dropdown navigation
        if self.showing_presets_dropdown {
            match key_lower.as_str() {
                "up" | "arrowup" => self.presets_select_prev(cx),
                "down" | "arrowdown" => self.presets_select_next(cx),
                "enter" => self.create_chat_with_preset(window, cx),
                "escape" => self.hide_presets_dropdown(cx),
                _ => {}
            }
            return;
        }

        // Default key handling (when no overlays are open)
        match key_lower.as_str() {
            "escape" => {
                if self.showing_attachments_picker {
                    self.hide_attachments_picker(cx);
                }
            }
            _ => {
                crate::logging::log(
                    "AI",
                    &format!("SimulateKey: Unhandled key '{}' in AI window", key_lower),
                );
            }
        }
    }

    /// Copy the last AI response to clipboard
    fn copy_last_response(&self, cx: &mut Context<Self>) {
        // Find the last assistant message
        if let Some(last_response) = self
            .current_messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                last_response.content.clone(),
            ));
            info!("Copied last response to clipboard");
        }
    }

    /// Copy the entire chat to clipboard
    fn copy_entire_chat(&self, cx: &mut Context<Self>) {
        let chat_text: String = self
            .current_messages
            .iter()
            .map(|m| {
                let role = if m.role == MessageRole::User {
                    "You"
                } else {
                    "AI"
                };
                format!("**{}**: {}\n\n", role, m.content)
            })
            .collect();
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(chat_text));
        info!("Copied entire chat to clipboard");
    }

    /// Copy the last code block from AI response
    fn copy_last_code_block(&self, cx: &mut Context<Self>) {
        // Find the last assistant message with a code block
        for msg in self.current_messages.iter().rev() {
            if msg.role == MessageRole::Assistant {
                // Simple regex-like search for code blocks
                if let Some(start) = msg.content.find("```") {
                    let after_start = &msg.content[start + 3..];
                    // Find the end of the language identifier (newline)
                    if let Some(lang_end) = after_start.find('\n') {
                        let code_start = &after_start[lang_end + 1..];
                        if let Some(end) = code_start.find("```") {
                            let code = &code_start[..end];
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                code.to_string(),
                            ));
                            info!("Copied last code block to clipboard");
                            return;
                        }
                    }
                }
            }
        }
        info!("No code block found to copy");
    }

    /// Paste image from clipboard as attachment
    fn paste_image_from_clipboard(&mut self, cx: &mut Context<Self>) {
        // Get the current clipboard text or image
        // Note: GPUI's clipboard API may not support raw image data directly
        // For now, we'll use a placeholder that can be enhanced later
        info!("Paste image from clipboard - checking for image data");
        // TODO: Implement proper image clipboard support when GPUI supports it
        cx.notify();
    }

    // === Model Picker Methods ===

    /// Show the model picker dropdown
    fn show_model_picker(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Find current model index
        self.model_picker_selected_index = self
            .selected_model
            .as_ref()
            .and_then(|sm| self.available_models.iter().position(|m| m.id == sm.id))
            .unwrap_or(0);
        self.showing_model_picker = true;
        cx.notify();
    }

    /// Hide the model picker dropdown
    fn hide_model_picker(&mut self, cx: &mut Context<Self>) {
        self.showing_model_picker = false;
        cx.notify();
    }

    /// Move selection up in model picker
    fn model_picker_select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.available_models.is_empty() {
            if self.model_picker_selected_index > 0 {
                self.model_picker_selected_index -= 1;
            } else {
                self.model_picker_selected_index = self.available_models.len() - 1;
            }
            cx.notify();
        }
    }

    /// Move selection down in model picker
    fn model_picker_select_next(&mut self, cx: &mut Context<Self>) {
        if !self.available_models.is_empty() {
            self.model_picker_selected_index =
                (self.model_picker_selected_index + 1) % self.available_models.len();
            cx.notify();
        }
    }

    /// Select the current model in the picker
    fn select_model_from_picker(&mut self, cx: &mut Context<Self>) {
        if let Some(model) = self.available_models.get(self.model_picker_selected_index) {
            self.selected_model = Some(model.clone());
            self.on_model_change(self.model_picker_selected_index, cx);
        }
        self.hide_model_picker(cx);
    }

    // === Presets Dropdown Methods ===

    /// Show the presets dropdown
    fn show_presets_dropdown(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.presets_selected_index = 0;
        self.showing_presets_dropdown = true;
        cx.notify();
    }

    /// Hide the presets dropdown
    fn hide_presets_dropdown(&mut self, cx: &mut Context<Self>) {
        self.showing_presets_dropdown = false;
        cx.notify();
    }

    /// Move selection up in presets dropdown
    fn presets_select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.presets.is_empty() {
            if self.presets_selected_index > 0 {
                self.presets_selected_index -= 1;
            } else {
                self.presets_selected_index = self.presets.len() - 1;
            }
            cx.notify();
        }
    }

    /// Move selection down in presets dropdown
    fn presets_select_next(&mut self, cx: &mut Context<Self>) {
        if !self.presets.is_empty() {
            self.presets_selected_index = (self.presets_selected_index + 1) % self.presets.len();
            cx.notify();
        }
    }

    /// Create a new chat with the selected preset
    fn create_chat_with_preset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(preset) = self.presets.get(self.presets_selected_index).cloned() {
            self.hide_presets_dropdown(cx);

            // Create new chat with system prompt
            let chat_id = self.create_chat(window, cx);
            if let Some(chat_id) = chat_id {
                // Add system message from preset
                if !preset.system_prompt.is_empty() {
                    let system_msg = Message::new(
                        chat_id,
                        crate::ai::model::MessageRole::System,
                        preset.system_prompt,
                    );
                    if let Err(e) = storage::save_message(&system_msg) {
                        tracing::error!(error = %e, "Failed to save system message");
                    }
                    // Reload messages to include system prompt
                    self.current_messages =
                        storage::get_chat_messages(&chat_id).unwrap_or_default();
                }

                // Set preferred model if specified
                if let Some(model_id) = preset.preferred_model {
                    if let Some(model) = self.available_models.iter().find(|m| m.id == model_id) {
                        self.selected_model = Some(model.clone());
                    }
                }

                cx.notify();
            }
        }
    }

    // === Attachments Picker Methods ===

    /// Show the attachments picker
    fn show_attachments_picker(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.showing_attachments_picker = true;
        cx.notify();
    }

    /// Hide the attachments picker
    fn hide_attachments_picker(&mut self, cx: &mut Context<Self>) {
        self.showing_attachments_picker = false;
        cx.notify();
    }

    /// Add a file attachment
    fn add_attachment(&mut self, path: String, cx: &mut Context<Self>) {
        if !self.pending_attachments.contains(&path) {
            self.pending_attachments.push(path);
            cx.notify();
        }
    }

    /// Remove a file attachment
    fn remove_attachment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.pending_attachments.len() {
            self.pending_attachments.remove(index);
            cx.notify();
        }
    }

    /// Clear all attachments
    fn clear_attachments(&mut self, cx: &mut Context<Self>) {
        self.pending_attachments.clear();
        cx.notify();
    }

    /// Hide all dropdowns (including closing the command bar vibrancy window)
    fn hide_all_dropdowns(&mut self, cx: &mut Context<Self>) {
        // Close command bar vibrancy window if open
        self.command_bar.close_app(cx);
        self.showing_model_picker = false;
        self.showing_presets_dropdown = false;
        self.showing_attachments_picker = false;
        cx.notify();
    }

    /// Initialize the chat window with pending messages from open_ai_window_with_chat.
    ///
    /// This creates a new chat with the provided messages and displays it immediately.
    /// Used for "Continue in Chat" functionality to transfer a conversation.
    fn initialize_with_pending_chat(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Take the pending messages from the global state
        let pending_messages = get_pending_chat()
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());

        let messages = match pending_messages {
            Some(msgs) if !msgs.is_empty() => msgs,
            _ => {
                crate::logging::log("AI", "No pending messages to initialize chat with");
                return;
            }
        };

        crate::logging::log(
            "AI",
            &format!("Initializing chat with {} messages", messages.len()),
        );

        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with the ChatPrompt source
        let mut chat = Chat::new(&model_id, &provider);
        chat.source = ChatSource::ChatPrompt;
        let chat_id = chat.id;

        // Generate title from the first user message (if any)
        if let Some((_, content)) = messages.iter().find(|(role, _)| *role == MessageRole::User) {
            let title = Chat::generate_title_from_content(content);
            chat.set_title(&title);
        }

        // Save chat to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat for transferred conversation");
            return;
        }

        // Save all messages to storage and build the current_messages list
        let mut saved_messages = Vec::new();
        for (role, content) in messages {
            let message = Message::new(chat_id, role, content);
            if let Err(e) = storage::save_message(&message) {
                tracing::error!(error = %e, "Failed to save message in transferred conversation");
                continue;
            }
            saved_messages.push(message);
        }

        // Update message preview with the last message
        if let Some(last_msg) = saved_messages.last() {
            let preview: String = last_msg.content.chars().take(60).collect();
            let preview = if preview.len() < last_msg.content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);
        }

        // Add chat to the list and select it
        self.chats.insert(0, chat);
        self.selected_chat_id = Some(chat_id);
        self.current_messages = saved_messages;

        // Scroll to bottom to show the latest messages
        self.messages_scroll_handle.scroll_to_bottom();

        info!(
            chat_id = %chat_id,
            message_count = self.current_messages.len(),
            "Chat initialized with transferred conversation"
        );

        cx.notify();
    }

    /// Create a new chat
    fn create_chat(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<ChatId> {
        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with selected model
        let chat = Chat::new(&model_id, &provider);
        let id = chat.id;

        // Save to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat");
            return None;
        }

        // Add to cache and select it
        self.chats.insert(0, chat);
        self.select_chat(id, window, cx);

        info!(chat_id = %id, model = model_id, "New chat created");
        Some(id)
    }

    /// Select a chat
    fn select_chat(&mut self, id: ChatId, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_chat_id = Some(id);

        // Load messages for this chat
        self.current_messages = storage::get_chat_messages(&id).unwrap_or_default();

        // Sync selected_model with the chat's stored model (BYOK per chat)
        if let Some(chat) = self.chats.iter().find(|c| c.id == id) {
            // Find the model in available_models that matches the chat's model_id
            self.selected_model = self
                .available_models
                .iter()
                .find(|m| m.id == chat.model_id)
                .cloned();

            if self.selected_model.is_none() && !chat.model_id.is_empty() {
                // Chat has a model_id but it's not in our available models
                // (provider may not be configured). Log for debugging.
                tracing::debug!(
                    chat_id = %id,
                    model_id = %chat.model_id,
                    provider = %chat.provider,
                    "Chat's model not found in available models (provider may not be configured)"
                );
            }
        }

        // Scroll to bottom to show latest messages
        self.messages_scroll_handle.scroll_to_bottom();

        // Clear streaming state for display purposes, but don't clear streaming_chat_id/generation
        // The streaming task may still be running for the previous chat - it will be
        // ignored via the generation guard when it tries to update
        self.is_streaming = false;
        self.streaming_content.clear();
        // Note: streaming_chat_id and streaming_generation are NOT cleared here
        // This allows the background streaming to complete and save to DB correctly
        // while UI shows the newly selected chat's messages

        cx.notify();
    }

    /// Delete the currently selected chat (soft delete)
    fn delete_selected_chat(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_chat_id {
            if let Err(e) = storage::delete_chat(&id) {
                tracing::error!(error = %e, "Failed to delete chat");
                return;
            }

            // Remove from visible list
            self.chats.retain(|c| c.id != id);

            // Select next chat and load its messages (or clear if no chats remain)
            self.selected_chat_id = self.chats.first().map(|c| c.id);
            self.current_messages = self
                .selected_chat_id
                .and_then(|new_id| storage::get_chat_messages(&new_id).ok())
                .unwrap_or_default();

            // Clear streaming state - if deleted chat was streaming, orphan the task
            // It will save to DB but won't corrupt UI since chat is deleted
            self.is_streaming = false;
            self.streaming_content.clear();
            // Also clear streaming context if the deleted chat was streaming
            if self.streaming_chat_id == Some(id) {
                self.streaming_chat_id = None;
            }

            cx.notify();
        }
    }

    /// Submit the current input as a message
    fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.input_state.read(cx).value().to_string();

        // Capture pending image before clearing
        let pending_image = self.pending_image.take();
        let has_image = pending_image.is_some();

        if let Some(ref image_base64) = pending_image {
            // Calculate approximate image size for logging
            let image_size_kb = image_base64.len() / 1024;
            crate::logging::log(
                "AI",
                &format!("Message includes attached image (~{}KB)", image_size_kb),
            );
        }

        if content.trim().is_empty() {
            return;
        }

        // Don't allow new messages while streaming for the CURRENT chat
        // (streaming for a different chat is fine - the guard handles it)
        if self.is_streaming && self.streaming_chat_id == self.selected_chat_id {
            return;
        }

        // If no chat selected, create a new one
        let chat_id = if let Some(id) = self.selected_chat_id {
            id
        } else {
            match self.create_chat(window, cx) {
                Some(id) => id,
                None => {
                    tracing::error!("Failed to create chat for message submission");
                    return;
                }
            }
        };

        // Update chat title if this is the first message
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            if chat.title == "New Chat" {
                let new_title = Chat::generate_title_from_content(&content);
                chat.set_title(&new_title);

                // Persist title update
                if let Err(e) = storage::update_chat_title(&chat_id, &new_title) {
                    tracing::error!(error = %e, "Failed to update chat title");
                }
            }
        }

        // Create and save user message with optional image
        let mut user_message = Message::user(chat_id, &content);

        // Attach image if present
        if let Some(image_base64) = pending_image {
            user_message
                .images
                .push(super::model::ImageAttachment::png(image_base64));
        }

        if let Err(e) = storage::save_message(&user_message) {
            tracing::error!(error = %e, "Failed to save user message");
            return;
        }

        // Add to current messages for display
        self.current_messages.push(user_message);

        // Scroll to bottom to show the new message
        self.messages_scroll_handle.scroll_to_bottom();

        // Update message preview cache
        let preview: String = content.chars().take(60).collect();
        let preview = if preview.len() < content.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };
        self.message_previews.insert(chat_id, preview);

        // Update chat timestamp and move to top of list
        self.touch_and_reorder_chat(chat_id);

        // Clear the input (pending image was already taken above)
        // Explicitly reset cursor to position 0 to fix cursor placement with placeholder
        self.input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
            state.set_selection(0, 0, window, cx);
        });

        info!(
            chat_id = %chat_id,
            content_len = content.len(),
            has_image = has_image,
            "User message submitted"
        );

        // Start streaming response
        self.start_streaming_response(chat_id, cx);

        cx.notify();
    }

    /// Start streaming an AI response (or mock response if no providers configured)
    fn start_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        // Check if we have a model selected - if not, use mock mode
        let use_mock_mode = self.selected_model.is_none() || self.available_models.is_empty();

        if use_mock_mode {
            info!(chat_id = %chat_id, "No AI providers configured - using mock mode");
            self.start_mock_streaming_response(chat_id, cx);
            return;
        }

        // Get the selected model
        let model = match &self.selected_model {
            Some(m) => m.clone(),
            None => {
                tracing::error!("No model selected for streaming");
                return;
            }
        };

        // Find the provider for this model
        let provider = match self.provider_registry.find_provider_for_model(&model.id) {
            Some(p) => p.clone(),
            None => {
                tracing::error!(model_id = model.id, "No provider found for model");
                return;
            }
        };

        // Build messages for the API call
        let api_messages: Vec<super::providers::ProviderMessage> = self
            .current_messages
            .iter()
            .map(|m| super::providers::ProviderMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
                images: m
                    .images
                    .iter()
                    .map(|img| super::providers::ProviderImage {
                        data: img.data.clone(),
                        media_type: img.media_type.clone(),
                    })
                    .collect(),
            })
            .collect();

        // Set streaming state with chat-scoping guards
        self.is_streaming = true;
        self.streaming_content.clear();
        self.streaming_chat_id = Some(chat_id);
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        let generation = self.streaming_generation;

        info!(
            chat_id = %chat_id,
            generation = generation,
            model = model.id,
            provider = model.provider,
            message_count = api_messages.len(),
            "Starting AI streaming response"
        );

        // Use a shared buffer for streaming content
        let shared_content = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let shared_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shared_error = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));

        let model_id = model.id.clone();
        let content_clone = shared_content.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let result = provider.stream_message(
                &api_messages,
                &model_id,
                Box::new(move |chunk| {
                    if let Ok(mut content) = content_clone.lock() {
                        content.push_str(&chunk);
                    }
                }),
            );

            match result {
                Ok(()) => {
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                Err(e) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates using background executor
        let content_for_poll = shared_content.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;
            loop {
                Timer::after(std::time::Duration::from_millis(50)).await;

                // Check if done or errored
                if done_for_poll.load(std::sync::atomic::Ordering::SeqCst) {
                    // Get final content
                    let final_content = content_for_poll.lock().ok().map(|c| c.clone());
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            // CRITICAL: Guard against stale updates from chat-switch
                            // If generation doesn't match, this is an old streaming task
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                tracing::debug!(
                                    expected_gen = generation,
                                    actual_gen = app.streaming_generation,
                                    expected_chat = %chat_id,
                                    actual_chat = ?app.streaming_chat_id,
                                    "Ignoring stale streaming completion (user switched chats)"
                                );
                                // Still save to DB below, but don't touch UI state
                                if let Some(err) = &error {
                                    tracing::error!(error = %err, chat_id = %chat_id, "Stale streaming error");
                                } else if let Some(content) = &final_content {
                                    // Save orphaned message to DB
                                    if !content.is_empty() {
                                        let assistant_message =
                                            Message::assistant(chat_id, content);
                                        if let Err(e) = storage::save_message(&assistant_message) {
                                            tracing::error!(error = %e, "Failed to save orphaned assistant message");
                                        } else {
                                            tracing::info!(
                                                chat_id = %chat_id,
                                                content_len = content.len(),
                                                "Orphaned streaming response saved to DB"
                                            );
                                        }
                                    }
                                }
                                return;
                            }

                            if let Some(err) = error {
                                tracing::error!(error = err, "Streaming error");
                                app.is_streaming = false;
                                app.streaming_content.clear();
                                app.streaming_chat_id = None;
                            } else if let Some(content) = final_content {
                                app.streaming_content = content;
                                app.finish_streaming(chat_id, generation, cx);
                            }
                            cx.notify();
                        })
                    });
                    break;
                }

                // Update with current content (only if generation matches)
                if let Ok(content) = content_for_poll.lock() {
                    if !content.is_empty() {
                        let current = content.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                // Guard: only update UI if this is the current streaming session
                                if app.streaming_generation != generation
                                    || app.streaming_chat_id != Some(chat_id)
                                {
                                    return; // Stale update, ignore
                                }
                                app.streaming_content = current;
                                // Auto-scroll to bottom as new content arrives
                                app.messages_scroll_handle.scroll_to_bottom();
                                cx.notify();
                            })
                        });
                    }
                }
            }
        })
        .detach();
    }

    /// Start a mock streaming response for testing/demo when no AI providers are configured
    fn start_mock_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        // Set streaming state with chat-scoping guards
        self.is_streaming = true;
        self.streaming_content.clear();
        self.streaming_chat_id = Some(chat_id);
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        let generation = self.streaming_generation;

        // Get the last user message to generate a contextual mock response
        let user_message = self
            .current_messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        // Generate a mock response based on the user's message
        let mock_response = generate_mock_response(&user_message);

        info!(
            chat_id = %chat_id,
            generation = generation,
            user_message_len = user_message.len(),
            mock_response_len = mock_response.len(),
            "Starting mock streaming response"
        );

        // Simulate streaming by revealing the response word by word
        let words: Vec<String> = mock_response
            .split_inclusive(char::is_whitespace)
            .map(|s| s.to_string())
            .collect();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;

            let mut accumulated = String::new();
            let mut delay_counter = 0u64;

            for word in words {
                // Vary delay slightly based on word position (30-60ms range)
                delay_counter = delay_counter.wrapping_add(17); // Simple pseudo-variation
                let delay = 30 + (delay_counter % 30);
                Timer::after(std::time::Duration::from_millis(delay)).await;

                accumulated.push_str(&word);

                let current_content = accumulated.clone();
                let should_break = cx
                    .update(|cx| {
                        this.update(cx, |app, cx| {
                            // Guard: only update UI if this is the current streaming session
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                return true; // Break out of loop - stale session
                            }
                            app.streaming_content = current_content;
                            // Auto-scroll to bottom as new content arrives
                            app.messages_scroll_handle.scroll_to_bottom();
                            cx.notify();
                            false
                        })
                        .unwrap_or(true)
                    })
                    .unwrap_or(true);

                if should_break {
                    // Session was superseded, save what we have to DB and exit
                    if !accumulated.is_empty() {
                        let assistant_message = Message::assistant(chat_id, &accumulated);
                        if let Err(e) = storage::save_message(&assistant_message) {
                            tracing::error!(error = %e, "Failed to save orphaned mock message");
                        } else {
                            tracing::info!(
                                chat_id = %chat_id,
                                content_len = accumulated.len(),
                                "Orphaned mock streaming saved to DB"
                            );
                        }
                    }
                    return;
                }
            }

            // Small delay before finishing
            Timer::after(std::time::Duration::from_millis(100)).await;

            // Finish streaming
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    app.finish_streaming(chat_id, generation, cx);
                })
            });
        })
        .detach();
    }

    /// Finish streaming and save the assistant message
    ///
    /// The `generation` parameter guards against stale completion calls.
    /// If the generation doesn't match, this is an orphaned streaming task
    /// and we should not update UI (message was already saved to DB by the guard).
    fn finish_streaming(&mut self, chat_id: ChatId, generation: u64, cx: &mut Context<Self>) {
        // Guard: verify this is still the current streaming session
        if self.streaming_generation != generation || self.streaming_chat_id != Some(chat_id) {
            tracing::debug!(
                expected_gen = generation,
                actual_gen = self.streaming_generation,
                "finish_streaming called with stale generation, ignoring"
            );
            return;
        }

        if !self.streaming_content.is_empty() {
            // Create and save assistant message
            let assistant_message = Message::assistant(chat_id, &self.streaming_content);
            if let Err(e) = storage::save_message(&assistant_message) {
                tracing::error!(error = %e, "Failed to save assistant message");
            }

            // Add to current messages (only if viewing this chat)
            if self.selected_chat_id == Some(chat_id) {
                self.current_messages.push(assistant_message);
            }

            // Update message preview
            let preview: String = self.streaming_content.chars().take(60).collect();
            let preview = if preview.len() < self.streaming_content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);

            // Update chat timestamp and move to top of list
            self.touch_and_reorder_chat(chat_id);

            info!(
                chat_id = %chat_id,
                content_len = self.streaming_content.len(),
                "Streaming response complete"
            );
        }

        self.is_streaming = false;
        self.streaming_content.clear();
        self.streaming_chat_id = None;
        cx.notify();
    }

    /// Get the currently selected chat
    fn get_selected_chat(&self) -> Option<&Chat> {
        self.selected_chat_id
            .and_then(|id| self.chats.iter().find(|c| c.id == id))
    }

    /// Render the search input
    fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Fixed height container to prevent layout shift when typing
        // Style the container and use appearance(false) on Input to remove its default white background
        div()
            .w_full()
            .h(px(36.)) // Fixed height to prevent layout shift
            .flex()
            .items_center()
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border.opacity(0.5))
            .bg(cx.theme().input) // Use theme's input background
            .child(
                Input::new(&self.search_state)
                    .w_full()
                    .small()
                    .appearance(false) // Remove default background/border (we provide our own)
                    .focus_bordered(false), // Disable default focus border
            )
    }

    /// Toggle sidebar visibility
    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    /// Copy the setup command to clipboard and show feedback
    fn copy_setup_command(&mut self, cx: &mut Context<Self>) {
        let setup_command = "export SCRIPT_KIT_ANTHROPIC_API_KEY=\"your-key-here\"";
        let item = gpui::ClipboardItem::new_string(setup_command.to_string());
        cx.write_to_clipboard(item);
        self.setup_copied_at = Some(std::time::Instant::now());
        info!("Setup command copied to clipboard");
        cx.notify();

        // Reset feedback after 2 seconds
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(std::time::Duration::from_millis(2000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |this, cx| {
                    this.setup_copied_at = None;
                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Check if we're showing "Copied!" feedback
    fn is_showing_copied_feedback(&self) -> bool {
        self.setup_copied_at
            .map(|t| t.elapsed() < std::time::Duration::from_millis(2000))
            .unwrap_or(false)
    }

    /// Show the API key configuration input
    fn show_api_key_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.showing_api_key_input = true;
        // Focus the API key input
        self.api_key_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
            state.set_selection(0, 0, window, cx);
        });
        cx.notify();
    }

    /// Hide the API key configuration input
    fn hide_api_key_input(&mut self, cx: &mut Context<Self>) {
        self.showing_api_key_input = false;
        cx.notify();
    }

    /// Submit the API key from the configuration input
    fn submit_api_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let api_key = self.api_key_input_state.read(cx).value().to_string();
        let api_key = api_key.trim();

        if api_key.is_empty() {
            info!("API key input is empty, ignoring submission");
            return;
        }

        // Save the API key to secrets storage
        if let Err(e) =
            crate::secrets::set_secret(crate::ai::config::env_vars::VERCEL_API_KEY, api_key)
        {
            tracing::error!(error = %e, "Failed to save Vercel API key");
            return;
        }

        info!("Vercel API key saved successfully");

        // Reinitialize the provider registry to pick up the new key
        self.provider_registry = ProviderRegistry::from_environment();
        self.available_models = self.provider_registry.get_all_models();

        // Select default model if available
        self.selected_model = self
            .available_models
            .iter()
            .find(|m| m.id.contains("haiku"))
            .or_else(|| self.available_models.first())
            .cloned();

        info!(
            providers = self.provider_registry.provider_ids().len(),
            models = self.available_models.len(),
            "Providers reinitialized after API key setup"
        );

        // Hide the input and show the welcome state
        self.showing_api_key_input = false;

        // Clear the input
        self.api_key_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        // Focus the main input
        self.focus_input(window, cx);

        cx.notify();
    }

    /// Render the sidebar toggle button using the Sidebar icon from our icon library
    fn render_sidebar_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Use opacity to indicate state - dimmed when collapsed
        let icon_color = if self.sidebar_collapsed {
            cx.theme().muted_foreground.opacity(0.5)
        } else {
            cx.theme().muted_foreground
        };

        div()
            .id("sidebar-toggle")
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.))
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.toggle_sidebar(cx);
                }),
            )
            .child(
                svg()
                    .external_path(LocalIconName::Sidebar.external_path())
                    .size(px(16.))
                    .text_color(icon_color),
            )
    }

    /// Render the chats sidebar with date groupings
    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If sidebar is collapsed, show a thin strip (toggle moves to main panel titlebar)
        if self.sidebar_collapsed {
            return div()
                .flex()
                .flex_col()
                .w(px(48.))
                .h_full()
                .bg(Self::get_vibrancy_sidebar_background()) // Semi-transparent sidebar
                .border_r_1()
                .border_color(cx.theme().sidebar_border)
                .items_center()
                // Spacer for traffic light area (toggle is now in main panel titlebar)
                .child(div().h(px(36.)))
                // New chat button below
                .child(
                    div().pt_1().child(
                        Button::new("new-chat-collapsed")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_chat(window, cx);
                            })),
                    ),
                )
                .into_any_element();
        }

        let selected_id = self.selected_chat_id;
        let date_groups = group_chats_by_date(&self.chats);

        // Build a custom sidebar with date groupings using divs
        // This gives us more control over the layout than SidebarGroup
        div()
            .flex()
            .flex_col()
            .w(px(240.))
            .h_full()
            .bg(Self::get_vibrancy_sidebar_background()) // Semi-transparent sidebar
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            // Top row - sidebar toggle on right (240px width clears traffic lights)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end() // Push toggle to right side
                    .w_full()
                    .h(px(36.)) // Match main panel titlebar height
                    .px_2()
                    .child(self.render_sidebar_toggle(cx)),
            )
            // Header with new chat button and search
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_2()
                    .pb_2()
                    .gap_2()
                    // New chat button row with preset dropdown option
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .w_full()
                            .gap_1()
                            // New chat button
                            .child(
                                Button::new("new-chat")
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Plus)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_chat(window, cx);
                                    })),
                            )
                            // Presets dropdown trigger
                            .child(
                                div()
                                    .id("presets-trigger")
                                    .px_1()
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|el| el.bg(cx.theme().sidebar_accent.opacity(0.5)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if this.showing_presets_dropdown {
                                            this.hide_presets_dropdown(cx);
                                        } else {
                                            this.hide_all_dropdowns(cx);
                                            this.show_presets_dropdown(window, cx);
                                        }
                                    }))
                                    .child(
                                        Icon::new(IconName::ChevronDown)
                                            .size(px(12.))
                                            .text_color(cx.theme().sidebar_foreground.opacity(0.7)),
                                    ),
                            ),
                    )
                    .child(self.render_search(cx)),
            )
            // Scrollable chat list with date groups
            // Note: overflow_y_scrollbar() wraps the element in a Scrollable container
            // min_h_0() is critical for flex containers - without it, the element won't shrink
            // below its content size and scrolling won't work properly
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0() // Critical: allows flex child to shrink and enable scrolling
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .px_2()
                            .pb_2()
                            .gap_3()
                            .children(date_groups.into_iter().map(|(group, chats)| {
                                self.render_date_group(group, chats, selected_id, cx)
                            }))
                            .overflow_y_scrollbar(),
                    ),
            )
            .into_any_element()
    }

    /// Render a date group section (Today, Yesterday, This Week, Older)
    fn render_date_group(
        &self,
        group: DateGroup,
        chats: Vec<&Chat>,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_1()
            // Group header
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .px_1()
                    .py_1()
                    .child(group.label()),
            )
            // Chat items
            .children(
                chats
                    .into_iter()
                    .map(|chat| self.render_chat_item(chat, selected_id, cx)),
            )
    }

    /// Render a single chat item with title and preview
    fn render_chat_item(
        &self,
        chat: &Chat,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let chat_id = chat.id;
        let is_selected = selected_id == Some(chat_id);

        let title: SharedString = if chat.title.is_empty() {
            "New Chat".into()
        } else {
            chat.title.clone().into()
        };

        let preview = self.message_previews.get(&chat_id).cloned();

        // Vibrancy-compatible selection/hover colors (white with alpha)
        // Same pattern as main menu list items - 0x54 = 33%, 0x26 = 15%
        // Formula: (hex_color << 8) | alpha
        let selected_bg = rgba((0xFFFFFF << 8) | 0x54);
        let hover_bg = rgba((0xFFFFFF << 8) | 0x26);

        // Text colors change based on selection state (matching main menu behavior)
        // Selected: bright white for high contrast against dark-tinted selection
        // Not selected: standard sidebar/muted colors
        let title_color = if is_selected {
            cx.theme().foreground // Bright white when selected
        } else {
            cx.theme().sidebar_foreground // Normal sidebar color when not
        };
        let description_color = if is_selected {
            cx.theme().sidebar_foreground // Lighter when selected (still readable)
        } else {
            cx.theme().muted_foreground // Muted when not selected
        };

        // Create a custom chat item with title and preview
        div()
            .id(SharedString::from(format!("chat-{}", chat_id)))
            .flex()
            .flex_col()
            .w_full()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .when(is_selected, |d| d.bg(selected_bg))
            .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_chat(chat_id, window, cx);
            }))
            .child(
                // Title - bright when selected, normal when not
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(title_color)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(title),
            )
            .when_some(preview, |d, preview_text| {
                // Clean up preview: skip markdown headings, find actual content
                let clean_preview: String = preview_text
                    .lines()
                    .map(|line| line.trim())
                    .find(|line| {
                        // Skip empty lines
                        !line.is_empty()
                        // Skip markdown headings
                        && !line.starts_with('#')
                        // Skip code fence markers
                        && !line.starts_with("```")
                        // Skip horizontal rules
                        && !line.chars().all(|c| c == '-' || c == '*' || c == '_')
                    })
                    .unwrap_or("")
                    .chars()
                    .take(50)
                    .collect();

                d.child(
                    // Preview - lighter when selected, muted when not
                    div()
                        .text_xs()
                        .text_color(description_color)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(clean_preview),
                )
            })
    }

    /// Render the input field - uses Input component's native styling
    fn render_input_with_cursor(
        &self,
        _border_color: gpui::Hsla,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Use Input's native appearance - no custom container
        // This avoids any potential visual conflicts
        Input::new(&self.input_state)
            .w_full()
            .appearance(true) // Use native Input styling
            .bordered(true)
            .focus_bordered(true)
    }

    /// Render the model picker button
    /// Clicking cycles to the next model; shows current model name
    fn render_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.available_models.is_empty() {
            let show_copied = self.is_showing_copied_feedback();

            // No models available - show actionable setup hint
            return div()
                .id("setup-hint")
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py(px(2.))
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                }))
                .child(if show_copied {
                    Icon::new(IconName::Check)
                        .size(px(12.))
                        .text_color(cx.theme().success)
                        .into_any_element()
                } else {
                    Icon::new(IconName::TriangleAlert)
                        .size(px(12.))
                        .text_color(cx.theme().warning)
                        .into_any_element()
                })
                .child(
                    div()
                        .text_xs()
                        .text_color(if show_copied {
                            cx.theme().success
                        } else {
                            cx.theme().muted_foreground
                        })
                        .child(if show_copied {
                            "Copied!"
                        } else {
                            "Setup Required"
                        }),
                )
                .when(!show_copied, |d| {
                    d.child(
                        div()
                            .px(px(4.))
                            .py(px(1.))
                            .rounded(px(3.))
                            .bg(cx.theme().muted)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("↵"),
                    )
                })
                .into_any_element();
        }

        // Get current model display name
        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| "Select Model".to_string())
            .into();

        // Model picker button - clicking opens dropdown
        Button::new("model-picker")
            .ghost()
            .xsmall()
            .icon(IconName::ChevronDown)
            .child(model_label)
            .on_click(cx.listener(|this, _, window, cx| {
                if this.showing_model_picker {
                    this.hide_model_picker(cx);
                } else {
                    this.show_model_picker(window, cx);
                }
            }))
            .into_any_element()
    }

    /// Cycle to the next model in the list
    fn cycle_model(&mut self, cx: &mut Context<Self>) {
        if self.available_models.is_empty() {
            return;
        }

        // Find current index
        let current_idx = self
            .selected_model
            .as_ref()
            .and_then(|sm| self.available_models.iter().position(|m| m.id == sm.id))
            .unwrap_or(0);

        // Cycle to next
        let next_idx = (current_idx + 1) % self.available_models.len();
        self.on_model_change(next_idx, cx);
    }

    /// Render the welcome state (no chat selected or empty chat)
    fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Show setup card if no providers are configured
        if self.available_models.is_empty() {
            return self.render_setup_card(cx).into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .text_color(cx.theme().foreground)
                    .child("Ask Anything"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Start a conversation with AI"),
            )
            .into_any_element()
    }

    /// Render the setup card when no API keys are configured
    /// Shows a Raycast-style prompt with a Configure Vercel AI Gateway button
    fn render_setup_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If showing API key input mode, render that instead
        if self.showing_api_key_input {
            return self.render_api_key_input(cx).into_any_element();
        }

        // Yellow/gold accent color for the button (Raycast style)
        let button_bg = hsla(45.0 / 360.0, 0.9, 0.55, 1.0); // Gold/yellow
        let button_text = hsla(0.0, 0.0, 0.1, 1.0); // Dark text

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap_5()
            .px_8()
            // Icon - muted settings icon at top
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(80.))
                    .rounded(px(20.))
                    .bg(cx.theme().muted.opacity(0.2))
                    .child(
                        svg()
                            .external_path(LocalIconName::Settings.external_path())
                            .size(px(40.))
                            .text_color(cx.theme().muted_foreground.opacity(0.5)),
                    ),
            )
            // Title
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().foreground)
                    .child("API Key Required"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(px(380.))
                    .child("Set up an API key to use the Ask AI feature. The easiest option is Vercel AI Gateway."),
            )
            // Configure Vercel AI Gateway button
            .child(
                div()
                    .id("configure-vercel-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .px_5()
                    .py_2()
                    .rounded_lg()
                    .bg(button_bg)
                    .cursor_pointer()
                    .border_1()
                    .border_color(button_bg.opacity(0.8))
                    .hover(|s| s.bg(button_bg.opacity(0.9)))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.show_api_key_input(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(LocalIconName::Settings.external_path())
                            .size(px(18.))
                            .text_color(button_text),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(button_text)
                            .child("Configure Vercel AI Gateway"),
                    ),
            )
            // Info text
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .mt_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("No restart required"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("After configuring, press Tab again to try"),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .mt_4()
                    // Enter to configure
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Enter"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to configure"),
                            ),
                    )
                    // Esc to go back
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Esc"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to go back"),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Render the API key input view (shown when user clicks Configure)
    fn render_api_key_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let input_border_color = cx.theme().accent;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap_5()
            .px_8()
            // Back arrow + title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("back-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.hide_api_key_input(cx);
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("←"),
                            ),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground)
                            .child("Enter Vercel API Key"),
                    ),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(px(400.))
                    .child("Get your API key from Vercel AI Gateway and paste it below."),
            )
            // Input field
            .child(
                div()
                    .w(px(400.))
                    .rounded_lg()
                    .border_1()
                    .border_color(input_border_color.opacity(0.6))
                    .overflow_hidden()
                    .child(
                        Input::new(&self.api_key_input_state)
                            .w_full()
                            .appearance(false)
                            .focus_bordered(false),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .mt_2()
                    // Enter to save
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Enter"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to save"),
                            ),
                    )
                    // Esc to go back
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Esc"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to go back"),
                            ),
                    ),
            )
    }

    /// Render a single message bubble
    fn render_message(&self, message: &Message, cx: &mut Context<Self>) -> impl IntoElement {
        let is_user = message.role == MessageRole::User;

        div()
            .flex()
            .flex_col()
            .w_full()
            .mb_3()
            .child(
                // Role label
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .mb_1()
                    .child(if is_user { "You" } else { "Assistant" }),
            )
            .child(
                // Message content
                div()
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .when(is_user, |d| d.bg(cx.theme().secondary))
                    .when(!is_user, |d| d.bg(cx.theme().muted.opacity(0.3)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(message.content.clone()),
                    ),
            )
    }

    /// Render streaming content (assistant response in progress)
    fn render_streaming_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .mb_3()
            .child(
                // Role label with streaming indicator
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mb_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child("Assistant"),
                    )
                    .child(
                        // Streaming indicator
                        div().text_xs().text_color(cx.theme().accent).child("●"),
                    ),
            )
            .child(
                // Streaming content
                div()
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .bg(cx.theme().muted.opacity(0.3))
                    .child(div().text_sm().text_color(cx.theme().foreground).child(
                        if self.streaming_content.is_empty() {
                            "...".to_string()
                        } else {
                            self.streaming_content.clone()
                        },
                    )),
            )
    }

    /// Render the messages area
    fn render_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let streaming_element = if self.is_streaming {
            Some(self.render_streaming_content(cx))
        } else {
            None
        };

        // Messages list with vertical scrollbar
        // Note: The container (in render_main_panel) handles flex_1 for sizing
        // We use size_full() here to fill the bounded container
        div()
            .id("messages-scroll-container")
            .flex()
            .flex_col()
            .p_3()
            .gap_3()
            .size_full()
            // Render all messages
            .children(
                self.current_messages
                    .iter()
                    .map(|msg| self.render_message(msg, cx)),
            )
            // Show streaming content if streaming
            .children(streaming_element)
            .overflow_y_scroll()
            .track_scroll(&self.messages_scroll_handle)
    }

    /// Render the main chat panel
    fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_selection = self.selected_chat_id.is_some();

        // Build titlebar
        let titlebar = div()
            .id("ai-titlebar")
            .flex()
            .items_center()
            .justify_between()
            .h(px(36.))
            .px_3()
            .bg(Self::get_vibrancy_sidebar_background()) // Semi-transparent sidebar
            .border_b_1()
            .border_color(cx.theme().border)
            // Left side: sidebar toggle (when collapsed) + chat title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .overflow_hidden()
                    // Sidebar toggle - only shown when sidebar is collapsed
                    .when(self.sidebar_collapsed, |d| {
                        d.child(self.render_sidebar_toggle(cx))
                    })
                    // Chat title (truncated)
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(
                                self.get_selected_chat()
                                    .map(|c| {
                                        if c.title.is_empty() {
                                            "New Chat".to_string()
                                        } else {
                                            c.title.clone()
                                        }
                                    })
                                    .unwrap_or_else(|| "AI Chat".to_string()),
                            ),
                    ),
            )
            .when(has_selection, |d| {
                d.child(
                    div().flex().items_center().gap_1().child(
                        Button::new("delete-chat")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Delete)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.delete_selected_chat(cx);
                            })),
                    ),
                )
            });

        // Build input area at bottom - Raycast-style layout:
        // Row 1: [+ icon] [input field with magenta border]
        // Row 2: [Model picker with spinner] ... [Submit ↵] | [Actions ⌘K]

        // Use theme accent color for input border (follows theme)
        let input_border_color = cx.theme().accent;

        // Check if we have a pending image to show
        let has_pending_image = self.pending_image.is_some();

        let input_area = div()
            .id("ai-input-area")
            .flex()
            .flex_col()
            .w_full()
            .bg(Self::get_vibrancy_sidebar_background()) // Semi-transparent sidebar
            .px_3()
            .pt_3()
            .pb_2() // Reduced bottom padding
            .gap_2()
            // Handle image file drops
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Pending image preview (shown above input when image is attached)
            .when(has_pending_image, |d| {
                d.child(self.render_pending_image_preview(cx))
            })
            // Input row with + icon and accent border
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .w_full()
                    // Plus button on the left - opens attachments picker
                    .child(
                        div()
                            .id("attachments-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.))
                            .rounded_full()
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.4))
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.showing_attachments_picker {
                                    this.hide_attachments_picker(cx);
                                } else {
                                    this.hide_all_dropdowns(cx);
                                    this.show_attachments_picker(window, cx);
                                }
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Plus.external_path())
                                    .size(px(14.))
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    // Input field with subtle accent border
                    .child(self.render_input_with_cursor(input_border_color, cx)),
            )
            // Bottom row: Model picker left, actions right (reduced padding)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .overflow_hidden()
                    // Left side: Model picker with potential spinner
                    .child(self.render_model_picker(cx))
                    // Right side: Submit and Actions as text labels
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .flex_shrink_0()
                            // Submit ↵ - clickable text
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px_2()
                                    .py(px(2.)) // Reduced vertical padding
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.submit_message(window, cx);
                                        }),
                                    )
                                    .child("Submit ↵"),
                            )
                            // Divider
                            .child(div().w(px(1.)).h(px(16.)).bg(cx.theme().border))
                            // Actions ⌘K - opens command bar with AI-specific actions
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px_2()
                                    .py(px(2.)) // Reduced vertical padding to match Submit
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.show_command_bar(window, cx);
                                        }),
                                    )
                                    .child("Actions ⌘K"),
                            ),
                    ),
            );

        // Determine what to show in the content area
        let has_messages = !self.current_messages.is_empty() || self.is_streaming;

        // Build main layout
        // Structure: titlebar (fixed) -> content area (flex_1, scrollable) -> input area (fixed)
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
            // Titlebar (fixed height)
            .child(titlebar)
            // Content area - this wrapper gets flex_1 to fill remaining space
            // The scrollable content goes inside this bounded container
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
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

    /// Get cached box shadows (computed once at construction)
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        self.cached_box_shadows.clone()
    }

    /// Compute box shadows from theme configuration (called once at construction)
    fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::load_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0)
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Update cached box shadows when theme changes
    pub fn update_theme(&mut self, _cx: &mut Context<Self>) {
        self.cached_box_shadows = Self::compute_box_shadows();
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // These use the same approach as the main window (render_script_list.rs)
    // to ensure vibrancy works correctly by using rgba() with hex colors
    // directly from the Script Kit theme.
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get main background color with vibrancy opacity
    /// Uses Script Kit theme hex colors directly (like main window)
    fn get_vibrancy_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get sidebar background color with vibrancy opacity
    fn get_vibrancy_sidebar_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        // Use title_bar background for sidebar (slightly different visual hierarchy)
        let bg_hex = sk_theme.colors.background.title_bar;
        // Sidebar uses title_bar opacity (0.65) for slightly more opaque
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }

    /// Get title bar background color with vibrancy opacity
    fn get_vibrancy_title_bar_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }
}

impl Focusable for AiApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Drop for AiApp {
    fn drop(&mut self) {
        // Clear the global window handle when AiApp is dropped
        // This ensures is_ai_window_open() returns false after the window closes
        // regardless of how it was closed (Cmd+W, traffic light, toggle, etc.)
        if let Some(window_handle) = AI_WINDOW.get() {
            if let Ok(mut guard) = window_handle.lock() {
                *guard = None;
                tracing::debug!("AiApp dropped - cleared global window handle");
            }
        }

        // Restore accessory app mode when AI window closes
        // This removes the app from Cmd+Tab and Dock (back to normal Script Kit behavior)
        // SAFETY: This runs on main thread (GPUI window lifecycle is main-thread only)
        crate::platform::set_accessory_app_mode();
    }
}

impl Render for AiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update cached theme values if theme has changed (hot-reload)
        self.maybe_update_theme_cache();

        // Persist bounds on change (ensures bounds saved even on traffic light close)
        self.maybe_persist_bounds(window);

        // Process command bar focus request FIRST (set after vibrancy window opens)
        // This ensures keyboard events route to the window's key handler for command bar navigation
        // CRITICAL: Must check this BEFORE focus_input to prevent input from stealing focus
        if self.needs_command_bar_focus {
            self.needs_command_bar_focus = false;
            self.focus_handle.focus(window, cx);
            crate::logging::log("AI", "Applied command bar focus in render");
        }
        // Process focus request flag (set by open_ai_window when bringing existing window to front)
        // Check both the instance flag and the global atomic flag
        // SKIP if command bar is open - the main focus_handle should have focus for arrow key routing
        else if !self.command_bar.is_open()
            && (self.needs_focus_input
                || AI_FOCUS_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst))
        {
            self.needs_focus_input = false;
            self.focus_input(window, cx);
        }

        // Process pending commands (for testing via stdin)
        for cmd in take_ai_commands() {
            match cmd {
                AiCommand::SetSearch(query) => {
                    self.search_state.update(cx, |state, cx| {
                        state.set_value(query.clone(), window, cx);
                    });
                    self.on_search_change(cx);
                    crate::logging::log("AI", &format!("Search filter set to: {}", query));
                }
                AiCommand::SetInput { text, submit } => {
                    self.input_state.update(cx, |state, cx| {
                        state.set_value(text.clone(), window, cx);
                        // Ensure cursor is at end of text with proper focus for editing
                        let text_len = state.text().len();
                        state.set_selection(text_len, text_len, window, cx);
                    });
                    crate::logging::log("AI", &format!("Input set to: {}", text));
                    if submit {
                        self.submit_message(window, cx);
                        crate::logging::log("AI", "Message submitted - streaming started");
                    }
                }
                AiCommand::SetInputWithImage {
                    text,
                    image_base64,
                    submit,
                } => {
                    self.input_state.update(cx, |state, cx| {
                        state.set_value(text.clone(), window, cx);
                        // Ensure cursor is at end of text with proper focus for editing
                        let text_len = state.text().len();
                        state.set_selection(text_len, text_len, window, cx);
                    });
                    // Store the pending image to be included with the next message
                    self.pending_image = Some(image_base64.clone());
                    crate::logging::log(
                        "AI",
                        &format!(
                            "Input set with image: {} chars text, {} chars base64",
                            text.len(),
                            image_base64.len()
                        ),
                    );
                    if submit {
                        self.submit_message(window, cx);
                        crate::logging::log(
                            "AI",
                            "Message with image submitted - streaming started",
                        );
                    }
                }
                AiCommand::InitializeWithPendingChat => {
                    self.initialize_with_pending_chat(window, cx);
                }
                AiCommand::ShowCommandBar => {
                    self.show_command_bar(window, cx);
                    crate::logging::log("AI", "Command bar shown via stdin command");
                }
                AiCommand::SimulateKey { key, modifiers } => {
                    self.handle_simulated_key(&key, &modifiers, window, cx);
                }
            }
        }

        let box_shadows = self.create_box_shadows();

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(Self::get_vibrancy_background()) // Semi-transparent for vibrancy
            .shadow(box_shadows)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // CRITICAL: Use capture_key_down to intercept keys BEFORE Input component handles them
            // This fires during the Capture phase (root->focused), before the Bubble phase (focused->root)
            // Without this, the Input component would consume arrow keys and command bar navigation fails
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Handle keyboard shortcuts
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // Debug: Log ALL key events to verify handler is firing
                crate::logging::log(
                    "AI",
                    &format!(
                        "AI capture_key_down: key='{}' command_bar_open={}",
                        key,
                        this.command_bar.is_open()
                    ),
                );

                // Handle Enter key for setup mode (no modifiers needed)
                if !modifiers.platform
                    && !modifiers.alt
                    && !modifiers.control
                    && matches!(key, "enter" | "return")
                    && this.available_models.is_empty()
                {
                    // In setup mode - Enter copies the setup command
                    this.copy_setup_command(cx);
                    window.activate_window();
                    cx.stop_propagation(); // Prevent further handling
                    return;
                }

                // Handle command bar navigation when it's open
                // This routes all relevant keys to the CommandBar
                // CRITICAL: Must stop propagation to prevent Input from consuming the keys
                if this.command_bar.is_open() {
                    crate::logging::log(
                        "AI",
                        &format!("AI capture_key_down (command_bar open): key='{}'", key),
                    );
                    match key {
                        "up" | "arrowup" => {
                            crate::logging::log(
                                "AI",
                                "AI window: routing UP to command_bar_select_prev",
                            );
                            this.command_bar_select_prev(cx);
                            cx.stop_propagation(); // Prevent Input from handling
                            return;
                        }
                        "down" | "arrowdown" => {
                            crate::logging::log(
                                "AI",
                                "AI window: routing DOWN to command_bar_select_next",
                            );
                            this.command_bar_select_next(cx);
                            cx.stop_propagation(); // Prevent Input from handling
                            return;
                        }
                        "enter" | "return" => {
                            this.execute_command_bar_action(window, cx);
                            cx.stop_propagation(); // Prevent Input from handling
                            return;
                        }
                        "escape" => {
                            this.hide_command_bar(cx);
                            cx.stop_propagation(); // Prevent further handling
                            return;
                        }
                        "backspace" | "delete" => {
                            this.command_bar_handle_backspace(cx);
                            cx.stop_propagation(); // Prevent Input from handling
                            return;
                        }
                        _ => {
                            // Handle printable characters for search (when no modifiers)
                            if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                // Get the character from the keystroke
                                if let Some(ch) = key.chars().next() {
                                    if ch.is_alphanumeric()
                                        || ch.is_whitespace()
                                        || ch == '-'
                                        || ch == '_'
                                    {
                                        this.command_bar_handle_char(ch, cx);
                                        cx.stop_propagation(); // Prevent Input from handling
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    // Don't fall through to other handlers when command bar is open
                    return;
                }

                // Handle model picker navigation
                if this.showing_model_picker {
                    match key {
                        "up" | "arrowup" => {
                            this.model_picker_select_prev(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "down" | "arrowdown" => {
                            this.model_picker_select_next(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "enter" | "return" => {
                            this.select_model_from_picker(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "escape" => {
                            this.hide_model_picker(cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle presets dropdown navigation
                if this.showing_presets_dropdown {
                    match key {
                        "up" | "arrowup" => {
                            this.presets_select_prev(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "down" | "arrowdown" => {
                            this.presets_select_next(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "enter" | "return" => {
                            this.create_chat_with_preset(window, cx);
                            cx.stop_propagation();
                            return;
                        }
                        "escape" => {
                            this.hide_presets_dropdown(cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle attachments picker
                if this.showing_attachments_picker && key == "escape" {
                    this.hide_attachments_picker(cx);
                    cx.stop_propagation();
                    return;
                }

                // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
                if modifiers.platform {
                    match key {
                        // Cmd+K to toggle command bar (like Raycast)
                        "k" => {
                            if this.command_bar.is_open() {
                                this.hide_command_bar(cx);
                            } else {
                                this.hide_all_dropdowns(cx);
                                this.show_command_bar(window, cx);
                            }
                        }
                        // Cmd+N for new chat (with Shift for presets)
                        "n" => {
                            if modifiers.shift {
                                // Cmd+Shift+N opens presets dropdown
                                this.hide_all_dropdowns(cx);
                                this.show_presets_dropdown(window, cx);
                            } else {
                                this.create_chat(window, cx);
                            }
                        }
                        // Cmd+M to toggle model picker
                        "m" => {
                            if this.showing_model_picker {
                                this.hide_model_picker(cx);
                            } else {
                                this.hide_all_dropdowns(cx);
                                this.show_model_picker(window, cx);
                            }
                        }
                        "enter" | "return" => this.submit_message(window, cx),
                        // Cmd+\ to toggle sidebar (like Raycast)
                        "\\" | "backslash" => this.toggle_sidebar(cx),
                        // Cmd+B also toggles sidebar (common convention)
                        "b" => this.toggle_sidebar(cx),
                        // Cmd+V for paste - check for images first
                        "v" => {
                            // Try to paste an image; if not found, let normal text paste happen
                            // We don't need to prevent the event since the Input handles text paste
                            this.handle_paste_for_image(cx);
                        }
                        // Cmd+W closes the AI window (standard macOS pattern)
                        "w" => {
                            // Save bounds before closing
                            let wb = window.window_bounds();
                            crate::window_state::save_window_from_gpui(
                                crate::window_state::WindowRole::Ai,
                                wb,
                            );
                            window.remove_window();
                        }
                        _ => {}
                    }
                }

                // Escape closes any open dropdown
                if key == "escape"
                    && (this.command_bar.is_open()
                        || this.showing_model_picker
                        || this.showing_presets_dropdown
                        || this.showing_attachments_picker)
                {
                    this.hide_all_dropdowns(cx);
                }
            }))
            .child(self.render_sidebar(cx))
            .child(self.render_main_panel(cx))
            // Overlay dropdowns (only one at a time)
            // NOTE: Command bar now renders in a separate vibrancy window (not inline)
            .when(self.showing_model_picker, |el| {
                el.child(self.render_model_picker_dropdown(cx))
            })
            .when(self.showing_presets_dropdown, |el| {
                el.child(self.render_presets_dropdown(cx))
            })
            .when(self.showing_attachments_picker, |el| {
                el.child(self.render_attachments_picker(cx))
            })
    }
}

impl AiApp {
    /// Render the command bar overlay (Raycast-style Cmd+K menu)
    /// NOTE: This is kept for reference but no longer used - command bar now uses separate vibrancy window
    #[allow(dead_code)]
    /// Render the command bar overlay (deprecated - CommandBar now uses separate vibrancy window)
    ///
    /// This function is kept for API compatibility but returns an empty element.
    /// The CommandBar component now manages its own window via `open_actions_window()`.
    #[allow(dead_code)]
    fn render_command_bar_overlay(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Command bar now renders in a separate vibrancy window (not inline)
        // See CommandBar component for window management
        div().id("command-bar-overlay-deprecated")
    }

    /// Render the model picker dropdown overlay
    fn render_model_picker_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let accent_fg = theme.accent_foreground;
        let fg = theme.foreground;

        // Build model items
        let model_items: Vec<_> = self
            .available_models
            .iter()
            .enumerate()
            .map(|(idx, model)| {
                let is_selected = idx == self.model_picker_selected_index;
                let model_id = model.id.clone();
                let display_name = model.display_name.clone();
                let provider = model.provider.clone();

                div()
                    .id(SharedString::from(format!("model-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.model_picker_selected_index = idx;
                        this.select_model_from_picker(cx);
                    }))
                    // Model name
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected { accent_fg } else { fg })
                                    .child(display_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_selected {
                                        accent_fg.opacity(0.7)
                                    } else {
                                        muted_fg
                                    })
                                    .child(format!("{} • {}", provider, model_id)),
                            ),
                    )
                    // Check mark if currently selected model
                    .when(
                        self.selected_model.as_ref().map(|m| &m.id) == Some(&model_id),
                        |el| {
                            el.child(
                                svg()
                                    .external_path(LocalIconName::Check.external_path())
                                    .size(px(14.))
                                    .text_color(if is_selected { accent_fg } else { accent }),
                            )
                        },
                    )
            })
            .collect();

        // Overlay
        div()
            .id("model-picker-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_end()
            .justify_center()
            .pb_20() // Position above the input area
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_model_picker(cx);
            }))
            .child(
                div()
                    .id("model-picker-container")
                    .w(px(350.0))
                    .max_h(px(300.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    .shadow_lg()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("Select Model"),
                    )
                    // Model list
                    .child(
                        div()
                            .id("model-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_1()
                            .children(model_items),
                    ),
            )
    }

    /// Render the presets dropdown overlay
    fn render_presets_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let accent_fg = theme.accent_foreground;
        let fg = theme.foreground;

        // Build preset items
        let preset_items: Vec<_> = self
            .presets
            .iter()
            .enumerate()
            .map(|(idx, preset)| {
                let is_selected = idx == self.presets_selected_index;
                let icon = preset.icon;
                let name = preset.name.to_string();
                let description = preset.description.to_string();

                div()
                    .id(SharedString::from(format!("preset-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.presets_selected_index = idx;
                        this.create_chat_with_preset(window, cx);
                    }))
                    // Icon
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(px(18.))
                            .text_color(if is_selected { accent_fg } else { muted_fg }),
                    )
                    // Name and description
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected { accent_fg } else { fg })
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_selected {
                                        accent_fg.opacity(0.7)
                                    } else {
                                        muted_fg
                                    })
                                    .child(description),
                            ),
                    )
            })
            .collect();

        // Overlay positioned near the new chat button
        div()
            .id("presets-dropdown-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_start()
            .justify_start()
            .pt_12()
            .pl_4()
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_presets_dropdown(cx);
            }))
            .child(
                div()
                    .id("presets-dropdown-container")
                    .w(px(300.0))
                    .max_h(px(350.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    .shadow_lg()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("New Chat with Preset"),
                    )
                    // Preset list
                    .child(
                        div()
                            .id("preset-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_1()
                            .children(preset_items),
                    )
                    // Footer hint
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(border_color)
                            .text_xs()
                            .text_color(muted_fg)
                            .child("Select a preset to start a new chat"),
                    ),
            )
    }

    /// Render the attachments picker overlay
    fn render_attachments_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let fg = theme.foreground;

        // Attachment options
        let options = [
            ("file", "Add File", LocalIconName::File, "Browse for a file"),
            ("image", "Add Image", LocalIconName::File, "Add an image"),
            (
                "clipboard",
                "Paste from Clipboard",
                LocalIconName::Copy,
                "⌘V",
            ),
        ];

        let option_items: Vec<_> = options
            .iter()
            .map(|(id, name, icon, hint)| {
                let id_str = *id;
                let name_str = name.to_string();
                let icon_name = *icon;
                let hint_str = hint.to_string();

                div()
                    .id(SharedString::from(format!("attach-{}", id_str)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .hover(|el| el.bg(accent.opacity(0.5)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.hide_attachments_picker(cx);
                        match id_str {
                            "file" => {
                                info!("File picker not implemented yet");
                            }
                            "image" => {
                                info!("Image picker not implemented yet");
                            }
                            "clipboard" => {
                                this.paste_image_from_clipboard(cx);
                            }
                            _ => {}
                        }
                    }))
                    // Icon
                    .child(
                        svg()
                            .external_path(icon_name.external_path())
                            .size(px(16.))
                            .text_color(muted_fg),
                    )
                    // Name
                    .child(div().flex_1().text_sm().text_color(fg).child(name_str))
                    // Hint
                    .child(div().text_xs().text_color(muted_fg).child(hint_str))
            })
            .collect();

        // Show pending attachments if any
        let pending_items: Vec<_> = self
            .pending_attachments
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                div()
                    .id(SharedString::from(format!("pending-{}", idx)))
                    .px_3()
                    .py_1()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(accent.opacity(0.2))
                    // File icon
                    .child(
                        svg()
                            .external_path(LocalIconName::File.external_path())
                            .size(px(14.))
                            .text_color(accent),
                    )
                    // Filename
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(filename),
                    )
                    // Remove button
                    .child(
                        div()
                            .id(SharedString::from(format!("remove-{}", idx)))
                            .cursor_pointer()
                            .hover(|el| el.text_color(gpui::red()))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_attachment(idx, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Close.external_path())
                                    .size(px(12.))
                                    .text_color(muted_fg),
                            ),
                    )
            })
            .collect();

        // Overlay
        div()
            .id("attachments-picker-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_end()
            .justify_start()
            .pb_20()
            .pl_4()
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_attachments_picker(cx);
            }))
            .child(
                div()
                    .id("attachments-picker-container")
                    .w(px(280.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    .shadow_lg()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("Add Attachment"),
                    )
                    // Pending attachments (if any)
                    .when(!self.pending_attachments.is_empty(), |el| {
                        el.child(
                            div()
                                .px_2()
                                .py_1()
                                .border_b_1()
                                .border_color(border_color)
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(pending_items),
                        )
                    })
                    // Options
                    .child(div().p_1().children(option_items)),
            )
    }
}

/// Initialize gpui-component theme and sync with Script Kit theme
fn ensure_theme_initialized(cx: &mut App) {
    // Use the shared theme sync function from src/theme/gpui_integration.rs
    crate::theme::sync_gpui_component_theme(cx);
    info!("AI window theme synchronized with Script Kit");
}

/// Toggle the AI window (open if closed, bring to front if open)
///
/// The AI window behaves as a NORMAL window (not a floating panel):
/// - Can go behind other windows when it loses focus
/// - Hotkey brings it to front and focuses it
/// - Does NOT affect other windows (main window, notes window)
/// - Does NOT hide the app when closed
pub fn open_ai_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("AI", "open_ai_window called - checking state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock.
    // WindowHandle is Copy, so we just dereference to get it out.
    let existing_handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    // Check if window already exists and is valid
    if let Some(handle) = existing_handle {
        // Window exists - check if it's valid (lock is released)
        let window_valid = handle
            .update(cx, |_root, window, _cx| {
                // Window is valid - bring it to front and focus it
                window.activate_window();
            })
            .is_ok();

        if window_valid {
            logging::log("AI", "AI window exists - bringing to front and focusing");

            // Ensure regular app mode (in case it was switched back to accessory)
            crate::platform::set_regular_app_mode();

            // Move the window to the display containing the mouse cursor
            // This ensures the AI window appears on the same screen as where the user is working
            let new_bounds = crate::platform::calculate_centered_bounds_on_mouse_display(size(
                px(900.),
                px(700.),
            ));
            let _ = handle.update(cx, |_root, window, cx| {
                crate::window_ops::queue_move(new_bounds, window, cx);
            });

            // Activate the app to ensure the window can receive focus
            cx.activate(true);

            // Request focus on the input field via the global flag.
            // AiApp checks this flag in render() and focuses if set.
            // This avoids the need for a global Entity<AiApp> reference which caused memory leaks.
            AI_FOCUS_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);

            // Notify to trigger re-render which will process the focus request
            let _ = handle.update(cx, |_root, _window, cx| {
                cx.notify();
            });

            return Ok(());
        }

        // Window handle was invalid, clear it
        logging::log("AI", "AI window handle was invalid - creating new");
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = None;
        }
    }

    // Create new window
    logging::log("AI", "Creating new AI window");
    info!("Opening new AI window");

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate position: try per-display saved position first, then centered on mouse display
    // Use mouse display positioning so AI window appears on the same screen as the cursor
    let displays = crate::platform::get_macos_displays();
    let bounds = if let Some((mouse_x, mouse_y)) = crate::platform::get_global_mouse_position() {
        if let Some((saved, _display)) =
            crate::window_state::get_ai_position_for_mouse_display(mouse_x, mouse_y, &displays)
        {
            // Use saved per-display position
            saved.to_gpui().get_bounds()
        } else {
            // Fall back to centered on mouse display
            crate::platform::calculate_centered_bounds_on_mouse_display(size(px(900.), px(700.)))
        }
    } else {
        // Mouse position unavailable, fall back to centered
        crate::platform::calculate_centered_bounds_on_mouse_display(size(px(900.), px(700.)))
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Script Kit AI".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        window_background,
        focus: true,
        show: true,
        // IMPORTANT: Use Normal window kind (not PopUp) so it behaves like a regular window
        // This allows it to go behind other windows and participate in normal window ordering
        kind: gpui::WindowKind::Normal,
        ..Default::default()
    };

    // Create a holder for the AiApp entity so we can focus it after window creation.
    // NOTE: This is a LOCAL holder, not stored globally, to avoid memory leaks.
    let ai_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<AiApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let ai_app_holder_clone = ai_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| AiApp::new(window, cx));
        // Store the AiApp entity temporarily for immediate focus after window creation
        *ai_app_holder_clone.lock().unwrap() = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // Activate the app and window so user can immediately start typing
    cx.activate(true);
    let _ = handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    });

    // Focus the input field immediately after window creation
    // Use the local entity reference (not stored globally to avoid leaks)
    if let Some(ai_app) = ai_app_holder.lock().ok().and_then(|mut h| h.take()) {
        let _ = handle.update(cx, |_root, window, cx| {
            ai_app.update(cx, |app, cx| {
                app.focus_input(window, cx);
            });
        });
    }

    // Store the window handle (release lock immediately after)
    {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(handle);
        }
    }

    // Switch to regular app mode so AI window appears in Cmd+Tab
    // This is unique to the AI window - other windows (main, notes) stay in accessory mode
    // The mode is restored to accessory when the AI window closes (see AiApp::drop)
    crate::platform::set_regular_app_mode();

    // NOTE: We do NOT configure as floating panel - this is a normal window
    // that can go behind other windows
    // However, we DO want vibrancy configuration for proper blur effect
    configure_ai_window_vibrancy();

    // NOTE: Theme hot-reload is now handled by the centralized ThemeService
    // (crate::theme::service::ensure_theme_service) which is started once at app init.
    // This eliminates per-window theme watcher tasks and their potential for leaks.

    Ok(())
}

/// Pending chat to initialize after window opens.
/// This is used by open_ai_window_with_chat to pass messages to the newly created window.
#[allow(clippy::type_complexity)]
static AI_PENDING_CHAT: std::sync::OnceLock<std::sync::Mutex<Option<Vec<(MessageRole, String)>>>> =
    std::sync::OnceLock::new();

fn get_pending_chat() -> &'static std::sync::Mutex<Option<Vec<(MessageRole, String)>>> {
    AI_PENDING_CHAT.get_or_init(|| std::sync::Mutex::new(None))
}

/// Open the AI window with an existing conversation.
///
/// This function:
/// 1. Opens the AI window (or brings it to front if already open)
/// 2. Creates a new chat with the provided messages
/// 3. Displays the chat immediately
///
/// Use this for "Continue in Chat" functionality to transfer a conversation
/// from the chat prompt to the AI window.
pub fn open_ai_window_with_chat(cx: &mut App, messages: Vec<(MessageRole, String)>) -> Result<()> {
    use crate::logging;

    logging::log(
        "AI",
        &format!(
            "open_ai_window_with_chat called with {} messages",
            messages.len()
        ),
    );

    // Store the pending chat messages
    if let Ok(mut pending) = get_pending_chat().lock() {
        *pending = Some(messages);
    }

    // Open or bring the window to front
    open_ai_window(cx)?;

    // Queue a command to initialize the chat with pending messages
    push_ai_command(AiCommand::InitializeWithPendingChat);

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }

    Ok(())
}

/// Close the AI window
pub fn close_ai_window(cx: &mut App) {
    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock
    // If handle.update() causes Drop to fire synchronously and tries to acquire
    // the same lock, we would deadlock. Taking the handle out first avoids this.
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            // Save window bounds per-display before closing
            let wb = window.window_bounds();
            let persisted = crate::window_state::PersistedWindowBounds::from_gpui(wb);
            let displays = crate::platform::get_macos_displays();
            // Find which display the window center is on
            if let Some(display) =
                crate::window_state::find_display_for_bounds(&persisted, &displays)
            {
                crate::window_state::save_ai_position_for_display(display, persisted);
            } else {
                // Fallback to legacy save if display not found
                crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Ai, wb);
            }
            window.remove_window();
        });
    }

    // Clear the focus request flag (no longer needed after window closes)
    AI_FOCUS_REQUESTED.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Check if the AI window is currently open
///
/// Returns true if the AI window exists and is valid.
/// This is used by other parts of the app to check if AI is open
/// without affecting it.
pub fn is_ai_window_open() -> bool {
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap();
    guard.is_some()
}

/// Set the search filter text in the AI window.
/// Used for testing the search functionality via stdin commands.
pub fn set_ai_search(cx: &mut App, query: &str) {
    use crate::logging;

    // Queue the command and notify the window to process it in render()
    // This avoids the need for a global Entity<AiApp> reference which caused memory leaks.
    push_ai_command(AiCommand::SetSearch(query.to_string()));

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
        logging::log("AI", &format!("Set AI search filter: {}", query));
    } else {
        logging::log("AI", "Cannot set search - AI window not found");
    }
}

/// Set the main input text in the AI window and optionally submit.
/// Used for testing the streaming functionality via stdin commands.
pub fn set_ai_input(cx: &mut App, text: &str, submit: bool) {
    use crate::logging;

    // Queue the command and notify the window to process it in render()
    // This avoids the need for a global Entity<AiApp> reference which caused memory leaks.
    push_ai_command(AiCommand::SetInput {
        text: text.to_string(),
        submit,
    });

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    } else {
        logging::log("AI", "Cannot set input - AI window not open");
    }
}

/// Set the main input text with an attached image in the AI window and optionally submit.
/// The image should be base64 encoded PNG data.
/// Used by AI commands like "Send Screen to AI Chat".
pub fn set_ai_input_with_image(cx: &mut App, text: &str, image_base64: &str, submit: bool) {
    use crate::logging;

    // Queue the command and notify the window to process it in render()
    push_ai_command(AiCommand::SetInputWithImage {
        text: text.to_string(),
        image_base64: image_base64.to_string(),
        submit,
    });

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    } else {
        logging::log("AI", "Cannot set input with image - AI window not open");
    }
}

/// Show the AI command bar (Cmd+K menu) in the AI window.
///
/// This is triggered by the stdin command `{"type":"showAiCommandBar"}`.
/// Opens the AI window if not already open, then shows the command bar overlay.
pub fn show_ai_command_bar(cx: &mut App) {
    use crate::logging;

    // First ensure the AI window is open
    if !is_ai_window_open() {
        if let Err(e) = open_ai_window(cx) {
            logging::log("AI", &format!("Failed to open AI window: {}", e));
            return;
        }
    }

    // Queue the command and notify the window to process it in render()
    // This avoids the need for direct entity access which caused memory leaks.
    push_ai_command(AiCommand::ShowCommandBar);

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
        logging::log("AI", "Showing AI command bar");
    } else {
        logging::log("AI", "Cannot show command bar - AI window handle not found");
    }
}

/// Simulate a key press in the AI window.
///
/// This is triggered by the stdin command `{"type":"simulateAiKey","key":"up","modifiers":["cmd"]}`.
/// Used for testing keyboard navigation in the AI window, especially the command bar.
pub fn simulate_ai_key(key: &str, modifiers: Vec<String>) {
    use crate::logging;

    // Check if AI window is open
    if !is_ai_window_open() {
        logging::log("AI", "Cannot simulate key - AI window not open");
        return;
    }

    // Queue the command
    push_ai_command(AiCommand::SimulateKey {
        key: key.to_string(),
        modifiers,
    });

    // The command is queued and will be processed in the next render cycle
    // We don't have `cx` here so can't notify, but that's okay since rendering happens continuously
    logging::log(
        "AI",
        &format!(
            "Queued SimulateKey: key='{}' - will process on next render",
            key
        ),
    );
}

/// Configure vibrancy and app switcher participation for the AI window.
///
/// This sets VibrantDark appearance and configures NSVisualEffectViews
/// for proper blur effect. Unlike other windows, the AI window:
/// - Does NOT float above other windows (stays at default level 0)
/// - DOES participate in Cmd+Tab app switcher cycling
///
/// This makes AI the only window that can be Cmd+Tab'd back to.
#[cfg(target_os = "macos")]
fn configure_ai_window_vibrancy() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - configure vibrancy
                        crate::platform::configure_secondary_window_vibrancy(window, "AI");

                        // Configure as a regular window that participates in Cmd+Tab:
                        // - Keep default window level (0) so it doesn't float
                        // - Add ParticipatesInCycle (128) so it appears in Cmd+Tab
                        // - Remove IgnoresCycle (64) if somehow set
                        // - Add MoveToActiveSpace (2) so it follows the user
                        let current: u64 = msg_send![window, collectionBehavior];
                        // Clear IgnoresCycle bit, set ParticipatesInCycle and MoveToActiveSpace
                        let desired: u64 = (current & !64) | 128 | 2;
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Log detailed breakdown of collection behavior bits
                        let has_participates = (desired & 128) != 0;
                        let has_ignores = (desired & 64) != 0;
                        let has_move_to_active = (desired & 2) != 0;

                        logging::log(
                            "PANEL",
                            &format!(
                                "AI window: Cmd+Tab config - behavior={}->{} [ParticipatesInCycle={}, IgnoresCycle={}, MoveToActiveSpace={}]",
                                current, desired, has_participates, has_ignores, has_move_to_active
                            ),
                        );
                        logging::log(
                            "PANEL",
                            "AI window: WILL appear in Cmd+Tab app switcher (unique among Script Kit windows)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: AI window not found by title for vibrancy config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_ai_window_vibrancy() {
    // No-op on non-macOS platforms
}

/// Configure the AI window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_ai_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - configure it

                        // NSFloatingWindowLevel = 3
                        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
                        let floating_level: i64 = 3;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // Get current collection behavior to preserve existing flags
                        let current: u64 = msg_send![window, collectionBehavior];
                        // OR in MoveToActiveSpace (2) + FullScreenAuxiliary (256)
                        let desired: u64 = current | 2 | 256;
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        // ═══════════════════════════════════════════════════════════════════════════
                        // VIBRANCY CONFIGURATION - Match main window for consistent blur
                        // ═══════════════════════════════════════════════════════════════════════════
                        crate::platform::configure_secondary_window_vibrancy(window, "AI");

                        logging::log(
                            "PANEL",
                            "AI window configured as floating panel (level=3, MoveToActiveSpace, vibrancy)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: AI window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_ai_as_floating_panel() {
    // No-op on non-macOS platforms
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that streaming state guards work correctly for chat-switch scenarios
    #[test]
    fn test_streaming_generation_guard_logic() {
        // Simulate the guard check logic used in streaming updates
        let update_chat_id = ChatId::new();
        let streaming_chat_id: Option<ChatId> = Some(update_chat_id);
        let streaming_generation: u64 = 5;

        // Scenario 1: Matching generation and chat - should NOT be stale
        let update_generation = 5;
        let is_stale =
            streaming_generation != update_generation || streaming_chat_id != Some(update_chat_id);
        assert!(
            !is_stale,
            "Matching generation and chat should not be stale"
        );

        // Scenario 2: Generation mismatch - should be stale (old streaming task)
        let old_generation = 4;
        let is_stale =
            streaming_generation != old_generation || streaming_chat_id != Some(update_chat_id);
        assert!(is_stale, "Old generation should be stale");

        // Scenario 3: Chat ID mismatch - should be stale (user switched chats)
        let different_chat_id = ChatId::new();
        let is_stale = streaming_generation != update_generation
            || streaming_chat_id != Some(different_chat_id);
        assert!(is_stale, "Different chat ID should be stale");

        // Scenario 4: No streaming chat - should be stale
        let no_streaming: Option<ChatId> = None;
        let is_stale =
            streaming_generation != update_generation || no_streaming != Some(update_chat_id);
        assert!(is_stale, "No streaming chat should be stale");
    }

    /// Test that generation counter wraps correctly
    #[test]
    fn test_streaming_generation_wrapping() {
        let mut generation: u64 = u64::MAX;

        // Simulate multiple streaming sessions
        for expected in [0, 1, 2, 3, 4] {
            generation = generation.wrapping_add(1);
            assert_eq!(generation, expected, "Generation should wrap correctly");
        }
    }

    /// Test the submit_message guard logic - should only block if streaming
    /// for the SAME chat
    #[test]
    fn test_submit_while_streaming_different_chat() {
        // Setup: streaming in chat A, trying to submit in chat B
        let chat_a = ChatId::new();
        let chat_b = ChatId::new();

        let is_streaming = true;
        let streaming_chat_id = Some(chat_a);
        let selected_chat_id = Some(chat_b);

        // The guard: block only if streaming AND same chat
        let should_block = is_streaming && streaming_chat_id == selected_chat_id;
        assert!(
            !should_block,
            "Should NOT block submission when streaming different chat"
        );

        // Same chat scenario should block
        let selected_chat_id = Some(chat_a);
        let should_block = is_streaming && streaming_chat_id == selected_chat_id;
        assert!(
            should_block,
            "Should block submission when streaming same chat"
        );
    }

    /// Test ChatId comparison behavior
    #[test]
    fn test_chat_id_equality() {
        let id1 = ChatId::new();
        let id2 = ChatId::new();
        let id1_copy = id1;

        assert_eq!(id1, id1_copy, "Same ID should be equal");
        assert_ne!(id1, id2, "Different IDs should not be equal");
        assert_eq!(Some(id1), Some(id1_copy), "Option<ChatId> equality works");
        assert_ne!(Some(id1), Some(id2), "Option<ChatId> inequality works");
        assert_ne!(Some(id1), None, "Some vs None inequality works");
    }
}
