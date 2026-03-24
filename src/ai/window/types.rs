use super::*;
use gpui::Pixels;

pub(super) const SP_1: Pixels = px(2.);
pub(super) const SP_2: Pixels = px(4.);
pub(super) const SP_3: Pixels = px(6.);
pub(super) const SP_4: Pixels = px(8.);
pub(super) const SP_5: Pixels = px(10.);
pub(super) const SP_6: Pixels = px(12.);
pub(super) const SP_7: Pixels = px(14.);
pub(super) const SP_8: Pixels = px(16.);
pub(super) const SP_9: Pixels = px(20.);
pub(super) const SP_10: Pixels = px(24.);

pub(super) const S0: Pixels = px(0.);
pub(super) const S1: Pixels = px(4.);
pub(super) const S2: Pixels = px(8.);
pub(super) const S3: Pixels = px(12.);
pub(super) const S4: Pixels = px(16.);
pub(super) const S5: Pixels = px(20.);
pub(super) const S6: Pixels = px(24.);
pub(super) const S7: Pixels = px(32.);
pub(super) const S8: Pixels = px(40.);
pub(super) const S9: Pixels = px(48.);

// -- Border radii --
pub(super) const R_SM: Pixels = px(8.);
pub(super) const R_MD: Pixels = px(10.);
pub(super) const R_LG: Pixels = px(12.);
pub(super) const R_XL: Pixels = px(16.);

// -- Icon sizes --
pub(super) const ICON_XS: Pixels = px(12.);
pub(super) const ICON_SM: Pixels = px(14.);
pub(super) const ICON_MD: Pixels = px(16.);

// -- Layout constants --
pub(super) const SIDEBAR_W: Pixels = px(240.);
pub(super) const TITLEBAR_H: Pixels = px(48.);
pub(super) const SIDEBAR_ROW_H: Pixels = px(52.);
pub(super) const COMPOSER_H: Pixels = px(40.);
/// Maximum height the composer input area can grow to (approx 6 lines).
pub(super) const COMPOSER_MAX_H: Pixels = px(200.);
pub(super) const SEARCH_H: Pixels = px(36.);
pub(super) const SIDEBAR_INSET_X: Pixels = S3;
pub(super) const PANEL_INSET_X: Pixels = S4;

// -- Setup card constants --
pub(super) const SETUP_ICON_CONTAINER_SIZE: Pixels = px(80.);
pub(super) const SETUP_DESCRIPTION_MAX_W: Pixels = px(380.);
pub(super) const SETUP_FEEDBACK_MAX_W: Pixels = px(340.);
pub(super) const SETUP_API_KEY_MAX_W: Pixels = px(400.);

// -- Sidebar search icon --
pub(super) const SIDEBAR_SEARCH_ICON_SIZE: Pixels = px(10.);

// -- Message bubble tokens --
pub(super) const MSG_PX: Pixels = S3;
pub(super) const MSG_PY: Pixels = S2;
pub(super) const MSG_GAP: Pixels = S6;
pub(super) const MSG_GAP_CONTINUATION: Pixels = S2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum AiWindowMode {
    #[default]
    Full,
    Mini,
}

impl AiWindowMode {
    pub(super) fn is_mini(self) -> bool {
        matches!(self, Self::Mini)
    }

    pub(super) fn default_width(self) -> f32 {
        match self {
            Self::Full => 900.0,
            Self::Mini => MINI_WINDOW_DEFAULT_W,
        }
    }

    pub(super) fn default_height(self) -> f32 {
        match self {
            Self::Full => 700.0,
            Self::Mini => MINI_WINDOW_DEFAULT_H,
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Full => "Script Kit AI",
            Self::Mini => "Mini AI",
        }
    }
}

/// Auto-collapse messages longer than this many characters.
/// Used by both the collapse decision logic and the toggle-button visibility gate.
pub(super) const MSG_COLLAPSE_CHAR_THRESHOLD: usize = 800;

