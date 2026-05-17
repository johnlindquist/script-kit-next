/// Delay between hiding the main window and starting a synchronous screenshot capture.
const AI_CAPTURE_HIDE_SETTLE_MS: u64 = 150;

/// Synthetic prompt ID used when the microphone-selection MiniPrompt is open.
/// Checked in `submit_arg_prompt_from_current_state` to intercept the submit
/// and persist the chosen device instead of sending a protocol message.
const BUILTIN_MIC_SELECT_PROMPT_ID: &str = "builtin:select-microphone";

/// Choice value representing "use system default" in the mic-selection prompt.
const BUILTIN_MIC_DEFAULT_VALUE: &str = "__system_default__";

/// Synthetic prompt ID for the dictation model download consent prompt.
/// Checked in `submit_arg_prompt_from_current_state` to intercept submit
/// and either start the Parakeet download or cancel.
const BUILTIN_DICTATION_MODEL_PROMPT_ID: &str = "builtin:dictation-model";

/// Choice value: user wants to download the Parakeet model.
const BUILTIN_DICTATION_MODEL_DOWNLOAD: &str = "download";

/// Choice value: user declines the download for now.
const BUILTIN_DICTATION_MODEL_CANCEL: &str = "cancel";

/// Choice value: user wants to hide the prompt while download continues.
const BUILTIN_DICTATION_MODEL_HIDE: &str = "builtin/dictation-model-hide";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DictationBuiltinAction {
    CurrentSurface,
    AgentChat,
    FrontmostApp,
    Notes,
}

impl DictationBuiltinAction {
    fn opening_message(self) -> &'static str {
        match self {
            Self::CurrentSurface => "Opening Dictation",
            Self::AgentChat => "Opening Dictation to Agent Chat",
            Self::FrontmostApp => "Opening Dictation to Frontmost App",
            Self::Notes => "Opening Dictation to Notes",
        }
    }

    fn failure_message(self) -> &'static str {
        match self {
            Self::CurrentSurface => "Failed to toggle dictation",
            Self::AgentChat => "Failed to toggle dictation to Agent Chat",
            Self::FrontmostApp => "Failed to toggle dictation to frontmost app",
            Self::Notes => "Failed to toggle dictation to notes",
        }
    }

    fn preflight_failure_message(self) -> &'static str {
        match self {
            Self::CurrentSurface => "Dictation start preflight failed",
            Self::AgentChat => "Dictation-to-Agent-Chat start preflight failed",
            Self::FrontmostApp => "Dictation-to-app start preflight failed",
            Self::Notes => "Dictation-to-notes start preflight failed",
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::CurrentSurface => "dictation_toggle",
            Self::AgentChat => "dictation_to_ai_toggle",
            Self::FrontmostApp => "dictation_to_frontmost_app_toggle",
            Self::Notes => "dictation_to_notes_toggle",
        }
    }

    fn forced_target(self) -> Option<crate::dictation::DictationTarget> {
        match self {
            Self::CurrentSurface => None,
            Self::AgentChat => Some(crate::dictation::DictationTarget::TabAiHarness),
            Self::FrontmostApp => Some(crate::dictation::DictationTarget::ExternalApp),
            Self::Notes => Some(crate::dictation::DictationTarget::NotesEditor),
        }
    }

    fn stop_fallback_target(self) -> crate::dictation::DictationTarget {
        self.forced_target()
            .unwrap_or(crate::dictation::DictationTarget::ExternalApp)
    }

    fn conceal_before_overlay(self) -> bool {
        matches!(self, Self::AgentChat | Self::FrontmostApp)
    }

    fn dispatch_start_before_overlay(self) -> bool {
        matches!(self, Self::CurrentSurface)
    }

    fn log_forced_route(self) -> bool {
        matches!(self, Self::FrontmostApp | Self::Notes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DictationStartPreflight {
    Ready,
    OpenedSetup,
    DownloadInProgress,
    OpenedModelPrompt,
    Failed,
}

impl DictationStartPreflight {
    fn success_detail(self) -> &'static str {
        match self {
            Self::Ready => "dictation_ready",
            Self::OpenedSetup => "dictation_setup_opened",
            Self::DownloadInProgress => "dictation_model_download_in_progress",
            Self::OpenedModelPrompt => "dictation_model_prompt_opened",
            Self::Failed => "dictation_preflight_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PasteSequentialBuiltinAction {
    PasteEntry(String),
    SequenceExhausted,
    HistoryEmpty,
}

impl PasteSequentialBuiltinAction {
    fn from_outcome(outcome: clipboard_history::PasteSequentialOutcome) -> Self {
        match outcome {
            clipboard_history::PasteSequentialOutcome::Pasted(entry_id) => {
                Self::PasteEntry(entry_id)
            }
            clipboard_history::PasteSequentialOutcome::Exhausted => Self::SequenceExhausted,
            clipboard_history::PasteSequentialOutcome::Empty => Self::HistoryEmpty,
        }
    }

    fn telemetry_event(&self) -> &'static str {
        match self {
            Self::PasteEntry(_) => "paste_entry",
            Self::SequenceExhausted => "sequence_exhausted",
            Self::HistoryEmpty => "history_empty",
        }
    }

    fn log_message(&self) -> &'static str {
        match self {
            Self::PasteEntry(_) => "Enqueuing sequential paste via serialized worker",
            Self::SequenceExhausted => "Sequential paste exhausted all entries",
            Self::HistoryEmpty => "No clipboard history available for sequential paste",
        }
    }

    fn success_detail(&self) -> &'static str {
        match self {
            Self::PasteEntry(_) => "paste_sequential",
            Self::SequenceExhausted => "paste_sequential_exhausted",
            Self::HistoryEmpty => "paste_sequential_empty",
        }
    }

    fn hud_message(&self) -> Option<&'static str> {
        match self {
            Self::PasteEntry(_) => None,
            Self::SequenceExhausted => Some("Sequence complete"),
            Self::HistoryEmpty => Some("No clipboard history"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSnapModeBuiltinAction {
    Disable,
    Simple,
    Expanded,
    Precision,
}

impl SettingsSnapModeBuiltinAction {
    fn from_command(command: builtins::SettingsCommandType) -> Option<Self> {
        match command {
            builtins::SettingsCommandType::DisableWindowSnapping => Some(Self::Disable),
            builtins::SettingsCommandType::SnapModeSimple => Some(Self::Simple),
            builtins::SettingsCommandType::SnapModeExpanded => Some(Self::Expanded),
            builtins::SettingsCommandType::SnapModePrecision => Some(Self::Precision),
            builtins::SettingsCommandType::ResetWindowPositions
            | builtins::SettingsCommandType::ChooseTheme
            | builtins::SettingsCommandType::SelectMicrophone
            | builtins::SettingsCommandType::DictationSetup => None,
        }
    }

    fn target_mode(self) -> window_control::SnapMode {
        match self {
            Self::Disable => window_control::SnapMode::Off,
            Self::Simple => window_control::SnapMode::Simple,
            Self::Expanded => window_control::SnapMode::Expanded,
            Self::Precision => window_control::SnapMode::Precision,
        }
    }

    fn hud_text(self) -> &'static str {
        match self {
            Self::Disable => "Window snapping disabled",
            Self::Simple => "Snap mode: Simple",
            Self::Expanded => "Snap mode: Expanded",
            Self::Precision => "Snap mode: Precision",
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Disable => "set_snap_mode::off",
            Self::Simple => "set_snap_mode::simple",
            Self::Expanded => "set_snap_mode::expanded",
            Self::Precision => "set_snap_mode::precision",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermissionAssistantBuiltinAction {
    Accessibility,
    ScreenRecording,
}

impl PermissionAssistantBuiltinAction {
    fn from_command(command: builtins::PermissionCommandType) -> Option<Self> {
        match command {
            builtins::PermissionCommandType::AllowAccessibility => Some(Self::Accessibility),
            builtins::PermissionCommandType::AllowScreenRecording => Some(Self::ScreenRecording),
            builtins::PermissionCommandType::CheckPermissions
            | builtins::PermissionCommandType::RequestAccessibility
            | builtins::PermissionCommandType::OpenAccessibilitySettings => None,
        }
    }

    fn panel(self) -> platform::permiso::PermisoPanel {
        match self {
            Self::Accessibility => platform::permiso::PermisoPanel::Accessibility,
            Self::ScreenRecording => platform::permiso::PermisoPanel::ScreenRecording,
        }
    }

    fn success_hud(self) -> &'static str {
        match self {
            Self::Accessibility => "Drag Script Kit into Accessibility",
            Self::ScreenRecording => "Drag Script Kit into Screen Recording",
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Accessibility => "allow_accessibility",
            Self::ScreenRecording => "allow_screen_recording",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::Accessibility => "allow_accessibility_failed",
            Self::ScreenRecording => "allow_screen_recording_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermissionCommandBuiltinAction {
    CheckPermissions,
    RequestAccessibility,
    OpenAccessibilitySettings,
    Assistant(PermissionAssistantBuiltinAction),
}

impl PermissionCommandBuiltinAction {
    fn from_command(command: builtins::PermissionCommandType) -> Self {
        match command {
            builtins::PermissionCommandType::CheckPermissions => Self::CheckPermissions,
            builtins::PermissionCommandType::RequestAccessibility => Self::RequestAccessibility,
            builtins::PermissionCommandType::OpenAccessibilitySettings => {
                Self::OpenAccessibilitySettings
            }
            builtins::PermissionCommandType::AllowAccessibility
            | builtins::PermissionCommandType::AllowScreenRecording => Self::Assistant(
                PermissionAssistantBuiltinAction::from_command(command)
                    .expect("permission assistant command should map to assistant action"),
            ),
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::CheckPermissions => "check_permissions",
            Self::RequestAccessibility => "request_accessibility",
            Self::OpenAccessibilitySettings => "open_accessibility_settings",
            Self::Assistant(action) => action.success_detail(),
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::OpenAccessibilitySettings => "open_accessibility_settings_failed",
            Self::Assistant(action) => action.failure_detail(),
            Self::CheckPermissions | Self::RequestAccessibility => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityOpenBuiltinAction {
    MiniMainWindow,
    ScratchPad,
    QuickTerminal,
    ClaudeCode,
    ProcessManager,
}

impl UtilityOpenBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::MiniMainWindow => Some(Self::MiniMainWindow),
            builtins::UtilityCommandType::ScratchPad => Some(Self::ScratchPad),
            builtins::UtilityCommandType::QuickTerminal => Some(Self::QuickTerminal),
            builtins::UtilityCommandType::ClaudeCode => Some(Self::ClaudeCode),
            builtins::UtilityCommandType::ProcessManager => Some(Self::ProcessManager),
            builtins::UtilityCommandType::StopAllProcesses
            | builtins::UtilityCommandType::DoInCurrentApp
            | builtins::UtilityCommandType::TurnThisIntoCommand
            | builtins::UtilityCommandType::CurrentAppCommands
            | builtins::UtilityCommandType::InspectCurrentContext
            | builtins::UtilityCommandType::TraceCurrentAppIntent
            | builtins::UtilityCommandType::VerifyCurrentAppRecipe
            | builtins::UtilityCommandType::ReplayCurrentAppRecipe => None,
        }
    }

    fn opening_message(self) -> Option<&'static str> {
        match self {
            Self::MiniMainWindow => Some("Opening Mini Main Window"),
            Self::ScratchPad | Self::QuickTerminal | Self::ClaudeCode | Self::ProcessManager => {
                None
            }
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::MiniMainWindow => "open_mini_main_window",
            Self::ScratchPad => "open_scratch_pad",
            Self::QuickTerminal => "open_quick_terminal",
            Self::ClaudeCode => "open_claude_code_terminal",
            Self::ProcessManager => "open_process_manager",
        }
    }

    fn opens_from_main_menu(self) -> bool {
        matches!(
            self,
            Self::ScratchPad | Self::QuickTerminal | Self::ClaudeCode
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityProcessBuiltinAction {
    StopAllProcesses,
}

impl UtilityProcessBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::StopAllProcesses => Some(Self::StopAllProcesses),
            _ => None,
        }
    }

    fn empty_hud(self) -> &'static str {
        match self {
            Self::StopAllProcesses => "No running scripts to stop.",
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::StopAllProcesses => "stop_all_processes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityContextBuiltinAction {
    InspectCurrentContext,
}

impl UtilityContextBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::InspectCurrentContext => {
                Some(Self::InspectCurrentContext)
            }
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::InspectCurrentContext => "inspect_current_context",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::InspectCurrentContext => "inspect_current_context_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityTraceBuiltinAction {
    CurrentAppIntent,
}

impl UtilityTraceBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::TraceCurrentAppIntent => Some(Self::CurrentAppIntent),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::CurrentAppIntent => "trace_current_app_intent",
        }
    }

    fn serialize_failure_detail(self) -> &'static str {
        match self {
            Self::CurrentAppIntent => "trace_current_app_intent_serialize_failed",
        }
    }

    fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::CurrentAppIntent => "trace_current_app_intent_capture_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityRecipeBuiltinAction {
    VerifyCurrentApp,
    ReplayCurrentApp,
    TurnThisIntoCommand,
}

impl UtilityRecipeBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::VerifyCurrentAppRecipe => Some(Self::VerifyCurrentApp),
            builtins::UtilityCommandType::ReplayCurrentAppRecipe => Some(Self::ReplayCurrentApp),
            builtins::UtilityCommandType::TurnThisIntoCommand => Some(Self::TurnThisIntoCommand),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe",
            Self::ReplayCurrentApp => "replay_current_app_recipe",
            Self::TurnThisIntoCommand => "turn_this_into_command",
        }
    }

    fn clipboard_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_clipboard_failed",
            Self::ReplayCurrentApp => "replay_current_app_recipe_clipboard_failed",
            Self::TurnThisIntoCommand => "turn_this_into_command_clipboard_failed",
        }
    }

    fn serialize_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_serialize_failed",
            Self::ReplayCurrentApp => "replay_current_app_recipe_serialize_failed",
            Self::TurnThisIntoCommand => "turn_this_into_command_serialize_failed",
        }
    }

    fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_capture_failed",
            Self::ReplayCurrentApp => "replay_current_app_recipe_capture_failed",
            Self::TurnThisIntoCommand => "turn_this_into_command_capture_failed",
        }
    }

    fn missing_query_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_missing_query",
            Self::ReplayCurrentApp => "replay_current_app_recipe_missing_query",
            Self::TurnThisIntoCommand => "turn_this_into_command_missing_query",
        }
    }

    fn drift_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_drift",
            Self::ReplayCurrentApp => "replay_current_app_recipe_drift",
            Self::TurnThisIntoCommand => "turn_this_into_command_drift",
        }
    }

    fn missing_entry_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_missing_entry_index",
            Self::ReplayCurrentApp => "replay_current_app_recipe_missing_entry_index",
            Self::TurnThisIntoCommand => "turn_this_into_command_missing_entry_index",
        }
    }

    fn open_palette_success_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_open_palette",
            Self::ReplayCurrentApp => "replay_current_app_recipe_open_palette",
            Self::TurnThisIntoCommand => "turn_this_into_command_open_palette",
        }
    }

    fn generate_script_success_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_generate_script",
            Self::ReplayCurrentApp => "replay_current_app_recipe_generate_script",
            Self::TurnThisIntoCommand => "turn_this_into_command_generate_script",
        }
    }

    fn unknown_action_failure_detail(self) -> &'static str {
        match self {
            Self::VerifyCurrentApp => "verify_current_app_recipe_unknown_action",
            Self::ReplayCurrentApp => "replay_current_app_recipe_unknown_action",
            Self::TurnThisIntoCommand => "turn_this_into_command_unknown_action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityDoInCurrentAppBuiltinAction {
    Submit,
}

impl UtilityDoInCurrentAppBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::DoInCurrentApp => Some(Self::Submit),
            _ => None,
        }
    }

    fn open_palette_success_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_open_palette",
        }
    }

    fn generate_script_success_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_generate_script_scheduled",
        }
    }

    fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_capture_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityCurrentAppCommandsBuiltinAction {
    Open,
}

impl UtilityCurrentAppCommandsBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Option<Self> {
        match command {
            builtins::UtilityCommandType::CurrentAppCommands => Some(Self::Open),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_current_app_commands",
        }
    }

    fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::Open => "current_app_commands_capture_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UtilityCommandBuiltinAction {
    Open(UtilityOpenBuiltinAction),
    Process(UtilityProcessBuiltinAction),
    Context(UtilityContextBuiltinAction),
    Trace(UtilityTraceBuiltinAction),
    Recipe(UtilityRecipeBuiltinAction),
    DoInCurrentApp(UtilityDoInCurrentAppBuiltinAction),
    CurrentAppCommands(UtilityCurrentAppCommandsBuiltinAction),
}

impl UtilityCommandBuiltinAction {
    fn from_command(command: builtins::UtilityCommandType) -> Self {
        match command {
            builtins::UtilityCommandType::MiniMainWindow
            | builtins::UtilityCommandType::ScratchPad
            | builtins::UtilityCommandType::QuickTerminal
            | builtins::UtilityCommandType::ClaudeCode
            | builtins::UtilityCommandType::ProcessManager => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .expect("utility open command should map to open action"),
            ),
            builtins::UtilityCommandType::StopAllProcesses => Self::Process(
                UtilityProcessBuiltinAction::from_command(command)
                    .expect("utility process command should map to process action"),
            ),
            builtins::UtilityCommandType::InspectCurrentContext => Self::Context(
                UtilityContextBuiltinAction::from_command(command)
                    .expect("utility context command should map to context action"),
            ),
            builtins::UtilityCommandType::TraceCurrentAppIntent => Self::Trace(
                UtilityTraceBuiltinAction::from_command(command)
                    .expect("utility trace command should map to trace action"),
            ),
            builtins::UtilityCommandType::VerifyCurrentAppRecipe
            | builtins::UtilityCommandType::ReplayCurrentAppRecipe
            | builtins::UtilityCommandType::TurnThisIntoCommand => Self::Recipe(
                UtilityRecipeBuiltinAction::from_command(command)
                    .expect("utility recipe command should map to recipe action"),
            ),
            builtins::UtilityCommandType::DoInCurrentApp => Self::DoInCurrentApp(
                UtilityDoInCurrentAppBuiltinAction::from_command(command)
                    .expect("utility do-in command should map to do-in action"),
            ),
            builtins::UtilityCommandType::CurrentAppCommands => Self::CurrentAppCommands(
                UtilityCurrentAppCommandsBuiltinAction::from_command(command)
                    .expect("utility current-app command should map to current-app action"),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuBarBuiltinAction {
    Execute,
}

impl MenuBarBuiltinAction {
    fn from_action(_action: &builtins::MenuBarActionInfo) -> Self {
        Self::Execute
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Execute => "menu_bar_action",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::Execute => "menu_bar_action_failed",
        }
    }

    fn unsupported_detail(self) -> &'static str {
        match self {
            Self::Execute => "menu_bar_action_unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SystemBuiltinAction {
    Dispatch,
}

impl SystemBuiltinAction {
    fn from_action(_action_type: &builtins::SystemActionType) -> Self {
        Self::Dispatch
    }

    fn handler_name(self) -> &'static str {
        match self {
            Self::Dispatch => "Executing system action via inner path",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurfaceOpenBuiltinAction {
    ClipboardHistory,
    Favorites,
    AppLauncher,
    DesignGallery,
    AiChat,
    EmojiPicker,
    Webcam,
    FileSearch,
    Settings,
    AcpHistory,
    AiVault,
    DictationHistory,
    SdkReference,
    ScriptTemplateCatalog,
}

impl SurfaceOpenBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::ClipboardHistory => Some(Self::ClipboardHistory),
            builtins::BuiltInFeature::Favorites => Some(Self::Favorites),
            builtins::BuiltInFeature::AppLauncher => Some(Self::AppLauncher),
            builtins::BuiltInFeature::DesignGallery => Some(Self::DesignGallery),
            builtins::BuiltInFeature::AiChat => Some(Self::AiChat),
            builtins::BuiltInFeature::EmojiPicker => Some(Self::EmojiPicker),
            builtins::BuiltInFeature::Webcam => Some(Self::Webcam),
            builtins::BuiltInFeature::FileSearch => Some(Self::FileSearch),
            builtins::BuiltInFeature::Settings => Some(Self::Settings),
            builtins::BuiltInFeature::AcpHistory => Some(Self::AcpHistory),
            builtins::BuiltInFeature::AiVault => Some(Self::AiVault),
            builtins::BuiltInFeature::DictationHistory => Some(Self::DictationHistory),
            builtins::BuiltInFeature::SdkReference => Some(Self::SdkReference),
            builtins::BuiltInFeature::NewScriptFromTemplate => Some(Self::ScriptTemplateCatalog),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::ClipboardHistory => "open_clipboard_history",
            Self::Favorites => "open_favorites_view",
            Self::AppLauncher => "open_app_launcher",
            Self::DesignGallery => "open_design_gallery",
            Self::AiChat => "open_ai_harness_dispatched",
            Self::EmojiPicker => "open_emoji_picker",
            Self::Webcam => "open_webcam",
            Self::FileSearch => "open_file_search",
            Self::Settings => "open_settings",
            Self::AcpHistory => "open_acp_history",
            Self::AiVault => "open_ai_vault",
            Self::DictationHistory => "open_dictation_history",
            Self::SdkReference => "open_sdk_reference",
            Self::ScriptTemplateCatalog => "open_script_template_catalog",
        }
    }

    fn log_message(self) -> &'static str {
        match self {
            Self::ClipboardHistory => "Opening Clipboard History",
            Self::Favorites => "Opening Favorites browse view",
            Self::AppLauncher => "Opening App Launcher",
            Self::DesignGallery => "Opening Design Gallery",
            Self::AiChat => "Opening Agent Chat",
            Self::EmojiPicker => "Opening Emoji Picker",
            Self::Webcam => "Opening Webcam",
            Self::FileSearch => "Opening File Search",
            Self::Settings => "Opening Settings",
            Self::AcpHistory => "Opening Agent Chat History",
            Self::AiVault => "Opening AI Vault",
            Self::DictationHistory => "Opening Dictation History",
            Self::SdkReference => "Opening SDK Reference",
            Self::ScriptTemplateCatalog => "Opening Script Template Catalog",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserTabsBuiltinAction {
    Open,
}

impl BrowserTabsBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::BrowserTabs => Some(Self::Open),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_browser_tabs",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::Open => "open_browser_tabs_failed",
        }
    }

    fn opening_message(self) -> &'static str {
        match self {
            Self::Open => "Opening Browser Tabs",
        }
    }

    fn loaded_message(self) -> &'static str {
        match self {
            Self::Open => "Loaded browser tabs",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Self::Open => "Search open browser tabs...",
        }
    }

    fn failure_message(self, error: &anyhow::Error) -> String {
        match self {
            Self::Open => format!("Failed to list browser tabs: {error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowSwitcherBuiltinAction {
    Open,
}

impl WindowSwitcherBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::WindowSwitcher => Some(Self::Open),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_window_switcher",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::Open => "open_window_switcher_failed",
        }
    }

    fn opening_message(self) -> &'static str {
        match self {
            Self::Open => "Opening Window Switcher",
        }
    }

    fn loaded_message(self) -> &'static str {
        match self {
            Self::Open => "Loaded windows",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Self::Open => "Search windows...",
        }
    }

    fn failure_message(self, error: &anyhow::Error) -> String {
        match self {
            Self::Open => format!("Failed to list windows: {error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppLaunchBuiltinAction {
    Launch,
}

impl AppLaunchBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::App(_) => Some(Self::Launch),
            _ => None,
        }
    }

    fn success_detail(self, app_name: &str) -> String {
        match self {
            Self::Launch => format!("launch_app::{app_name}"),
        }
    }

    fn not_found_detail(self, app_name: &str) -> String {
        match self {
            Self::Launch => format!("launch_app_not_found::{app_name}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesBuiltinAction {
    Open,
}

impl NotesBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::Notes => Some(Self::Open),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_notes",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::Open => "open_notes_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncToGithubBuiltinAction {
    Dispatch,
}

impl SyncToGithubBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::SyncToGithub => Some(Self::Dispatch),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Dispatch => "sync_to_github_dispatched",
        }
    }
}

#[cfg(feature = "storybook")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesignExplorerBuiltinAction {
    Open,
}

