#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityOpenBuiltinAction {
    MainWindow,
    ScratchPad,
    QuickTerminal,
    ClaudeCode,
    ProcessManager,
}

impl UtilityOpenBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::MainWindow => Some(Self::MainWindow),
            crate::builtins::UtilityCommandType::ScratchPad => Some(Self::ScratchPad),
            crate::builtins::UtilityCommandType::QuickTerminal => Some(Self::QuickTerminal),
            crate::builtins::UtilityCommandType::ClaudeCode => Some(Self::ClaudeCode),
            crate::builtins::UtilityCommandType::ProcessManager => Some(Self::ProcessManager),
            crate::builtins::UtilityCommandType::StopAllProcesses
            | crate::builtins::UtilityCommandType::ScriptKitSelfie
            | crate::builtins::UtilityCommandType::DoInCurrentApp
            | crate::builtins::UtilityCommandType::TurnThisIntoCommand
            | crate::builtins::UtilityCommandType::CurrentAppCommands => None,
        }
    }

    pub fn opening_message(self) -> Option<&'static str> {
        match self {
            Self::MainWindow => Some("Opening Main Window"),
            Self::ScratchPad | Self::QuickTerminal | Self::ClaudeCode | Self::ProcessManager => {
                None
            }
        }
    }

    pub fn success_detail(self) -> &'static str {
        match self {
            Self::MainWindow => "open_main_window",
            Self::ScratchPad => "open_scratch_pad",
            Self::QuickTerminal => "open_quick_terminal",
            Self::ClaudeCode => "open_claude_code_terminal",
            Self::ProcessManager => "open_process_manager",
        }
    }

    pub fn opens_from_main_menu(self) -> bool {
        matches!(
            self,
            Self::ScratchPad | Self::QuickTerminal | Self::ClaudeCode
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityProcessBuiltinAction {
    StopAllProcesses,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityProcessBuiltinOutcome {
    NoRunningProcesses,
    StopRequested { process_count: usize },
}

impl UtilityProcessBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::StopAllProcesses => Some(Self::StopAllProcesses),
            _ => None,
        }
    }

    pub fn empty_hud(self) -> &'static str {
        match self {
            Self::StopAllProcesses => "No running scripts to stop.",
        }
    }

    pub fn success_hud(self, process_count: usize) -> String {
        match self {
            Self::StopAllProcesses => {
                format!("Stopped {process_count} running script process(es).")
            }
        }
    }

    pub fn success_detail(self) -> &'static str {
        match self {
            Self::StopAllProcesses => "stop_all_processes",
        }
    }

    pub fn outcome(self, process_count: usize) -> UtilityProcessBuiltinOutcome {
        let _ = self;
        match process_count {
            0 => UtilityProcessBuiltinOutcome::NoRunningProcesses,
            process_count => UtilityProcessBuiltinOutcome::StopRequested { process_count },
        }
    }
}

impl UtilityProcessBuiltinOutcome {
    pub fn should_stop_processes(self) -> bool {
        matches!(self, Self::StopRequested { .. })
    }

    pub fn process_count(self) -> usize {
        match self {
            Self::NoRunningProcesses => 0,
            Self::StopRequested { process_count } => process_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilitySelfieBuiltinAction {
    Selfie,
}

impl UtilitySelfieBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::ScriptKitSelfie => Some(Self::Selfie),
            _ => None,
        }
    }

    pub fn starting_hud(self, state: &str) -> String {
        match self {
            Self::Selfie => format!("Capturing Script Kit selfie: {state}"),
        }
    }

    pub fn saved_hud(self, receipt: &crate::platform::ScriptKitSelfieReceipt) -> String {
        match self {
            Self::Selfie => format!("Selfie saved: {}", receipt.png_path),
        }
    }

    pub fn failure_message(self, error: &dyn std::fmt::Display) -> String {
        match self {
            Self::Selfie => format!("Script Kit Selfie failed: {error}"),
        }
    }

    pub fn success_detail(self) -> &'static str {
        match self {
            Self::Selfie => "script_kit_selfie_saved",
        }
    }

    pub fn failure_detail(self) -> &'static str {
        match self {
            Self::Selfie => "script_kit_selfie_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityRecipeBuiltinAction {
    TurnThisIntoCommand,
}