/// Result of the pure collapse-decision computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CollapseDecision {
    pub(super) char_count: usize,
    pub(super) threshold: usize,
    pub(super) should_collapse: bool,
}

/// Pure function: decide whether a message should auto-collapse based on length.
///
/// This does NOT consider explicit user overrides (expanded/collapsed sets);
/// callers must check those first.
pub(super) fn compute_collapse_decision(char_count: usize) -> CollapseDecision {
    let should_collapse = char_count > MSG_COLLAPSE_CHAR_THRESHOLD;
    tracing::debug!(
        char_count,
        threshold = MSG_COLLAPSE_CHAR_THRESHOLD,
        should_collapse,
        "ai_message_collapse_decision"
    );
    CollapseDecision {
        char_count,
        threshold: MSG_COLLAPSE_CHAR_THRESHOLD,
        should_collapse,
    }
}

/// Welcome screen suggestion card definitions.
/// Shared between render_welcome (UI) and render_keydown (Cmd+1-4 shortcuts).
pub(super) const WELCOME_SUGGESTIONS: [(&str, &str); 4] = [
    ("Write a script", "to monitor my clipboard for changes"),
    ("Create a menu bar shortcut", "that opens my project folder"),
    ("Help me build", "a quick search for my bookmarks"),
    ("Generate a scriptlet", "that reformats selected text"),
];

#[cfg(test)]
mod message_spacing_tests {
    use super::*;

    #[test]
    fn test_message_spacing_constants_preserve_transition_separation_when_using_scale_tokens() {
        assert_eq!(MSG_PX, S3);
        assert_eq!(MSG_PY, S2);
        assert_eq!(MSG_GAP, S6);
        assert_eq!(MSG_GAP_CONTINUATION, S2);
        assert!(MSG_GAP / MSG_GAP_CONTINUATION > 1.0);
    }
}

#[cfg(test)]
mod layout_token_tests {
    use super::*;

    #[test]
    fn test_layout_tokens_define_expected_4px_grid_and_component_sizing() {
        assert_eq!(S0, px(0.));
        assert_eq!(S1, px(4.));
        assert_eq!(S2, px(8.));
        assert_eq!(S3, px(12.));
        assert_eq!(S4, px(16.));
        assert_eq!(S5, px(20.));
        assert_eq!(S6, px(24.));
        assert_eq!(S7, px(32.));
        assert_eq!(S8, px(40.));
        assert_eq!(S9, px(48.));

        assert_eq!(R_SM, px(8.));
        assert_eq!(R_MD, px(10.));
        assert_eq!(R_LG, px(12.));
        assert_eq!(R_XL, px(16.));

        assert_eq!(SIDEBAR_ROW_H, px(52.));
        assert_eq!(COMPOSER_H, px(40.));
        assert_eq!(SEARCH_H, px(36.));
        assert_eq!(SIDEBAR_INSET_X, S3);
        assert_eq!(PANEL_INSET_X, S4);
        assert_eq!(TITLEBAR_H, px(48.));

        assert_eq!(SETUP_ICON_CONTAINER_SIZE, px(80.));
        assert_eq!(SETUP_DESCRIPTION_MAX_W, px(380.));
        assert_eq!(SETUP_FEEDBACK_MAX_W, px(340.));
        assert_eq!(SETUP_API_KEY_MAX_W, px(400.));
        assert_eq!(SIDEBAR_SEARCH_ICON_SIZE, px(10.));
    }
}

#[cfg(test)]
mod ai_window_mode_tests {
    use super::*;

    #[test]
    fn test_ai_window_mode_defaults_to_full() {
        assert_eq!(AiWindowMode::default(), AiWindowMode::Full);
        assert!(!AiWindowMode::Full.is_mini());
        assert!(AiWindowMode::Mini.is_mini());
    }