#[cfg(feature = "storybook")]
impl DesignExplorerBuiltinAction {
    fn from_feature(feature: &builtins::BuiltInFeature) -> Option<Self> {
        match feature {
            builtins::BuiltInFeature::DesignExplorer => Some(Self::Open),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_design_explorer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KitStoreBuiltinAction {
    BrowseKits,
    InstalledKits,
    UpdateAllKits,
}

impl KitStoreBuiltinAction {
    fn from_command(command: builtins::KitStoreCommandType) -> Self {
        match command {
            builtins::KitStoreCommandType::BrowseKits => Self::BrowseKits,
            builtins::KitStoreCommandType::InstalledKits => Self::InstalledKits,
            builtins::KitStoreCommandType::UpdateAllKits => Self::UpdateAllKits,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::BrowseKits => "browse_kits_dispatched",
            Self::InstalledKits => "installed_kits",
            Self::UpdateAllKits => "update_all_kits_dispatched",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KitStoreUpdateAllResult {
    updated: usize,
    failed: usize,
}

impl KitStoreUpdateAllResult {
    fn message(self) -> String {
        if self.failed == 0 {
            format!("Updated {} kit(s) successfully", self.updated)
        } else {
            format!("Updated {} kit(s), {} failed", self.updated, self.failed)
        }
    }

    fn is_failure(self) -> bool {
        self.failed > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesCommandBuiltinAction {
    OpenNotes,
    NewNote,
    SearchNotes,
    QuickCapture,
}

impl NotesCommandBuiltinAction {
    fn from_command(command: builtins::NotesCommandType) -> Self {
        match command {
            builtins::NotesCommandType::OpenNotes => Self::OpenNotes,
            builtins::NotesCommandType::NewNote => Self::NewNote,
            builtins::NotesCommandType::SearchNotes => Self::SearchNotes,
            builtins::NotesCommandType::QuickCapture => Self::QuickCapture,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::OpenNotes => "notes_command::OpenNotes",
            Self::NewNote => "notes_command::NewNote",
            Self::SearchNotes => "notes_command::SearchNotes",
            Self::QuickCapture => "notes_command::QuickCapture",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::OpenNotes => "notes_command_failed::OpenNotes",
            Self::NewNote => "notes_command_failed::NewNote",
            Self::SearchNotes => "notes_command_failed::SearchNotes",
            Self::QuickCapture => "notes_command_failed::QuickCapture",
        }
    }

    fn opens_notes_window(self) -> bool {
        matches!(self, Self::OpenNotes | Self::NewNote | Self::SearchNotes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptCommandBuiltinAction {
    NewScript,
    NewExtension,
}

impl ScriptCommandBuiltinAction {
    fn from_command(command: builtins::ScriptCommandType) -> Self {
        match command {
            builtins::ScriptCommandType::NewScript => Self::NewScript,
            builtins::ScriptCommandType::NewExtension => Self::NewExtension,
        }
    }

    fn naming_target(self) -> prompts::NamingTarget {
        match self {
            Self::NewScript => prompts::NamingTarget::Script,
            Self::NewExtension => prompts::NamingTarget::Extension,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::NewScript => "script_command::NewScript",
            Self::NewExtension => "script_command::NewExtension",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrecencyCommandBuiltinAction {
    ClearSuggested,
}

impl FrecencyCommandBuiltinAction {
    fn from_command(command: builtins::FrecencyCommandType) -> Self {
        match command {
            builtins::FrecencyCommandType::ClearSuggested => Self::ClearSuggested,
        }
    }

    fn hud_text(self) -> &'static str {
        match self {
            Self::ClearSuggested => "Suggested items cleared",
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::ClearSuggested => "clear_suggested",
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::ClearSuggested => "clear_suggested_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsCommandBuiltinAction {
    ResetWindowPositions,
    ChooseTheme,
    DictationSetup,
    SelectMicrophone,
    SnapMode(SettingsSnapModeBuiltinAction),
}

impl SettingsCommandBuiltinAction {
    fn from_command(command: builtins::SettingsCommandType) -> Self {
        match command {
            builtins::SettingsCommandType::ResetWindowPositions => Self::ResetWindowPositions,
            builtins::SettingsCommandType::ChooseTheme => Self::ChooseTheme,
            builtins::SettingsCommandType::DictationSetup => Self::DictationSetup,
            builtins::SettingsCommandType::SelectMicrophone => Self::SelectMicrophone,
            builtins::SettingsCommandType::DisableWindowSnapping
            | builtins::SettingsCommandType::SnapModeSimple
            | builtins::SettingsCommandType::SnapModeExpanded
            | builtins::SettingsCommandType::SnapModePrecision => Self::SnapMode(
                SettingsSnapModeBuiltinAction::from_command(command)
                    .expect("snap mode settings command should map to snap mode action"),
            ),
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::ResetWindowPositions => "reset_window_positions",
            Self::ChooseTheme => "choose_theme",
            Self::DictationSetup => "dictation_setup",
            Self::SelectMicrophone => "select_microphone",
            Self::SnapMode(action) => action.success_detail(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiPresetViewBuiltinAction {
    Create,
    Search,
}

impl AiPresetViewBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::CreateAiPreset => Some(Self::Create),
            builtins::AiCommandType::SearchAiPresets => Some(Self::Search),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Create => "ai_create_preset",
            Self::Search => "ai_search_presets",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiCaptureBuiltinAction {
    FullScreen,
    FocusedWindow,
    SelectedText,
    BrowserTab,
}

impl AiCaptureBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::SendScreenToAi => Some(Self::FullScreen),
            builtins::AiCommandType::SendFocusedWindowToAi => Some(Self::FocusedWindow),
            builtins::AiCommandType::SendSelectedTextToAi => Some(Self::SelectedText),
            builtins::AiCommandType::SendBrowserTabToAi => Some(Self::BrowserTab),
            _ => None,
        }
    }

    fn capture_kind(self) -> crate::ai::TabAiCaptureKind {
        match self {
            Self::FullScreen => crate::ai::TabAiCaptureKind::FullScreen,
            Self::FocusedWindow => crate::ai::TabAiCaptureKind::FocusedWindow,
            Self::SelectedText => crate::ai::TabAiCaptureKind::SelectedText,
            Self::BrowserTab => crate::ai::TabAiCaptureKind::BrowserTab,
        }
    }

    fn prompt(self) -> &'static str {
        match self {
            Self::FullScreen => "Capture and analyze the full screen.",
            Self::FocusedWindow => "Capture and analyze the focused window.",
            Self::SelectedText => "Use the current selected text as the primary subject.",
            Self::BrowserTab => {
                "Use the current browser tab URL and page context as the primary subject."
            }
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::FullScreen => "ai_send_screen_routed_to_harness",
            Self::FocusedWindow => "ai_send_focused_window_routed_to_harness",
            Self::SelectedText => "ai_send_selected_text_routed_to_harness",
            Self::BrowserTab => "ai_send_browser_tab_routed_to_harness",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiGenerateBuiltinAction {
    NewScript,
    CurrentAppScript,
}

impl AiGenerateBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::GenerateScript => Some(Self::NewScript),
            builtins::AiCommandType::GenerateScriptFromCurrentApp => Some(Self::CurrentAppScript),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::NewScript => "ai_generate_script_routed_to_harness",
            Self::CurrentAppScript => "ai_generate_script_from_current_app_routed_to_harness",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiPresetFileBuiltinAction {
    Import,
    Export,
}

impl AiPresetFileBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::ImportAiPresets => Some(Self::Import),
            builtins::AiCommandType::ExportAiPresets => Some(Self::Export),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::Import => "ai_import_presets_dispatched",
            Self::Export => "ai_export_presets_dispatched",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiUnavailableBuiltinAction {
    ScreenAreaCapture,
}

impl AiUnavailableBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::SendScreenAreaToAi => Some(Self::ScreenAreaCapture),
            _ => None,
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::ScreenAreaCapture => {
                "Send Screen Area to Agent Chat is unavailable until selected-area capture is attached to Agent Chat."
            }
        }
    }

    fn failure_detail(self) -> &'static str {
        match self {
            Self::ScreenAreaCapture => "ai_send_screen_area_unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiLegacyHarnessBuiltinAction {
    OpenAi,
    MiniAi,
    NewConversation,
    ClearConversation,
}

impl AiLegacyHarnessBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Option<Self> {
        match command {
            builtins::AiCommandType::OpenAi => Some(Self::OpenAi),
            builtins::AiCommandType::MiniAi => Some(Self::MiniAi),
            builtins::AiCommandType::NewConversation => Some(Self::NewConversation),
            builtins::AiCommandType::ClearConversation => Some(Self::ClearConversation),
            _ => None,
        }
    }

    fn success_detail(self) -> &'static str {
        match self {
            Self::OpenAi => "ai_OpenAi_routed_to_harness",
            Self::MiniAi => "ai_MiniAi_routed_to_harness",
            Self::NewConversation => "ai_NewConversation_routed_to_harness",
            Self::ClearConversation => "ai_ClearConversation_routed_to_harness",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiCommandBuiltinAction {
    Generate(AiGenerateBuiltinAction),
    Capture(AiCaptureBuiltinAction),
    Unavailable(AiUnavailableBuiltinAction),
    PresetView(AiPresetViewBuiltinAction),
    PresetFile(AiPresetFileBuiltinAction),
    LegacyHarness(AiLegacyHarnessBuiltinAction),
}

impl AiCommandBuiltinAction {
    fn from_command(command: builtins::AiCommandType) -> Self {
        match command {
            builtins::AiCommandType::GenerateScript => Self::Generate(
                AiGenerateBuiltinAction::from_command(command)
                    .expect("generate command should map to generate action"),
            ),
            builtins::AiCommandType::GenerateScriptFromCurrentApp => Self::Generate(
                AiGenerateBuiltinAction::from_command(command)
                    .expect("current-app generate command should map to generate action"),
            ),
            builtins::AiCommandType::SendScreenToAi => Self::Capture(
                AiCaptureBuiltinAction::from_command(command)
                    .expect("screen command should map to capture action"),
            ),
            builtins::AiCommandType::SendFocusedWindowToAi => Self::Capture(
                AiCaptureBuiltinAction::from_command(command)
                    .expect("focused-window command should map to capture action"),
            ),
            builtins::AiCommandType::SendSelectedTextToAi => Self::Capture(
                AiCaptureBuiltinAction::from_command(command)
                    .expect("selected-text command should map to capture action"),
            ),
            builtins::AiCommandType::SendBrowserTabToAi => Self::Capture(
                AiCaptureBuiltinAction::from_command(command)
                    .expect("browser-tab command should map to capture action"),
            ),
            builtins::AiCommandType::SendScreenAreaToAi => Self::Unavailable(
                AiUnavailableBuiltinAction::from_command(command)
                    .expect("screen-area command should map to unavailable action"),
            ),
            builtins::AiCommandType::CreateAiPreset => Self::PresetView(
                AiPresetViewBuiltinAction::from_command(command)
                    .expect("create preset command should map to preset view action"),
            ),
            builtins::AiCommandType::SearchAiPresets => Self::PresetView(
                AiPresetViewBuiltinAction::from_command(command)
                    .expect("search presets command should map to preset view action"),
            ),
            builtins::AiCommandType::ImportAiPresets => Self::PresetFile(
                AiPresetFileBuiltinAction::from_command(command)
                    .expect("import presets command should map to preset file action"),
            ),
            builtins::AiCommandType::ExportAiPresets => Self::PresetFile(
                AiPresetFileBuiltinAction::from_command(command)
                    .expect("export presets command should map to preset file action"),
            ),
            builtins::AiCommandType::OpenAi
            | builtins::AiCommandType::MiniAi
            | builtins::AiCommandType::NewConversation
            | builtins::AiCommandType::ClearConversation => Self::LegacyHarness(
                AiLegacyHarnessBuiltinAction::from_command(command)
                    .expect("legacy command should map to legacy harness action"),
            ),
        }
    }
}

/// Generate a stable semantic ID for a built-in prompt choice.
///
/// Format: `{prompt_id}:choice:{index}:{value_slug}`
///
/// `prompt_id` already contains the `builtin:` prefix (e.g. `builtin:select-microphone`).
fn builtin_choice_semantic_id(prompt_id: &str, index: usize, value: &str) -> String {
    crate::protocol::generate_semantic_id(&format!("{prompt_id}:choice"), index, value)
}

/// Typed progress events sent from the blocking download thread to the
/// async context for updating the in-prompt progress display.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DictationModelProgressEvent {
    Downloading {
        percentage: u8,
        downloaded_bytes: u64,
        total_bytes: u64,
        speed_bytes_per_sec: u64,
        eta_seconds: Option<u64>,
    },
    Extracting,
}

/// Simple rolling-window speed tracker for download progress.
struct SpeedTracker {
    last_bytes: u64,
    last_time: std::time::Instant,
    speed: u64,
}

impl SpeedTracker {
    fn new() -> Self {
        Self {
            last_bytes: 0,
            last_time: std::time::Instant::now(),
            speed: 0,
        }
    }

    fn update(&mut self, downloaded: u64) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();
        if elapsed >= 0.5 {
            let delta = downloaded.saturating_sub(self.last_bytes);
            self.speed = (delta as f64 / elapsed) as u64;
            self.last_bytes = downloaded;
            self.last_time = now;
        }
    }

    fn speed_bytes_per_sec(&self) -> u64 {
        self.speed
    }
}

/// Phases tracked by the UI coalescing emitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DictationModelUiPhase {
    Downloading,
    Extracting,
}

/// Snapshot of the last UI-visible state, used to decide whether a new
/// progress event is worth publishing.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DictationModelUiSnapshot {
    phase: DictationModelUiPhase,
    percentage: u8,
    eta_bucket_seconds: Option<u64>,
}

impl DictationModelUiSnapshot {
    fn downloading(percentage: u8, eta_seconds: Option<u64>) -> Self {
        Self {
            phase: DictationModelUiPhase::Downloading,
            percentage,
            eta_bucket_seconds: bucket_dictation_eta_seconds(eta_seconds),
        }
    }

    fn extracting() -> Self {
        Self {
            phase: DictationModelUiPhase::Extracting,
            percentage: 100,
            eta_bucket_seconds: Some(0),
        }
    }
}

/// Gates cosmetic UI updates so the download thread is never blocked on
/// repaints.  Publishes on meaningful change or after a ~300 ms heartbeat.
#[derive(Debug, Default)]
struct DictationModelUiEmitter {
    last_emit_at: Option<std::time::Instant>,
    last_snapshot: Option<DictationModelUiSnapshot>,
}

impl DictationModelUiEmitter {
    fn should_emit(&self, now: std::time::Instant, next: &DictationModelUiSnapshot) -> bool {
        const HEARTBEAT: std::time::Duration = std::time::Duration::from_millis(300);

        let Some(last_snapshot) = self.last_snapshot.as_ref() else {
            return true;
        };
        let Some(last_emit_at) = self.last_emit_at else {
            return true;
        };

        if last_snapshot.phase != next.phase {
            return true;
        }
        if last_snapshot.percentage != next.percentage {
            return true;
        }
        if last_snapshot.eta_bucket_seconds != next.eta_bucket_seconds {
            return true;
        }

        now.duration_since(last_emit_at) >= HEARTBEAT
    }

    fn record_emit(&mut self, now: std::time::Instant, next: &DictationModelUiSnapshot) {
        self.last_emit_at = Some(now);
        self.last_snapshot = Some(next.clone());
    }
}

/// Bucket ETA seconds into human-friendly steps so minor fluctuations
/// don't trigger a UI repaint.
fn bucket_dictation_eta_seconds(eta_seconds: Option<u64>) -> Option<u64> {
    eta_seconds.map(|value| match value {
        0..=15 => value,
        16..=60 => value - (value % 5),
        61..=300 => value - (value % 15),
        _ => value - (value % 60),
    })
}

/// Prevent overlapping Parakeet model downloads when the dictation hotkey is
/// pressed repeatedly while the model is still missing.
static PARAKEET_MODEL_DOWNLOAD_IN_PROGRESS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

static DICTATION_MODEL_PROMPT_STATUS: std::sync::OnceLock<
    parking_lot::Mutex<crate::dictation::DictationModelStatus>,
> = std::sync::OnceLock::new();

fn dictation_model_prompt_status(
) -> &'static parking_lot::Mutex<crate::dictation::DictationModelStatus> {
    DICTATION_MODEL_PROMPT_STATUS.get_or_init(|| {
        parking_lot::Mutex::new(crate::dictation::DictationModelStatus::NotDownloaded)
    })
}

static PARAKEET_MODEL_DOWNLOAD_CANCEL: std::sync::OnceLock<
    parking_lot::Mutex<Option<std::sync::Arc<std::sync::atomic::AtomicBool>>>,
> = std::sync::OnceLock::new();

fn parakeet_model_download_cancel_slot(
) -> &'static parking_lot::Mutex<Option<std::sync::Arc<std::sync::atomic::AtomicBool>>> {
    PARAKEET_MODEL_DOWNLOAD_CANCEL.get_or_init(|| parking_lot::Mutex::new(None))
}

#[cfg(test)]
fn ai_open_failure_message(error: impl std::fmt::Display) -> String {
    format!("Failed to open AI: {}", error)
}

#[derive(Debug)]
enum DeferredAiCapturedText {
    Ready(String),
    Empty(String),
}

fn ai_capture_hide_settle_duration() -> std::time::Duration {
    std::time::Duration::from_millis(AI_CAPTURE_HIDE_SETTLE_MS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiCommandWindowPlan {
    KeepMainWindowVisible,
    HideMainWindowDeferred,
    HideMainWindowForCapture,
}

impl AiCommandWindowPlan {
    fn from_command(cmd_type: &builtins::AiCommandType) -> Self {
        // All active AI commands now route to the harness terminal, which is
        // a view inside the main window — keep it visible.
        match cmd_type {
            builtins::AiCommandType::GenerateScript
            | builtins::AiCommandType::GenerateScriptFromCurrentApp
            | builtins::AiCommandType::SendScreenToAi
            | builtins::AiCommandType::SendFocusedWindowToAi
            | builtins::AiCommandType::SendSelectedTextToAi
            | builtins::AiCommandType::SendBrowserTabToAi
            | builtins::AiCommandType::SendScreenAreaToAi => Self::KeepMainWindowVisible,
            // Legacy aliases (OpenAi, MiniAi, NewConversation, ClearConversation)
            // also open the harness terminal inside the main window.
            cmd if cmd.is_legacy_harness_alias() => Self::KeepMainWindowVisible,
            // Preset commands (debug-only) retain their original behavior.
            _ => Self::HideMainWindowDeferred,
        }
    }

    fn keeps_main_window_visible(self) -> bool {
        matches!(self, Self::KeepMainWindowVisible)
    }

    fn uses_hide_then_capture_flow(self) -> bool {
        matches!(self, Self::HideMainWindowForCapture)
    }
}

fn ai_command_keeps_main_window_visible(cmd_type: &builtins::AiCommandType) -> bool {
    AiCommandWindowPlan::from_command(cmd_type).keeps_main_window_visible()
}

fn ai_command_uses_hide_then_capture_flow(cmd_type: &builtins::AiCommandType) -> bool {
    AiCommandWindowPlan::from_command(cmd_type).uses_hide_then_capture_flow()
}

#[cfg(test)]
fn favorites_loaded_message(count: usize) -> String {
    if count == 1 {
        "Loaded 1 favorite".to_string()
    } else {
        format!("Loaded {} favorites", count)
    }
}

#[cfg(test)]
fn created_file_path_for_feedback(path: &std::path::Path) -> std::path::PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    match std::env::current_dir() {
        Ok(current_dir) => current_dir.join(path),
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn applescript_list_literal(values: &[String]) -> String {
    let escaped_values = values
        .iter()
        .map(|value| format!("\"{}\"", crate::utils::escape_applescript_string(value)))
        .join(", ");
    format!("{{{}}}", escaped_values)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn choose_from_list(
    prompt: &str,
    ok_button: &str,
    values: &[String],
) -> Result<Option<String>, String> {
    if values.is_empty() {
        return Ok(None);
    }

    let list_literal = applescript_list_literal(values);
    let script = format!(
        r#"set selectedItem to choose from list {list_literal} with prompt "{prompt}" OK button name "{ok_button}" cancel button name "Cancel" without multiple selections allowed
if selectedItem is false then
    return ""
end if
return item 1 of selectedItem"#,
        list_literal = list_literal,
        prompt = crate::utils::escape_applescript_string(prompt),
        ok_button = crate::utils::escape_applescript_string(ok_button),
    );

    let selected = crate::platform::run_osascript(&script, "builtin_picker_choose_from_list")
        .map_err(|error| error.to_string())?;
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Retained for potential future AppleScript-based pickers
fn prompt_for_text(
    prompt: &str,
    default_value: &str,
    ok_button: &str,
) -> Result<Option<String>, String> {
    let script = format!(
        r#"try
set dialogResult to display dialog "{prompt}" default answer "{default_value}" buttons {{"Cancel", "{ok_button}"}} default button "{ok_button}"
return text returned of dialogResult
on error number -128
return ""
end try"#,
        prompt = crate::utils::escape_applescript_string(prompt),
        default_value = crate::utils::escape_applescript_string(default_value),
        ok_button = crate::utils::escape_applescript_string(ok_button),
    );

    let value = crate::platform::run_osascript(&script, "builtin_picker_prompt_for_text")
        .map_err(|error| error.to_string())?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[cfg(test)]
fn emoji_picker_label(emoji: &script_kit_gpui::emoji::Emoji) -> String {
    format!("{}  {}", emoji.emoji, emoji.name)
}

impl ScriptListApp {
    fn spawn_send_screen_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendScreenToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling screen capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_screen_screenshot() })
                .await;

            match capture_result {
                Ok((png_data, width, height)) => {
                    let size_bytes = png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendScreenToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting screen capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Screen capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &png_data,
                    );
                    let message = format!(
                        "[Screenshot captured: {}x{} pixels]\n\nPlease analyze this screenshot.",
                        width, height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendScreenToAi",
                        trace_id = %trace_id,
                        width,
                        height,
                        size_bytes,
                        "Screen captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        this.open_ai_window_after_already_hidden(
                            "SendScreenToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendScreenToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture screen for AI"
                    );
                    let message = format!("Failed to capture screen: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn spawn_send_focused_window_to_ai_after_hide(
        &mut self,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendFocusedWindowToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling focused window capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_focused_window_screenshot() })
                .await;

            match capture_result {
                Ok(capture) => {
                    let size_bytes = capture.png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendFocusedWindowToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting window capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Window capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let fallback_warning = capture.used_fallback.then(|| {
                        format!(
                            "No focused window found — captured '{}'",
                            capture.window_title
                        )
                    });
                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &capture.png_data,
                    );
                    let message = format!(
                        "[Window: {} - {}x{} pixels]\n\nPlease analyze this window screenshot.",
                        capture.window_title, capture.width, capture.height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendFocusedWindowToAi",
                        trace_id = %trace_id,
                        window_title = %capture.window_title,
                        width = capture.width,
                        height = capture.height,
                        size_bytes,
                        used_fallback = capture.used_fallback,
                        "Focused window captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        if let Some(warning_message) = fallback_warning {
                            this.toast_manager.push(
                                components::toast::Toast::warning(warning_message, &this.theme)
                                    .duration_ms(Some(TOAST_WARNING_MS)),
                            );
                            cx.notify();
                        }

                        this.open_ai_window_after_already_hidden(
                            "SendFocusedWindowToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendFocusedWindowToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture focused window for AI"
                    );
                    let message = format!("Failed to capture window: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn spawn_send_screen_area_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "SendScreenAreaToAi",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling screen area capture for AI"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let capture_result = cx
                .background_executor()
                .spawn(async { platform::capture_screen_area() })
                .await;

            match capture_result {
                Ok(Some(capture)) => {
                    let size_bytes = capture.png_data.len();
                    if size_bytes > crate::prompts::chat::MAX_IMAGE_BYTES {
                        tracing::warn!(
                            category = "AI",
                            event = "ai_capture_rejected",
                            source_action = "SendScreenAreaToAi",
                            trace_id = %trace_id,
                            size_bytes,
                            max_bytes = crate::prompts::chat::MAX_IMAGE_BYTES,
                            "Rejecting screen area capture larger than 10 MB"
                        );
                        this.update(cx, |this, cx| {
                            this.show_error_toast(
                                "Screen area capture exceeds 10 MB limit".to_string(),
                                cx,
                            );
                        })
                        .ok();
                        return;
                    }

                    let base64_data = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &capture.png_data,
                    );
                    let message = format!(
                        "[Screen area captured: {}x{} pixels]\n\nPlease analyze this selected screen area.",
                        capture.width, capture.height
                    );

                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_completed",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        width = capture.width,
                        height = capture.height,
                        size_bytes,
                        "Screen area captured for AI"
                    );

                    this.update(cx, |this, cx| {
                        this.open_ai_window_after_already_hidden(
                            "SendScreenAreaToAi",
                            &trace_id,
                            DeferredAiWindowAction::SetInputWithImage {
                                text: message,
                                image_base64: base64_data,
                                submit: false,
                            },
                            cx,
                        );
                    })
                    .ok();
                }
                Ok(None) => {
                    tracing::info!(
                        category = "AI",
                        event = "ai_capture_cancelled",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        "Screen area selection cancelled by user"
                    );
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action = "SendScreenAreaToAi",
                        trace_id = %trace_id,
                        error = %error,
                        "Failed to capture screen area for AI"
                    );
                    let message = format!("Failed to capture screen area: {}", error);
                    this.update(cx, |this, cx| {
                        this.show_error_toast(message, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_capture_text_to_ai_after_already_hidden<C, F>(
        &mut self,
        source_action: &'static str,
        trace_id: &str,
        capture_kind: &'static str,
        capture_fn: C,
        format_fn: F,
        cx: &mut Context<Self>,
    ) where
        C: FnOnce() -> Result<DeferredAiCapturedText, String> + Send + 'static,
        F: FnOnce(String) -> String + Send + 'static,
    {
        let trace_id = trace_id.to_string();

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action,
            trace_id = %trace_id,
            capture_kind,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Scheduled deferred AI text capture"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let (result_tx, result_rx) =
                async_channel::bounded::<Result<DeferredAiCapturedText, String>>(1);

            let trace_id_for_thread = trace_id.clone();
            std::thread::spawn(move || {
                let started_at = std::time::Instant::now();
                let result = capture_fn();

                let (success, result_state) = match &result {
                    Ok(DeferredAiCapturedText::Ready(_)) => (true, "ready"),
                    Ok(DeferredAiCapturedText::Empty(_)) => (true, "empty"),
                    Err(_) => (false, "error"),
                };

                tracing::info!(
                    category = "AI",
                    event = "ai_capture_completed",
                    source_action,
                    trace_id = %trace_id_for_thread,
                    capture_kind,
                    result_state,
                    success,
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Deferred AI text capture finished"
                );

                let _ = result_tx.send_blocking(result);
            });

            let Ok(result) = result_rx.recv().await else {
                return;
            };

            let _ = this.update(cx, |this, cx| match result {
                Ok(DeferredAiCapturedText::Ready(captured)) => {
                    this.open_ai_window_after_already_hidden(
                        source_action,
                        &trace_id,
                        DeferredAiWindowAction::SetInput {
                            text: format_fn(captured),
                            submit: false,
                        },
                        cx,
                    );
                }
                Ok(DeferredAiCapturedText::Empty(message)) => {
                    this.toast_manager.push(
                        components::toast::Toast::info(message, &this.theme)
                            .duration_ms(Some(TOAST_INFO_MS)),
                    );
                    cx.notify();
                }
                Err(error) => {
                    tracing::error!(
                        category = "AI",
                        event = "ai_capture_failed",
                        source_action,
                        trace_id = %trace_id,
                        capture_kind,
                        error = %error,
                        "Deferred AI text capture failed"
                    );
                    let message = format!("Failed to capture content for Agent Chat: {}", error);
                    this.toast_manager.push(
                        components::toast::Toast::error(message, &this.theme)
                            .duration_ms(Some(TOAST_CRITICAL_MS)),
                    );
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn spawn_send_selected_text_to_ai_after_hide(
        &mut self,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) {
        self.spawn_capture_text_to_ai_after_already_hidden(
            "SendSelectedTextToAi",
            trace_id,
            "selected_text",
            || {
                crate::selected_text::get_selected_text()
                    .map_err(|error| error.to_string())
                    .map(|text| {
                        let trimmed = text.trim().to_string();
                        if trimmed.is_empty() {
                            DeferredAiCapturedText::Empty(
                                "No text selected. Select some text first.".to_string(),
                            )
                        } else {
                            DeferredAiCapturedText::Ready(trimmed)
                        }
                    })
            },
            |text| {
                format!(
                    "I've selected the following text:\n\n```\n{}\n```\n\nPlease help me with this.",
                    text
                )
            },
            cx,
        );
    }

    fn spawn_send_browser_tab_to_ai_after_hide(&mut self, trace_id: &str, cx: &mut Context<Self>) {
        self.spawn_capture_text_to_ai_after_already_hidden(
            "SendBrowserTabToAi",
            trace_id,
            "browser_url",
            || {
                platform::get_focused_browser_tab_url()
                    .map_err(|error| error.to_string())
                    .map(|url| {
                        let trimmed = url.trim().to_string();
                        if trimmed.is_empty() {
                            DeferredAiCapturedText::Empty(
                                "No browser URL found in the frontmost tab.".to_string(),
                            )
                        } else {
                            DeferredAiCapturedText::Ready(trimmed)
                        }
                    })
            },
            |url| {
                format!(
                    "I'm looking at this webpage:\n\n{}\n\nPlease help me analyze or understand its content.",
                    url
                )
            },
            cx,
        );
    }

    fn spawn_generate_script_from_current_app_after_hide(
        &mut self,
        trace_id: String,
        query_override: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let fallback_query = query_override.unwrap_or_else(|| self.filter_text.clone());

        tracing::info!(
            category = "AI",
            event = "ai_capture_scheduled",
            source_action = "GenerateScriptFromCurrentApp",
            trace_id = %trace_id,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling context capture for script generation"
        );

        platform::defer_hide_main_window(cx);

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            let snapshot_result = cx
                .background_executor()
                .spawn(async { crate::menu_bar::load_frontmost_menu_snapshot() })
                .await;

            let selected_text = match crate::selected_text::get_selected_text() {
                Ok(text) if !text.trim().is_empty() => Some(text),
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(
                        trace_id = %trace_id,
                        error = %error,
                        "ai_generate_script_from_current_app.selected_text_unavailable"
                    );
                    None
                }
            };

            let browser_url = match platform::get_focused_browser_tab_url() {
                Ok(url) if !url.trim().is_empty() => Some(url),
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(
                        trace_id = %trace_id,
                        error = %error,
                        "ai_generate_script_from_current_app.browser_url_unavailable"
                    );
                    None
                }
            };

            // Build prompt outside entity borrow so we can show window safely.
            let prompt_or_error = match snapshot_result {
                Ok(snapshot) => {
                    let user_request =
                        crate::menu_bar::current_app_commands::normalize_generate_script_from_current_app_request(
                            Some(fallback_query.as_str()),
                        );

                    let (prompt, receipt) =
                        crate::menu_bar::current_app_commands::build_generate_script_prompt_from_snapshot(
                            snapshot,
                            user_request,
                            selected_text.as_deref(),
                            browser_url.as_deref(),
                        );

                    tracing::info!(
                        trace_id = %trace_id,
                        app_name = %receipt.app_name,
                        bundle_id = %receipt.bundle_id,
                        total_menu_items = receipt.total_menu_items,
                        included_menu_items = receipt.included_menu_items,
                        included_user_request = receipt.included_user_request,
                        included_selected_text = receipt.included_selected_text,
                        included_browser_url = receipt.included_browser_url,
                        "ai_generate_script_from_current_app.prompt_ready"
                    );

                    Ok(prompt)
                }
                Err(error) => Err(error),
            };

            match prompt_or_error {
                Ok(prompt) => {
                    // Platform calls — trigger macOS delegate callbacks.
                    // Safe here: no AppCell borrow is active.
                    script_kit_gpui::set_main_window_visible(true);
                    tracing::info!(
                        trace_id = %trace_id,
                        "ai_generate_script_from_current_app.showing_window"
                    );
                    crate::platform::show_main_window_without_activation();

                    // GPUI state changes inside entity borrow.
                    let _ = this.update(cx, |app, cx| {
                        app.dispatch_ai_script_generation_from_query(prompt, cx);
                    });
                }
                Err(error) => {
                    let _ = this.update(cx, |app, cx| {
                        let message = format!("Failed to capture current app context: {}", error);
                        app.show_error_toast(message.clone(), cx);
                        tracing::error!(
                            trace_id = %trace_id,
                            error = %error,
                            "ai_generate_script_from_current_app.capture_failed"
                        );
                    });
                }
            }
        })
        .detach();
    }

    /// Like `spawn_generate_script_from_current_app_after_hide`, but reuses an
    /// already-built recipe instead of recapturing live context after hide.
    ///
    /// This eliminates prompt drift: the prompt copied in the recipe is
    /// byte-for-byte the prompt sent to the AI generation path.
    fn spawn_generate_script_from_recipe_after_hide(
        &mut self,
        trace_id: String,
        recipe: crate::menu_bar::current_app_commands::CurrentAppCommandRecipe,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            category = "AI",
            event = "ai_recipe_generation_scheduled",
            source_action = "TurnThisIntoCommand",
            trace_id = %trace_id,
            recipe_prompt_bytes = recipe.prompt.len(),
            recipe_bundle_id = %recipe.prompt_receipt.bundle_id,
            recipe_included_selected_text = recipe.prompt_receipt.included_selected_text,
            recipe_included_browser_url = recipe.prompt_receipt.included_browser_url,
            hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
            "Deferring main window hide and scheduling recipe-based script generation (no recapture)"
        );

        platform::defer_hide_main_window(cx);

        let prompt =
            crate::menu_bar::current_app_commands::build_generated_script_prompt_from_recipe(
                &recipe,
            );

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(ai_capture_hide_settle_duration())
                .await;

            tracing::info!(
                trace_id = %trace_id,
                recipe_prompt_bytes = prompt.len(),
                recipe_bundle_id = %recipe.prompt_receipt.bundle_id,
                recipe_included_selected_text = recipe.prompt_receipt.included_selected_text,
                recipe_included_browser_url = recipe.prompt_receipt.included_browser_url,
                "ai_generate_script_from_recipe.prompt_ready"
            );

            // Platform calls — trigger macOS delegate callbacks.
            // Safe here: no AppCell borrow is active.
            script_kit_gpui::set_main_window_visible(true);
            tracing::info!(
                trace_id = %trace_id,
                "ai_generate_script_from_recipe.showing_window"
            );
            crate::platform::show_main_window_without_activation();

            // GPUI state changes inside entity borrow.
            let _ = this.update(cx, |app, cx| {
                app.dispatch_ai_script_generation_from_query(prompt, cx);
            });
        })
        .detach();
    }

    /// Schedule the DoInCurrentApp→GenerateScript flow, capturing selected
    /// text and the focused browser URL off the UI thread.
    ///
    /// Both `get_selected_text()` (AX-first, clipboard fallback) and
    /// `get_focused_browser_tab_url()` (single `osascript` call gated by the
    /// in-process frontmost-app tracker) can block for hundreds of
    /// milliseconds. Running them on `cx.background_executor()` keeps the
    /// launcher responsive while macOS answers; the memory lookup, recipe
    /// build, and dispatch run back on the main thread once capture completes.
    fn spawn_generate_script_from_current_app_with_capture(
        &mut self,
        trace_id: String,
        raw_query_owned: String,
        snapshot_for_recipe: crate::menu_bar::FrontmostMenuSnapshot,
        entries: Vec<crate::builtins::BuiltInEntry>,
        snapshot_receipt: crate::menu_bar::FrontmostMenuSnapshotReceipt,
        snapshot_pid: i32,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            trace_id = %trace_id,
            raw_query = %raw_query_owned,
            "do_in_current_app.spawn_context_capture"
        );

        cx.spawn(async move |this, cx| {
            let capture_started_at = std::time::Instant::now();
            let (selected_text, browser_url) = cx
                .background_executor()
                .spawn(async {
                    let selected_text = crate::selected_text::get_selected_text()
                        .ok()
                        .filter(|text| !text.trim().is_empty());
                    let browser_url = crate::platform::get_focused_browser_tab_url()
                        .ok()
                        .filter(|url| !url.trim().is_empty());
                    (selected_text, browser_url)
                })
                .await;

            tracing::info!(
                trace_id = %trace_id,
                capture_ms = capture_started_at.elapsed().as_millis() as u64,
                has_selected_text = selected_text.is_some(),
                has_browser_url = browser_url.is_some(),
                "do_in_current_app.context_capture_complete"
            );

            let _ = this.update(cx, |this, cx| {
                this.continue_generate_script_from_current_app_after_capture(
                    trace_id,
                    raw_query_owned,
                    snapshot_for_recipe,
                    entries,
                    snapshot_receipt,
                    snapshot_pid,
                    selected_text,
                    browser_url,
                    cx,
                );
            });
        })
        .detach();
    }

    /// Continuation of `spawn_generate_script_from_current_app_with_capture`,
    /// invoked back on the main thread once the blocking capture finishes.
    fn continue_generate_script_from_current_app_after_capture(
        &mut self,
        trace_id: String,
        raw_query_owned: String,
        snapshot_for_recipe: crate::menu_bar::FrontmostMenuSnapshot,
        entries: Vec<crate::builtins::BuiltInEntry>,
        snapshot_receipt: crate::menu_bar::FrontmostMenuSnapshotReceipt,
        snapshot_pid: i32,
        selected_text: Option<String>,
        browser_url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let memory_decision = crate::ai::resolve_current_app_automation_from_memory(
            &raw_query_owned,
            &snapshot_for_recipe,
            &entries,
            selected_text.as_deref(),
            browser_url.as_deref(),
        );

        if let Ok(ref decision) = memory_decision {
            if let Some(ref replay) = decision.replay {
                tracing::info!(
                    category = "CURRENT_APP_AUTOMATION_MEMORY",
                    trace_id = %trace_id,
                    action = %decision.action,
                    best_score = decision.best_score,
                    matched_slug = decision
                        .matched
                        .as_ref()
                        .map(|entry| entry.slug.as_str())
                        .unwrap_or(""),
                    reason = %decision.reason,
                    "do_in_current_app.memory_resolved"
                );

                match decision.action.as_str() {
                    "replay_recipe" => match replay.action.as_str() {
                        "execute_entry" => {
                            if let Some(entry_index) = replay.selected_entry_index {
                                if entry_index < entries.len() {
                                    let entry = entries[entry_index].clone();
                                    let dctx = crate::action_helpers::DispatchContext {
                                        trace_id: trace_id.clone(),
                                        surface: crate::action_helpers::DispatchSurface::Builtin,
                                        action_id: entry.id.clone(),
                                    };
                                    let _ = self.execute_builtin_inner(
                                        &entry,
                                        Some(&raw_query_owned),
                                        &dctx,
                                        cx,
                                    );
                                    return;
                                }
                            }
                        }
                        "open_command_palette" => {
                            let filter = replay.verification.live_recipe.effective_query.clone();
                            self.present_current_app_commands_entries(
                                entries.clone(),
                                &snapshot_receipt,
                                snapshot_pid,
                                &filter,
                                cx,
                            );
                            return;
                        }
                        "generate_script" => {
                            self.spawn_generate_script_from_recipe_after_hide(
                                trace_id.clone(),
                                replay.verification.live_recipe.clone(),
                                cx,
                            );
                            return;
                        }
                        _ => {}
                    },
                    "repair_recipe" => {
                        self.spawn_generate_script_from_recipe_after_hide(
                            trace_id.clone(),
                            replay.verification.live_recipe.clone(),
                            cx,
                        );
                        return;
                    }
                    _ => {}
                }
            }
        }

        let recipe = crate::menu_bar::current_app_commands::build_current_app_command_recipe(
            snapshot_for_recipe,
            Some(&raw_query_owned),
            selected_text.as_deref(),
            browser_url.as_deref(),
        );

        match serde_json::to_string_pretty(&recipe) {
            Ok(json) => {
                tracing::info!(
                    category = "CURRENT_APP_RECIPE",
                    trace_id = %trace_id,
                    app_name = %recipe.prompt_receipt.app_name,
                    bundle_id = %recipe.prompt_receipt.bundle_id,
                    effective_query = %recipe.effective_query,
                    route = %recipe.trace.action,
                    suggested_script_name = %recipe.suggested_script_name,
                    included_selected_text = recipe.prompt_receipt.included_selected_text,
                    included_browser_url = recipe.prompt_receipt.included_browser_url,
                    json_bytes = json.len(),
                    "do_in_current_app.recipe_prepared"
                );
            }
            Err(error) => {
                tracing::warn!(
                    trace_id = %trace_id,
                    error = %error,
                    "do_in_current_app.recipe_serialize_failed"
                );
            }
        }

        self.spawn_generate_script_from_recipe_after_hide(trace_id, recipe, cx);
    }

    fn system_action_feedback_message(
        &self,
        action_type: &builtins::SystemActionType,
    ) -> Option<String> {
        let dark_mode_enabled = if matches!(action_type, builtins::SystemActionType::ToggleDarkMode)
        {
            system_actions::is_dark_mode().ok()
        } else {
            None
        };

        builtins::system_action_hud_message(*action_type, dark_mode_enabled)
    }

    /// Shared dispatch for system actions — used by both the normal and confirmed paths.
    /// Maps a `SystemActionType` to its implementation, handles special cases
    /// (TestConfirmation, QuitScriptKit), and routes the result through
    /// `handle_system_action_result`.
    /// Structured outcome logger for builtin execution paths.
    ///
    /// Emits a single log line with all fields needed for machine consumption
    /// and human debugging: builtin_id, trace_id, surface, handler, status,
    /// error_code, and duration_ms.
    fn log_builtin_outcome(
        builtin_id: &str,
        dctx: &crate::action_helpers::DispatchContext,
        handler: &str,
        outcome: &crate::action_helpers::DispatchOutcome,
        start: &std::time::Instant,
    ) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let trace_id = outcome
            .trace_id
            .as_deref()
            .unwrap_or(dctx.trace_id.as_str());

        match outcome.status {
            crate::action_helpers::ActionOutcomeStatus::Error => {
                tracing::error!(
                    category = "BUILTIN",
                    builtin_id = %builtin_id,
                    trace_id = %trace_id,
                    surface = %dctx.surface,
                    handler,
                    status = %outcome.status,
                    error_code = outcome.error_code,
                    duration_ms,
                    detail = ?outcome.detail,
                    "Builtin execution finished"
                );
            }
            _ => {
                tracing::info!(
                    category = "BUILTIN",
                    builtin_id = %builtin_id,
                    trace_id = %trace_id,
                    surface = %dctx.surface,
                    handler,
                    status = %outcome.status,
                    error_code = outcome.error_code,
                    duration_ms,
                    detail = ?outcome.detail,
                    "Builtin execution finished"
                );
            }
        }
    }

    /// Build a success outcome carrying the dispatch context's trace_id.
    fn builtin_success(
        dctx: &crate::action_helpers::DispatchContext,
        detail: impl Into<String>,
    ) -> crate::action_helpers::DispatchOutcome {
        crate::action_helpers::DispatchOutcome::success()
            .with_trace_id(dctx.trace_id.clone())
            .with_detail(detail)
    }

    /// Build an error outcome carrying the dispatch context's trace_id.
    fn builtin_error(
        dctx: &crate::action_helpers::DispatchContext,
        code: &'static str,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> crate::action_helpers::DispatchOutcome {
        crate::action_helpers::DispatchOutcome::error(code, message)
            .with_trace_id(dctx.trace_id.clone())
            .with_detail(detail)
    }

    fn execute_menu_bar_builtin(
        &mut self,
        action_state: MenuBarBuiltinAction,
        action: &builtins::MenuBarActionInfo,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            bundle_id = %action.bundle_id,
            "Executing menu bar action"
        );
        #[cfg(target_os = "macos")]
        {
            match script_kit_gpui::menu_executor::execute_menu_action(
                &action.bundle_id,
                &action.menu_path,
            ) {
                Ok(()) => {
                    self.close_and_reset_window(cx);
                    Self::builtin_success(dctx, action_state.success_detail())
                }
                Err(e) => {
                    let message = format!("Menu action failed: {}", e);
                    self.show_error_toast(message.clone(), cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        message,
                        action_state.failure_detail(),
                    )
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.show_unsupported_platform_toast("Menu bar actions", cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
                "Menu bar actions only supported on macOS",
                action_state.unsupported_detail(),
            )
        }
    }

    fn execute_system_builtin(
        &mut self,
        action: SystemBuiltinAction,
        action_type: &builtins::SystemActionType,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            action_type = ?action_type,
            action = ?action,
            action_name = %action.handler_name(),
            "Executing system action via inner path"
        );
        self.dispatch_system_action(action_type, dctx, cx)
    }

    fn dispatch_system_action(
        &mut self,
        action_type: &builtins::SystemActionType,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let start = std::time::Instant::now();

        tracing::info!(
            category = "BUILTIN",
            builtin_id = %dctx.action_id,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            action_type = ?action_type,
            status = "dispatched",
            "system_action_dispatch"
        );

        #[cfg(target_os = "macos")]
        {
            use builtins::SystemActionType;

            let result = match action_type {
                // Power management
                SystemActionType::EmptyTrash => system_actions::empty_trash(),
                SystemActionType::LockScreen => system_actions::lock_screen(),
                SystemActionType::Sleep => system_actions::sleep(),
                SystemActionType::Restart => system_actions::restart(),
                SystemActionType::ShutDown => system_actions::shut_down(),
                SystemActionType::LogOut => system_actions::log_out(),

                // UI controls
                SystemActionType::ToggleDarkMode => system_actions::toggle_dark_mode(),
                SystemActionType::ShowDesktop => system_actions::show_desktop(),
                SystemActionType::MissionControl => system_actions::mission_control(),
                SystemActionType::Launchpad => system_actions::launchpad(),
                SystemActionType::ForceQuitApps => system_actions::force_quit_apps(),

                // Volume controls (preset levels)
                SystemActionType::Volume0 => system_actions::set_volume(0),
                SystemActionType::Volume25 => system_actions::set_volume(25),
                SystemActionType::Volume50 => system_actions::set_volume(50),
                SystemActionType::Volume75 => system_actions::set_volume(75),
                SystemActionType::Volume100 => system_actions::set_volume(100),
                SystemActionType::VolumeMute => system_actions::volume_mute(),

                // Dev/test actions
                #[cfg(debug_assertions)]
                SystemActionType::TestConfirmation => {
                    self.toast_manager.push(
                        components::toast::Toast::success("Confirmation test passed!", &self.theme)
                            .duration_ms(Some(TOAST_SUCCESS_MS)),
                    );
                    cx.notify();
                    return Self::builtin_success(dctx, "system_action_test_confirmation");
                }

                // App control
                SystemActionType::QuitScriptKit => {
                    Self::prepare_script_kit_shutdown();
                    cx.quit();
                    return Self::builtin_success(dctx, "quit_script_kit");
                }

                // System utilities
                SystemActionType::ToggleDoNotDisturb => system_actions::toggle_do_not_disturb(),
                SystemActionType::StartScreenSaver => system_actions::start_screen_saver(),

                // System Preferences
                SystemActionType::OpenSystemPreferences => {
                    system_actions::open_system_preferences_main()
                }
                SystemActionType::OpenPrivacySettings => system_actions::open_privacy_settings(),
                SystemActionType::OpenDisplaySettings => system_actions::open_display_settings(),
                SystemActionType::OpenSoundSettings => system_actions::open_sound_settings(),
                SystemActionType::OpenNetworkSettings => system_actions::open_network_settings(),
                SystemActionType::OpenKeyboardSettings => system_actions::open_keyboard_settings(),
                SystemActionType::OpenBluetoothSettings => {
                    system_actions::open_bluetooth_settings()
                }
                SystemActionType::OpenNotificationsSettings => {
                    system_actions::open_notifications_settings()
                }
            };

            self.handle_system_action_result(result, action_type, dctx, start.elapsed(), cx)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = action_type;
            self.show_unsupported_platform_toast("System actions", cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_UNSUPPORTED_PLATFORM,
                "System actions are not supported on this platform",
                "system_action_unsupported_platform",
            )
        }
    }

    /// Shared result handler for system actions — shows HUD on success, Toast on error.
    /// Returns a `DispatchOutcome` for structured logging at the call boundary.
    fn handle_system_action_result(
        &mut self,
        result: Result<(), String>,
        action_type: &builtins::SystemActionType,
        dctx: &crate::action_helpers::DispatchContext,
        elapsed: std::time::Duration,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let duration_ms = elapsed.as_millis() as u64;
        match result {
            Ok(()) => {
                tracing::info!(
                    category = "BUILTIN",
                    builtin_id = %dctx.action_id,
                    trace_id = %dctx.trace_id,
                    surface = %dctx.surface,
                    action_type = ?action_type,
                    status = "success",
                    duration_ms,
                    "system_action_dispatch"
                );
                if let Some(message) = self.system_action_feedback_message(action_type) {
                    cx.notify();
                    self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                    self.hide_main_and_reset(cx);
                } else {
                    self.close_and_reset_window(cx);
                }
                Self::builtin_success(dctx, format!("system_action::{action_type:?}"))
            }
            Err(error) => {
                tracing::error!(
                    category = "BUILTIN",
                    builtin_id = %dctx.action_id,
                    trace_id = %dctx.trace_id,
                    surface = %dctx.surface,
                    action_type = ?action_type,
                    status = "error",
                    error_code = crate::action_helpers::ERROR_LAUNCH_FAILED,
                    duration_ms,
                    error = %error,
                    "system_action_dispatch"
                );
                self.show_error_toast(format!("System action failed: {}", error), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_LAUNCH_FAILED,
                    format!("System action failed: {}", error),
                    format!("system_action::{action_type:?}; error={error}"),
                )
            }
        }
    }

    fn prepare_script_kit_shutdown() {
        tracing::info!(
            category = "UI",
            event = "prepare_script_kit_shutdown",
            "prepare_script_kit_shutdown"
        );
        PROCESS_MANAGER.kill_all_processes();
        PROCESS_MANAGER.remove_main_pid();
    }

    fn quit_script_kit_confirm_options() -> crate::confirm::ParentConfirmOptions {
        crate::confirm::ParentConfirmOptions::destructive(
            "Quit Script Kit",
            "Quit Script Kit and stop all running processes?",
            "Quit",
        )
    }

    fn builtin_confirmation_options(
        entry_id: &str,
        entry_name: &str,
    ) -> crate::confirm::ParentConfirmOptions {
        match entry_id {
            "builtin/quit-script-kit" => Self::quit_script_kit_confirm_options(),
            "builtin/shut-down" => crate::confirm::ParentConfirmOptions::destructive(
                "Shut Down Mac",
                "Shut down this Mac now?",
                "Shut Down",
            ),
            "builtin/restart" => crate::confirm::ParentConfirmOptions::destructive(
                "Restart Mac",
                "Restart this Mac now?",
                "Restart",
            ),
            "builtin/log-out" => crate::confirm::ParentConfirmOptions::destructive(
                "Log Out",
                "Log out of the current macOS session?",
                "Log Out",
            ),
            "builtin/empty-trash" => crate::confirm::ParentConfirmOptions::destructive(
                "Empty Trash",
                "Empty Trash now? This cannot be undone.",
                "Empty Trash",
            ),
            "builtin/sleep" => crate::confirm::ParentConfirmOptions {
                title: "Sleep Mac".into(),
                body: "Put this Mac to sleep now?".into(),
                confirm_text: "Sleep".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
            "builtin/force-quit" => crate::confirm::ParentConfirmOptions::destructive(
                "Open Force Quit Apps",
                "Open Force Quit Apps?",
                "Open",
            ),
            "builtin/stop-all-processes" => crate::confirm::ParentConfirmOptions::destructive(
                "Stop All Processes",
                "Stop all running Script Kit processes?",
                "Stop All",
            ),
            "builtin/clear-suggested" => crate::confirm::ParentConfirmOptions::destructive(
                "Clear Suggested",
                "Clear suggested items and reset their ranking data?",
                "Clear Suggested",
            ),
            "builtin/sync-to-github" => crate::confirm::ParentConfirmOptions {
                title: "Sync to GitHub".into(),
                body: "Write safe .gitignore exclusions, commit Script Kit changes, and sync this workspace to GitHub?".into(),
                confirm_text: "Sync".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
            "builtin/test-confirmation" => crate::confirm::ParentConfirmOptions {
                title: "Test Confirmation".into(),
                body: "Open the confirmation test action?".into(),
                confirm_text: "Run Test".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
            _ => crate::confirm::ParentConfirmOptions {
                title: "Confirm".into(),
                body: format!("Are you sure you want to {}?", entry_name).into(),
                confirm_text: "Continue".into(),
                cancel_text: "Cancel".into(),
                ..Default::default()
            },
        }
    }

    pub(crate) fn execute_builtin(
        &mut self,
        entry: &builtins::BuiltInEntry,
        cx: &mut Context<Self>,
    ) {
        self.execute_builtin_with_query(entry, None, cx);
    }

    fn execute_builtin_with_query(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let start = std::time::Instant::now();
        let dctx = crate::action_helpers::DispatchContext::for_builtin(&entry.id);

        tracing::info!(
            category = "BUILTIN",
            builtin_id = %entry.id,
            builtin_name = %entry.name,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            "Builtin execution started"
        );

        // Clear any stale actions popup from previous view
        self.clear_actions_popup_state();

        // Check if this command requires confirmation - open modal if so.
        // Quit Script Kit goes through this same path (see builtin_confirmation_options
        // and DEFAULT_CONFIRMATION_COMMANDS) — the parent confirm popup is a separate
        // native window with its own focus, so Tab/Esc work without competing with the
        // launcher or any other view's key handlers.
        if self.config.requires_confirmation(&entry.id) {
            let confirmation_start = std::time::Instant::now();
            let entry_id = entry.id.clone();
            let query_owned = query_override.map(|s| s.to_string());
            let dctx_owned = dctx.clone();
            let confirm_options = Self::builtin_confirmation_options(&entry.id, &entry.name);

            // Spawn a task to show confirmation dialog via shared parent dialog helper
            cx.spawn(async move |this, cx| {
                match crate::confirm::confirm_with_parent_dialog(
                    cx,
                    confirm_options,
                    &dctx_owned.trace_id,
                )
                .await
                {
                    Ok(true) => {
                        let _ = this.update(cx, |this, cx| {
                            this.handle_builtin_confirmation(
                                entry_id,
                                true,
                                query_owned,
                                &dctx_owned,
                                cx,
                            );
                        });
                    }
                    Ok(false) => {
                        let outcome = crate::action_helpers::DispatchOutcome::cancelled()
                            .with_trace_id(dctx_owned.trace_id.clone())
                            .with_detail("builtin_confirmation_cancelled");
                        let _ = this.update(cx, |_, _| {
                            Self::log_builtin_outcome(
                                &entry_id,
                                &dctx_owned,
                                "confirmation_gate",
                                &outcome,
                                &confirmation_start,
                            );
                        });
                    }
                    Err(e) => {
                        let _ = this.update(cx, |this, cx| {
                            tracing::error!(
                                builtin_id = %entry_id,
                                trace_id = %dctx_owned.trace_id,
                                error = %e,
                                "failed to open confirmation modal"
                            );
                            this.show_error_toast_with_code(
                                "Failed to open confirmation dialog",
                                Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                cx,
                            );
                            let outcome = Self::builtin_error(
                                &dctx_owned,
                                crate::action_helpers::ERROR_MODAL_FAILED,
                                "Failed to open confirmation dialog",
                                format!("confirmation_modal_error={e}"),
                            );
                            Self::log_builtin_outcome(
                                &entry_id,
                                &dctx_owned,
                                "confirmation_gate",
                                &outcome,
                                &confirmation_start,
                            );
                        });
                    }
                }
            })
            .detach();

            tracing::info!(
                category = "BUILTIN",
                trace_id = %dctx.trace_id,
                builtin_id = %entry.id,
                status = "awaiting_confirmation",
                duration_ms = start.elapsed().as_millis() as u64,
                "Builtin execution deferred to confirmation modal"
            );
            return; // Wait for modal callback
        }

        // All builtins now return DispatchOutcome — system actions are handled
        // inside execute_builtin_inner as well.
        let outcome = self.execute_builtin_inner(entry, query_override, &dctx, cx);

        Self::log_builtin_outcome(&entry.id, &dctx, "builtin_execution", &outcome, &start);
    }

    /// Open a filterable main-window builtin view with a consistent UX contract.
    ///
    /// Every filterable builtin should go through this helper so that focus,
    /// placeholder, filter reset, hover clearing, resize, and opened-from-menu
    /// state are always set the same way.
    ///
    /// `expanded` picks the window sizing contract: `false` matches the main
    /// menu's compact 480×440 (Mini) window for light-weight pickers like
    /// emoji / apps / browser tabs / window switcher; `true` uses the wide
    /// 750×500 (Full) window for info-heavy views like clipboard history.
    fn open_builtin_filterable_view(
        &mut self,
        view: AppView,
        placeholder: &str,
        expanded: bool,
        cx: &mut Context<Self>,
    ) {
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(placeholder.to_string());
        self.current_view = view;
        self.hovered_index = None;
        self.opened_from_main_menu = true;
        if expanded {
            self.set_main_window_mode_state_only(
                MainWindowMode::Full,
                cx,
                "open_builtin_filterable_view",
            );
            resize_to_view_sync(ViewType::ScriptList, 0);
        } else {
            self.set_main_window_mode_state_only(
                MainWindowMode::Mini,
                cx,
                "open_builtin_filterable_view",
            );
            resize_to_view_sync(ViewType::MiniMainWindow, 0);
        }
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    fn open_theme_chooser_view(&mut self, cx: &mut Context<Self>) {
        self.theme_before_chooser = Some(self.theme.clone());
        let start_index = theme::presets::find_current_preset_index(&self.theme);

        self.open_builtin_filterable_view(
            AppView::ThemeChooserView {
                filter: String::new(),
                selected_index: start_index,
            },
            "Search themes or paste #hex...",
            true,
            cx,
        );

        let item_count = theme::presets::presets_cached().len();
        let old_count = self.theme_chooser_list_state.item_count();
        if old_count != item_count {
            self.theme_chooser_list_state
                .splice(0..old_count, item_count);
        }
        self.theme_chooser_list_state
            .scroll_to_reveal_item(start_index);
    }

    fn open_mini_main_window(&mut self, cx: &mut Context<Self>) {
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search scripts, apps, and commands…".to_string());
        self.show_script_list_with_main_filter_focus();
        self.set_main_window_mode_state_only(MainWindowMode::Mini, cx, "open_mini_main_window");
        self.hovered_index = None;
        self.selected_index = 0;
        self.opened_from_main_menu = true;
        self.invalidate_grouped_cache();
        self.sync_list_state();
        let (grouped_items, _) = self.get_grouped_results_cached();
        let item_count = grouped_items.len();
        // Skip section headers — select first actual item so cmd+k works immediately
        let first_selectable =
            crate::list_item::GroupedListState::from_items(&grouped_items).first_selectable;
        self.selected_index = first_selectable;
        tracing::info!(
            event = "open_mini_main_window",
            item_count = item_count,
            selected_index = self.selected_index,
            first_selectable = first_selectable,
            grouped_cache_key = %self.main_menu_result_caches.grouped_cache_key(),
            computed_filter = %self.computed_filter_text,
            filter_text = %self.filter_text,
            pending_filter_sync = self.pending_filter_sync,
            "open_mini_main_window: items={}, selected={}",
            item_count,
            self.selected_index,
        );
        resize_to_view_sync(ViewType::MiniMainWindow, item_count);
        cx.notify();
    }

    fn open_ai_vault_source_filter(&mut self, cx: &mut Context<Self>) {
        let filter_text = "vault: ".to_string();
        self.cancel_history_filter_render_pending_if_obsolete(&filter_text);
        self.filter_text = filter_text.clone();
        self.computed_filter_text = filter_text.clone();
        self.pending_programmatic_filter_echo = Some(filter_text.clone());
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search AI Vault sessions...".to_string());
        self.set_menu_syntax_mode_from_filter(&filter_text);
        self.show_script_list_with_main_filter_focus();
        self.hovered_index = None;
        self.selected_index = 0;
        self.opened_from_main_menu = true;
        self.main_menu_fallback_state.clear();
        self.invalidate_grouped_cache();
        self.filter_coalescer.reset();
        self.maybe_start_root_file_search(&filter_text, cx);
        self.reconcile_script_list_after_filter_change("open_ai_vault_source_filter", cx);
        let (grouped_items, _) = self.get_grouped_results_cached();
        let item_count = grouped_items.len();
        self.set_main_window_mode_state_only(
            MainWindowMode::Mini,
            cx,
            "open_ai_vault_source_filter",
        );
        resize_to_view_sync(ViewType::MiniMainWindow, item_count);
        cx.notify();
    }

    /// Open a filterable builtin view with an initial filter value.
    ///
    /// Same UX contract as [`open_builtin_filterable_view`] but pre-fills the
    /// filter input instead of clearing it. Used by `DoInCurrentApp` to open
    /// the command palette with the user's query already typed.
    ///
    /// See `open_builtin_filterable_view` for the meaning of `expanded`.
    fn open_builtin_filterable_view_with_filter(
        &mut self,
        view: AppView,
        filter: &str,
        placeholder: &str,
        expanded: bool,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            view = ?view,
            filter = %filter,
            placeholder = %placeholder,
            expanded,
            "open_builtin_filterable_view_with_filter — setting current_view and filter"
        );
        self.filter_text = filter.to_string();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(placeholder.to_string());
        self.current_view = view;
        self.hovered_index = None;
        self.opened_from_main_menu = true;
        if expanded {
            self.set_main_window_mode_state_only(
                MainWindowMode::Full,
                cx,
                "open_builtin_filterable_view_with_filter",
            );
            resize_to_view_sync(ViewType::ScriptList, 0);
        } else {
            self.set_main_window_mode_state_only(
                MainWindowMode::Mini,
                cx,
                "open_builtin_filterable_view_with_filter",
            );
            resize_to_view_sync(ViewType::MiniMainWindow, 0);
        }
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    pub(crate) fn present_current_app_commands_entries(
        &mut self,
        entries: Vec<crate::builtins::BuiltInEntry>,
        receipt: &crate::menu_bar::FrontmostMenuSnapshotReceipt,
        pid: i32,
        filter: &str,
        cx: &mut Context<Self>,
    ) {
        let session = crate::menu_bar::CurrentAppCommandsSession::from_entries_and_receipt(
            entries,
            receipt.clone(),
            pid,
        );
        self.present_current_app_commands_session(session, filter, cx);
    }

    pub(crate) fn present_current_app_commands_session(
        &mut self,
        session: crate::menu_bar::CurrentAppCommandsSession,
        filter: &str,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            pid = session.pid,
            app_name = %session.app_name,
            bundle_id = %session.bundle_id,
            top_level_menu_count = session.top_level_menu_count,
            leaf_entry_count = session.leaf_entry_count,
            placeholder = %session.placeholder,
            source = session.source,
            filter = %filter,
            "current_app_commands.present_session"
        );

        self.current_app_commands_session = Some(session.clone());
        self.cached_current_app_entries = session.entries.clone();
        self.open_builtin_filterable_view_with_filter(
            AppView::CurrentAppCommandsView {
                filter: filter.to_string(),
                selected_index: 0,
            },
            filter,
            &session.placeholder,
            false,
            cx,
        );
        self.current_app_commands_scroll_handle
            .scroll_to_item(0, gpui::ScrollStrategy::Top);

        if self.cached_current_app_entries.is_empty() {
            tracing::info!(
                pid = session.pid,
                app_name = %session.app_name,
                bundle_id = %session.bundle_id,
                "current_app_commands.present_empty_state"
            );
        }
    }

    pub(crate) fn open_current_app_commands_from_tray(
        &mut self,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let session = crate::menu_bar::capture_current_app_commands_session()
            .map_err(|e| anyhow::anyhow!("Failed to load frontmost app menu bar: {e}"))?;
        self.present_current_app_commands_session(session, "", cx);
        Ok(())
    }

    pub(crate) fn refresh_current_app_commands_session_if_needed(
        &mut self,
        filter: &str,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<bool> {
        let Some(existing) = self.current_app_commands_session.clone() else {
            return Ok(false);
        };

        let live_identity = crate::menu_bar::load_live_current_app_commands_identity();
        if !crate::menu_bar::current_app_commands_session_identity_changed(
            &existing,
            live_identity.as_ref(),
        ) {
            return Ok(false);
        }

        let refreshed = match crate::menu_bar::capture_current_app_commands_session() {
            Ok(session) => session,
            Err(error) => {
                self.invalidate_current_app_commands_session(filter, cx);
                return Err(error);
            }
        };
        let previous_app_name = existing.app_name.clone();
        let refreshed_app_name = refreshed.app_name.clone();
        tracing::info!(
            previous_pid = existing.pid,
            new_pid = refreshed.pid,
            previous_bundle_id = %existing.bundle_id,
            new_bundle_id = %refreshed.bundle_id,
            previous_app_name = %previous_app_name,
            new_app_name = %refreshed_app_name,
            filter = %filter,
            "current_app_commands.session_switched"
        );
        self.present_current_app_commands_session(refreshed, filter, cx);
        self.show_hud(
            format!("Current app changed: {previous_app_name} -> {refreshed_app_name}"),
            Some(HUD_SHORT_MS),
            cx,
        );
        tracing::info!(
            previous_app_name = %previous_app_name,
            new_app_name = %refreshed_app_name,
            filter = %filter,
            "current_app_commands.session_switch_hud_shown"
        );
        Ok(true)
    }

    pub(crate) fn invalidate_current_app_commands_session(
        &mut self,
        filter: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(existing) = &self.current_app_commands_session {
            tracing::warn!(
                previous_pid = existing.pid,
                previous_bundle_id = %existing.bundle_id,
                previous_app_name = %existing.app_name,
                filter = %filter,
                "current_app_commands.session_invalidated"
            );
        }

        self.current_app_commands_session = None;
        self.cached_current_app_entries.clear();
        self.filter_text = filter.to_string();
        self.pending_placeholder = Some("Search current app commands…".to_string());

        if let AppView::CurrentAppCommandsView {
            filter: current_filter,
            selected_index,
        } = &mut self.current_view
        {
            *current_filter = filter.to_string();
            *selected_index = 0;
        }

        cx.notify();
    }

    pub(crate) fn execute_selected_current_app_command(
        &mut self,
        original_entry_index: usize,
        cx: &mut Context<Self>,
    ) {
        let filter = match &self.current_view {
            AppView::CurrentAppCommandsView { filter, .. } => filter.clone(),
            _ => String::new(),
        };

        match self.refresh_current_app_commands_session_if_needed(&filter, cx) {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    entry_index = original_entry_index,
                    "current_app_commands.refresh_failed_before_execute"
                );
                self.show_error_toast(
                    format!("Failed to refresh current app commands: {error}"),
                    cx,
                );
                return;
            }
        }

        let Some(entry) = self
            .cached_current_app_entries
            .get(original_entry_index)
            .cloned()
        else {
            tracing::warn!(
                entry_index = original_entry_index,
                cached_entry_count = self.cached_current_app_entries.len(),
                "current_app_commands.execute_selected_missing_index"
            );
            return;
        };

        tracing::info!(
            entry_id = %entry.id,
            entry_name = %entry.name,
            entry_index = original_entry_index,
            "current_app_commands.execute_selected_resolved"
        );
        self.execute_builtin(&entry, cx);
    }

    /// Inner builtin executor — runs the actual action logic.
    /// Called from both the normal path (after confirmation check) and the
    /// confirmed path (after modal approval), ensuring a single implementation.
    ///
    /// Returns a `DispatchOutcome` so callers can log the real result instead
    /// of a synthetic success.
    fn execute_builtin_inner(
        &mut self,
        entry: &builtins::BuiltInEntry,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive ClipboardHistory");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::PasteSequentially => {
                self.execute_paste_sequential_builtin(dctx, cx)
            }
            builtins::BuiltInFeature::Favorites => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive Favorites");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::AppLauncher => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive AppLauncher");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::App(app_name) => {
                let launch_action = AppLaunchBuiltinAction::from_feature(&entry.feature)
                    .expect("app launch arm should only receive App");
                self.execute_app_launch_builtin(launch_action, app_name, dctx, cx)
            }
            builtins::BuiltInFeature::WindowSwitcher => {
                let window_switcher_action =
                    WindowSwitcherBuiltinAction::from_feature(&entry.feature)
                        .expect("window switcher arm should only receive WindowSwitcher");
                self.execute_window_switcher_builtin(window_switcher_action, dctx, cx)
            }
            builtins::BuiltInFeature::BrowserTabs => {
                let browser_tabs_action = BrowserTabsBuiltinAction::from_feature(&entry.feature)
                    .expect("browser tabs arm should only receive BrowserTabs");
                self.execute_browser_tabs_builtin(browser_tabs_action, dctx, cx)
            }
            builtins::BuiltInFeature::DesignGallery => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive DesignGallery");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            #[cfg(feature = "storybook")]
            builtins::BuiltInFeature::DesignExplorer => {
                let design_action = DesignExplorerBuiltinAction::from_feature(&entry.feature)
                    .expect("design explorer arm should only receive DesignExplorer");
                self.execute_design_explorer_builtin(design_action, dctx, cx)
            }
            builtins::BuiltInFeature::AiChat => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive AiChat");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::Notes => {
                let notes_action = NotesBuiltinAction::from_feature(&entry.feature)
                    .expect("notes arm should only receive Notes");
                self.execute_notes_builtin(notes_action, dctx, cx)
            }
            builtins::BuiltInFeature::EmojiPicker => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive EmojiPicker");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::SyncToGithub => {
                let sync_action = SyncToGithubBuiltinAction::from_feature(&entry.feature)
                    .expect("sync-to-github arm should only receive SyncToGithub");
                self.execute_sync_to_github_builtin(sync_action, dctx, cx)
            }
            builtins::BuiltInFeature::MenuBarAction(action) => {
                let menu_action = MenuBarBuiltinAction::from_action(action);
                self.execute_menu_bar_builtin(menu_action, action, dctx, cx)
            }

            // =========================================================================
            // System Actions
            // =========================================================================
            builtins::BuiltInFeature::SystemAction(action_type) => {
                let system_action = SystemBuiltinAction::from_action(action_type);
                self.execute_system_builtin(system_action, action_type, dctx, cx)
            }

            // NOTE: Window Actions removed - now handled by window-management extension
            // SDK tileWindow() still works via protocol messages in execute_script.rs

            // =========================================================================
            // Notes Commands
            // =========================================================================
            builtins::BuiltInFeature::NotesCommand(cmd_type) => {
                tracing::info!(
                    target: "script_kit::keyboard",
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    notes_command = ?cmd_type,
                    event = "notes_command_handoff_preserving_launcher_context",
                    filter_text_len = self.filter_text.len(),
                    show_actions_popup = self.show_actions_popup,
                    "Executing notes command (preserving launcher context)"
                );

                let notes_action = NotesCommandBuiltinAction::from_command(*cmd_type);
                self.execute_notes_command_builtin(notes_action, dctx, cx)
            }

            // =========================================================================
            // AI Commands
            // =========================================================================
            builtins::BuiltInFeature::AiCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    ai_command = ?cmd_type,
                    "Executing AI command"
                );

                let window_plan = AiCommandWindowPlan::from_command(cmd_type);
                self.apply_ai_command_window_plan(window_plan, cmd_type, cx);

                let ai_action = AiCommandBuiltinAction::from_command(*cmd_type);
                self.execute_ai_command_builtin(ai_action, query_override, dctx, cx)
            }

            // =========================================================================
            // Script Commands
            // =========================================================================
            builtins::BuiltInFeature::ScriptCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    script_command = ?cmd_type,
                    "Executing script command"
                );

                let script_action = ScriptCommandBuiltinAction::from_command(*cmd_type);
                self.execute_script_command_builtin(script_action, dctx, cx)
            }

            // =========================================================================
            // Permission Commands
            // =========================================================================
            builtins::BuiltInFeature::PermissionCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    permission_command = ?cmd_type,
                    "Executing permission command"
                );

                let permission_action = PermissionCommandBuiltinAction::from_command(*cmd_type);
                self.execute_permission_command_builtin(permission_action, dctx, cx)
            }

            // =========================================================================
            // Frecency/Suggested Commands
            // =========================================================================
            builtins::BuiltInFeature::FrecencyCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    frecency_command = ?cmd_type,
                    "Executing frecency command"
                );

                let frecency_action = FrecencyCommandBuiltinAction::from_command(*cmd_type);
                self.execute_frecency_command_builtin(frecency_action, dctx, cx)
            }

            // =========================================================================
            // Settings Commands (Reset Window Positions, etc.)
            // =========================================================================
            builtins::BuiltInFeature::SettingsCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    settings_command = ?cmd_type,
                    "Executing settings command"
                );

                let settings_action = SettingsCommandBuiltinAction::from_command(*cmd_type);
                self.execute_settings_command_builtin(settings_action, dctx, cx)
            }

            // =========================================================================
            // Utility Commands (Scratch Pad, Quick Terminal, Claude Code Harness, Process Manager)
            // =========================================================================
            builtins::BuiltInFeature::UtilityCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    utility_command = ?cmd_type,
                    "Executing utility command"
                );

                let utility_action = UtilityCommandBuiltinAction::from_command(*cmd_type);
                self.execute_utility_command_builtin(utility_action, query_override, dctx, cx)
            }

            // =========================================================================
            // Kit Store Commands
            // =========================================================================
            builtins::BuiltInFeature::KitStoreCommand(cmd_type) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    kit_store_command = ?cmd_type,
                    "Executing kit store command"
                );