impl UtilityRecipeBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::TurnThisIntoCommand => {
                Some(Self::TurnThisIntoCommand)
            }
            _ => None,
        }
    }

    pub fn success_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command",
        }
    }

    pub fn clipboard_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_clipboard_failed",
        }
    }

    pub fn serialize_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_serialize_failed",
        }
    }

    pub fn serialize_failure_message(self, error: &dyn std::fmt::Display) -> String {
        match self {
            Self::TurnThisIntoCommand => {
                format!("Failed to serialize current app command recipe: {error}")
            }
        }
    }

    pub fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_capture_failed",
        }
    }

    pub fn missing_query_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_missing_query",
        }
    }

    pub fn drift_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_drift",
        }
    }

    pub fn missing_entry_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_missing_entry_index",
        }
    }

    pub fn open_palette_success_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_open_palette",
        }
    }

    pub fn generate_script_success_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_generate_script",
        }
    }

    pub fn copied_recipe_hud(self, suggested_script_name: &str) -> String {
        match self {
            Self::TurnThisIntoCommand => {
                format!("Automation recipe copied: {suggested_script_name}")
            }
        }
    }

    pub fn unknown_action_failure_detail(self) -> &'static str {
        match self {
            Self::TurnThisIntoCommand => "turn_this_into_command_unknown_action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityDoInCurrentAppBuiltinAction {
    Submit,
}

impl UtilityDoInCurrentAppBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::DoInCurrentApp => Some(Self::Submit),
            _ => None,
        }
    }

    pub fn open_palette_success_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_open_palette",
        }
    }

    pub fn generate_script_success_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_generate_script_scheduled",
        }
    }

    pub fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::Submit => "do_in_current_app_capture_failed",
        }
    }

    pub fn capture_failure_message(self, error: &dyn std::fmt::Display) -> String {
        match self {
            Self::Submit => format!("Failed to load frontmost app menu bar: {error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityCurrentAppCommandsBuiltinAction {
    Open,
}

impl UtilityCurrentAppCommandsBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Option<Self> {
        match command {
            crate::builtins::UtilityCommandType::CurrentAppCommands => Some(Self::Open),
            _ => None,
        }
    }

    pub fn success_detail(self) -> &'static str {
        match self {
            Self::Open => "open_current_app_commands",
        }
    }

    pub fn capture_failure_detail(self) -> &'static str {
        match self {
            Self::Open => "current_app_commands_capture_failed",
        }
    }

    pub fn capture_failure_message(self, error: &dyn std::fmt::Display) -> String {
        match self {
            Self::Open => format!("Failed to load frontmost app menu bar: {error}"),
        }
    }

    pub fn refresh_failure_message(self, error: &dyn std::fmt::Display) -> String {
        match self {
            Self::Open => format!("Failed to refresh current app commands: {error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityCommandBuiltinAction {
    Open(UtilityOpenBuiltinAction),
    Process(UtilityProcessBuiltinAction),
    Selfie(UtilitySelfieBuiltinAction),
    Recipe(UtilityRecipeBuiltinAction),
    DoInCurrentApp(UtilityDoInCurrentAppBuiltinAction),
    CurrentAppCommands(UtilityCurrentAppCommandsBuiltinAction),
}

impl UtilityCommandBuiltinAction {
    pub fn from_command(command: crate::builtins::UtilityCommandType) -> Self {
        match command {
            crate::builtins::UtilityCommandType::MainWindow => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .unwrap_or(UtilityOpenBuiltinAction::MainWindow),
            ),
            crate::builtins::UtilityCommandType::ScratchPad => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .unwrap_or(UtilityOpenBuiltinAction::ScratchPad),
            ),
            crate::builtins::UtilityCommandType::QuickTerminal => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .unwrap_or(UtilityOpenBuiltinAction::QuickTerminal),
            ),
            crate::builtins::UtilityCommandType::ClaudeCode => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .unwrap_or(UtilityOpenBuiltinAction::ClaudeCode),
            ),
            crate::builtins::UtilityCommandType::ProcessManager => Self::Open(
                UtilityOpenBuiltinAction::from_command(command)
                    .unwrap_or(UtilityOpenBuiltinAction::ProcessManager),
            ),
            crate::builtins::UtilityCommandType::StopAllProcesses => Self::Process(
                UtilityProcessBuiltinAction::from_command(command)
                    .unwrap_or(UtilityProcessBuiltinAction::StopAllProcesses),
            ),
            crate::builtins::UtilityCommandType::ScriptKitSelfie => Self::Selfie(
                UtilitySelfieBuiltinAction::from_command(command)
                    .unwrap_or(UtilitySelfieBuiltinAction::Selfie),
            ),
            crate::builtins::UtilityCommandType::TurnThisIntoCommand => Self::Recipe(
                UtilityRecipeBuiltinAction::from_command(command)
                    .unwrap_or(UtilityRecipeBuiltinAction::TurnThisIntoCommand),
            ),
            crate::builtins::UtilityCommandType::DoInCurrentApp => Self::DoInCurrentApp(
                UtilityDoInCurrentAppBuiltinAction::from_command(command)
                    .unwrap_or(UtilityDoInCurrentAppBuiltinAction::Submit),
            ),
            crate::builtins::UtilityCommandType::CurrentAppCommands => Self::CurrentAppCommands(
                UtilityCurrentAppCommandsBuiltinAction::from_command(command)
                    .unwrap_or(UtilityCurrentAppCommandsBuiltinAction::Open),
            ),
        }
    }
}