    #[test]
    fn test_ai_window_mode_uses_expected_titles_and_dimensions() {
        assert_eq!(AiWindowMode::Full.default_width(), 900.0);
        assert_eq!(AiWindowMode::Full.default_height(), 700.0);
        assert_eq!(AiWindowMode::Full.title(), "Script Kit AI");

        assert_eq!(AiWindowMode::Mini.default_width(), MINI_WINDOW_DEFAULT_W);
        assert_eq!(AiWindowMode::Mini.default_height(), MINI_WINDOW_DEFAULT_H);
        assert_eq!(AiWindowMode::Mini.title(), "Mini AI");
    }

    #[test]
    fn mini_layout_contract_uses_expected_dimensions() {
        assert_eq!(MINI_WINDOW_DEFAULT_W, 720.0);
        assert_eq!(MINI_WINDOW_DEFAULT_H, 440.0);
        assert_eq!(MINI_TITLEBAR_H, px(44.));
        assert_eq!(MINI_CONTENT_MAX_W, px(760.));
        assert_eq!(MINI_HISTORY_OVERLAY_W, px(320.));
        assert_eq!(MINI_HISTORY_OVERLAY_MAX_H, px(420.));
        assert_eq!(MINI_HISTORY_OVERLAY_TOP, px(48.));
        assert_eq!(MINI_BTN_SIZE, px(28.));
    }

    #[test]
    fn test_ai_window_mode_command_names_match_variants() {
        assert_eq!(
            AiCommand::SetWindowMode(AiWindowMode::Full).name(),
            "set_window_mode_full"
        );
        assert_eq!(
            AiCommand::SetWindowMode(AiWindowMode::Mini).name(),
            "set_window_mode_mini"
        );
    }
}

// -- Image thumbnails --
pub(super) const IMG_THUMBNAIL_SIZE: Pixels = px(120.);
pub(super) const IMG_PENDING_THUMB_SIZE: Pixels = px(36.);
pub(super) const IMG_PENDING_THUMB_RADIUS: Pixels = SP_2; // 4px

// -- Titlebar layout --
pub(super) const TITLEBAR_TRAFFIC_LIGHT_ZONE_W: Pixels = px(80.);
pub(super) const TITLEBAR_LEFT_PADDING: Pixels = px(64.);

// -- Mini window layout --
pub(super) const MINI_WINDOW_DEFAULT_W: f32 = 720.0;
pub(super) const MINI_WINDOW_DEFAULT_H: f32 = 440.0;
pub(super) const MINI_TITLEBAR_H: Pixels = px(44.);
pub(super) const MINI_CONTENT_MAX_W: Pixels = px(760.);
pub(super) const MINI_HISTORY_OVERLAY_TOP: Pixels = px(48.);
pub(super) const MINI_HISTORY_OVERLAY_W: Pixels = px(320.);
pub(super) const MINI_HISTORY_OVERLAY_MAX_H: Pixels = px(420.);
pub(super) const MINI_BTN_SIZE: Pixels = px(28.);

// -- Overlay layout --
pub(super) const ATTACHMENTS_PICKER_BOTTOM_INSET: Pixels = px(80.);

// -- Animation constants --
pub(super) const ANIM_CYCLE_MS: u64 = 1200;

// -- Streaming cursor opacity range: base + amplitude * sin(t) --
pub(super) const CURSOR_OPACITY_BASE: f32 = 0.7;
pub(super) const CURSOR_OPACITY_AMP: f32 = 0.3;

// -- Dot separator --
pub(super) const DOT_SIZE: Pixels = px(3.);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct StreamingSessionKey {
    pub(super) chat_id: ChatId,
    pub(super) generation: u64,
}

pub(super) fn should_retry_existing_user_turn(messages: &[Message]) -> bool {
    messages
        .last()
        .map(|message| message.role == MessageRole::User)
        .unwrap_or(false)
}