                let kit_action = KitStoreBuiltinAction::from_command(*cmd_type);
                self.execute_kit_store_builtin(kit_action, dctx, cx)
            }

            // =========================================================================
            // File Search (Directory Navigation)
            // =========================================================================
            builtins::BuiltInFeature::Webcam => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive Webcam");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::Dictation => self.execute_dictation_builtin_action(
                DictationBuiltinAction::CurrentSurface,
                dctx,
                cx,
            ),
            builtins::BuiltInFeature::DictationToAiHarness => {
                self.execute_dictation_builtin_action(DictationBuiltinAction::AgentChat, dctx, cx)
            }
            builtins::BuiltInFeature::DictationToFrontmostApp => self
                .execute_dictation_builtin_action(DictationBuiltinAction::FrontmostApp, dctx, cx),
            builtins::BuiltInFeature::DictationToNotes => {
                self.execute_dictation_builtin_action(DictationBuiltinAction::Notes, dctx, cx)
            }
            builtins::BuiltInFeature::FileSearch => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive FileSearch");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            // =========================================================================
            // Settings Hub
            // =========================================================================
            builtins::BuiltInFeature::Settings => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive Settings");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            // =========================================================================
            // ACP Conversation History
            // =========================================================================
            builtins::BuiltInFeature::AcpHistory => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive AcpHistory");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::AiVault => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive AiVault");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            builtins::BuiltInFeature::DictationHistory => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive DictationHistory");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            // =========================================================================
            // SDK Reference — browse kit://sdk-reference functions while authoring
            // =========================================================================
            builtins::BuiltInFeature::SdkReference => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive SdkReference");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
            // =========================================================================
            // New Script from Template — browse kit://script-templates catalog, pick
            // a starter, then hand off to the naming prompt with template metadata
            // threaded through so handle_naming_dialog_completion can overwrite the
            // freshly-created file with render_script_template_file before the
            // editor opens. See `src/render_builtins/script_templates.rs` and
            // `src/app_impl/naming_dialog.rs`.
            // =========================================================================
            builtins::BuiltInFeature::NewScriptFromTemplate => {
                let open_action = SurfaceOpenBuiltinAction::from_feature(&entry.feature)
                    .expect("surface open arm should only receive NewScriptFromTemplate");
                self.execute_surface_open_builtin(open_action, dctx, cx)
            }
        }
    }

    // =========================================================================
    // Dictation helpers — overlay pump, transcript delivery, scheduled cleanup
    // =========================================================================

    fn execute_settings_command_builtin(
        &mut self,
        action: SettingsCommandBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            SettingsCommandBuiltinAction::ResetWindowPositions => {
                self.reset_window_positions_to_default_main_menu(cx);
                Self::builtin_success(dctx, action.success_detail())
            }
            SettingsCommandBuiltinAction::ChooseTheme => {
                self.open_theme_chooser_view(cx);
                Self::builtin_success(dctx, action.success_detail())
            }
            SettingsCommandBuiltinAction::DictationSetup => {
                self.open_dictation_model_prompt(cx);
                Self::builtin_success(dctx, action.success_detail())
            }
            SettingsCommandBuiltinAction::SelectMicrophone => {
                self.execute_select_microphone_builtin(dctx, cx)
            }
            SettingsCommandBuiltinAction::SnapMode(snap_action) => {
                self.execute_settings_snap_mode_builtin(snap_action, dctx, cx)
            }
        }
    }

    fn execute_surface_open_builtin(
        &mut self,
        action: SurfaceOpenBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            SurfaceOpenBuiltinAction::ClipboardHistory => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                self.focused_clipboard_entry_id = self
                    .cached_clipboard_entries
                    .first()
                    .map(|entry| entry.id.clone());
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = self.cached_clipboard_entries.len(),
                    "Loaded clipboard entries"
                );

                self.open_builtin_filterable_view(
                    AppView::ClipboardHistoryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search clipboard history...",
                    true,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::Favorites => {
                tracing::info!(
                    category = "BUILTIN",
                    action = "open_favorites_view",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );

                self.open_builtin_filterable_view(
                    AppView::FavoritesBrowseView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search favorites...",
                    false,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::AppLauncher => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.apps = app_launcher::scan_applications().clone();
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = self.apps.len(),
                    "Loaded applications"
                );
                self.invalidate_filter_cache();
                self.invalidate_grouped_cache();
                self.sync_list_state();

                self.open_builtin_filterable_view(
                    AppView::AppLauncherView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search applications...",
                    false,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::DesignGallery => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );

                self.open_builtin_filterable_view(
                    AppView::DesignGalleryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search designs...",
                    false,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::AiChat => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.open_tab_ai_acp_with_entry_intent(None, cx);
            }
            SurfaceOpenBuiltinAction::EmojiPicker => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.emoji_frequent_snapshot = crate::emoji_usage::load_frequent_snapshot(
                    crate::emoji_usage::EMOJI_FREQUENT_LIMIT,
                );
                self.open_builtin_filterable_view(
                    AppView::EmojiPickerView {
                        filter: String::new(),
                        selected_index: 0,
                        selected_category: None,
                    },
                    "Search Emoji & Symbols...",
                    false,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::Webcam => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.opened_from_main_menu = true;
                self.open_webcam(cx);
            }
            SurfaceOpenBuiltinAction::FileSearch => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.opened_from_main_menu = true;
                self.open_file_search(String::new(), cx);
            }
            SurfaceOpenBuiltinAction::Settings => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.open_builtin_filterable_view(
                    AppView::SettingsView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search settings...",
                    false,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::AcpHistory => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.open_builtin_filterable_view(
                    AppView::AcpHistoryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search conversation history...",
                    true,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::AiVault => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.open_ai_vault_source_filter(cx);
            }
            SurfaceOpenBuiltinAction::DictationHistory => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    "{}",
                    action.log_message()
                );
                self.open_builtin_filterable_view(
                    AppView::DictationHistoryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search dictation history...",
                    true,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::SdkReference => {
                let entries = crate::mcp_resources::sdk_reference_entries_for_ui();
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    entry_count = entries.len(),
                    "{}",
                    action.log_message()
                );
                self.open_builtin_filterable_view(
                    AppView::SdkReferenceView {
                        filter: String::new(),
                        selected_index: 0,
                        entries,
                    },
                    "Search SDK functions…",
                    true,
                    cx,
                );
            }
            SurfaceOpenBuiltinAction::ScriptTemplateCatalog => {
                let templates = crate::mcp_resources::script_template_entries_for_ui();
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    template_count = templates.len(),
                    "{}",
                    action.log_message()
                );
                self.open_builtin_filterable_view(
                    AppView::ScriptTemplateCatalogView {
                        filter: String::new(),
                        selected_index: 0,
                        templates,
                    },
                    "Search starter templates…",
                    true,
                    cx,
                );
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_browser_tabs_builtin(
        &mut self,
        action: BrowserTabsBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            "{}",
            action.opening_message()
        );
        match crate::browser_tabs::list_open_tabs() {
            Ok(tabs) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = tabs.len(),
                    "{}",
                    action.loaded_message()
                );
                self.cached_browser_tabs = tabs;

                let domains =
                    crate::browser_tabs::domains_needing_favicons(&self.cached_browser_tabs);
                if !domains.is_empty() {
                    let task = cx.spawn(async move |this, cx| {
                        cx.background_executor()
                            .spawn(async move {
                                crate::browser_tabs::fetch_favicons_blocking(&domains);
                            })
                            .await;
                        let _ = cx.update(|cx| {
                            let _ = this.update(cx, |_, cx| cx.notify());
                        });
                    });
                    task.detach();
                }

                self.open_builtin_filterable_view(
                    AppView::BrowserTabsView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    action.placeholder(),
                    false,
                    cx,
                );

                Self::builtin_success(dctx, action.success_detail())
            }
            Err(error) => {
                let message = action.failure_message(&error);
                self.show_error_toast(message.clone(), cx);
                cx.notify();
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.failure_detail(),
                )
            }
        }
    }

    fn execute_app_launch_builtin(
        &mut self,
        action: AppLaunchBuiltinAction,
        app_name: &str,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            app = %app_name,
            "Launching app"
        );
        let apps = app_launcher::scan_applications();
        if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
            if let Err(error) = app_launcher::launch_application(app) {
                let message = format!("Failed to launch {}: {}", app_name, error);
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_LAUNCH_FAILED,
                    message,
                    action.success_detail(app_name),
                )
            } else {
                self.close_and_reset_window(cx);
                Self::builtin_success(dctx, action.success_detail(app_name))
            }
        } else {
            let message = format!("App not found: {}", app_name);
            self.show_error_toast(message.clone(), cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_ACTION_FAILED,
                message,
                action.not_found_detail(app_name),
            )
        }
    }

    fn execute_notes_builtin(
        &mut self,
        action: NotesBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            target: "script_kit::keyboard",
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            event = "notes_handoff_preserving_launcher_context",
            filter_text_len = self.filter_text.len(),
            show_actions_popup = self.show_actions_popup,
            "Opening Notes window (preserving launcher context)"
        );

        // Close companion UI before hiding so it does not stay stale.
        if crate::confirm::is_confirm_window_open() {
            crate::confirm::route_key_to_confirm_popup("escape", cx);
        }
        if crate::actions::is_actions_window_open() {
            crate::actions::close_actions_window(cx);
            self.mark_actions_popup_closed();
            self.mark_filter_resync_after_actions_if_needed();
            self.pop_focus_overlay(cx);
        }

        self.pending_focus = None;
        script_kit_gpui::set_main_window_visible(false);
        platform::defer_hide_main_window(cx);
        if let Err(error) = notes::open_notes_window_without_launcher_restore(cx) {
            script_kit_gpui::set_main_window_visible(true);
            platform::show_main_window_without_activation();
            let message = format!("Failed to open Notes: {}", error);
            self.show_error_toast(message.clone(), cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_LAUNCH_FAILED,
                message,
                action.failure_detail(),
            )
        } else {
            Self::builtin_success(dctx, action.success_detail())
        }
    }

    fn execute_sync_to_github_builtin(
        &mut self,
        action: SyncToGithubBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            "Sync to GitHub requested"
        );
        self.show_hud(
            "Syncing Script Kit to GitHub...".to_string(),
            Some(HUD_SHORT_MS),
            cx,
        );

        let dctx_owned = dctx.clone();
        cx.spawn(async move |this, cx| {
            let sync_result = cx
                .background_executor()
                .spawn(async { crate::sync::github::sync_to_github_workspace() })
                .await;

            let _ = this.update(cx, |this, cx| match sync_result {
                Ok(report) => {
                    tracing::info!(
                        category = "BUILTIN",
                        trace_id = %dctx_owned.trace_id,
                        workspace = %report.workspace.display(),
                        dry_run = report.dry_run,
                        step_count = report.steps.len(),
                        "Sync to GitHub completed"
                    );
                    this.show_hud(report.summary_message(), Some(HUD_MEDIUM_MS), cx);
                    this.close_and_reset_window(cx);
                }
                Err(error) => {
                    tracing::error!(
                        category = "BUILTIN",
                        trace_id = %dctx_owned.trace_id,
                        error = %error,
                        "Sync to GitHub failed"
                    );
                    this.show_error_toast(format!("GitHub sync failed: {error}"), cx);
                    cx.notify();
                }
            });
        })
        .detach();

        Self::builtin_success(dctx, action.success_detail())
    }

    #[cfg(feature = "storybook")]
    fn execute_design_explorer_builtin(
        &mut self,
        action: DesignExplorerBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            "Opening Design Explorer"
        );

        let explorer = cx.new(|cx| {
            let mut browser = script_kit_gpui::storybook::StoryBrowser::new(cx);
            browser.configure_for_design_explorer(Some(
                script_kit_gpui::storybook::StorySurface::MainMenu,
            ));
            browser.open_compare_mode();
            let _ = browser.select_variant_id("current-main-menu");
            tracing::info!(
                event = "design_explorer_opened",
                surface = "main-menu",
                preview_mode = "compare",
                variant_id = "current-main-menu",
                "Opened in-app design explorer on the compare-ready Main Menu surface"
            );
            browser
        });

        self.current_view = AppView::DesignExplorerView { entity: explorer };
        cx.notify();

        Self::builtin_success(dctx, action.success_detail())
    }

    fn apply_ai_command_window_plan(
        &mut self,
        plan: AiCommandWindowPlan,
        cmd_type: &builtins::AiCommandType,
        cx: &mut Context<Self>,
    ) {
        match plan {
            AiCommandWindowPlan::KeepMainWindowVisible => {}
            AiCommandWindowPlan::HideMainWindowDeferred => {
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::defer_hide_main_window(cx);
            }
            AiCommandWindowPlan::HideMainWindowForCapture => {
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                tracing::debug!(
                    action = ?cmd_type,
                    hide_settle_ms = AI_CAPTURE_HIDE_SETTLE_MS,
                    "Deferring main window hide to async capture flow"
                );
            }
        }
    }

    fn execute_window_switcher_builtin(
        &mut self,
        action: WindowSwitcherBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            "{}",
            action.opening_message()
        );
        match window_control::list_windows() {
            Ok(windows) => {
                tracing::info!(
                    category = "BUILTIN",
                    trace_id = %dctx.trace_id,
                    count = windows.len(),
                    "{}",
                    action.loaded_message()
                );
                self.cached_windows = windows;

                self.open_builtin_filterable_view(
                    AppView::WindowSwitcherView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    action.placeholder(),
                    false,
                    cx,
                );

                Self::builtin_success(dctx, action.success_detail())
            }
            Err(error) => {
                let message = action.failure_message(&error);
                self.show_error_toast(message.clone(), cx);
                cx.notify();
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.failure_detail(),
                )
            }
        }
    }

    fn execute_ai_capture_builtin(
        &mut self,
        action: AiCaptureBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        self.open_tab_ai_chat_with_capture_kind(
            Some(action.prompt().to_string()),
            action.capture_kind(),
            cx,
        );
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_ai_command_builtin(
        &mut self,
        action: AiCommandBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            AiCommandBuiltinAction::Generate(generate_action) => {
                self.execute_ai_generate_builtin(generate_action, query_override, dctx, cx)
            }
            AiCommandBuiltinAction::Capture(capture_action) => {
                self.execute_ai_capture_builtin(capture_action, dctx, cx)
            }
            AiCommandBuiltinAction::Unavailable(unavailable_action) => {
                self.execute_ai_unavailable_builtin(unavailable_action, dctx, cx)
            }
            AiCommandBuiltinAction::PresetView(preset_action) => {
                match preset_action {
                    AiPresetViewBuiltinAction::Create => {
                        tracing::info!(
                            action = "create_ai_preset",
                            trace_id = %dctx.trace_id,
                            "Opening create AI preset form"
                        );
                    }
                    AiPresetViewBuiltinAction::Search => {
                        tracing::info!(
                            action = "search_ai_presets",
                            trace_id = %dctx.trace_id,
                            "Opening AI presets search"
                        );
                    }
                }
                self.execute_ai_preset_view_builtin(preset_action, dctx, cx)
            }
            AiCommandBuiltinAction::PresetFile(file_action) => {
                match file_action {
                    AiPresetFileBuiltinAction::Import => {
                        tracing::info!(
                            action = "import_ai_presets",
                            "Opening file picker for AI preset import"
                        );
                    }
                    AiPresetFileBuiltinAction::Export => {
                        tracing::info!(
                            action = "export_ai_presets",
                            trace_id = %dctx.trace_id,
                            "Opening save dialog for AI preset export"
                        );
                    }
                }
                self.execute_ai_preset_file_builtin(file_action, dctx, cx)
            }
            AiCommandBuiltinAction::LegacyHarness(legacy_action) => {
                self.execute_ai_legacy_harness_builtin(legacy_action, dctx, cx)
            }
        }
    }

    fn execute_ai_generate_builtin(
        &mut self,
        action: AiGenerateBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let query = query_override.unwrap_or(&self.filter_text);
        match action {
            AiGenerateBuiltinAction::NewScript => {
                let request =
                    crate::menu_bar::current_app_commands::normalize_generate_script_request(Some(
                        query,
                    ))
                    .map(str::to_string);
                if let Some(request) = request {
                    self.open_tab_ai_chat_with_entry_intent(Some(request), cx);
                } else {
                    self.open_tab_ai_acp_with_entry_intent(None, cx);
                }
            }
            AiGenerateBuiltinAction::CurrentAppScript => {
                let request = crate::menu_bar::current_app_commands::normalize_generate_script_from_current_app_request(Some(query));
                let intent = if let Some(request) = request {
                    format!(
                        "Generate a Script Kit script for the frontmost app \
                         using the current menu, selection, and browser context. \
                         User request: {request}"
                    )
                } else {
                    "Generate a Script Kit script for the frontmost app \
                     using the current menu, selection, and browser context."
                        .to_string()
                };
                self.open_tab_ai_chat_with_entry_intent(Some(intent), cx);
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_ai_preset_file_builtin(
        &mut self,
        action: AiPresetFileBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            AiPresetFileBuiltinAction::Import => {
                let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some("Select AI presets JSON file".into()),
                    allowed_extensions: vec!["json".into()],
                });

                cx.spawn(async move |this, cx| {
                    match rx.await {
                        Ok(Ok(Some(paths))) => {
                            if let Some(path) = paths.first() {
                                // Validate file contents before importing
                                let import_result = cx
                                    .background_executor()
                                    .spawn({
                                        let path = path.clone();
                                        async move {
                                            let contents =
                                                std::fs::read_to_string(&path).map_err(|e| {
                                                    format!("Failed to read file: {}", e)
                                                })?;
                                            ai::presets::validate_presets_json(&contents).map_err(
                                                |e| format!("Invalid preset file: {}", e),
                                            )?;
                                            ai::presets::import_presets_from_file(&path)
                                                .map_err(|e| format!("Import failed: {}", e))
                                        }
                                    })
                                    .await;

                                let _ = this.update(cx, |this, cx| {
                                    match import_result {
                                        Ok(total) => {
                                            tracing::info!(
                                                total = total,
                                                action = "import_presets_success",
                                                "Imported AI presets via file picker"
                                            );
                                            this.show_hud(
                                                format!("Imported presets ({} total)", total),
                                                Some(HUD_SHORT_MS),
                                                cx,
                                            );
                                            ai::reload_ai_presets(cx);
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                error = %e,
                                                action = "import_presets_failed",
                                                "Failed to import presets"
                                            );
                                            this.show_error_toast(
                                                format!("Failed to import presets: {}", e),
                                                cx,
                                            );
                                        }
                                    }
                                    cx.notify();
                                });
                            }
                        }
                        Ok(Ok(None)) => {
                            tracing::info!(
                                action = "import_presets_cancelled",
                                "User cancelled import file picker"
                            );
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(error = %e, "Import file picker returned error");
                        }
                        Err(_) => {
                            tracing::warn!("Import file picker channel closed unexpectedly");
                        }
                    }
                })
                .detach();
            }
            AiPresetFileBuiltinAction::Export => {
                let default_dir = ai::presets::get_presets_path()
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(crate::setup::get_kit_path);

                let rx = cx.prompt_for_new_path(&default_dir, Some("ai-presets-export.json"));

                cx.spawn(async move |this, cx| match rx.await {
                    Ok(Ok(Some(path))) => {
                        let export_result = cx
                            .background_executor()
                            .spawn({
                                let path = path.clone();
                                async move {
                                    ai::presets::export_presets_to_file(&path)
                                        .map_err(|e| format!("Export failed: {}", e))
                                }
                            })
                            .await;

                        let _ = this.update(cx, |this, cx| {
                            match export_result {
                                Ok(count) => {
                                    tracing::info!(
                                        count = count,
                                        path = %path.display(),
                                        action = "export_presets_success",
                                        "Exported AI presets via file picker"
                                    );
                                    this.show_hud(
                                        format!("Exported {} presets", count),
                                        Some(HUD_SHORT_MS),
                                        cx,
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(
                                        error = %e,
                                        action = "export_presets_failed",
                                        "Failed to export presets"
                                    );
                                    this.show_error_toast(
                                        format!("Failed to export presets: {}", e),
                                        cx,
                                    );
                                }
                            }
                            cx.notify();
                        });
                    }
                    Ok(Ok(None)) => {
                        tracing::info!(
                            action = "export_presets_cancelled",
                            "User cancelled export save dialog"
                        );
                    }
                    Ok(Err(e)) => {
                        tracing::warn!(error = %e, "Export save dialog returned error");
                    }
                    Err(_) => {
                        tracing::warn!("Export save dialog channel closed unexpectedly");
                    }
                })
                .detach();
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_ai_unavailable_builtin(
        &mut self,
        action: AiUnavailableBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let message = action.message();
        self.toast_manager.push(
            components::toast::Toast::error(message, &self.theme).duration_ms(Some(TOAST_ERROR_MS)),
        );
        cx.notify();
        Self::builtin_error(
            dctx,
            crate::action_helpers::ERROR_ACTION_FAILED,
            message.to_string(),
            action.failure_detail(),
        )
    }

    fn execute_ai_legacy_harness_builtin(
        &mut self,
        action: AiLegacyHarnessBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        self.open_tab_ai_acp_with_entry_intent(None, cx);
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_ai_preset_view_builtin(
        &mut self,
        action: AiPresetViewBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            AiPresetViewBuiltinAction::Create => {
                self.current_view = AppView::CreateAiPresetView {
                    name: String::new(),
                    system_prompt: String::new(),
                    model: String::new(),
                    active_field: 0,
                };
                self.pending_focus = Some(FocusTarget::AppRoot);
            }
            AiPresetViewBuiltinAction::Search => {
                self.current_view = AppView::SearchAiPresetsView {
                    filter: String::new(),
                    selected_index: 0,
                };
                self.pending_focus = Some(FocusTarget::MainFilter);
            }
        }

        cx.notify();
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_settings_snap_mode_builtin(
        &mut self,
        action: SettingsSnapModeBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let target_mode = action.target_mode();

        let previous = window_control::current_snap_mode();
        let runtime_active = window_control::is_snap_runtime_active();

        let mode = match window_control::persist_snap_mode(target_mode) {
            Ok(mode) => mode,
            Err(error) => {
                tracing::error!(
                    category = "WINDOW",
                    trace_id = %dctx.trace_id,
                    %error,
                    ?target_mode,
                    "Failed to persist snap mode from built-in command"
                );
                self.show_hud(
                    format!("Failed to update snap mode: {error}"),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                return Self::builtin_error(
                    dctx,
                    "set_snap_mode_failed",
                    "Failed to save snap mode",
                    error.to_string(),
                );
            }
        };

        if runtime_active {
            let runtime_result = if mode == window_control::SnapMode::Off {
                window_control::cancel_snap_runtime(cx)
            } else {
                window_control::refresh_snap_runtime_for_mode(cx)
            };

            if let Err(error) = runtime_result {
                tracing::warn!(
                    category = "WINDOW",
                    trace_id = %dctx.trace_id,
                    %error,
                    ?mode,
                    "Failed to apply runtime transition after snap mode change"
                );
            }
        }

        tracing::info!(
            category = "WINDOW",
            trace_id = %dctx.trace_id,
            previous = ?previous,
            ?mode,
            runtime_active,
            "Updated snap mode from built-in command"
        );

        self.show_hud(action.hud_text().to_string(), Some(HUD_SHORT_MS), cx);
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_select_microphone_builtin(
        &mut self,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let prefs = crate::config::load_user_preferences();
        let menu_items = match crate::dictation::list_input_device_menu_items(
            prefs.dictation.selected_device_id.as_deref(),
        ) {
            Ok(items) => items,
            Err(error) => {
                tracing::error!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to enumerate microphone devices"
                );
                self.show_hud(
                    format!("Failed to list microphones: {error}"),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                return Self::builtin_error(
                    dctx,
                    "select_microphone_failed",
                    "Failed to list microphones",
                    error.to_string(),
                );
            }
        };

        let mut start_index: usize = 0;
        let choices: Vec<Choice> = menu_items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let value = match &item.action {
                    crate::dictation::DictationDeviceSelectionAction::UseSystemDefault => {
                        BUILTIN_MIC_DEFAULT_VALUE.to_string()
                    }
                    crate::dictation::DictationDeviceSelectionAction::UseDevice(id) => id.0.clone(),
                };
                let name = if item.is_selected {
                    if start_index == 0 && idx > 0 {
                        start_index = idx;
                    }
                    format!("{} (current)", item.title)
                } else {
                    item.title.clone()
                };
                Choice {
                    name,
                    value: value.clone(),
                    description: Some(item.subtitle.clone()),
                    key: None,
                    semantic_id: Some(builtin_choice_semantic_id(
                        BUILTIN_MIC_SELECT_PROMPT_ID,
                        idx,
                        &value,
                    )),
                }
            })
            .collect();

        // Follow the canonical ShowMini pattern from prompt_handler
        // (not open_builtin_filterable_view which targets MainFilter focus)
        let choice_count = choices.len();
        tracing::info!(
            category = "AUTOMATION",
            prompt_id = BUILTIN_MIC_SELECT_PROMPT_ID,
            choice_count = choice_count,
            selected_index = start_index,
            semantic_ids_populated = choices.iter().all(|c| c.semantic_id.is_some()),
            "opened_builtin_microphone_prompt"
        );
        self.opened_from_main_menu = true;
        self.arg_input.clear();
        self.arg_selected_index = start_index;
        self.focused_input = FocusedInput::ArgPrompt;
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Select microphone...".to_string());
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.current_view = AppView::MiniPrompt {
            id: BUILTIN_MIC_SELECT_PROMPT_ID.to_string(),
            placeholder: "Select microphone...".to_string(),
            choices,
        };
        resize_to_view_sync(ViewType::MiniPrompt, choice_count.min(5));
        cx.notify();

        Self::builtin_success(dctx, "select_microphone")
    }

    fn execute_script_command_builtin(
        &mut self,
        action: ScriptCommandBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        self.show_naming_dialog(action.naming_target(), cx);
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_frecency_command_builtin(
        &mut self,
        action: FrecencyCommandBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        self.frecency_store.clear();
        if let Err(e) = self.frecency_store.save() {
            let message = format!("Failed to clear suggested: {}", e);
            self.show_error_toast(message.clone(), cx);
            cx.notify();
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_ACTION_FAILED,
                message,
                action.failure_detail(),
            )
        } else {
            tracing::info!(
                trace_id = %dctx.trace_id,
                "Cleared all suggested items"
            );
            self.invalidate_grouped_cache();
            self.reset_to_script_list(cx);
            resize_to_view_sync(ViewType::ScriptList, 0);
            self.show_hud(action.hud_text().to_string(), Some(HUD_SHORT_MS), cx);
            cx.notify();
            Self::builtin_success(dctx, action.success_detail())
        }
    }

    fn execute_notes_command_builtin(
        &mut self,
        action: NotesCommandBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        // Close companion UI before hiding so it does not stay stale.
        if crate::confirm::is_confirm_window_open() {
            crate::confirm::route_key_to_confirm_popup("escape", cx);
        }
        if crate::actions::is_actions_window_open() {
            crate::actions::close_actions_window(cx);
            self.mark_actions_popup_closed();
            self.mark_filter_resync_after_actions_if_needed();
            self.pop_focus_overlay(cx);
        }

        self.pending_focus = None;
        script_kit_gpui::set_main_window_visible(false);
        platform::defer_hide_main_window(cx);

        let result = if action.opens_notes_window() {
            notes::open_notes_window_without_launcher_restore(cx)
        } else {
            notes::quick_capture(cx)
        };

        if let Err(e) = result {
            script_kit_gpui::set_main_window_visible(true);
            platform::show_main_window_without_activation();
            let message = format!("Notes command failed: {}", e);
            self.show_error_toast(message.clone(), cx);
            Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_LAUNCH_FAILED,
                message,
                action.failure_detail(),
            )
        } else {
            Self::builtin_success(dctx, action.success_detail())
        }
    }

    fn execute_kit_store_builtin(
        &mut self,
        action: KitStoreBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        self.opened_from_main_menu = true;

        match action {
            KitStoreBuiltinAction::BrowseKits => {
                self.current_view = AppView::BrowseKitsView {
                    query: String::new(),
                    selected_index: 0,
                    results: Vec::new(),
                };
                self.pending_focus = Some(FocusTarget::AppRoot);
                cx.notify();

                cx.spawn(async move |this, cx| {
                    let results = cx
                        .background_executor()
                        .spawn(async { Self::kit_store_search_results("") })
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        if let AppView::BrowseKitsView {
                            results: view_results,
                            ..
                        } = &mut this.current_view
                        {
                            *view_results = results;
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            KitStoreBuiltinAction::InstalledKits => {
                let kits = Self::kit_store_list_installed();
                tracing::info!(
                    trace_id = %dctx.trace_id,
                    installed_count = kits.len(),
                    "Loaded installed kits"
                );
                self.current_view = AppView::InstalledKitsView {
                    selected_index: 0,
                    kits,
                };
                self.pending_focus = Some(FocusTarget::AppRoot);
                cx.notify();
            }
            KitStoreBuiltinAction::UpdateAllKits => {
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async {
                            let kits = script_kit_gpui::kit_store::storage::list_installed_kits()
                                .unwrap_or_default();
                            let mut updated = 0usize;
                            let mut failed = 0usize;
                            for kit in &kits {
                                let pull_output = std::process::Command::new("git")
                                    .arg("-C")
                                    .arg(&kit.path)
                                    .arg("pull")
                                    .arg("--ff-only")
                                    .output();
                                match pull_output {
                                    Ok(output) if output.status.success() => {
                                        updated += 1;
                                    }
                                    _ => {
                                        failed += 1;
                                        tracing::warn!(
                                            kit_name = %kit.name,
                                            "Kit update-all failed for kit"
                                        );
                                    }
                                }
                            }
                            KitStoreUpdateAllResult { updated, failed }
                        })
                        .await;

                    let _ = this.update(cx, |this, cx| {
                        let message = result.message();
                        if result.is_failure() {
                            this.toast_manager.push(
                                components::toast::Toast::error(message, &this.theme)
                                    .duration_ms(Some(TOAST_ERROR_MS)),
                            );
                        } else {
                            this.show_hud(message, Some(HUD_MEDIUM_MS), cx);
                        }
                        cx.notify();
                    });
                })
                .detach();
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_utility_open_builtin(
        &mut self,
        action: UtilityOpenBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        if let Some(message) = action.opening_message() {
            tracing::info!(
                category = "BUILTIN",
                trace_id = %dctx.trace_id,
                "{}", message
            );
        }

        if action.opens_from_main_menu() {
            self.opened_from_main_menu = true;
        }

        match action {
            UtilityOpenBuiltinAction::MiniMainWindow => self.open_mini_main_window(cx),
            UtilityOpenBuiltinAction::ScratchPad => self.open_scratch_pad(cx),
            UtilityOpenBuiltinAction::QuickTerminal => self.open_quick_terminal(None, cx),
            UtilityOpenBuiltinAction::ClaudeCode => self.open_claude_code_terminal(cx),
            UtilityOpenBuiltinAction::ProcessManager => {
                let processes =
                    crate::process_manager::PROCESS_MANAGER.get_active_processes_sorted();
                tracing::info!(
                    trace_id = %dctx.trace_id,
                    active_process_count = processes.len(),
                    "process_manager.open_view"
                );

                self.cached_processes = processes;
                self.open_builtin_filterable_view(
                    AppView::ProcessManagerView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search running scripts...",
                    false,
                    cx,
                );
                self.start_process_manager_refresh(cx);
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_utility_command_builtin(
        &mut self,
        action: UtilityCommandBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            UtilityCommandBuiltinAction::Open(open_action) => {
                self.execute_utility_open_builtin(open_action, dctx, cx)
            }
            UtilityCommandBuiltinAction::Process(process_action) => {
                self.execute_utility_process_builtin(process_action, dctx, cx)
            }
            UtilityCommandBuiltinAction::Context(context_action) => {
                self.execute_utility_context_builtin(context_action, dctx, cx)
            }
            UtilityCommandBuiltinAction::Trace(trace_action) => {
                self.execute_utility_trace_builtin(trace_action, query_override, dctx, cx)
            }
            UtilityCommandBuiltinAction::Recipe(recipe_action) => match recipe_action {
                UtilityRecipeBuiltinAction::VerifyCurrentApp => {
                    self.execute_utility_verify_recipe_builtin(recipe_action, dctx, cx)
                }
                UtilityRecipeBuiltinAction::ReplayCurrentApp => {
                    self.execute_utility_replay_recipe_builtin(recipe_action, dctx, cx)
                }
                UtilityRecipeBuiltinAction::TurnThisIntoCommand => self
                    .execute_utility_turn_this_into_command_builtin(
                        recipe_action,
                        query_override,
                        dctx,
                        cx,
                    ),
            },
            UtilityCommandBuiltinAction::DoInCurrentApp(do_in_action) => self
                .execute_utility_do_in_current_app_builtin(do_in_action, query_override, dctx, cx),
            UtilityCommandBuiltinAction::CurrentAppCommands(current_app_action) => {
                self.execute_utility_current_app_commands_builtin(current_app_action, dctx, cx)
            }
        }
    }

    fn execute_utility_process_builtin(
        &mut self,
        action: UtilityProcessBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
        tracing::info!(
            trace_id = %dctx.trace_id,
            requested_count = process_count,
            "process_manager.stop_all"
        );

        if process_count == 0 {
            self.show_hud(action.empty_hud().to_string(), Some(HUD_2200_MS), cx);
        } else {
            crate::process_manager::PROCESS_MANAGER.kill_all_processes();
            self.show_hud(
                format!("Stopped {} running script process(es).", process_count),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            self.close_and_reset_window(cx);
        }
        Self::builtin_success(dctx, action.success_detail())
    }

    fn execute_utility_context_builtin(
        &mut self,
        action: UtilityContextBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            trace_id = %dctx.trace_id,
            "context_snapshot.inspect_requested"
        );

        let started_at = std::time::Instant::now();

        let snapshot = crate::context_snapshot::capture_context_snapshot(
            &crate::context_snapshot::CaptureContextOptions::default(),
        );

        match serde_json::to_string_pretty(&snapshot) {
            Ok(json) => {
                let receipt =
                    crate::context_snapshot::build_inspection_receipt(&snapshot, json.len());

                tracing::info!(
                    category = "CONTEXT",
                    event = "context_snapshot_copied",
                    trace_id = %dctx.trace_id,
                    schema_version = receipt.schema_version,
                    warning_count = receipt.warning_count,
                    has_selected_text = receipt.has_selected_text,
                    has_frontmost_app = receipt.has_frontmost_app,
                    top_level_menu_count = receipt.top_level_menu_count,
                    has_browser = receipt.has_browser,
                    has_focused_window = receipt.has_focused_window,
                    json_bytes = receipt.json_bytes,
                    status = %receipt.status,
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Copied current context snapshot to clipboard"
                );

                cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                let hud_message = crate::context_snapshot::build_inspection_hud_message(&receipt);
                self.show_hud(hud_message, Some(HUD_MEDIUM_MS), cx);
                self.close_and_reset_window(cx);

                Self::builtin_success(dctx, action.success_detail())
            }
            Err(e) => {
                let message = format!("Failed to serialize context snapshot: {}", e);
                tracing::error!(
                    trace_id = %dctx.trace_id,
                    error = %e,
                    "context_snapshot.serialize_failed"
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.failure_detail(),
                )
            }
        }
    }

    fn execute_utility_trace_builtin(
        &mut self,
        action: UtilityTraceBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let raw_query_owned = query_override.unwrap_or(&self.filter_text).to_string();
        let effective_query =
            crate::menu_bar::current_app_commands::normalize_trace_current_app_intent_request(
                Some(&raw_query_owned),
            )
            .unwrap_or_default();

        tracing::info!(
            trace_id = %dctx.trace_id,
            raw_query = %raw_query_owned,
            effective_query = %effective_query,
            "current_app_intent_trace.requested"
        );

        match crate::menu_bar::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let trace_receipt =
                    crate::menu_bar::current_app_commands::build_current_app_intent_trace_receipt(
                        snapshot,
                        Some(&raw_query_owned),
                    );

                match serde_json::to_string_pretty(&trace_receipt) {
                    Ok(json) => {
                        tracing::info!(
                            category = "CURRENT_APP_TRACE",
                            trace_id = %dctx.trace_id,
                            app_name = %trace_receipt.app_name,
                            bundle_id = %trace_receipt.bundle_id,
                            raw_query = %trace_receipt.raw_query,
                            effective_query = %trace_receipt.effective_query,
                            normalized_query = %trace_receipt.normalized_query,
                            action = %trace_receipt.action,
                            filtered_entries = trace_receipt.filtered_entries,
                            exact_matches = trace_receipt.exact_matches,
                            candidate_count = trace_receipt.candidates.len(),
                            has_prompt_preview = trace_receipt.prompt_preview.is_some(),
                            json_bytes = json.len(),
                            "current_app_intent_trace.copied"
                        );

                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                        self.show_hud(
                            format!(
                                "Copied app intent trace: {} ({} exact / {} filtered)",
                                trace_receipt.action,
                                trace_receipt.exact_matches,
                                trace_receipt.filtered_entries,
                            ),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                        self.close_and_reset_window(cx);
                        Self::builtin_success(dctx, action.success_detail())
                    }
                    Err(e) => {
                        let message =
                            format!("Failed to serialize current app intent trace: {}", e);
                        tracing::error!(
                            trace_id = %dctx.trace_id,
                            error = %e,
                            "current_app_intent_trace.serialize_failed"
                        );
                        self.show_error_toast(message.clone(), cx);
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                            action.serialize_failure_detail(),
                        )
                    }
                }
            }
            Err(e) => {
                let message = format!(
                    "Failed to inspect current app intent: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                    e
                );
                tracing::warn!(
                    trace_id = %dctx.trace_id,
                    error = %e,
                    "current_app_intent_trace.capture_failed"
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_utility_verify_recipe_builtin(
        &mut self,
        action: UtilityRecipeBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            trace_id = %dctx.trace_id,
            "verify_current_app_recipe.requested"
        );

        let stored_recipe =
            match crate::menu_bar::current_app_commands::load_current_app_command_recipe_from_clipboard() {
                Ok(recipe) => recipe,
                Err(error) => {
                    let message = format!("Verify Current App Recipe failed: {}", error);
                    self.show_error_toast(message.clone(), cx);
                    return Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        message,
                        action.clipboard_failure_detail(),
                    );
                }
            };

        match crate::menu_bar::current_app_commands::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let selected_text = crate::selected_text::get_selected_text()
                    .ok()
                    .filter(|text| !text.trim().is_empty());

                let browser_url = crate::platform::get_focused_browser_tab_url()
                    .ok()
                    .filter(|url| !url.trim().is_empty());

                let verification =
                    crate::menu_bar::current_app_commands::verify_current_app_command_recipe(
                        &stored_recipe,
                        snapshot,
                        selected_text.as_deref(),
                        browser_url.as_deref(),
                    );

                match serde_json::to_string_pretty(&verification) {
                    Ok(json) => {
                        tracing::info!(
                            category = "CURRENT_APP_RECIPE_VERIFY",
                            trace_id = %dctx.trace_id,
                            expected_bundle_id = %verification.expected_bundle_id,
                            actual_bundle_id = %verification.actual_bundle_id,
                            expected_route = %verification.expected_route,
                            actual_route = %verification.actual_route,
                            prompt_matches = verification.prompt_matches,
                            selected_text_expected = verification.selected_text_expected,
                            selected_text_present = verification.selected_text_present,
                            browser_url_expected = verification.browser_url_expected,
                            browser_url_present = verification.browser_url_present,
                            warning_count = verification.warning_count,
                            status = %verification.status,
                            json_bytes = json.len(),
                            "verify_current_app_recipe.completed"
                        );

                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));
                        self.show_hud(
                            crate::menu_bar::current_app_commands::build_current_app_command_verification_hud_message(
                                &verification,
                            ),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                        self.close_and_reset_window(cx);

                        Self::builtin_success(dctx, action.success_detail())
                    }
                    Err(error) => {
                        let message = format!(
                            "Failed to serialize current app recipe verification: {}",
                            error
                        );
                        self.show_error_toast(message.clone(), cx);
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                            action.serialize_failure_detail(),
                        )
                    }
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to verify current app recipe: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                    error
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_utility_replay_recipe_builtin(
        &mut self,
        action: UtilityRecipeBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            trace_id = %dctx.trace_id,
            "replay_current_app_recipe.requested"
        );

        let stored_recipe =
            match crate::menu_bar::current_app_commands::load_current_app_command_recipe_from_clipboard(
            ) {
                Ok(recipe) => recipe,
                Err(error) => {
                    let message = format!("Replay Current App Recipe failed: {}", error);
                    self.show_error_toast(message.clone(), cx);
                    return Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        message,
                        action.clipboard_failure_detail(),
                    );
                }
            };

        match crate::menu_bar::current_app_commands::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let snapshot_pid = snapshot.pid;
                let (entries, snapshot_receipt) = snapshot.clone().into_entries_with_receipt();

                let selected_text = crate::selected_text::get_selected_text()
                    .ok()
                    .filter(|text| !text.trim().is_empty());

                let browser_url = crate::platform::get_focused_browser_tab_url()
                    .ok()
                    .filter(|url| !url.trim().is_empty());

                let replay_receipt =
                    crate::menu_bar::current_app_commands::build_replay_current_app_recipe_receipt(
                        &stored_recipe,
                        &entries,
                        snapshot,
                        selected_text.as_deref(),
                        browser_url.as_deref(),
                    );

                tracing::info!(
                    category = "CURRENT_APP_RECIPE_REPLAY",
                    trace_id = %dctx.trace_id,
                    action = %replay_receipt.action,
                    status = %replay_receipt.verification.status,
                    warning_count = replay_receipt.verification.warning_count,
                    expected_bundle_id = %replay_receipt.verification.expected_bundle_id,
                    actual_bundle_id = %replay_receipt.verification.actual_bundle_id,
                    expected_route = %replay_receipt.verification.expected_route,
                    actual_route = %replay_receipt.verification.actual_route,
                    selected_entry_index = replay_receipt.selected_entry_index,
                    "replay_current_app_recipe.resolved"
                );

                if replay_receipt.verification.warning_count > 0 {
                    let json = match serde_json::to_string_pretty(&replay_receipt) {
                        Ok(json) => json,
                        Err(error) => {
                            let message = format!(
                                "Failed to serialize replay current app recipe receipt: {}",
                                error
                            );
                            self.show_error_toast(message.clone(), cx);
                            return Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                message,
                                action.serialize_failure_detail(),
                            );
                        }
                    };

                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));

                    let message =
                        crate::menu_bar::current_app_commands::build_replay_current_app_recipe_hud_message(
                            &replay_receipt,
                        );

                    self.show_error_toast(
                        format!("{}. Copied replay report to clipboard.", message),
                        cx,
                    );

                    return Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        message,
                        action.drift_failure_detail(),
                    );
                }

                match replay_receipt.action.as_str() {
                    "execute_entry" => {
                        let Some(entry_index) = replay_receipt.selected_entry_index else {
                            let message =
                                "Replay Current App Recipe resolved to execute_entry without an entry index"
                                    .to_string();
                            self.show_error_toast(message.clone(), cx);
                            return Self::builtin_error(
                                dctx,
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                message,
                                action.missing_entry_failure_detail(),
                            );
                        };

                        let entry = entries[entry_index].clone();
                        self.execute_builtin_inner(
                            &entry,
                            Some(
                                replay_receipt
                                    .verification
                                    .live_recipe
                                    .effective_query
                                    .as_str(),
                            ),
                            dctx,
                            cx,
                        )
                    }
                    "open_command_palette" => {
                        let filter = replay_receipt
                            .verification
                            .live_recipe
                            .effective_query
                            .clone();
                        self.present_current_app_commands_entries(
                            entries,
                            &snapshot_receipt,
                            snapshot_pid,
                            &filter,
                            cx,
                        );

                        Self::builtin_success(dctx, action.open_palette_success_detail())
                    }
                    "generate_script" => {
                        self.spawn_generate_script_from_recipe_after_hide(
                            dctx.trace_id.to_string(),
                            replay_receipt.verification.live_recipe.clone(),
                            cx,
                        );
                        Self::builtin_success(dctx, action.generate_script_success_detail())
                    }
                    other => {
                        let message = format!(
                            "Replay Current App Recipe resolved to unsupported action: {}",
                            other
                        );
                        self.show_error_toast(message.clone(), cx);
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                            action.unknown_action_failure_detail(),
                        )
                    }
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to replay current app recipe: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                    error
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_utility_turn_this_into_command_builtin(
        &mut self,
        action: UtilityRecipeBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let raw_query_owned = query_override.unwrap_or(&self.filter_text).to_string();

        let effective_query =
            crate::menu_bar::current_app_commands::normalize_turn_this_into_a_command_request(
                Some(&raw_query_owned),
            )
            .unwrap_or_default();

        if effective_query.is_empty() {
            let message =
                "Type what you want to automate after \"Turn This Into a Command\"".to_string();
            self.show_error_toast(message.clone(), cx);
            return Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_ACTION_FAILED,
                message,
                action.missing_query_failure_detail(),
            );
        }

        tracing::info!(
            trace_id = %dctx.trace_id,
            raw_query = %raw_query_owned,
            effective_query = %effective_query,
            "turn_this_into_command.requested"
        );

        match crate::menu_bar::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let selected_text = crate::selected_text::get_selected_text()
                    .ok()
                    .filter(|text| !text.trim().is_empty());

                let browser_url = crate::platform::get_focused_browser_tab_url()
                    .ok()
                    .filter(|url| !url.trim().is_empty());

                let recipe =
                    crate::menu_bar::current_app_commands::build_current_app_command_recipe(
                        snapshot,
                        Some(&raw_query_owned),
                        selected_text.as_deref(),
                        browser_url.as_deref(),
                    );

                match serde_json::to_string_pretty(&recipe) {
                    Ok(json) => {
                        tracing::info!(
                            category = "CURRENT_APP_RECIPE",
                            trace_id = %dctx.trace_id,
                            app_name = %recipe.prompt_receipt.app_name,
                            bundle_id = %recipe.prompt_receipt.bundle_id,
                            effective_query = %recipe.effective_query,
                            route = %recipe.trace.action,
                            suggested_script_name = %recipe.suggested_script_name,
                            candidate_count = recipe.trace.candidates.len(),
                            included_selected_text = recipe.prompt_receipt.included_selected_text,
                            included_browser_url = recipe.prompt_receipt.included_browser_url,
                            json_bytes = json.len(),
                            "turn_this_into_command.recipe_copied"
                        );

                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(json));

                        self.show_hud(
                            format!("Automation recipe copied: {}", recipe.suggested_script_name,),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );

                        self.spawn_generate_script_from_recipe_after_hide(
                            dctx.trace_id.to_string(),
                            recipe.clone(),
                            cx,
                        );

                        Self::builtin_success(dctx, action.success_detail())
                    }
                    Err(e) => {
                        let message =
                            format!("Failed to serialize current app command recipe: {}", e);
                        tracing::error!(
                            trace_id = %dctx.trace_id,
                            error = %e,
                            "turn_this_into_command.serialize_failed"
                        );
                        self.show_error_toast(message.clone(), cx);
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            message,
                            action.serialize_failure_detail(),
                        )
                    }
                }
            }
            Err(e) => {
                let message = format!(
                    "Failed to capture current app command recipe: {}. Check Accessibility permission in System Settings → Privacy & Security → Accessibility, then refocus the target app and try again.",
                    e
                );
                tracing::warn!(
                    trace_id = %dctx.trace_id,
                    error = %e,
                    "turn_this_into_command.capture_failed"
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_utility_do_in_current_app_builtin(
        &mut self,
        action: UtilityDoInCurrentAppBuiltinAction,
        query_override: Option<&str>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        let raw_query_owned = query_override.unwrap_or(&self.filter_text).to_string();
        let raw_query_safe = crate::logging::log_user_value(&raw_query_owned);
        let filter_text_safe = crate::logging::log_user_value(&self.filter_text);
        tracing::info!(
            target: "script_kit::do_in_trace",
            event = "DO_IN_TRACE execution.entry",
            trace_id = %dctx.trace_id,
            raw_query_preview = %raw_query_safe,
            raw_query_bytes = raw_query_safe.raw_bytes,
            raw_query_safe_bytes = raw_query_safe.safe_bytes,
            raw_query_truncated = raw_query_safe.truncated,
            filter_text_preview = %filter_text_safe,
            filter_text_bytes = filter_text_safe.raw_bytes,
            filter_text_safe_bytes = filter_text_safe.safe_bytes,
            filter_text_truncated = filter_text_safe.truncated,
            query_override = ?query_override,
            current_view = ?self.current_view,
            "DO_IN_TRACE execution.entry"
        );
        tracing::info!(
            trace_id = %dctx.trace_id,
            raw_query = %raw_query_owned,
            filter_text = %self.filter_text,
            query_override = ?query_override,
            "do_in_current_app.execution_entry — raw inputs"
        );
        let effective_query =
            crate::menu_bar::current_app_commands::effective_do_in_current_app_query_for_submission(
                &raw_query_owned,
                query_override,
            );
        let effective_query_for_router =
            (!effective_query.is_empty()).then_some(effective_query.as_str());
        let effective_query_safe = crate::logging::log_user_value(&effective_query);

        tracing::info!(
            target: "script_kit::do_in_trace",
            event = "DO_IN_TRACE execution.normalized",
            trace_id = %dctx.trace_id,
            query_preview = %effective_query_safe,
            query_bytes = effective_query_safe.raw_bytes,
            query_safe_bytes = effective_query_safe.safe_bytes,
            query_truncated = effective_query_safe.truncated,
            raw_query_preview = %raw_query_safe,
            "DO_IN_TRACE execution.normalized"
        );
        tracing::info!(
            trace_id = %dctx.trace_id,
            query = %effective_query,
            "do_in_current_app.requested"
        );

        match crate::menu_bar::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let snapshot_for_recipe = snapshot.clone();
                let snapshot_pid = snapshot.pid;
                let (entries, snapshot_receipt) = snapshot.into_entries_with_receipt();

                let (resolved_action, intent_receipt) =
                    crate::menu_bar::current_app_commands::resolve_do_in_current_app_intent(
                        &entries,
                        effective_query_for_router,
                    );

                tracing::info!(
                    target: "script_kit::do_in_trace",
                    event = "DO_IN_TRACE execution.resolved",
                    trace_id = %dctx.trace_id,
                    app_name = %snapshot_receipt.app_name,
                    bundle_id = %snapshot_receipt.bundle_id,
                    leaf_entry_count = snapshot_receipt.leaf_entry_count,
                    query_preview = %effective_query_safe,
                    raw_query_preview = %raw_query_safe,
                    filtered_entries = intent_receipt.filtered_entries,
                    exact_matches = intent_receipt.exact_matches,
                    resolved_action = intent_receipt.action,
                    "DO_IN_TRACE execution.resolved"
                );
                tracing::info!(
                    trace_id = %dctx.trace_id,
                    app_name = %snapshot_receipt.app_name,
                    bundle_id = %snapshot_receipt.bundle_id,
                    leaf_entry_count = snapshot_receipt.leaf_entry_count,
                    query = %effective_query,
                    filtered_entries = intent_receipt.filtered_entries,
                    exact_matches = intent_receipt.exact_matches,
                    resolved_action = intent_receipt.action,
                    "do_in_current_app.resolved"
                );

                match resolved_action {
                    crate::menu_bar::current_app_commands::DoInCurrentAppAction::OpenCommandPalette => {
                        tracing::info!(
                            target: "script_kit::do_in_trace",
                            event = "DO_IN_TRACE execution.open_palette",
                            trace_id = %dctx.trace_id,
                            cached_entries = entries.len(),
                            filter_preview = %effective_query_safe,
                            placeholder = %snapshot_receipt.placeholder,
                            "DO_IN_TRACE execution.open_palette"
                        );
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            cached_entries = entries.len(),
                            filter = %effective_query,
                            placeholder = %snapshot_receipt.placeholder,
                            "do_in_current_app.action → OpenCommandPalette — switching to CurrentAppCommandsView"
                        );
                        self.present_current_app_commands_entries(
                            entries,
                            &snapshot_receipt,
                            snapshot_pid,
                            &effective_query,
                            cx,
                        );
                        Self::builtin_success(dctx, action.open_palette_success_detail())
                    }
                    crate::menu_bar::current_app_commands::DoInCurrentAppAction::ExecuteEntry(
                        entry_index,
                    ) => {
                        tracing::info!(
                            target: "script_kit::do_in_trace",
                            event = "DO_IN_TRACE execution.execute_entry",
                            trace_id = %dctx.trace_id,
                            entry_index = entry_index,
                            entry_name = %entries[entry_index].name,
                            query_preview = %effective_query_safe,
                            raw_query_preview = %raw_query_safe,
                            "DO_IN_TRACE execution.execute_entry"
                        );
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            entry_index = entry_index,
                            entry_name = %entries[entry_index].name,
                            "do_in_current_app.action → ExecuteEntry — running menu command directly"
                        );
                        let entry = entries[entry_index].clone();
                        self.execute_builtin_inner(&entry, effective_query_for_router, dctx, cx)
                    }
                    crate::menu_bar::current_app_commands::DoInCurrentAppAction::GenerateScript => {
                        tracing::info!(
                            target: "script_kit::do_in_trace",
                            event = "DO_IN_TRACE execution.generate_script",
                            trace_id = %dctx.trace_id,
                            query_preview = %effective_query_safe,
                            raw_query_preview = %raw_query_safe,
                            "DO_IN_TRACE execution.generate_script"
                        );
                        tracing::info!(
                            trace_id = %dctx.trace_id,
                            query = %effective_query,
                            "do_in_current_app.action → GenerateScript — scheduling async context capture before recipe flow"
                        );

                        self.spawn_generate_script_from_current_app_with_capture(
                            dctx.trace_id.to_string(),
                            effective_query.clone(),
                            snapshot_for_recipe,
                            entries,
                            snapshot_receipt.clone(),
                            snapshot_pid,
                            cx,
                        );

                        Self::builtin_success(dctx, action.generate_script_success_detail())
                    }
                }
            }
            Err(e) => {
                let message = format!("Failed to load frontmost app menu bar: {}", e);
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_utility_current_app_commands_builtin(
        &mut self,
        action: UtilityCurrentAppCommandsBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            trace_id = %dctx.trace_id,
            "current_app_commands.open_requested"
        );
        match crate::menu_bar::load_frontmost_menu_snapshot() {
            Ok(snapshot) => {
                let pid = snapshot.pid;
                let (entries, receipt) = snapshot.into_entries_with_receipt();

                tracing::info!(
                    trace_id = %dctx.trace_id,
                    pid,
                    app_name = %receipt.app_name,
                    bundle_id = %receipt.bundle_id,
                    top_level_menu_count = receipt.top_level_menu_count,
                    leaf_entry_count = receipt.leaf_entry_count,
                    placeholder = %receipt.placeholder,
                    source = receipt.source,
                    "current_app_commands.snapshot_ready"
                );

                self.present_current_app_commands_entries(entries, &receipt, pid, "", cx);
                Self::builtin_success(dctx, action.success_detail())
            }
            Err(e) => {
                let message = format!("Failed to load frontmost app menu bar: {}", e);
                tracing::warn!(
                    trace_id = %dctx.trace_id,
                    error = %e,
                    "current_app_commands.capture_failed"
                );
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    message,
                    action.capture_failure_detail(),
                )
            }
        }
    }

    fn execute_permission_command_builtin(
        &mut self,
        action: PermissionCommandBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match action {
            PermissionCommandBuiltinAction::CheckPermissions => {
                let status = permissions_wizard::check_all_permissions();
                if status.all_granted() {
                    self.show_hud(
                        "All permissions granted!".to_string(),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                } else {
                    let missing: Vec<_> = status
                        .missing_permissions()
                        .iter()
                        .map(|p| p.permission_type.name())
                        .collect();
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            format!("Missing permissions: {}", missing.join(", ")),
                            &self.theme,
                        )
                        .duration_ms(Some(TOAST_WARNING_MS)),
                    );
                }
                cx.notify();
                Self::builtin_success(dctx, action.success_detail())
            }
            PermissionCommandBuiltinAction::RequestAccessibility => {
                let granted = permissions_wizard::request_accessibility_permission();
                if granted {
                    self.show_hud(
                        "Accessibility permission granted!".to_string(),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                } else {
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "Accessibility permission not granted. Some features may not work.",
                            &self.theme,
                        )
                        .duration_ms(Some(TOAST_WARNING_MS)),
                    );
                }
                cx.notify();
                Self::builtin_success(dctx, action.success_detail())
            }
            PermissionCommandBuiltinAction::OpenAccessibilitySettings => {
                if let Err(e) = permissions_wizard::open_accessibility_settings() {
                    let message = format!("Failed to open settings: {}", e);
                    self.show_error_toast(message.clone(), cx);
                    Self::builtin_error(
                        dctx,
                        crate::action_helpers::ERROR_LAUNCH_FAILED,
                        message,
                        action.failure_detail(),
                    )
                } else {
                    self.close_and_reset_window(cx);
                    Self::builtin_success(dctx, action.success_detail())
                }
            }
            PermissionCommandBuiltinAction::Assistant(assistant_action) => {
                self.execute_permission_assistant_builtin(assistant_action, dctx, cx)
            }
        }
    }

    fn execute_permission_assistant_builtin(
        &mut self,
        action: PermissionAssistantBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        match platform::permiso::PermisoAssistant::present_retained(action.panel()) {
            Ok(()) => {
                self.show_hud(action.success_hud().to_string(), Some(HUD_SHORT_MS), cx);
                Self::builtin_success(dctx, action.success_detail())
            }
            Err(error) => {
                let message = format!("Failed to open Permission Assistant: {}", error);
                self.show_error_toast(message.clone(), cx);
                Self::builtin_error(
                    dctx,
                    crate::action_helpers::ERROR_LAUNCH_FAILED,
                    message,
                    action.failure_detail(),
                )
            }
        }
    }

    fn execute_paste_sequential_builtin(
        &mut self,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            action = "paste_sequential",
            event = "trigger",
            trace_id = %dctx.trace_id,
            "Paste Sequentially triggered"
        );

        let paste_action = PasteSequentialBuiltinAction::from_outcome(
            clipboard_history::advance_paste_sequence(&mut self.paste_sequential_state),
        );

        match &paste_action {
            PasteSequentialBuiltinAction::PasteEntry(entry_id) => {
                tracing::info!(
                    action = "paste_sequential",
                    event = paste_action.telemetry_event(),
                    entry_id = %entry_id,
                    trace_id = %dctx.trace_id,
                    "{}", paste_action.log_message()
                );
                match clipboard_history::enqueue_sequential_paste(entry_id.clone()) {
                    Ok(()) => {
                        clipboard_history::commit_paste_sequence(&mut self.paste_sequential_state);
                        self.hide_main_and_reset(cx);
                        Self::builtin_success(dctx, paste_action.success_detail())
                    }
                    Err(clipboard_history::EnqueuePasteError::WorkerDisconnected) => {
                        tracing::error!(
                            action = "paste_sequential",
                            event = "enqueue_failed",
                            error_code = "worker_disconnected",
                            trace_id = %dctx.trace_id,
                            "Paste worker is not running"
                        );
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                "Paste worker crashed — restart Script Kit",
                                &self.theme,
                            )
                            .duration_ms(Some(TOAST_CRITICAL_MS)),
                        );
                        cx.notify();
                        Self::builtin_error(
                            dctx,
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Paste worker crashed",
                            "paste_sequential_worker_disconnected",
                        )
                    }
                }
            }
            PasteSequentialBuiltinAction::SequenceExhausted
            | PasteSequentialBuiltinAction::HistoryEmpty => {
                tracing::info!(
                    action = "paste_sequential",
                    event = paste_action.telemetry_event(),
                    trace_id = %dctx.trace_id,
                    "{}", paste_action.log_message()
                );
                if let Some(message) = paste_action.hud_message() {
                    self.show_hud(message.to_string(), Some(HUD_SHORT_MS), cx);
                }
                Self::builtin_success(dctx, paste_action.success_detail())
            }
        }
    }

    fn execute_dictation_builtin_action(
        &mut self,
        action: DictationBuiltinAction,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        tracing::info!(
            category = "BUILTIN",
            trace_id = %dctx.trace_id,
            "{}", action.opening_message()
        );

        let is_start_edge = !crate::dictation::is_dictation_recording();
        if is_start_edge {
            let preflight = self.prepare_dictation_builtin_start(action, cx);
            if preflight != DictationStartPreflight::Ready {
                return Self::builtin_success(dctx, preflight.success_detail());
            }
        }

        let dictation_target = if is_start_edge {
            self.dictation_start_target(action)
        } else {
            crate::dictation::get_dictation_target()
                .unwrap_or_else(|| action.stop_fallback_target())
        };

        match crate::dictation::toggle_dictation(dictation_target) {
            Ok(crate::dictation::DictationToggleOutcome::Started) => {
                self.handle_dictation_started(action, dictation_target, cx);
            }
            Ok(crate::dictation::DictationToggleOutcome::Stopped(Some(capture))) => {
                self.begin_dictation_transcription(capture, dictation_target, cx);
            }
            Ok(crate::dictation::DictationToggleOutcome::Stopped(None)) => {
                let _ = crate::dictation::close_dictation_overlay(cx);
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
            Err(error) => {
                tracing::error!(
                    category = "DICTATION",
                    error = %error,
                    failure_message = action.failure_message(),
                    "Dictation toggle failed"
                );
                let _ = crate::dictation::update_dictation_overlay(
                    crate::dictation::DictationOverlayState {
                        phase: crate::dictation::DictationSessionPhase::Failed(error.to_string()),
                        ..Default::default()
                    },
                    cx,
                );
                self.schedule_dictation_overlay_close(cx, std::time::Duration::from_millis(800));
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
        }

        Self::builtin_success(dctx, action.success_detail())
    }

    fn prepare_dictation_builtin_start(
        &mut self,
        action: DictationBuiltinAction,
        cx: &mut Context<Self>,
    ) -> DictationStartPreflight {
        if self.open_dictation_setup_if_microphone_not_ready(cx) {
            return DictationStartPreflight::OpenedSetup;
        }

        if !crate::dictation::is_parakeet_model_available() {
            if PARAKEET_MODEL_DOWNLOAD_IN_PROGRESS.load(std::sync::atomic::Ordering::Acquire) {
                self.open_dictation_model_prompt(cx);
                return DictationStartPreflight::DownloadInProgress;
            }
            tracing::info!(
                category = "DICTATION",
                "Parakeet model not downloaded, opening consent prompt"
            );
            self.open_dictation_model_prompt(cx);
            return DictationStartPreflight::OpenedModelPrompt;
        }

        if let Err(error) = self.ensure_dictation_builtin_target_available(action) {
            let error_text = error.to_string();
            tracing::error!(
                category = "DICTATION",
                error = %error_text,
                action = ?action,
                "{}", action.preflight_failure_message()
            );
            self.show_error_toast(format!("Dictation unavailable: {error_text}"), cx);
            return DictationStartPreflight::Failed;
        }

        DictationStartPreflight::Ready
    }

    fn ensure_dictation_builtin_target_available(
        &self,
        action: DictationBuiltinAction,
    ) -> anyhow::Result<()> {
        if let Some(target) = action.forced_target() {
            self.ensure_dictation_delivery_target_available_for(target)
        } else {
            self.ensure_dictation_delivery_target_available()
        }
    }

    fn dictation_start_target(
        &self,
        action: DictationBuiltinAction,
    ) -> crate::dictation::DictationTarget {
        action
            .forced_target()
            .unwrap_or_else(|| self.resolve_dictation_target())
    }

    fn handle_dictation_started(
        &mut self,
        action: DictationBuiltinAction,
        dictation_target: crate::dictation::DictationTarget,
        cx: &mut Context<Self>,
    ) {
        let _ = crate::dictation::set_dictation_target_cycle(
            self.dictation_target_cycle_for(dictation_target),
        );

        if action.log_forced_route() {
            tracing::info!(
                category = "DICTATION",
                ?dictation_target,
                target_label = dictation_target.overlay_label(),
                "Starting forced-route dictation"
            );
        }

        if action.conceal_before_overlay() {
            platform::conceal_main_window();
        }

        let orch_target =
            crate::window_orchestrator::executor::to_orchestrator_target(&dictation_target);
        if action.dispatch_start_before_overlay() {
            self.dispatch_window_event(
                crate::window_orchestrator::WindowEvent::StartDictation {
                    target: orch_target,
                },
                cx,
            );
            self.start_dictation_overlay_session(cx);
        } else {
            self.start_dictation_overlay_session(cx);
            self.dispatch_window_event(
                crate::window_orchestrator::WindowEvent::StartDictation {
                    target: orch_target,
                },
                cx,
            );
        }
    }

    /// Open the overlay, register the abort callback, start the pump.
    ///
    /// Shared by both `BuiltInFeature::Dictation` and
    /// `BuiltInFeature::DictationToAiHarness` so confirm/resume fixes
    /// only need one change.
    fn start_dictation_overlay_session(&mut self, cx: &mut Context<Self>) {
        let _ = crate::dictation::begin_overlay_session();
        let app_entity = cx.entity().downgrade();
        crate::dictation::set_overlay_abort_callback(|cx| {
            if let Err(error) = crate::dictation::abort_dictation() {
                tracing::error!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to abort dictation from overlay"
                );
            }
            let _ = crate::dictation::close_dictation_overlay(cx);
        });
        crate::dictation::set_overlay_submit_callback(move |cx| {
            if let Some(app) = app_entity.upgrade() {
                app.update(cx, |this, cx| {
                    this.submit_active_dictation_from_overlay(cx);
                });
            }
        });
        let _ = crate::dictation::open_dictation_overlay(cx);
        let _ = crate::dictation::update_dictation_overlay(
            crate::dictation::DictationOverlayState {
                phase: crate::dictation::DictationSessionPhase::Recording,
                ..Default::default()
            },
            cx,
        );
        self.spawn_dictation_overlay_pump(cx);
    }

    /// Stop the active recording from the overlay Stop action and continue
    /// through the same transcription/delivery path as the dictation hotkey.
    fn submit_active_dictation_from_overlay(&mut self, cx: &mut Context<Self>) {
        let dictation_target = crate::dictation::get_dictation_target()
            .unwrap_or_else(|| self.resolve_dictation_target());

        match crate::dictation::toggle_dictation(dictation_target) {
            Ok(crate::dictation::DictationToggleOutcome::Stopped(Some(capture))) => {
                let _ = crate::dictation::begin_overlay_session();
                let _ = crate::dictation::open_dictation_overlay(cx);
                self.begin_dictation_transcription(capture, dictation_target, cx);
            }
            Ok(crate::dictation::DictationToggleOutcome::Stopped(None)) => {
                let _ = crate::dictation::close_dictation_overlay(cx);
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
            Ok(crate::dictation::DictationToggleOutcome::Started) => {
                tracing::warn!(
                    category = "DICTATION",
                    ?dictation_target,
                    "Overlay submit started dictation unexpectedly"
                );
                self.start_dictation_overlay_session(cx);
            }
            Err(error) => {
                tracing::error!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to stop dictation from overlay"
                );
                let _ = crate::dictation::open_dictation_overlay(cx);
                let _ = crate::dictation::update_dictation_overlay(
                    crate::dictation::DictationOverlayState {
                        phase: crate::dictation::DictationSessionPhase::Failed(error.to_string()),
                        ..Default::default()
                    },
                    cx,
                );
                self.schedule_dictation_overlay_close(cx, std::time::Duration::from_millis(800));
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
        }
    }

    /// Transition a completed capture into the transcribing overlay state
    /// and kick off async transcription.
    ///
    /// Shared by both dictation entry points so the handoff cannot drift.
    fn begin_dictation_transcription(
        &mut self,
        capture: crate::dictation::CompletedDictationCapture,
        target: crate::dictation::DictationTarget,
        cx: &mut Context<Self>,
    ) {
        let _ = crate::dictation::update_dictation_overlay(
            crate::dictation::DictationOverlayState {
                phase: crate::dictation::DictationSessionPhase::Transcribing,
                elapsed: capture.audio_duration,
                ..Default::default()
            },
            cx,
        );
        let audio_duration = capture.audio_duration;
        let chunks = capture.chunks;
        cx.spawn(async move |this, cx| {
            let transcript_result = cx
                .background_executor()
                .spawn(async move { crate::dictation::transcribe_captured_audio(&chunks) })
                .await;
            let _ = this.update(cx, |this, cx| {
                Self::handle_dictation_transcript(
                    this,
                    transcript_result,
                    audio_duration,
                    target,
                    cx,
                );
            });
        })
        .detach();
    }

    /// Periodically snapshot the live capture session and push state to the
    /// dictation overlay.  Runs every 16 ms (~60 fps) for smooth waveform
    /// animation and stops automatically when the session ends.
    fn spawn_dictation_overlay_pump(&mut self, cx: &mut Context<Self>) {
        let gen = crate::dictation::overlay_generation();
        cx.spawn(async move |_this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(16))
                    .await;
                // Bail if a newer overlay session has started.
                if crate::dictation::overlay_generation() != gen {
                    tracing::debug!(
                        category = "DICTATION",
                        "Overlay pump detected generation change, stopping"
                    );
                    break;
                }
                let Some(state) = crate::dictation::snapshot_overlay_state() else {
                    break;
                };
                cx.update(|cx| {
                    let _ = crate::dictation::update_dictation_overlay(state, cx);
                });
            }
        })
        .detach();
    }

    /// Handle the result of background transcription: deliver the transcript
    /// to the target surface that was active when dictation started, update
    /// the overlay, and schedule cleanup timers.
    fn handle_dictation_transcript(
        &mut self,
        result: anyhow::Result<Option<String>>,
        audio_duration: std::time::Duration,
        target: crate::dictation::DictationTarget,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(Some(transcript)) => {
                let history_entry =
                    crate::dictation::record_dictation_history(&transcript, audio_duration, target);
                tracing::info!(
                    category = "DICTATION",
                    event = "dictation_history_recorded_before_delivery",
                    entry_id = %history_entry.id,
                    target = %history_entry.target,
                );
                let history_entry_id = history_entry.id.clone();
                // Route delivery based on the target that was captured at
                // session start, not the current UI state.
                let mut delivery_insertion_range: Option<serde_json::Value> = None;
                let delivered_internally = match target {
                    crate::dictation::DictationTarget::MainWindowFilter => {
                        if !self.can_accept_dictation_into_main_filter() {
                            self.reset_to_script_list(cx);
                        }
                        self.try_set_main_window_filter_from_dictation(transcript.clone(), cx)
                    }
                    crate::dictation::DictationTarget::MainWindowPrompt => {
                        self.try_set_prompt_input(transcript.clone(), cx)
                    }
                    crate::dictation::DictationTarget::NotesEditor => {
                        match notes::inject_text_into_notes(&mut **cx, &transcript) {
                            Ok(insertion_range) => {
                                delivery_insertion_range = Some(insertion_range);
                                true
                            }
                            Err(error) => {
                                tracing::warn!(
                                    category = "DICTATION",
                                    error = %error,
                                    "Notes delivery failed, falling back to frontmost app"
                                );
                                false
                            }
                        }
                    }
                    crate::dictation::DictationTarget::AiChatComposer => {
                        match ai::set_ai_input(&mut **cx, &transcript, false) {
                            Ok(()) => true,
                            Err(error) => {
                                tracing::warn!(
                                    category = "DICTATION",
                                    error = %error,
                                    "AI chat delivery failed, falling back to frontmost app"
                                );
                                false
                            }
                        }
                    }
                    crate::dictation::DictationTarget::TabAiHarness => {
                        self.seed_acp_dictation_return_origin();
                        if crate::ai::acp::chat_window::is_chat_window_open() {
                            tracing::info!(
                                category = "DICTATION",
                                event = "dictation_acp_detached_closed_for_embedded_reveal",
                                "Closing detached ACP before ACP-targeted dictation reveal"
                            );
                            crate::ai::acp::chat_window::close_chat_window(&mut **cx);
                        }
                        self.open_tab_ai_acp_with_entry_intent_suppressing_focused_part(
                            Some(transcript.clone()),
                            cx,
                        );
                        // Let the orchestrator reveal the main window as ACP
                        // chat and focus the composer after the view is
                        // seeded with the dictated prompt.
                        self.dispatch_window_event(
                            crate::window_orchestrator::WindowEvent::FinishDictation,
                            cx,
                        );
                        true
                    }
                    crate::dictation::DictationTarget::ExternalApp => false,
                };

                if delivered_internally {
                    let destination = match target {
                        crate::dictation::DictationTarget::MainWindowFilter => {
                            crate::dictation::DictationDestination::MainWindowFilter
                        }
                        crate::dictation::DictationTarget::MainWindowPrompt => {
                            crate::dictation::DictationDestination::ActivePrompt
                        }
                        crate::dictation::DictationTarget::NotesEditor => {
                            crate::dictation::DictationDestination::NotesEditor
                        }
                        crate::dictation::DictationTarget::AiChatComposer => {
                            crate::dictation::DictationDestination::AiChatComposer
                        }
                        crate::dictation::DictationTarget::TabAiHarness => {
                            crate::dictation::DictationDestination::TabAiHarness
                        }
                        crate::dictation::DictationTarget::ExternalApp => {
                            crate::dictation::DictationDestination::FrontmostApp
                        }
                    };
                    let insertion_range = match destination {
                        crate::dictation::DictationDestination::MainWindowFilter
                        | crate::dictation::DictationDestination::ActivePrompt
                        | crate::dictation::DictationDestination::AiChatComposer
                        | crate::dictation::DictationDestination::TabAiHarness => {
                            Some(serde_json::json!({
                                "available": true,
                                "unit": "utf8Bytes",
                                "start": 0,
                                "end": transcript.len(),
                                "insertedLength": transcript.len(),
                                "operation": "replaceInput",
                                "source": "deliveryPipeline",
                                "redacted": true,
                            }))
                        }
                        crate::dictation::DictationDestination::NotesEditor => {
                            delivery_insertion_range
                        }
                        crate::dictation::DictationDestination::FrontmostApp => None,
                    };
                    tracing::info!(
                        category = "DICTATION",
                        ?target,
                        ?destination,
                        transcript_len = transcript.len(),
                        "Internal dictation delivery complete"
                    );
                    let _ = crate::dictation::record_delivery_receipt(
                        &transcript,
                        audio_duration,
                        target,
                        destination,
                        true,
                        &history_entry_id,
                        insertion_range,
                    );

                    let _ = crate::dictation::close_dictation_overlay(cx);
                    if matches!(target, crate::dictation::DictationTarget::MainWindowFilter)
                        && !script_kit_gpui::is_main_window_visible()
                    {
                        script_kit_gpui::set_main_window_visible(true);
                        crate::platform::ensure_main_panel_configured(
                            "builtin_execution::dictation_main_filter_delivery",
                        );
                        crate::platform::show_main_window_without_activation();
                    }
                    self.schedule_dictation_transcriber_cleanup(
                        cx,
                        std::time::Duration::from_secs(300),
                    );
                    // Notify orchestrator that dictation is complete.
                    // TabAiHarness dispatches this earlier (before overlay
                    // scheduling) to trigger immediate RevealMain; other
                    // targets dispatch here for state bookkeeping.
                    if !matches!(target, crate::dictation::DictationTarget::TabAiHarness) {
                        self.dispatch_window_event(
                            crate::window_orchestrator::WindowEvent::FinishDictation,
                            cx,
                        );
                    }
                } else {
                    // Guard: verify that a tracked external app target exists
                    // before attempting to paste to the frontmost app.
                    if let Err(error) = Self::ensure_dictation_frontmost_target_available() {
                        let error_text = error.to_string();
                        tracing::error!(
                            category = "DICTATION",
                            error = %error_text,
                            "Failed to resolve frontmost-app dictation target"
                        );
                        self.show_error_toast(format!("Dictation paste failed: {error_text}"), cx);
                        self.schedule_dictation_overlay_close(
                            cx,
                            std::time::Duration::from_millis(150),
                        );
                        self.schedule_dictation_transcriber_cleanup(
                            cx,
                            std::time::Duration::from_secs(300),
                        );
                        self.dispatch_window_event(
                            crate::window_orchestrator::WindowEvent::AbortDictation,
                            cx,
                        );
                        return;
                    }

                    let Some(target_bundle_id) =
                        crate::frontmost_app_tracker::get_last_real_app_bundle_id()
                    else {
                        tracing::error!(
                            category = "DICTATION",
                            "Frontmost-app dictation target disappeared before paste"
                        );
                        self.show_error_toast(
                            "Dictation paste failed: no tracked frontmost app is available"
                                .to_string(),
                            cx,
                        );
                        self.schedule_dictation_overlay_close(
                            cx,
                            std::time::Duration::from_millis(150),
                        );
                        self.schedule_dictation_transcriber_cleanup(
                            cx,
                            std::time::Duration::from_secs(300),
                        );
                        self.dispatch_window_event(
                            crate::window_orchestrator::WindowEvent::AbortDictation,
                            cx,
                        );
                        return;
                    };
                    tracing::info!(
                        category = "DICTATION",
                        target_bundle_id = %target_bundle_id,
                        transcript_len = transcript.len(),
                        "Preparing frontmost-app dictation paste"
                    );

                    cx.spawn(async move |this, cx| {
                        // Close overlay, hide Script Kit, and explicitly
                        // activate the tracked target app so macOS moves
                        // keyboard focus there before the CGEvent paste.
                        let yield_focus_result = match this.update(
                            cx,
                            |this, cx| this.yield_focus_for_dictation_paste(&target_bundle_id, cx),
                        ) {
                            Ok(result) => result,
                            Err(error) => Err(anyhow::anyhow!(
                                "failed to update app state before paste: {error}"
                            )),
                        };

                        if let Err(error) = yield_focus_result {
                            let error_text = error.to_string();
                            if this.update(cx, |this, cx| {
                                tracing::error!(
                                    category = "DICTATION",
                                    error = %error_text,
                                    "Failed to yield focus before dictation paste"
                                );
                                this.show_error_toast(
                                    format!(
                                        "Dictation paste failed before paste step: {error_text}"
                                    ),
                                    cx,
                                );
                                this.schedule_dictation_transcriber_cleanup(
                                    cx,
                                    std::time::Duration::from_secs(300),
                                );
                            }).is_err() {
                                tracing::warn!(
                                    category = "DICTATION",
                                    error = %error_text,
                                    "Yield-focus failure could not be surfaced (entity released)"
                                );
                            }
                            return;
                        }

                        // Let macOS settle focus back to the target app.
                        cx.background_executor()
                            .timer(Self::dictation_focus_settle_duration())
                            .await;

                        tracing::info!(
                            category = "DICTATION",
                            target_bundle_id = %target_bundle_id,
                            transcript_len = transcript.len(),
                            "Focus yielded to target app; pasting transcript"
                        );

                        let paste_result = cx
                            .background_executor()
                            .spawn({
                                let transcript = transcript.clone();
                                async move {
                                    crate::text_injector::TextInjector::new()
                                        .paste_text(&transcript)
                                }
                            })
                            .await;

                        if this.update(cx, |this, cx| {
                            match paste_result {
                                Ok(()) => {
                                    let _ = crate::dictation::record_delivery_receipt(
                                        &transcript,
                                        audio_duration,
                                        target,
                                        crate::dictation::DictationDestination::FrontmostApp,
                                        false,
                                        &history_entry_id,
                                        None,
                                    );
                                    tracing::info!(
                                        category = "DICTATION",
                                        destination = ?crate::dictation::DictationDestination::FrontmostApp,
                                        transcript_len = transcript.len(),
                                        "Transcript delivered"
                                    );
                                }
                                Err(ref error) => {
                                    tracing::error!(
                                        category = "DICTATION",
                                        error = %error,
                                        "Failed to paste dictation transcript"
                                    );
                                    this.show_error_toast(
                                        format!("Dictation paste failed: {error}"),
                                        cx,
                                    );
                                }
                            }
                            this.schedule_dictation_transcriber_cleanup(
                                cx,
                                std::time::Duration::from_secs(300),
                            );
                        }).is_err() {
                            tracing::warn!(
                                category = "DICTATION",
                                transcript_len = transcript.len(),
                                "Paste result could not be reported (entity released)"
                            );
                        }
                    })
                    .detach();
                }
            }
            Ok(None) => {
                // No speech detected — close overlay quietly and treat the
                // session as an abort. For ACP dictation, a successful finish
                // means "reveal ACP with a transcript"; without transcript
                // there is nothing to seed or submit.
                tracing::info!(
                    category = "DICTATION",
                    ?target,
                    "No dictation transcript recognized; aborting delivery"
                );
                self.schedule_dictation_overlay_close(cx, std::time::Duration::from_millis(150));
                self.schedule_dictation_transcriber_cleanup(
                    cx,
                    std::time::Duration::from_secs(300),
                );
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
            Err(error) => {
                let error_text = error.to_string();
                let model_path = crate::dictation::resolve_default_model_path();
                tracing::error!(
                    category = "DICTATION",
                    error = %error_text,
                    model_path = %model_path.display(),
                    "Transcription failed"
                );

                if error_text.contains("Parakeet model not downloaded") {
                    let _ = crate::dictation::close_dictation_overlay(cx);
                    self.dispatch_window_event(
                        crate::window_orchestrator::WindowEvent::AbortDictation,
                        cx,
                    );
                    self.open_dictation_model_prompt(cx);
                    self.schedule_dictation_transcriber_cleanup(
                        cx,
                        std::time::Duration::from_secs(300),
                    );
                    return;
                } else {
                    self.show_error_toast(
                        format!("Dictation transcription failed: {error_text}"),
                        cx,
                    );
                }

                let _ = crate::dictation::update_dictation_overlay(
                    crate::dictation::DictationOverlayState {
                        phase: crate::dictation::DictationSessionPhase::Failed(error_text),
                        elapsed: audio_duration,
                        ..Default::default()
                    },
                    cx,
                );
                self.schedule_dictation_overlay_close(cx, std::time::Duration::from_millis(800));
                self.schedule_dictation_transcriber_cleanup(
                    cx,
                    std::time::Duration::from_secs(300),
                );
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::AbortDictation,
                    cx,
                );
            }
        }
    }

    pub(crate) fn deliver_stdin_dictation_result(
        &mut self,
        transcript: String,
        target_label: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<crate::dictation::DictationTarget, String> {
        let target = Self::dictation_target_from_stdin_label(target_label)
            .or_else(crate::dictation::get_dictation_target)
            .unwrap_or_else(|| self.resolve_dictation_target());

        if crate::dictation::is_dictation_recording() {
            crate::dictation::abort_dictation()
                .map_err(|error| format!("failed to stop active dictation capture: {error}"))?;
        }

        let result = if transcript.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(transcript))
        };
        self.handle_dictation_transcript(result, std::time::Duration::ZERO, target, cx);
        Ok(target)
    }

    fn dictation_target_from_stdin_label(
        target_label: Option<&str>,
    ) -> Option<crate::dictation::DictationTarget> {
        let normalized = target_label?
            .trim()
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect::<String>();

        match normalized.as_str() {
            "mainwindowfilter" | "scriptkit" | "launcher" | "filter" => {
                Some(crate::dictation::DictationTarget::MainWindowFilter)
            }
            "mainwindowprompt" | "prompt" => {
                Some(crate::dictation::DictationTarget::MainWindowPrompt)
            }
            "noteseditor" | "notes" => Some(crate::dictation::DictationTarget::NotesEditor),
            "aichatcomposer" | "aichat" | "legacyai" => {
                Some(crate::dictation::DictationTarget::AiChatComposer)
            }
            "tabaiharness" | "acp" | "acpchat" | "ai" => {
                Some(crate::dictation::DictationTarget::TabAiHarness)
            }
            "externalapp" | "frontmostapp" | "frontmost" | "app" => {
                Some(crate::dictation::DictationTarget::ExternalApp)
            }
            _ => None,
        }
    }

    const DICTATION_FOCUS_SETTLE_MS: u64 = 120;

    fn dictation_focus_settle_duration() -> std::time::Duration {
        std::time::Duration::from_millis(Self::DICTATION_FOCUS_SETTLE_MS)
    }

    /// Start downloading the Parakeet model in the background, showing
    /// progress via in-prompt updates and HUD fallback.
    fn start_parakeet_model_download(&mut self, cx: &mut Context<Self>) {
        if PARAKEET_MODEL_DOWNLOAD_IN_PROGRESS
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_err()
        {
            self.show_hud(
                "Dictation model download already in progress".to_string(),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }

        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        *parakeet_model_download_cancel_slot().lock() = Some(cancel.clone());
        // Shallow channel — cosmetic updates use try_send so the download
        // thread is never blocked on UI repaints.
        let (progress_tx, progress_rx) = async_channel::bounded::<DictationModelProgressEvent>(4);
        let ui_emitter =
            std::sync::Arc::new(parking_lot::Mutex::new(DictationModelUiEmitter::default()));

        // Spawn a concurrent reader that updates the in-prompt progress
        // display as events arrive.  HUD only shows when the rich prompt
        // is not visible — they no longer compete.
        cx.spawn({
            let progress_rx = progress_rx.clone();
            async move |this, cx| {
                while let Ok(event) = progress_rx.recv().await {
                    let _ = this.update(cx, |this, cx| match event {
                        DictationModelProgressEvent::Downloading {
                            percentage,
                            downloaded_bytes,
                            total_bytes,
                            speed_bytes_per_sec,
                            eta_seconds,
                        } => {
                            let prompt_visible = this.is_dictation_model_prompt_visible();
                            this.update_dictation_model_prompt_if_visible(
                                crate::dictation::DictationModelStatus::Downloading {
                                    percentage,
                                    downloaded_bytes,
                                    total_bytes,
                                    speed_bytes_per_sec,
                                    eta_seconds,
                                },
                                cx,
                            );
                            if !prompt_visible {
                                let summary = crate::dictation::download::format_progress_summary(
                                    percentage,
                                    downloaded_bytes,
                                    total_bytes,
                                    speed_bytes_per_sec,
                                    eta_seconds,
                                );
                                this.show_hud(
                                    format!("Downloading model\u{2026} {summary}"),
                                    Some(HUD_SHORT_MS),
                                    cx,
                                );
                            }
                        }
                        DictationModelProgressEvent::Extracting => {
                            let prompt_visible = this.is_dictation_model_prompt_visible();
                            this.update_dictation_model_prompt_if_visible(
                                crate::dictation::DictationModelStatus::Extracting,
                                cx,
                            );
                            if !prompt_visible {
                                this.show_hud(
                                    "Extracting dictation model\u{2026}".to_string(),
                                    Some(HUD_SHORT_MS),
                                    cx,
                                );
                            }
                        }
                    });
                }
            }
        })
        .detach();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn({
                    let cancel = cancel.clone();
                    async move {
                        let speed_tracker =
                            std::sync::Arc::new(parking_lot::Mutex::new(SpeedTracker::new()));
                        let ui_emitter = ui_emitter.clone();
                        crate::dictation::download::download_parakeet_model(
                            {
                                let speed_tracker = speed_tracker.clone();
                                let ui_emitter = ui_emitter.clone();
                                let progress_tx = progress_tx;
                                move |phase, progress| {
                                    match phase {
                                        crate::dictation::download::DownloadPhase::Downloading => {
                                            let pct = progress.percentage();
                                            let speed = {
                                                let mut tracker = speed_tracker.lock();
                                                tracker.update(progress.downloaded);
                                                tracker.speed_bytes_per_sec()
                                            };
                                            let eta =
                                                crate::dictation::download::estimate_eta_seconds(
                                                    progress, speed,
                                                );

                                            let snapshot =
                                                DictationModelUiSnapshot::downloading(pct, eta);
                                            let now = std::time::Instant::now();
                                            let should_emit = {
                                                let emitter = ui_emitter.lock();
                                                emitter.should_emit(now, &snapshot)
                                            };

                                            if should_emit {
                                                let sent = progress_tx
                                                    .try_send(
                                                        DictationModelProgressEvent::Downloading {
                                                            percentage: pct,
                                                            downloaded_bytes: progress.downloaded,
                                                            total_bytes: progress.total,
                                                            speed_bytes_per_sec: speed,
                                                            eta_seconds: eta,
                                                        },
                                                    )
                                                    .is_ok();
                                                if sent {
                                                    tracing::info!(
                                                        category = "DICTATION",
                                                        pct,
                                                        downloaded = progress.downloaded,
                                                        total = progress.total,
                                                        speed,
                                                        "Model download progress"
                                                    );
                                                    let mut emitter = ui_emitter.lock();
                                                    emitter.record_emit(now, &snapshot);
                                                }
                                            }
                                        }
                                        crate::dictation::download::DownloadPhase::Extracting => {
                                            tracing::info!(
                                                category = "DICTATION",
                                                "Extracting dictation model"
                                            );
                                            let snapshot = DictationModelUiSnapshot::extracting();
                                            let now = std::time::Instant::now();
                                            // Extracting is critical — use blocking send
                                            // so it always reaches the UI.
                                            if progress_tx
                                                .send_blocking(
                                                    DictationModelProgressEvent::Extracting,
                                                )
                                                .is_ok()
                                            {
                                                let mut emitter = ui_emitter.lock();
                                                emitter.record_emit(now, &snapshot);
                                            }
                                        }
                                        crate::dictation::download::DownloadPhase::Failed(_)
                                        | crate::dictation::download::DownloadPhase::Cancelled
                                        | crate::dictation::download::DownloadPhase::Complete => {}
                                    }
                                }
                            },
                            cancel,
                        )
                    }
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                *parakeet_model_download_cancel_slot().lock() = None;
                PARAKEET_MODEL_DOWNLOAD_IN_PROGRESS
                    .store(false, std::sync::atomic::Ordering::Release);
                match result {
                    Ok(_path) => {
                        tracing::info!(category = "DICTATION", "Parakeet model download complete");
                        this.update_dictation_model_prompt_if_visible(
                            crate::dictation::DictationModelStatus::Available,
                            cx,
                        );
                        this.show_hud(
                            "Dictation model ready \u{2014} press hotkey to dictate".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    Err(error) if error.to_string().contains("cancelled") => {
                        let cancelled = "model download cancelled".to_string();
                        tracing::info!(category = "DICTATION", "Parakeet model download cancelled");
                        this.update_dictation_model_prompt_if_visible(
                            crate::dictation::DictationModelStatus::DownloadFailed(cancelled),
                            cx,
                        );
                        this.show_hud(
                            "Dictation model download cancelled".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    Err(error) => {
                        let raw_error = error.to_string();
                        let error_text =
                            crate::dictation::download::classify_download_error(&error);
                        tracing::error!(
                            category = "DICTATION",
                            error = %raw_error,
                            user_error = %error_text,
                            "Parakeet model download failed"
                        );
                        this.update_dictation_model_prompt_if_visible(
                            crate::dictation::DictationModelStatus::DownloadFailed(
                                error_text.clone(),
                            ),
                            cx,
                        );
                        this.show_error_toast(
                            format!("Dictation model download failed: {error_text}"),
                            cx,
                        );
                    }
                }
            });
        })
        .detach();
    }

    /// Build the title, placeholder, and choices for the dictation model
    /// prompt based on the current `DictationModelStatus`.  Pure function
    /// with no side effects — suitable for unit testing.
    fn build_dictation_model_prompt(
        status: crate::dictation::DictationModelStatus,
    ) -> (String, String, Vec<Choice>) {
        use crate::dictation::DictationModelStatus;

        let archive_size =
            crate::dictation::download::format_bytes(crate::dictation::PARAKEET_MODEL_ARCHIVE_SIZE);

        match status {
            DictationModelStatus::NotDownloaded => (
                "Download local dictation model".to_string(),
                format!(
                    "{archive_size} download \u{00b7} transcription is local after Parakeet installs \u{00b7} resumable if interrupted"
                ),
                vec![
                    Choice {
                        name: format!("Download Parakeet model ({archive_size})"),
                        value: BUILTIN_DICTATION_MODEL_DOWNLOAD.to_string(),
                        description: Some("Required for local dictation".to_string()),
                        key: None,
                        semantic_id: Some(builtin_choice_semantic_id(
                            BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_DOWNLOAD,
                        )),
                    },
                    Choice {
                        name: "Not now".to_string(),
                        value: BUILTIN_DICTATION_MODEL_CANCEL.to_string(),
                        description: Some("Leave dictation unchanged".to_string()),
                        key: None,
                        semantic_id: Some(builtin_choice_semantic_id(
                            BUILTIN_DICTATION_MODEL_PROMPT_ID, 1, BUILTIN_DICTATION_MODEL_CANCEL,
                        )),
                    },
                ],
            ),
            DictationModelStatus::Downloading {
                percentage,
                downloaded_bytes,
                total_bytes,
                speed_bytes_per_sec,
                eta_seconds,
            } => {
                let summary = crate::dictation::download::format_progress_summary(
                    percentage,
                    downloaded_bytes,
                    total_bytes,
                    speed_bytes_per_sec,
                    eta_seconds,
                );
                (
                    format!("Downloading local dictation model\u{2026} {percentage}%"),
                    summary,
                    vec![
                        Choice {
                            name: "Cancel download".to_string(),
                            value: BUILTIN_DICTATION_MODEL_CANCEL.to_string(),
                            description: Some(
                                "Stop now; retry resumes from the partial file".to_string(),
                            ),
                            key: None,
                            semantic_id: Some(builtin_choice_semantic_id(
                                BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_CANCEL,
                            )),
                        },
                        Choice {
                            name: "Hide".to_string(),
                            value: BUILTIN_DICTATION_MODEL_HIDE.to_string(),
                            description: Some("Download continues in background".to_string()),
                            key: None,
                            semantic_id: Some(builtin_choice_semantic_id(
                                BUILTIN_DICTATION_MODEL_PROMPT_ID, 1, BUILTIN_DICTATION_MODEL_HIDE,
                            )),
                        },
                    ],
                )
            }
            DictationModelStatus::Extracting => (
                "Installing local dictation model\u{2026}".to_string(),
                "Download finished. Installing model files locally.".to_string(),
                vec![Choice {
                    name: "Hide".to_string(),
                    value: BUILTIN_DICTATION_MODEL_HIDE.to_string(),
                    description: Some("Extraction continues in background".to_string()),
                    key: None,
                    semantic_id: Some(builtin_choice_semantic_id(
                        BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_HIDE,
                    )),
                }],
            ),
            DictationModelStatus::DownloadFailed(ref error)
                if error.to_ascii_lowercase().contains("cancelled") =>
            {
                (
                    "Download cancelled".to_string(),
                    "Partial download kept. Retry resumes from where you stopped.".to_string(),
                    vec![
                        Choice {
                            name: "Retry download".to_string(),
                            value: BUILTIN_DICTATION_MODEL_DOWNLOAD.to_string(),
                            description: Some(
                                "Resume the Parakeet model download".to_string(),
                            ),
                            key: None,
                            semantic_id: Some(builtin_choice_semantic_id(
                                BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_DOWNLOAD,
                            )),
                        },
                        Choice {
                            name: "Done".to_string(),
                            value: BUILTIN_DICTATION_MODEL_HIDE.to_string(),
                            description: Some("Close this prompt".to_string()),
                            key: None,
                            semantic_id: Some(builtin_choice_semantic_id(
                                BUILTIN_DICTATION_MODEL_PROMPT_ID, 1, BUILTIN_DICTATION_MODEL_HIDE,
                            )),
                        },
                    ],
                )
            }
            DictationModelStatus::DownloadFailed(error) => (
                "Dictation model download failed".to_string(),
                error,
                vec![
                    Choice {
                        name: "Retry download".to_string(),
                        value: BUILTIN_DICTATION_MODEL_DOWNLOAD.to_string(),
                        description: Some(
                            "Try the Parakeet model download again".to_string(),
                        ),
                        key: None,
                        semantic_id: Some(builtin_choice_semantic_id(
                            BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_DOWNLOAD,
                        )),
                    },
                    Choice {
                        name: "Not now".to_string(),
                        value: BUILTIN_DICTATION_MODEL_CANCEL.to_string(),
                        description: Some("Leave dictation unchanged".to_string()),
                        key: None,
                        semantic_id: Some(builtin_choice_semantic_id(
                            BUILTIN_DICTATION_MODEL_PROMPT_ID, 1, BUILTIN_DICTATION_MODEL_CANCEL,
                        )),
                    },
                ],
            ),
            DictationModelStatus::Available => (
                "Dictation model ready".to_string(),
                "Start dictation from the launcher or configured hotkey; no default is assumed."
                    .to_string(),
                vec![Choice {
                    name: "Done".to_string(),
                    value: BUILTIN_DICTATION_MODEL_HIDE.to_string(),
                    description: Some("Close this prompt".to_string()),
                    key: None,
                    semantic_id: Some(builtin_choice_semantic_id(
                        BUILTIN_DICTATION_MODEL_PROMPT_ID, 0, BUILTIN_DICTATION_MODEL_HIDE,
                    )),
                }],
            ),
        }
    }

    fn preferred_dictation_model_prompt_index(
        status: &crate::dictation::DictationModelStatus,
    ) -> usize {
        match status {
            crate::dictation::DictationModelStatus::Downloading { .. } => 1,
            _ => 0,
        }
    }

    /// Render the dictation model prompt with the given status, replacing
    /// whatever is currently on screen.
    fn render_dictation_model_prompt(
        &mut self,
        status: crate::dictation::DictationModelStatus,
        cx: &mut Context<Self>,
    ) {
        let previous_status = dictation_model_prompt_status().lock().clone();
        let phase_changed =
            std::mem::discriminant(&previous_status) != std::mem::discriminant(&status);
        let prompt_visible = self.is_dictation_model_prompt_visible();
        *dictation_model_prompt_status().lock() = status.clone();
        let (title, placeholder, choices) = Self::build_dictation_model_prompt(status.clone());
        if !prompt_visible || phase_changed {
            self.arg_selected_index = Self::preferred_dictation_model_prompt_index(&status)
                .min(choices.len().saturating_sub(1));
        }
        self.open_builtin_filterable_view(
            AppView::MiniPrompt {
                id: BUILTIN_DICTATION_MODEL_PROMPT_ID.to_string(),
                placeholder,
                choices,
            },
            &title,
            false,
            cx,
        );
    }

    /// Returns `true` when the dictation model prompt is currently on-screen.
    fn is_dictation_model_prompt_visible(&self) -> bool {
        matches!(
            &self.current_view,
            AppView::MiniPrompt { id, .. } if id == BUILTIN_DICTATION_MODEL_PROMPT_ID
        )
    }

    /// If the dictation model prompt is currently visible, update it in-place
    /// with the new status.  Otherwise this is a no-op.
    fn update_dictation_model_prompt_if_visible(
        &mut self,
        status: crate::dictation::DictationModelStatus,
        cx: &mut Context<Self>,
    ) {
        // Always persist the latest status so reopening a hidden prompt
        // shows the current state instead of stale progress.
        *dictation_model_prompt_status().lock() = status.clone();

        let is_visible = matches!(
            &self.current_view,
            AppView::MiniPrompt { id, .. } if id == BUILTIN_DICTATION_MODEL_PROMPT_ID
        );
        if is_visible {
            self.render_dictation_model_prompt(status, cx);
        }
    }

    /// Open the dictation model prompt in its initial `NotDownloaded` state.
    fn open_dictation_model_prompt(&mut self, cx: &mut Context<Self>) {
        let status = if crate::dictation::is_parakeet_model_available() {
            crate::dictation::DictationModelStatus::Available
        } else {
            dictation_model_prompt_status().lock().clone()
        };
        self.render_dictation_model_prompt(status, cx);
    }

    /// Handle a selection from the dictation model download prompt.
    fn handle_dictation_model_selection(&mut self, value: &str, cx: &mut Context<Self>) {
        match value {
            BUILTIN_DICTATION_MODEL_DOWNLOAD => {
                tracing::info!(
                    category = "DICTATION",
                    "User accepted Parakeet model download"
                );
                // Transition the prompt to downloading state instead of closing it.
                self.render_dictation_model_prompt(
                    crate::dictation::DictationModelStatus::Downloading {
                        percentage: 0,
                        downloaded_bytes: 0,
                        total_bytes: crate::dictation::PARAKEET_MODEL_ARCHIVE_SIZE,
                        speed_bytes_per_sec: 0,
                        eta_seconds: None,
                    },
                    cx,
                );
                self.start_parakeet_model_download(cx);
            }
            BUILTIN_DICTATION_MODEL_CANCEL => {
                if PARAKEET_MODEL_DOWNLOAD_IN_PROGRESS.load(std::sync::atomic::Ordering::Acquire) {
                    tracing::info!(
                        category = "DICTATION",
                        "User requested Parakeet model download cancellation"
                    );
                    if let Some(cancel) = parakeet_model_download_cancel_slot().lock().clone() {
                        cancel.store(true, std::sync::atomic::Ordering::Release);
                    }
                    self.show_hud(
                        "Cancelling dictation model download\u{2026}".to_string(),
                        Some(HUD_SHORT_MS),
                        cx,
                    );
                } else {
                    tracing::info!(
                        category = "DICTATION",
                        "User declined Parakeet model download"
                    );
                    self.reset_to_script_list(cx);
                }
            }
            BUILTIN_DICTATION_MODEL_HIDE => {
                tracing::info!(category = "DICTATION", "User hid Parakeet model prompt");
                self.reset_to_script_list(cx);
            }
            _ => {
                tracing::info!(
                    category = "DICTATION",
                    "User declined Parakeet model download"
                );
                self.reset_to_script_list(cx);
            }
        }
    }

    /// Ensure a new dictation session has somewhere valid to send text.
    ///
    /// Allowed start conditions:
    /// - the launcher/main filter is active, or
    /// - a Script Kit prompt is active and can accept dictated text, or
    /// - the frontmost-app tracker already has a previously tracked external target.
    fn open_dictation_setup_if_microphone_not_ready(&mut self, cx: &mut Context<Self>) -> bool {
        let permission = crate::dictation::microphone_permission_status();
        if matches!(
            permission,
            crate::dictation::DictationMicrophonePermissionStatus::Granted
                | crate::dictation::DictationMicrophonePermissionStatus::Unknown
        ) {
            return false;
        }

        self.show_hud(
            "Dictation needs microphone permission".to_string(),
            Some(HUD_SHORT_MS),
            cx,
        );
        self.open_dictation_model_prompt(cx);
        true
    }

    fn ensure_dictation_delivery_target_available(&self) -> anyhow::Result<()> {
        if self.can_accept_dictation_into_main_filter() || self.can_accept_dictation_into_prompt() {
            return Ok(());
        }
        Self::ensure_dictation_frontmost_target_available()
    }

    /// Verify that the frontmost-app tracker has a previously-tracked
    /// external app target before attempting a dictation paste.
    fn ensure_dictation_frontmost_target_available() -> anyhow::Result<()> {
        use anyhow::Context as _;
        crate::frontmost_app_tracker::get_last_real_app_bundle_id()
            .context("no previously tracked frontmost app is available for dictation paste")?;
        Ok(())
    }

    /// Validate that a specific delivery target is reachable before
    /// starting dictation.  Script Kit internal targets (harness, notes,
    /// AI composer, prompt) are always available.  `ExternalApp` requires
    /// the frontmost-app tracker to have a previously-tracked target.
    fn ensure_dictation_delivery_target_available_for(
        &self,
        target: crate::dictation::DictationTarget,
    ) -> anyhow::Result<()> {
        match target {
            crate::dictation::DictationTarget::ExternalApp => {
                Self::ensure_dictation_frontmost_target_available()
            }
            crate::dictation::DictationTarget::MainWindowFilter
            | crate::dictation::DictationTarget::MainWindowPrompt
            | crate::dictation::DictationTarget::NotesEditor
            | crate::dictation::DictationTarget::AiChatComposer
            | crate::dictation::DictationTarget::TabAiHarness => Ok(()),
        }
    }

    /// Build the small per-session destination cycle exposed by the overlay
    /// badge. Keep it intentionally tight: the primary target plus an
    /// ACP chat-submit fallback, except external-app sessions which expose
    /// the main launcher filter as the alternate target.
    fn dictation_target_cycle_for(
        &self,
        target: crate::dictation::DictationTarget,
    ) -> Vec<crate::dictation::DictationTarget> {
        match target {
            crate::dictation::DictationTarget::ExternalApp => vec![
                crate::dictation::DictationTarget::ExternalApp,
                crate::dictation::DictationTarget::MainWindowFilter,
            ],
            crate::dictation::DictationTarget::MainWindowFilter => vec![
                crate::dictation::DictationTarget::MainWindowFilter,
                crate::dictation::DictationTarget::TabAiHarness,
            ],
            crate::dictation::DictationTarget::MainWindowPrompt => vec![
                crate::dictation::DictationTarget::MainWindowPrompt,
                crate::dictation::DictationTarget::TabAiHarness,
            ],
            crate::dictation::DictationTarget::NotesEditor => vec![
                crate::dictation::DictationTarget::NotesEditor,
                crate::dictation::DictationTarget::ExternalApp,
            ],
            crate::dictation::DictationTarget::AiChatComposer => vec![
                crate::dictation::DictationTarget::AiChatComposer,
                crate::dictation::DictationTarget::ExternalApp,
            ],
            crate::dictation::DictationTarget::TabAiHarness => vec![
                crate::dictation::DictationTarget::TabAiHarness,
                crate::dictation::DictationTarget::ExternalApp,
            ],
        }
    }

    /// Resolve the dictation delivery target, optionally overriding to
    /// `TabAiHarness` so a dedicated "dictate to Agent Chat" action can
    /// force harness delivery even when the harness is not already on-screen.
    pub(crate) fn resolve_dictation_target_with_override(
        &self,
        force_tab_ai_harness: bool,
    ) -> crate::dictation::DictationTarget {
        if force_tab_ai_harness {
            crate::dictation::DictationTarget::TabAiHarness
        } else {
            self.resolve_dictation_target()
        }
    }

    /// Determine the delivery target for a new dictation session based on
    /// which Script Kit surface is currently active.
    ///
    /// Priority: notes editor > AI chat composer > launcher main filter >
    /// active prompt > external app.
    fn resolve_dictation_target(&self) -> crate::dictation::DictationTarget {
        let target = if matches!(self.current_view, AppView::QuickTerminalView { .. }) {
            crate::dictation::DictationTarget::TabAiHarness
        } else if notes::is_notes_window_open() {
            crate::dictation::DictationTarget::NotesEditor
        } else if ai::is_ai_window_open() {
            crate::dictation::DictationTarget::AiChatComposer
        } else if self.can_accept_dictation_into_main_filter() {
            crate::dictation::DictationTarget::MainWindowFilter
        } else if self.can_accept_dictation_into_prompt() {
            crate::dictation::DictationTarget::MainWindowPrompt
        } else {
            crate::dictation::DictationTarget::ExternalApp
        };
        tracing::info!(
            category = "DICTATION",
            ?target,
            current_view = ?std::mem::discriminant(&self.current_view),
            notes_open = notes::is_notes_window_open(),
            ai_open = ai::is_ai_window_open(),
            accepts_main_filter = self.can_accept_dictation_into_main_filter(),
            accepts_prompt = self.can_accept_dictation_into_prompt(),
            "Resolved dictation target"
        );
        target
    }

    /// Close the dictation overlay and hide Script Kit so macOS naturally
    /// returns keyboard focus to the previously-active window before the
    /// CGEvent Cmd+V paste fires.
    ///
    /// Script Kit is a non-activating accessory app (NSPanel with
    /// NonactivatingPanel style), so when our panels close via `orderOut:`,
    /// macOS automatically restores focus to the window that was active
    /// before — no explicit `activate` call is needed.  Avoiding AppleScript
    /// `tell application id … to activate` is important because that can
    /// reorder windows within multi-window apps like Chrome, causing the
    /// paste to land in the wrong window.
    fn yield_focus_for_dictation_paste(
        &mut self,
        target_bundle_id: &str,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        use anyhow::Context as _;
        tracing::info!(
            category = "DICTATION",
            target_bundle_id = %target_bundle_id,
            "Yielding focus for dictation paste (non-activating panel dismiss)"
        );
        crate::dictation::close_dictation_overlay(cx)
            .context("failed to close dictation overlay before paste")?;
        if script_kit_gpui::is_main_window_visible() {
            script_kit_gpui::set_main_window_visible(false);
            platform::defer_hide_main_window(cx);
        }
        Ok(())
    }

    /// Schedule the overlay window to close after a delay.
    fn schedule_dictation_overlay_close(
        &mut self,
        cx: &mut Context<Self>,
        delay: std::time::Duration,
    ) {
        let gen = crate::dictation::overlay_generation();
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            // Only close if the overlay hasn't been replaced by a newer session.
            if crate::dictation::overlay_generation() != gen {
                tracing::debug!(
                    category = "DICTATION",
                    "Scheduled overlay close skipped — generation changed"
                );
                return;
            }
            cx.update(|cx| {
                let _ = crate::dictation::close_dictation_overlay(cx);
            });
        })
        .detach();
    }

    /// Bring the main window back after a delay.
    ///
    /// Used by the dictation-to-AI path: the main window is concealed so
    /// the overlay is visible, and once the overlay closes we reveal the
    /// main window with the newly-opened ACP chat view.  The delay must
    /// be slightly longer than `schedule_dictation_overlay_close` so the
    /// overlay is gone before the main window reappears.
    fn schedule_deferred_main_window_reveal(
        &mut self,
        cx: &mut Context<Self>,
        delay: std::time::Duration,
    ) {
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|_cx| {
                platform::show_main_window_without_activation();
            });
        })
        .detach();
    }

    /// Schedule the cached transcriber to be unloaded after an idle timeout.
    fn schedule_dictation_transcriber_cleanup(
        &mut self,
        cx: &mut Context<Self>,
        delay: std::time::Duration,
    ) {
        cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|_cx| {
                crate::dictation::maybe_unload_transcriber();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod builtin_execution_ai_feedback_tests {
    use super::{
        ai_capture_hide_settle_duration, ai_command_keeps_main_window_visible,
        ai_command_uses_hide_then_capture_flow, ai_open_failure_message,
        created_file_path_for_feedback, emoji_picker_label, favorites_loaded_message,
        AiCommandWindowPlan, AI_CAPTURE_HIDE_SETTLE_MS,
    };
    use crate::builtins::AiCommandType;
    use script_kit_gpui::emoji::{Emoji, EmojiCategory};
    use std::path::PathBuf;

    #[test]
    fn all_active_ai_commands_keep_main_window_visible_for_harness() {
        // All active AI commands now route to the harness terminal (a view
        // inside the main window), so they must all keep the window visible.
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::GenerateScript
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::GenerateScriptFromCurrentApp
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::SendScreenToAi
        ));
        assert!(ai_command_keeps_main_window_visible(&AiCommandType::OpenAi));
        assert!(ai_command_keeps_main_window_visible(&AiCommandType::MiniAi));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::NewConversation
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::ClearConversation
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::SendFocusedWindowToAi
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::SendSelectedTextToAi
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::SendBrowserTabToAi
        ));
        assert!(ai_command_keeps_main_window_visible(
            &AiCommandType::SendScreenAreaToAi
        ));
    }

    #[test]
    fn test_ai_open_failure_message_includes_error_details() {
        assert_eq!(
            ai_open_failure_message("window init failed"),
            "Failed to open AI: window init failed"
        );
    }

    #[test]
    fn test_favorites_loaded_message_uses_singular_for_one() {
        assert_eq!(favorites_loaded_message(1), "Loaded 1 favorite");
    }

    #[test]
    fn test_favorites_loaded_message_uses_plural_for_many() {
        assert_eq!(favorites_loaded_message(3), "Loaded 3 favorites");
    }

    #[test]
    fn no_ai_commands_use_hide_then_capture_flow_after_harness_redirect() {
        // Legacy capture flow is no longer used — all active AI commands
        // route to the harness terminal which captures context inline.
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::GenerateScriptFromCurrentApp
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendScreenToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendFocusedWindowToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendScreenAreaToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendSelectedTextToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::SendBrowserTabToAi
        ));
        assert!(!ai_command_uses_hide_then_capture_flow(
            &AiCommandType::MiniAi
        ));
    }

    #[test]
    fn ai_command_window_plan_names_harness_visible_and_preset_hide_paths() {
        assert_eq!(
            AiCommandWindowPlan::from_command(&AiCommandType::GenerateScript),
            AiCommandWindowPlan::KeepMainWindowVisible
        );
        assert_eq!(
            AiCommandWindowPlan::from_command(&AiCommandType::SendScreenToAi),
            AiCommandWindowPlan::KeepMainWindowVisible
        );
        assert_eq!(
            AiCommandWindowPlan::from_command(&AiCommandType::OpenAi),
            AiCommandWindowPlan::KeepMainWindowVisible
        );
        assert_eq!(
            AiCommandWindowPlan::from_command(&AiCommandType::CreateAiPreset),
            AiCommandWindowPlan::HideMainWindowDeferred
        );
        assert!(
            !AiCommandWindowPlan::from_command(&AiCommandType::SendBrowserTabToAi)
                .uses_hide_then_capture_flow()
        );
    }

    #[test]
    fn test_ai_capture_hide_settle_duration_matches_constant() {
        assert_eq!(
            ai_capture_hide_settle_duration(),
            std::time::Duration::from_millis(AI_CAPTURE_HIDE_SETTLE_MS)
        );
    }

    #[test]
    fn test_ai_capture_hide_settle_duration_waits_150ms() {
        assert_eq!(AI_CAPTURE_HIDE_SETTLE_MS, 150);
        assert_eq!(
            ai_capture_hide_settle_duration(),
            std::time::Duration::from_millis(150)
        );
    }

    #[test]
    fn test_emoji_picker_label_includes_emoji_and_name() {
        let emoji = Emoji {
            emoji: "🚀",
            name: "rocket",
            keywords: &["launch", "ship"],
            category: EmojiCategory::TravelPlaces,
        };

        assert_eq!(emoji_picker_label(&emoji), "🚀  rocket");
    }

    #[test]
    fn test_created_file_path_for_feedback_returns_same_path_when_already_absolute() {
        let absolute_path = PathBuf::from("/tmp/new-script.ts");
        let feedback_path = created_file_path_for_feedback(&absolute_path);

        assert_eq!(feedback_path, absolute_path);
    }

    #[test]
    fn test_created_file_path_for_feedback_joins_current_dir_when_relative() {
        let relative_path = PathBuf::from("new-script.ts");
        let current_dir = std::env::current_dir().expect("current dir should be available");
        let feedback_path = created_file_path_for_feedback(&relative_path);

        assert_eq!(feedback_path, current_dir.join(relative_path));
    }
}