#[cfg(test)]
mod utility_builtin_action_tests {
    use super::*;
    use crate::builtins::UtilityCommandType;

    const ALL_COMMANDS: [UtilityCommandType; 10] = [
        UtilityCommandType::MainWindow,
        UtilityCommandType::ScratchPad,
        UtilityCommandType::QuickTerminal,
        UtilityCommandType::ClaudeCode,
        UtilityCommandType::ProcessManager,
        UtilityCommandType::ScriptKitSelfie,
        UtilityCommandType::StopAllProcesses,
        UtilityCommandType::DoInCurrentApp,
        UtilityCommandType::TurnThisIntoCommand,
        UtilityCommandType::CurrentAppCommands,
    ];

    #[test]
    fn every_utility_command_lands_on_its_own_action() {
        let actions: Vec<_> = ALL_COMMANDS
            .into_iter()
            .map(UtilityCommandBuiltinAction::from_command)
            .collect();
        assert_eq!(
            actions,
            vec![
                UtilityCommandBuiltinAction::Open(UtilityOpenBuiltinAction::MainWindow),
                UtilityCommandBuiltinAction::Open(UtilityOpenBuiltinAction::ScratchPad),
                UtilityCommandBuiltinAction::Open(UtilityOpenBuiltinAction::QuickTerminal),
                UtilityCommandBuiltinAction::Open(UtilityOpenBuiltinAction::ClaudeCode),
                UtilityCommandBuiltinAction::Open(UtilityOpenBuiltinAction::ProcessManager),
                UtilityCommandBuiltinAction::Selfie(UtilitySelfieBuiltinAction::Selfie),
                UtilityCommandBuiltinAction::Process(UtilityProcessBuiltinAction::StopAllProcesses),
                UtilityCommandBuiltinAction::DoInCurrentApp(
                    UtilityDoInCurrentAppBuiltinAction::Submit
                ),
                UtilityCommandBuiltinAction::Recipe(
                    UtilityRecipeBuiltinAction::TurnThisIntoCommand
                ),
                UtilityCommandBuiltinAction::CurrentAppCommands(
                    UtilityCurrentAppCommandsBuiltinAction::Open
                ),
            ]
        );
        for command in ALL_COMMANDS {
            let leaf_count = [
                UtilityOpenBuiltinAction::from_command(command).is_some(),
                UtilityProcessBuiltinAction::from_command(command).is_some(),
                UtilitySelfieBuiltinAction::from_command(command).is_some(),
                UtilityRecipeBuiltinAction::from_command(command).is_some(),
                UtilityDoInCurrentAppBuiltinAction::from_command(command).is_some(),
                UtilityCurrentAppCommandsBuiltinAction::from_command(command).is_some(),
            ]
            .into_iter()
            .filter(|claimed| *claimed)
            .count();
            assert_eq!(
                leaf_count, 1,
                "{command:?} must belong to exactly one action"
            );
        }
    }

    #[test]
    fn open_actions_say_what_they_open_and_which_start_from_the_main_menu() {
        let open = |command| UtilityOpenBuiltinAction::from_command(command).unwrap();
        let rows: Vec<_> = [
            UtilityCommandType::MainWindow,
            UtilityCommandType::ScratchPad,
            UtilityCommandType::QuickTerminal,
            UtilityCommandType::ClaudeCode,
            UtilityCommandType::ProcessManager,
        ]
        .into_iter()
        .map(|command| {
            let action = open(command);
            (
                action.opening_message(),
                action.success_detail(),
                action.opens_from_main_menu(),
            )
        })
        .collect();
        assert_eq!(
            rows,
            vec![
                (Some("Opening Main Window"), "open_main_window", false),
                (None, "open_scratch_pad", true),
                (None, "open_quick_terminal", true),
                (None, "open_claude_code_terminal", true),
                (None, "open_process_manager", false),
            ]
        );
    }

