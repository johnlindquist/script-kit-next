/// Receipt types for the main-window Execution Contract rail.
///
/// These types describe "what Enter will do" and "what Tab will send"
/// for the currently selected item in the ScriptList view.  They are
/// serializable so an AI agent (or test harness) can inspect the
/// contract without parsing the UI.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum MainWindowPreflightActionKind {
    RunScript,
    RunSnippet,
    RunCommand,
    LaunchApp,
    SwitchWindow,
    OpenFile,
    RunAgent,
    RunFallback,
    OpenSkill,
    AskAi,
    InspectIssues,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MainWindowPreflightAction {
    pub kind: MainWindowPreflightActionKind,
    pub label: String,
    pub subject: String,
    pub type_label: String,
    pub source_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RootPassiveSourceReceipt {
    pub enabled: bool,
    pub frame_count: usize,
    pub cache_generation: u64,
    pub frame_generation: u64,
    pub refreshing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RootPassiveFrameReceipt {
    pub query: String,
    pub browser_tabs: RootPassiveSourceReceipt,
    pub browser_history: RootPassiveSourceReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MainWindowPreflightReceipt {
    pub filter_text: String,
    pub selected_index: usize,
    pub selected_result_key: Option<String>,
    pub visible_result_key_fingerprint: String,
    pub visible_result_count: usize,
    pub root_passive_frame: Option<RootPassiveFrameReceipt>,
    pub enter_action: MainWindowPreflightAction,
    pub tab_action: Option<MainWindowPreflightAction>,
    pub warnings: Vec<String>,
}