/// Derive display text for chat title/preview from authored content.
///
/// When the user's authored content (after stripping directives) is empty,
/// falls back to descriptive placeholders based on what attachments exist.
/// This prevents raw directive lines (`@selection`, `@context`) from leaking
/// into titles or sidebar previews.
pub(super) fn ai_window_outbound_display_source(
    authored_content: &str,
    has_pending_image: bool,
    has_context_parts: bool,
) -> String {
    if authored_content.trim().is_empty() && has_pending_image {
        "Image attachment".to_string()
    } else if authored_content.trim().is_empty() && has_context_parts {
        "Context attachment".to_string()
    } else {
        authored_content.to_string()
    }
}

pub(super) fn ai_window_can_submit_message(
    content: &str,
    has_pending_image: bool,
    has_pending_context_parts: bool,
) -> bool {
    !content.trim().is_empty() || has_pending_image || has_pending_context_parts
}

pub(super) fn ai_window_prune_deleted_message_ui_state(
    collapsed_messages: &mut std::collections::HashSet<String>,
    expanded_messages: &mut std::collections::HashSet<String>,
    deleted_message_ids: &[String],
) {
    for message_id in deleted_message_ids {
        collapsed_messages.remove(message_id);
        expanded_messages.remove(message_id);
    }
}

pub(super) fn ai_window_queue_command_if_open(
    pending_commands: &mut Vec<AiCommand>,
    window_is_open: bool,
    command: AiCommand,
) -> bool {
    if !window_is_open {
        return false;
    }

    pending_commands.push(command);
    true
}

pub(super) fn should_persist_stale_completion(
    suppressed_sessions: &mut std::collections::HashSet<StreamingSessionKey>,
    session_key: StreamingSessionKey,
) -> bool {
    !suppressed_sessions.remove(&session_key)
}

/// A preset configuration for starting new chats.
///
/// Uses owned `String` fields so presets can be loaded from disk (user-created)
/// or constructed from static defaults.
#[derive(Clone, Debug)]
pub(super) struct AiPreset {
    /// Unique identifier
    pub(super) id: String,
    /// Display name
    pub(super) name: String,
    /// Description shown in dropdown
    pub(super) description: String,
    /// System prompt to use
    pub(super) system_prompt: String,
    /// Icon name
    pub(super) icon: LocalIconName,
    /// Preferred model ID (if any)
    pub(super) preferred_model: Option<String>,
}

/// A recently used model+provider configuration (for "Last Used Settings" in dropdown)
#[derive(Clone, Debug, PartialEq)]
pub(super) struct LastUsedSetting {
    /// Model ID (e.g., "claude-3-5-sonnet-20241022")
    pub(super) model_id: String,
    /// Provider name (e.g., "anthropic")
    pub(super) provider: String,
    /// Display name for the model
    pub(super) display_name: String,
    /// Provider display name
    pub(super) provider_display_name: String,
}

/// Internal enum for handling new chat dropdown selection
pub(super) enum NewChatAction {
    Model { model_id: String, provider: String },
    Preset { index: usize },
}

impl AiPreset {
    /// Get default presets
    pub(super) fn default_presets() -> Vec<AiPreset> {
        vec![
            AiPreset {
                id: "general".to_string(),
                name: "General Assistant".to_string(),
                description: "Helpful AI assistant for any task".to_string(),
                system_prompt: "You are a helpful AI assistant.".to_string(),
                icon: LocalIconName::Star,
                preferred_model: None,
            },
            AiPreset {
                id: "coder".to_string(),
                name: "Code Assistant".to_string(),
                description: "Expert programmer and debugger".to_string(),
                system_prompt: "You are an expert programmer. Write clean, efficient, well-documented code. Explain your reasoning.".to_string(),
                icon: LocalIconName::Code,
                preferred_model: None,
            },
            AiPreset {
                id: "writer".to_string(),
                name: "Writing Assistant".to_string(),
                description: "Help with writing and editing".to_string(),
                system_prompt: "You are a skilled writer and editor. Help improve writing clarity, grammar, and style.".to_string(),
                icon: LocalIconName::FileCode,
                preferred_model: None,
            },
            AiPreset {
                id: "researcher".to_string(),
                name: "Research Assistant".to_string(),
                description: "Deep analysis and research".to_string(),
                system_prompt: "You are a thorough researcher. Analyze topics deeply, cite sources when possible, and provide comprehensive answers.".to_string(),
                icon: LocalIconName::MagnifyingGlass,
                preferred_model: None,
            },
            AiPreset {
                id: "creative".to_string(),
                name: "Creative Partner".to_string(),
                description: "Brainstorming and creative ideas".to_string(),
                system_prompt: "You are a creative partner. Help brainstorm ideas, think outside the box, and explore possibilities.".to_string(),
                icon: LocalIconName::BoltFilled,
                preferred_model: None,
            },
        ]
    }

