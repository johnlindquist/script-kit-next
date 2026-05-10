use crate::computer_use::window_observation::ComputerUseWindowObservationV1;
use crate::protocol::{
    AutomationInspectSnapshot, AutomationWindowTarget, PixelProbe, TargetWindowBounds,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseListAppWindowsRequest {
    pub pid: i32,
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

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerUseAppWindowInfo {
    pub native_window_id: u32,
    pub title: Option<String>,
    pub bounds: TargetWindowBounds,
    pub is_on_screen: bool,
    pub layer: i64,
    pub z_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<ComputerUseWindowObservationV1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseListAppsSnapshot {
    pub apps: Vec<ComputerUseRunningAppInfo>,
    pub frontmost_pid: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputerUseListAppWindowsSnapshot {
    pub app: Option<ComputerUseRunningAppInfo>,
    pub windows: Vec<ComputerUseAppWindowInfo>,
    pub warnings: Vec<String>,
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

    fn list_app_windows(
        &self,
        request: ComputerUseListAppWindowsRequest,
    ) -> Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError>;
}
