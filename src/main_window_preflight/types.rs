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
    RunAgent,
    RunFallback,
    AskAi,
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
pub(crate) struct MainWindowPreflightReceipt {
    pub filter_text: String,
    pub selected_index: usize,
    pub enter_action: MainWindowPreflightAction,
    pub tab_action: Option<MainWindowPreflightAction>,
    pub warnings: Vec<String>,
}