    /// Load all presets: defaults merged with user-saved presets from disk.
    ///
    /// User presets with the same ID as a default preset will replace the default.
    pub(super) fn load_all_presets() -> Vec<AiPreset> {
        let mut presets = Self::default_presets();

        match crate::ai::presets::load_presets() {
            Ok(saved) => {
                for saved_preset in saved {
                    let ai_preset = Self::from_saved(saved_preset);
                    if let Some(pos) = presets.iter().position(|p| p.id == ai_preset.id) {
                        presets[pos] = ai_preset;
                    } else {
                        presets.push(ai_preset);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load saved presets, using defaults only");
            }
        }

        presets
    }

    /// Convert a saved preset from disk into an in-memory AiPreset.
    fn from_saved(saved: crate::ai::presets::SavedAiPreset) -> AiPreset {
        let icon = match saved.icon.as_str() {
            "code" => LocalIconName::Code,
            "file-code" | "filecode" => LocalIconName::FileCode,
            "magnifying-glass" | "search" => LocalIconName::MagnifyingGlass,
            "bolt" | "bolt-filled" => LocalIconName::BoltFilled,
            "terminal" => LocalIconName::Terminal,
            "pencil" => LocalIconName::Pencil,
            "settings" => LocalIconName::Settings,
            _ => LocalIconName::Star,
        };

        AiPreset {
            id: saved.id,
            name: saved.name,
            description: saved.description,
            system_prompt: saved.system_prompt,
            icon,
            preferred_model: saved.preferred_model,
        }
    }
}

/// Date group categories for sidebar organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DateGroup {
    Today,
    Yesterday,
    ThisWeek,
    Older,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarRow {
    Header { group: DateGroup, is_first: bool },
    Chat { chat_id: ChatId },
}

impl DateGroup {
    /// Get the display label for this group
    pub(super) fn label(&self) -> &'static str {
        match self {
            DateGroup::Today => "Today",
            DateGroup::Yesterday => "Yesterday",
            DateGroup::ThisWeek => "This Week",
            DateGroup::Older => "Older",
        }
    }
}

/// Determine which date group a date belongs to
pub(super) fn get_date_group(date: NaiveDate, today: NaiveDate) -> DateGroup {
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
pub(super) fn group_chats_by_date(chats: &[Chat]) -> Vec<(DateGroup, Vec<&Chat>)> {
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

pub(super) fn build_sidebar_rows_for_chats(chats: &[Chat]) -> Vec<SidebarRow> {
    let date_groups = group_chats_by_date(chats);
    let mut rows = Vec::with_capacity(chats.len() + date_groups.len());

    for (group_index, (group, group_chats)) in date_groups.into_iter().enumerate() {
        rows.push(SidebarRow::Header {
            group,
            is_first: group_index == 0,
        });
        rows.extend(
            group_chats
                .into_iter()
                .map(|chat| SidebarRow::Chat { chat_id: chat.id }),
        );
    }

    rows
}

/// Generate a contextual mock AI response based on the user's message
/// Used for demo/testing when no AI providers are configured
pub(super) fn generate_mock_response(user_message: &str) -> String {
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
pub(super) static AI_WINDOW: std::sync::OnceLock<
    std::sync::Mutex<Option<gpui::WindowHandle<Root>>>,
> = std::sync::OnceLock::new();

/// Global flag to request input focus in the AI window.
/// This replaces the problematic AI_APP_ENTITY which caused memory leaks.
/// The flag is checked in AiApp::render() and cleared after use.
pub(super) static AI_FOCUS_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Global mirror of the current AiWindowMode, updated by set_window_mode()
/// and read by close_ai_window() to determine the correct WindowRole for
/// bounds persistence. This avoids relying on window title string comparison.
/// 0 = Full, 1 = Mini.
pub(super) static AI_CURRENT_WINDOW_MODE: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(0);

impl AiWindowMode {
    pub(super) fn to_u8(self) -> u8 {
        match self {
            Self::Full => 0,
            Self::Mini => 1,
        }
    }
    pub(super) fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Mini,
            _ => Self::Full,
        }
    }
}

/// Pending commands for the AI window (for testing via stdin).
/// These are processed in AiApp::render() to avoid needing a global entity reference.
pub(super) static AI_PENDING_COMMANDS: std::sync::OnceLock<std::sync::Mutex<Vec<AiCommand>>> =
    std::sync::OnceLock::new();

/// Commands that can be sent to the AI window (for testing)
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartChatResolvedMetadata {
    pub(crate) model_id: String,
    pub(crate) provider: String,
}

#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub(super) enum AiCommand {
    SetWindowMode(AiWindowMode),
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
    /// Add a file attachment to the pending attachment list
    AddAttachment {
        path: String,
    },
    /// Initialize the chat with pending messages from open_ai_window_with_chat
    InitializeWithPendingChat,
    /// Show the command bar overlay (Cmd+K menu)
    ShowCommandBar,
    /// Apply a preset by ID (opens a new chat with the preset's system prompt and model)
    ApplyPreset {
        preset_id: String,
    },
    /// Reload presets from disk (after create/import)
    ReloadPresets,
    /// Simulate a key press (for testing)
    SimulateKey {
        key: String,
        modifiers: Vec<KeyModifier>,
    },
    /// Start a new chat with a user message (from SDK aiStartChat).
    /// The ChatId is pre-generated by the caller so it can be returned immediately.
    StartChat {
        chat_id: ChatId,
        message: String,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        image: Option<String>,
        system_prompt: Option<String>,
        model_id: Option<String>,
        provider: Option<String>,
        on_created: Option<std::sync::Arc<dyn Fn(String, String) + Send + Sync + 'static>>,
        /// If true, trigger AI streaming response after creating the user message.
        submit: bool,
    },
}

impl AiCommand {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::SetWindowMode(AiWindowMode::Full) => "set_window_mode_full",
            Self::SetWindowMode(AiWindowMode::Mini) => "set_window_mode_mini",
            Self::SetSearch(_) => "set_search",
            Self::SetInput { submit: true, .. } => "set_input_submit",
            Self::SetInput { submit: false, .. } => "set_input",
            Self::SetInputWithImage { submit: true, .. } => "set_input_with_image_submit",
            Self::SetInputWithImage { submit: false, .. } => "set_input_with_image",
            Self::AddAttachment { .. } => "add_attachment",
            Self::InitializeWithPendingChat => "initialize_with_pending_chat",
            Self::ShowCommandBar => "show_command_bar",
            Self::ApplyPreset { .. } => "apply_preset",
            Self::ReloadPresets => "reload_presets",
            Self::SimulateKey { .. } => "simulate_key",
            Self::StartChat { submit: true, .. } => "start_chat_submit",
            Self::StartChat { submit: false, .. } => "start_chat",
        }
    }
}