    #[test]
    fn stop_all_processes_stops_only_when_something_is_running() {
        let action = UtilityProcessBuiltinAction::StopAllProcesses;

        let idle = action.outcome(0);
        assert_eq!(idle, UtilityProcessBuiltinOutcome::NoRunningProcesses);
        assert!(!idle.should_stop_processes());
        assert_eq!(idle.process_count(), 0);
        assert_eq!(action.empty_hud(), "No running scripts to stop.");

        let busy = action.outcome(3);
        assert_eq!(
            busy,
            UtilityProcessBuiltinOutcome::StopRequested { process_count: 3 }
        );
        assert!(busy.should_stop_processes());
        assert_eq!(busy.process_count(), 3);
        assert_eq!(
            action.success_hud(busy.process_count()),
            "Stopped 3 running script process(es)."
        );
        assert_eq!(action.success_detail(), "stop_all_processes");
    }

    #[test]
    fn selfie_words_name_the_state_the_saved_file_and_the_failure() {
        let bounds = crate::platform::ScriptKitSelfieBounds {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let receipt = crate::platform::ScriptKitSelfieReceipt {
            schema_version: 1,
            command_id: "builtin/script-kit-selfie".to_string(),
            receipt_id: "receipt".to_string(),
            created_at: "now".to_string(),
            state: "main".to_string(),
            shortcut: "cmd+alt+1".to_string(),
            capture_method: "test".to_string(),
            png_path: "/tmp/selfie.png".to_string(),
            receipt_path: "/tmp/selfie.json".to_string(),
            window_bounds: bounds,
            monitor_bounds: bounds,
            crop_bounds: bounds,
            image_width: 1,
            image_height: 1,
        };
        let action = UtilitySelfieBuiltinAction::Selfie;
        assert_eq!(
            action.starting_hud("main"),
            "Capturing Script Kit selfie: main"
        );
        assert_eq!(action.saved_hud(&receipt), "Selfie saved: /tmp/selfie.png");
        assert_eq!(
            action.failure_message(&"no display"),
            "Script Kit Selfie failed: no display"
        );
        assert_eq!(action.success_detail(), "script_kit_selfie_saved");
        assert_eq!(action.failure_detail(), "script_kit_selfie_failed");
    }

    #[test]
    fn turn_this_into_a_command_keeps_its_words_and_details() {
        let action = UtilityRecipeBuiltinAction::TurnThisIntoCommand;
        assert_eq!(
            action.copied_recipe_hud("open-downloads"),
            "Automation recipe copied: open-downloads"
        );
        assert_eq!(
            action.serialize_failure_message(&"bad json"),
            "Failed to serialize current app command recipe: bad json"
        );
        assert_eq!(
            [
                action.success_detail(),
                action.clipboard_failure_detail(),
                action.serialize_failure_detail(),
                action.capture_failure_detail(),
                action.missing_query_failure_detail(),
                action.drift_failure_detail(),
                action.missing_entry_failure_detail(),
                action.open_palette_success_detail(),
                action.generate_script_success_detail(),
                action.unknown_action_failure_detail(),
            ],
            [
                "turn_this_into_command",
                "turn_this_into_command_clipboard_failed",
                "turn_this_into_command_serialize_failed",
                "turn_this_into_command_capture_failed",
                "turn_this_into_command_missing_query",
                "turn_this_into_command_drift",
                "turn_this_into_command_missing_entry_index",
                "turn_this_into_command_open_palette",
                "turn_this_into_command_generate_script",
                "turn_this_into_command_unknown_action",
            ]
        );
    }

    #[test]
    fn current_app_actions_say_when_the_menu_bar_cannot_be_read() {
        let submit = UtilityDoInCurrentAppBuiltinAction::Submit;
        assert_eq!(
            submit.capture_failure_message(&"denied"),
            "Failed to load frontmost app menu bar: denied"
        );
        assert_eq!(
            [
                submit.open_palette_success_detail(),
                submit.generate_script_success_detail(),
                submit.capture_failure_detail(),
            ],
            [
                "do_in_current_app_open_palette",
                "do_in_current_app_generate_script_scheduled",
                "do_in_current_app_capture_failed",
            ]
        );

        let open = UtilityCurrentAppCommandsBuiltinAction::Open;
        assert_eq!(
            open.capture_failure_message(&"denied"),
            "Failed to load frontmost app menu bar: denied"
        );
        assert_eq!(
            open.refresh_failure_message(&"gone"),
            "Failed to refresh current app commands: gone"
        );
        assert_eq!(open.success_detail(), "open_current_app_commands");
        assert_eq!(
            open.capture_failure_detail(),
            "current_app_commands_capture_failed"
        );
    }
}
