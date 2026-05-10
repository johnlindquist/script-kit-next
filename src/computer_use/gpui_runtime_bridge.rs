use crate::computer_use::runtime_bridge::{
    ComputerUseInspectRequest, ComputerUseListAppsRequest, ComputerUseListAppsSnapshot,
    ComputerUseRunningAppInfo, ComputerUseRuntimeBridge, ComputerUseRuntimeError,
};
use crate::protocol::AutomationInspectSnapshot;
use std::sync::mpsc::{self, SyncSender};
use std::sync::RwLock;
use std::time::Duration;

pub struct GpuiComputerUseRuntimeBridge {
    sender: RwLock<Option<async_channel::Sender<GpuiComputerUseRequest>>>,
    timeout: Duration,
}

pub enum GpuiComputerUseRequest {
    InspectAutomationWindow {
        request_id: String,
        request: ComputerUseInspectRequest,
        response_tx: SyncSender<Result<AutomationInspectSnapshot, ComputerUseRuntimeError>>,
    },
    ListRunningApps {
        request_id: String,
        request: ComputerUseListAppsRequest,
        response_tx: SyncSender<Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError>>,
    },
}

impl GpuiComputerUseRuntimeBridge {
    pub fn new(timeout: Duration) -> Self {
        Self {
            sender: RwLock::new(None),
            timeout,
        }
    }

    pub fn install(&self, sender: async_channel::Sender<GpuiComputerUseRequest>) {
        if let Ok(mut guard) = self.sender.write() {
            *guard = Some(sender);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.sender.write() {
            *guard = None;
        }
    }
}

impl GpuiComputerUseRequest {
    pub fn respond_inspect(
        self,
        result: Result<AutomationInspectSnapshot, ComputerUseRuntimeError>,
    ) {
        if let Self::InspectAutomationWindow { response_tx, .. } = self {
            let _ = response_tx.send(result);
        }
    }

    pub fn respond_list_apps(
        self,
        result: Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError>,
    ) {
        if let Self::ListRunningApps { response_tx, .. } = self {
            let _ = response_tx.send(result);
        }
    }
}

impl ComputerUseRuntimeBridge for GpuiComputerUseRuntimeBridge {
    fn inspect_automation_window(
        &self,
        request: ComputerUseInspectRequest,
    ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError> {
        let sender = self
            .sender
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(ComputerUseRuntimeError::Unavailable)?;

        let request_id = format!("mcp-computer-see:{}", uuid::Uuid::new_v4());
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        sender
            .try_send(GpuiComputerUseRequest::InspectAutomationWindow {
                request_id,
                request,
                response_tx,
            })
            .map_err(|_| ComputerUseRuntimeError::Unavailable)?;

        response_rx
            .recv_timeout(self.timeout)
            .map_err(|_| ComputerUseRuntimeError::Timeout)?
    }

    fn list_running_apps(
        &self,
        request: ComputerUseListAppsRequest,
    ) -> Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError> {
        let sender = self
            .sender
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(ComputerUseRuntimeError::Unavailable)?;

        let request_id = format!("mcp-computer-list-apps:{}", uuid::Uuid::new_v4());
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        sender
            .try_send(GpuiComputerUseRequest::ListRunningApps {
                request_id,
                request,
                response_tx,
            })
            .map_err(|_| ComputerUseRuntimeError::Unavailable)?;

        response_rx
            .recv_timeout(self.timeout)
            .map_err(|_| ComputerUseRuntimeError::Timeout)?
    }
}

#[cfg(target_os = "macos")]
pub fn list_running_apps_on_gpui_thread(
    request: &ComputerUseListAppsRequest,
) -> Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let workspace_class = Class::get("NSWorkspace").ok_or_else(|| {
            ComputerUseRuntimeError::Failed("NSWorkspace unavailable".to_string())
        })?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        if workspace.is_null() {
            return Err(ComputerUseRuntimeError::Failed(
                "NSWorkspace sharedWorkspace returned null".to_string(),
            ));
        }

        let frontmost_app: *mut Object = msg_send![workspace, frontmostApplication];
        let frontmost_pid = if frontmost_app.is_null() {
            None
        } else {
            let pid: i32 = msg_send![frontmost_app, processIdentifier];
            (pid > 0).then_some(pid)
        };

        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        if running_apps.is_null() {
            return Err(ComputerUseRuntimeError::Failed(
                "NSWorkspace runningApplications returned null".to_string(),
            ));
        }

        let count: usize = msg_send![running_apps, count];
        let mut apps = Vec::with_capacity(count);
        for index in 0..count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: index];
            if app.is_null() {
                continue;
            }

            let pid: i32 = msg_send![app, processIdentifier];
            if pid <= 0 {
                continue;
            }

            let activation_policy_raw: i64 = msg_send![app, activationPolicy];
            let activation_policy = activation_policy_name(activation_policy_raw);
            if !request.include_background && activation_policy != "regular" {
                continue;
            }

            let is_hidden: bool = msg_send![app, isHidden];
            if !request.include_hidden && is_hidden {
                continue;
            }

            apps.push(ComputerUseRunningAppInfo {
                pid,
                bundle_id: nsstring_to_string(msg_send![app, bundleIdentifier]),
                name: nsstring_to_string(msg_send![app, localizedName])
                    .unwrap_or_else(|| format!("pid:{pid}")),
                is_active: msg_send![app, isActive],
                is_hidden,
                activation_policy: activation_policy.to_string(),
            });
        }

        apps.sort_by(|left, right| {
            right
                .is_active
                .cmp(&left.is_active)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.pid.cmp(&right.pid))
        });

        Ok(ComputerUseListAppsSnapshot {
            apps,
            frontmost_pid,
        })
    }
}

#[cfg(not(target_os = "macos"))]
pub fn list_running_apps_on_gpui_thread(
    _request: &ComputerUseListAppsRequest,
) -> Result<ComputerUseListAppsSnapshot, ComputerUseRuntimeError> {
    Ok(ComputerUseListAppsSnapshot {
        apps: Vec::new(),
        frontmost_pid: None,
    })
}

fn activation_policy_name(raw: i64) -> &'static str {
    match raw {
        0 => "regular",
        1 => "accessory",
        2 => "prohibited",
        _ => "unknown",
    }
}

#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(value: *mut objc::runtime::Object) -> Option<String> {
    use objc::{msg_send, sel, sel_impl};

    if value.is_null() {
        return None;
    }

    let utf8: *const i8 = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;
    }

    std::ffi::CStr::from_ptr(utf8)
        .to_str()
        .ok()
        .map(ToString::to_string)
}