pub(super) fn get_pending_commands() -> &'static std::sync::Mutex<Vec<AiCommand>> {
    AI_PENDING_COMMANDS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

pub(super) fn push_ai_command(cmd: AiCommand) {
    if let Ok(mut cmds) = get_pending_commands().lock() {
        cmds.push(cmd);
    }
}

pub(super) fn take_ai_commands() -> Vec<AiCommand> {
    get_pending_commands()
        .lock()
        .ok()
        .map(|mut cmds| std::mem::take(&mut *cmds))
        .unwrap_or_default()
}

// === SDK State Bridge ===
// These globals allow SDK handlers (which run outside the UI thread) to read
// the current AI window state without needing an Entity<AiApp> reference.
// AiApp updates these at every state transition; SDK handlers read them.

/// Snapshot of the AI window's streaming state, published by AiApp for SDK handler consumption.
#[derive(Debug, Clone, Default)]
pub(crate) struct AiStreamingSnapshot {
    pub(crate) is_streaming: bool,
    pub(crate) chat_id: Option<String>,
    pub(crate) partial_content: Option<String>,
}

/// Currently active (selected) chat ID in the AI window.
static AI_ACTIVE_CHAT_ID: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();

/// Current streaming state snapshot.
static AI_STREAMING_SNAPSHOT: std::sync::OnceLock<std::sync::Mutex<AiStreamingSnapshot>> =
    std::sync::OnceLock::new();