#[cfg(test)]
mod dictation_model_prompt_tests {
    use super::*;

    #[test]
    fn downloading_prompt_shows_progress_bar_with_bytes_and_speed() {
        let (title, placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::Downloading {
                percentage: 35,
                downloaded_bytes: 175_000_000,
                total_bytes: 500_000_000,
                speed_bytes_per_sec: 10_485_760,
                eta_seconds: Some(31),
            },
        );
        assert!(
            title.contains("35%"),
            "title must show percentage, got: {title}"
        );
        assert!(
            placeholder.contains("166.9 MB"),
            "placeholder must show downloaded bytes, got: {placeholder}"
        );
        assert!(
            placeholder.contains("10.0 MB/s"),
            "placeholder must show speed, got: {placeholder}"
        );
        assert!(
            placeholder.contains("ETA"),
            "placeholder must show ETA, got: {placeholder}"
        );
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].name, "Cancel download");
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_CANCEL);
        assert_eq!(choices[1].name, "Hide");
        assert_eq!(choices[1].value, BUILTIN_DICTATION_MODEL_HIDE);
    }

    #[test]
    fn downloading_prompt_prefers_hide_after_phase_change() {
        let index = ScriptListApp::preferred_dictation_model_prompt_index(
            &crate::dictation::DictationModelStatus::Downloading {
                percentage: 0,
                downloaded_bytes: 0,
                total_bytes: crate::dictation::PARAKEET_MODEL_ARCHIVE_SIZE,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
            },
        );

        let (_, _, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::Downloading {
                percentage: 0,
                downloaded_bytes: 0,
                total_bytes: crate::dictation::PARAKEET_MODEL_ARCHIVE_SIZE,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
            },
        );

        assert_eq!(choices[index].value, BUILTIN_DICTATION_MODEL_HIDE);
    }

    #[test]
    fn failed_prompt_offers_retry() {
        let (title, placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::DownloadFailed("network timeout".to_string()),
        );
        assert_eq!(title, "Dictation model download failed");
        assert_eq!(placeholder, "network timeout");
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_DOWNLOAD);
        assert_eq!(choices[1].value, BUILTIN_DICTATION_MODEL_CANCEL);
    }

    #[test]
    fn cancelled_prompt_offers_retry_and_done() {
        let (title, placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::DownloadFailed(
                "model download cancelled".to_string(),
            ),
        );
        assert_eq!(title, "Download cancelled");
        assert!(
            placeholder.contains("Partial download kept"),
            "cancelled placeholder must mention partial file, got: {placeholder}"
        );
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].name, "Retry download");
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_DOWNLOAD);
        assert_eq!(choices[1].name, "Done");
        assert_eq!(choices[1].value, BUILTIN_DICTATION_MODEL_HIDE);
    }

    #[test]
    fn not_downloaded_prompt_offers_download_and_cancel() {
        let (title, placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::NotDownloaded,
        );
        assert_eq!(title, "Download local dictation model");
        assert!(
            placeholder.contains("fully local transcription")
                || placeholder.contains("resumable if interrupted"),
            "placeholder must mention local transcription or resumability, got: {placeholder}"
        );
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_DOWNLOAD);
        assert_eq!(choices[1].value, BUILTIN_DICTATION_MODEL_CANCEL);
    }

    #[test]
    fn extracting_prompt_offers_hide() {
        let (title, _placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::Extracting,
        );
        assert_eq!(title, "Installing local dictation model\u{2026}");
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_HIDE);
    }

    #[test]
    fn available_prompt_offers_done() {
        let (title, _placeholder, choices) = ScriptListApp::build_dictation_model_prompt(
            crate::dictation::DictationModelStatus::Available,
        );
        assert_eq!(title, "Dictation model ready");
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].value, BUILTIN_DICTATION_MODEL_HIDE);
    }
}
