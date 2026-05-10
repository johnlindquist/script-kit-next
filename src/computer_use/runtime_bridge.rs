use crate::protocol::{AutomationInspectSnapshot, AutomationWindowTarget, PixelProbe};
use crate::tray::TrayMenuObservation;

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseInspectRequest {
    pub target: Option<AutomationWindowTarget>,
    pub hi_dpi: Option<bool>,
    pub probes: Vec<PixelProbe>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseListAppsRequest {
    pub include_hidden: bool,
    pub include_background: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerUseRunningAppInfo {
    pub pid: i32,
    pub bundle_id: Option<String>,
    pub name: String,
    pub is_active: bool,
    pub is_hidden: bool,
    pub activation_policy: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseListAppsSnapshot {
    pub apps: Vec<ComputerUseRunningAppInfo>,
    pub frontmost_pid: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputerUseRuntimeError {
    Unavailable,
    Disconnected,
    Timeout,
    Failed(String),
}

impl ComputerUseRuntimeError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unavailable => "runtime_unavailable",
            Self::Disconnected => "runtime_disconnected",
            Self::Timeout => "runtime_timeout",
            Self::Failed(_) => "inspection_failed",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Unavailable => {
                "computer/see requires a live automation runtime bridge to inspectAutomationWindow"
                    .to_string()
            }
            Self::Disconnected => "computer-use runtime bridge disconnected".to_string(),
            Self::Timeout => "computer-use runtime bridge timed out".to_string(),
            Self::Failed(message) => message.clone(),
        }
    }
}

pub trait ComputerUseRuntimeBridge: Send + Sync {
    fn inspect_automation_window(
        &self,
        request: ComputerUseInspectRequest,
    ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError>;

    fn list_running_apps(
        &self,
        request: ComputerUseListAppsRequest,
    ) -> Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError>;

    fn list_tray_menu(&self) -> Result<TrayMenuObservation, ComputerUseRuntimeError>;
}