fn active_chat_id_slot() -> &'static std::sync::Mutex<Option<String>> {
    AI_ACTIVE_CHAT_ID.get_or_init(|| std::sync::Mutex::new(None))
}

fn streaming_snapshot_slot() -> &'static std::sync::Mutex<AiStreamingSnapshot> {
    AI_STREAMING_SNAPSHOT.get_or_init(|| std::sync::Mutex::new(AiStreamingSnapshot::default()))
}

/// Update the active chat ID (called by AiApp on chat selection).
pub(super) fn publish_active_chat_id(chat_id: Option<&ChatId>) {
    if let Ok(mut slot) = active_chat_id_slot().lock() {
        *slot = chat_id.map(|id| id.as_str());
    }
}

/// Update the streaming state snapshot (called by AiApp on streaming transitions).
pub(super) fn publish_streaming_state(snapshot: AiStreamingSnapshot) {
    if let Ok(mut slot) = streaming_snapshot_slot().lock() {
        *slot = snapshot;
    }
}

/// Read the current active chat ID (called by SDK handlers).
pub(crate) fn get_active_chat_id() -> Option<String> {
    active_chat_id_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.clone())
}

/// Read the current streaming state (called by SDK handlers).
pub(crate) fn get_streaming_snapshot() -> AiStreamingSnapshot {
    streaming_snapshot_slot()
        .lock()
        .ok()
        .map(|slot| slot.clone())
        .unwrap_or_default()
}

/// Clear all SDK-visible state (called when AI window closes).
pub(crate) fn clear_sdk_state() {
    if let Ok(mut slot) = active_chat_id_slot().lock() {
        *slot = None;
    }
    if let Ok(mut slot) = streaming_snapshot_slot().lock() {
        *slot = AiStreamingSnapshot::default();
    }
}

// === Keyboard Shortcut Registry ===
// Single source of truth for all AI window keyboard shortcuts.
// Used by both the Cmd+/ overlay and visible hint chips in the sidebar.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AiShortcutItem {
    pub(super) keys: &'static str,
    pub(super) description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AiShortcutSection {
    pub(super) title: &'static str,
    pub(super) items: &'static [AiShortcutItem],
}

pub(super) const AI_SHORTCUTS_NAVIGATION: &[AiShortcutItem] = &[
    AiShortcutItem {
        keys: "\u{2318}J",
        description: "Recent chats (mini)",
    },
    AiShortcutItem {
        keys: "\u{2318}B",
        description: "Toggle sidebar",
    },
    AiShortcutItem {
        keys: "\u{2318}\u{21e7}F",
        description: "Focus search",
    },
    AiShortcutItem {
        keys: "\u{2318}[",
        description: "Previous chat",
    },
    AiShortcutItem {
        keys: "\u{2318}]",
        description: "Next chat",
    },
];

