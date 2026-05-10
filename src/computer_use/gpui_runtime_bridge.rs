use crate::computer_use::runtime_bridge::{
    ComputerUseAppWindowInfo, ComputerUseInspectRequest, ComputerUseListAppWindowsRequest,
    ComputerUseListAppWindowsSnapshot, ComputerUseListAppsRequest, ComputerUseListAppsSnapshot,
    ComputerUseRunningAppInfo, ComputerUseRuntimeBridge, ComputerUseRuntimeError,
};
use crate::computer_use::window_observation::{
    computer_use_window_observation_v1, window_capture_selection_candidates_v1,
    window_duplicate_groups_v1, window_list_candidate_v1, window_own_process_policy_v1,
    window_title_fallbacks_v1, WindowCaptureSelectionObservationInputV1,
    WindowDuplicateObservationInputV1, WindowTitleFallbackObservationInputV1,
};
use crate::protocol::{AutomationInspectSnapshot, TargetWindowBounds};
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
    ListAppWindows {
        request_id: String,
        request: ComputerUseListAppWindowsRequest,
        response_tx: SyncSender<Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError>>,
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

    pub fn respond_list_app_windows(
        self,
        result: Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError>,
    ) {
        if let Self::ListAppWindows { response_tx, .. } = self {
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

    fn list_app_windows(
        &self,
        request: ComputerUseListAppWindowsRequest,
    ) -> Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError> {
        let sender = self
            .sender
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(ComputerUseRuntimeError::Unavailable)?;

        let request_id = format!("mcp-computer-list-app-windows:{}", uuid::Uuid::new_v4());
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        sender
            .try_send(GpuiComputerUseRequest::ListAppWindows {
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

#[cfg(target_os = "macos")]
pub fn list_app_windows_on_gpui_thread(
    request: &ComputerUseListAppWindowsRequest,
) -> Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError> {
    let apps = list_running_apps_on_gpui_thread(&ComputerUseListAppsRequest {
        include_hidden: true,
        include_background: true,
    })?;
    let app = apps.apps.into_iter().find(|app| app.pid == request.pid);

    let Some(app) = app else {
        return Ok(ComputerUseListAppWindowsSnapshot {
            app: None,
            windows: Vec::new(),
            warnings: Vec::new(),
        });
    };

    let mut warnings = Vec::new();
    if crate::platform::screen_capture_access_preflight() == Some(false) {
        warnings.push("screenRecordingNotGrantedWindowTitlesMayBeRedacted".to_string());
    }

    let windows = core_graphics_windows_for_pid(request.pid)?;
    Ok(ComputerUseListAppWindowsSnapshot {
        app: Some(app),
        windows,
        warnings,
    })
}

#[cfg(target_os = "macos")]
fn core_graphics_windows_for_pid(
    pid: i32,
) -> Result<Vec<ComputerUseAppWindowInfo>, ComputerUseRuntimeError> {
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionaryRef;
    use core_foundation::string::CFString;

    const K_CG_NULL_WINDOW_ID: u32 = 0;
    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGWindowListCopyWindowInfo(
            option: u32,
            relative_to_window: u32,
        ) -> core_foundation::array::CFArrayRef;
    }

    let window_info_list = unsafe {
        CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, K_CG_NULL_WINDOW_ID)
    };
    if window_info_list.is_null() {
        return Err(ComputerUseRuntimeError::Failed(
            "CGWindowListCopyWindowInfo returned null".to_string(),
        ));
    }

    let info_array: CFArray = unsafe { CFArray::wrap_under_create_rule(window_info_list) };
    let k_owner_pid = CFString::new("kCGWindowOwnerPID");
    let k_window_number = CFString::new("kCGWindowNumber");
    let k_window_name = CFString::new("kCGWindowName");
    let k_window_bounds = CFString::new("kCGWindowBounds");
    let k_window_is_on_screen = CFString::new("kCGWindowIsOnscreen");
    let k_window_layer = CFString::new("kCGWindowLayer");
    let k_window_alpha = CFString::new("kCGWindowAlpha");
    let k_window_sharing_state = CFString::new("kCGWindowSharingState");

    let mut windows = Vec::new();
    for index in 0..info_array.len() {
        let Some(item_ref) = info_array.get(index) else {
            continue;
        };
        let dict_ref = *item_ref as CFDictionaryRef;
        if dict_ref.is_null() {
            continue;
        }

        if cf_number_i64(dict_ref, &k_owner_pid) != Some(pid as i64) {
            continue;
        }

        let native_window_id = match cf_number_i64(dict_ref, &k_window_number) {
            Some(value) if value >= 0 => value as u32,
            _ => continue,
        };
        let Some(bounds) = cf_bounds(dict_ref, &k_window_bounds) else {
            continue;
        };

        let is_on_screen = cf_bool(dict_ref, &k_window_is_on_screen).unwrap_or(true);
        let layer = cf_number_i64(dict_ref, &k_window_layer).unwrap_or(0);
        let alpha = cf_number_f64(dict_ref, &k_window_alpha);
        let sharing_state = cf_number_i64(dict_ref, &k_window_sharing_state);
        let is_current_process_window = u32::try_from(pid).ok() == Some(std::process::id());
        let own_process_window_policy = window_own_process_policy_v1(
            is_current_process_window,
            if is_current_process_window {
                ns_window_is_excluded_from_windows_menu(native_window_id)
            } else {
                None
            },
        );
        let mut observation =
            computer_use_window_observation_v1(&bounds, is_on_screen, layer, alpha, sharing_state);
        observation.own_process_window_policy = own_process_window_policy;
        let own_process_window_policy_status = observation
            .own_process_window_policy
            .as_ref()
            .map(|policy| policy.status.clone());
        observation.list_candidate = Some(window_list_candidate_v1(
            &bounds,
            layer,
            alpha,
            own_process_window_policy_status,
        ));

        windows.push(ComputerUseAppWindowInfo {
            native_window_id,
            title: cf_string(dict_ref, &k_window_name),
            bounds,
            is_on_screen,
            layer,
            z_order: windows.len() as u32,
            observation: Some(observation),
        });
    }

    let duplicate_groups = window_duplicate_groups_v1(
        &windows
            .iter()
            .map(|window| WindowDuplicateObservationInputV1 {
                native_window_id: window.native_window_id,
                bounds: window.bounds.clone(),
                is_on_screen: window.is_on_screen,
                z_order: window.z_order,
            })
            .collect::<Vec<_>>(),
    );

    for (window, duplicate_group) in windows.iter_mut().zip(duplicate_groups) {
        if let Some(observation) = &mut window.observation {
            observation.duplicate_group = duplicate_group;
        }
    }

    let title_fallbacks = window_title_fallbacks_v1(
        &windows
            .iter()
            .map(|window| {
                let observation = window
                    .observation
                    .as_ref()
                    .expect("CoreGraphics windows are annotated before title fallback");

                WindowTitleFallbackObservationInputV1 {
                    title: window.title.clone(),
                    capture_candidate_status: observation.capture_candidate.status.clone(),
                    duplicate_group_status: observation
                        .duplicate_group
                        .as_ref()
                        .map(|group| group.status.clone()),
                }
            })
            .collect::<Vec<_>>(),
    );

    for (window, title_fallback) in windows.iter_mut().zip(title_fallbacks) {
        if let Some(observation) = &mut window.observation {
            observation.title_fallback = title_fallback;
        }
    }

    let capture_selection_candidates = window_capture_selection_candidates_v1(
        &windows
            .iter()
            .map(|window| {
                let observation = window
                    .observation
                    .as_ref()
                    .expect("CoreGraphics windows are annotated before capture selection");

                WindowCaptureSelectionObservationInputV1 {
                    capture_candidate_status: observation.capture_candidate.status.clone(),
                    capture_candidate_reason: observation.capture_candidate.reason.clone(),
                    duplicate_group_status: observation
                        .duplicate_group
                        .as_ref()
                        .map(|group| group.status.clone()),
                    title_fallback_status: observation
                        .title_fallback
                        .as_ref()
                        .map(|fallback| fallback.status.clone()),
                    own_process_window_policy_status: observation
                        .own_process_window_policy
                        .as_ref()
                        .map(|policy| policy.status.clone()),
                }
            })
            .collect::<Vec<_>>(),
    );

    for (window, capture_selection_candidate) in
        windows.iter_mut().zip(capture_selection_candidates)
    {
        if let Some(observation) = &mut window.observation {
            observation.capture_selection_candidate = Some(capture_selection_candidate);
        }
    }

    Ok(windows)
}

#[cfg(target_os = "macos")]
fn ns_window_is_excluded_from_windows_menu(native_window_id: u32) -> Option<bool> {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    let window_number = native_window_id as isize;
    unsafe {
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        if app.is_null() {
            return None;
        }

        let window: *mut Object = msg_send![app, windowWithWindowNumber: window_number];
        if window.is_null() {
            return None;
        }

        let is_excluded: bool = msg_send![window, isExcludedFromWindowsMenu];
        Some(is_excluded)
    }
}

#[cfg(target_os = "macos")]
fn cf_dictionary_value(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<core_foundation::base::CFTypeRef> {
    use core_foundation::base::TCFType;
    use std::ffi::c_void;

    let mut value: *const c_void = std::ptr::null();
    let found = unsafe {
        core_foundation::dictionary::CFDictionaryGetValueIfPresent(
            dict_ref,
            key.as_concrete_TypeRef() as *const c_void,
            &mut value,
        )
    };
    (found != 0 && !value.is_null()).then_some(value as core_foundation::base::CFTypeRef)
}

#[cfg(target_os = "macos")]
fn cf_number_i64(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<i64> {
    use core_foundation::base::TCFType;
    use core_foundation::number::CFNumber;

    let value = cf_dictionary_value(dict_ref, key)?;
    let number = unsafe { CFNumber::wrap_under_get_rule(value as *const _) };
    number.to_i64()
}

#[cfg(target_os = "macos")]
fn cf_number_f64(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<f64> {
    use core_foundation::base::CFTypeRef;
    use std::ffi::c_void;

    const K_CF_NUMBER_DOUBLE_TYPE: i32 = 13;
    const K_CF_NUMBER_SINT64_TYPE: i32 = 4;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFNumberGetValue(number: CFTypeRef, number_type: i32, value_ptr: *mut c_void) -> bool;
    }

    let value = cf_dictionary_value(dict_ref, key)?;
    let mut double_value = 0.0_f64;
    if unsafe {
        CFNumberGetValue(
            value,
            K_CF_NUMBER_DOUBLE_TYPE,
            &mut double_value as *mut f64 as *mut c_void,
        )
    } {
        return Some(double_value);
    }

    let mut int_value = 0_i64;
    if unsafe {
        CFNumberGetValue(
            value,
            K_CF_NUMBER_SINT64_TYPE,
            &mut int_value as *mut i64 as *mut c_void,
        )
    } {
        return Some(int_value as f64);
    }

    None
}

#[cfg(target_os = "macos")]
fn cf_string(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<String> {
    use core_foundation::base::TCFType;

    let value = cf_dictionary_value(dict_ref, key)?;
    let string =
        unsafe { core_foundation::string::CFString::wrap_under_get_rule(value as *const _) };
    Some(string.to_string())
}

#[cfg(target_os = "macos")]
fn cf_bool(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<bool> {
    use core_foundation::base::CFTypeRef;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFBooleanGetValue(boolean: CFTypeRef) -> bool;
    }

    let value = cf_dictionary_value(dict_ref, key)?;
    Some(unsafe { CFBooleanGetValue(value) })
}

#[cfg(target_os = "macos")]
fn cf_bounds(
    dict_ref: core_foundation::dictionary::CFDictionaryRef,
    key: &core_foundation::string::CFString,
) -> Option<TargetWindowBounds> {
    let value = cf_dictionary_value(dict_ref, key)?;
    let bounds_ref = value as core_foundation::dictionary::CFDictionaryRef;
    if bounds_ref.is_null() {
        return None;
    }

    let x = cf_number_i64(bounds_ref, &core_foundation::string::CFString::new("X"))? as i32;
    let y = cf_number_i64(bounds_ref, &core_foundation::string::CFString::new("Y"))? as i32;
    let width = cf_number_i64(bounds_ref, &core_foundation::string::CFString::new("Width"))?;
    let height = cf_number_i64(
        bounds_ref,
        &core_foundation::string::CFString::new("Height"),
    )?;

    Some(TargetWindowBounds {
        x,
        y,
        width: width.max(0) as u32,
        height: height.max(0) as u32,
    })
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

#[cfg(not(target_os = "macos"))]
pub fn list_app_windows_on_gpui_thread(
    _request: &ComputerUseListAppWindowsRequest,
) -> Result<ComputerUseListAppWindowsSnapshot, ComputerUseRuntimeError> {
    Ok(ComputerUseListAppWindowsSnapshot {
        app: None,
        windows: Vec::new(),
        warnings: Vec::new(),
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