pub(super) const AI_SHORTCUTS_CHAT: &[AiShortcutItem] = &[
    AiShortcutItem {
        keys: "\u{2318}N",
        description: "New chat",
    },
    AiShortcutItem {
        keys: "\u{2318}\u{21e7}N",
        description: "New chat with preset",
    },
    AiShortcutItem {
        keys: "\u{2318}\u{21e7}\u{232B}",
        description: "Delete chat",
    },
];

pub(super) const AI_SHORTCUTS_INPUT: &[AiShortcutItem] = &[
    AiShortcutItem {
        keys: "Enter",
        description: "Send message",
    },
    AiShortcutItem {
        keys: "Shift+Enter",
        description: "Insert newline",
    },
    AiShortcutItem {
        keys: "\u{2318}L",
        description: "Focus input",
    },
    AiShortcutItem {
        keys: "\u{2191}",
        description: "Edit last message (empty input)",
    },
    AiShortcutItem {
        keys: "Esc",
        description: "Stop streaming / close",
    },
];

pub(super) const AI_SHORTCUTS_ACTIONS: &[AiShortcutItem] = &[
    AiShortcutItem {
        keys: "\u{2318}K",
        description: "Open actions",
    },
    AiShortcutItem {
        keys: "\u{21e7}\u{2318}M",
        description: "Toggle mini / full mode",
    },
    AiShortcutItem {
        keys: "\u{2318}\u{21e7}C",
        description: "Copy last response",
    },
    AiShortcutItem {
        keys: "\u{2318}\u{21e7}E",
        description: "Export chat as markdown",
    },
    AiShortcutItem {
        keys: "\u{2318}/",
        description: "Toggle shortcuts overlay",
    },
];

pub(super) const AI_SHORTCUT_SECTIONS: &[AiShortcutSection] = &[
    AiShortcutSection {
        title: "Navigation",
        items: AI_SHORTCUTS_NAVIGATION,
    },
    AiShortcutSection {
        title: "Chat",
        items: AI_SHORTCUTS_CHAT,
    },
    AiShortcutSection {
        title: "Input",
        items: AI_SHORTCUTS_INPUT,
    },
    AiShortcutSection {
        title: "Actions",
        items: AI_SHORTCUTS_ACTIONS,
    },
];

// NOTE: AI_APP_ENTITY was removed to prevent memory leaks.
// The entity was being kept alive by this global reference and by theme watcher tasks,
// causing the AiApp to never be dropped even after the window closed.
// Instead, we use AI_FOCUS_REQUESTED (AtomicBool) which AiApp checks in render().

#[cfg(test)]
mod outbound_display_source_tests {
    use super::*;

    #[test]
    fn outbound_display_source_uses_context_attachment_for_empty_authored_content() {
        assert_eq!(
            ai_window_outbound_display_source("", false, true),
            "Context attachment"
        );
    }

    #[test]
    fn outbound_display_source_uses_authored_content_when_present() {
        assert_eq!(
            ai_window_outbound_display_source("Summarize this.", false, true),
            "Summarize this."
        );
    }

    #[test]
    fn outbound_display_source_prefers_image_over_context_when_both_empty() {
        assert_eq!(
            ai_window_outbound_display_source("", true, true),
            "Image attachment"
        );
    }

    #[test]
    fn outbound_display_source_returns_empty_string_for_no_attachments() {
        assert_eq!(ai_window_outbound_display_source("", false, false), "");
    }

    #[test]
    fn outbound_display_source_trims_whitespace_only_authored_content() {
        assert_eq!(
            ai_window_outbound_display_source("   ", false, true),
            "Context attachment"
        );
    }
}
