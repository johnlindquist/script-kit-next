//! MCP computer-use tools.
//!
//! Iteration 1 exposes `computer/see` as the agent-facing name for Script Kit's
//! existing `inspectAutomationWindow` snapshot contract. Native input actions
//! remain deferred until they can cite stable inspection receipts.

use crate::computer_use::runtime_bridge::{
    ComputerUseAppWindowInfo, ComputerUseInspectRequest, ComputerUseListAppWindowsRequest,
    ComputerUseListAppsRequest, ComputerUseRunningAppInfo, ComputerUseRuntimeBridge,
    ComputerUseRuntimeError,
};
use crate::computer_use::types::ComputerUseSeeArgs;
use crate::frontmost_app_tracker::{get_cached_menu_snapshot, get_last_real_app};
use crate::mcp_kit_tools::{ToolContent, ToolDefinition, ToolResult};
use crate::menu_bar::MenuBarItem;
use crate::protocol::{
    AutomationWindowInfo, DisplayInfo, TargetWindowBounds, AUTOMATION_WINDOW_SCHEMA_VERSION,
};
use serde_json::Value;

pub const COMPUTER_USE_NAMESPACE: &str = "computer/";
pub const COMPUTER_SEE_TOOL: &str = "computer/see";
pub const COMPUTER_LIST_WINDOWS_TOOL: &str = "computer/list_windows";
pub const COMPUTER_GET_WINDOW_TOOL: &str = "computer/get_window";
pub const COMPUTER_GET_FOCUSED_WINDOW_TOOL: &str = "computer/get_focused_window";
pub const COMPUTER_LIST_APPS_TOOL: &str = "computer/list_apps";
pub const COMPUTER_GET_APP_TOOL: &str = "computer/get_app";
pub const COMPUTER_LIST_APP_WINDOWS_TOOL: &str = "computer/list_app_windows";
pub const COMPUTER_GET_FRONTMOST_APP_TOOL: &str = "computer/get_frontmost_app";
pub const COMPUTER_LIST_MENUS_TOOL: &str = "computer/list_menus";
pub const COMPUTER_LIST_TRAY_MENU_TOOL: &str = "computer/list_tray_menu";
pub const COMPUTER_LIST_SCREENS_TOOL: &str = "computer/list_screens";
pub const COMPUTER_LIST_PERMISSIONS_TOOL: &str = "computer/list_permissions";
const COMPUTER_APPS_SCHEMA_VERSION: u32 = 1;
const COMPUTER_APP_WINDOWS_SCHEMA_VERSION: u32 = 1;
const COMPUTER_FRONTMOST_APP_SCHEMA_VERSION: u32 = 1;
const COMPUTER_MENUS_SCHEMA_VERSION: u32 = 1;
const COMPUTER_SCREENS_SCHEMA_VERSION: u32 = 1;
const COMPUTER_PERMISSIONS_SCHEMA_VERSION: u32 = 1;

pub fn get_computer_use_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: COMPUTER_SEE_TOOL.to_string(),
            description:
                "Inspect a Script Kit automation window and return a state-first computer-use observation."
                    .to_string(),
            input_schema: computer_see_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_WINDOWS_TOOL.to_string(),
            description: "List registered Script Kit automation windows without interacting with them."
                .to_string(),
            input_schema: computer_list_windows_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_GET_WINDOW_TOOL.to_string(),
            description: "Return one registered Script Kit automation window by stable automation window id without screenshots, native focus changes, or runtime inspection."
                .to_string(),
            input_schema: computer_get_window_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_GET_FOCUSED_WINDOW_TOOL.to_string(),
            description: "Return the focused Script Kit automation window from the automation registry without screenshots, native focus changes, or runtime inspection."
                .to_string(),
            input_schema: computer_get_focused_window_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_APPS_TOOL.to_string(),
            description: "List running GUI applications without launching, quitting, focusing, hiding, or sending input."
                .to_string(),
            input_schema: computer_list_apps_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_GET_APP_TOOL.to_string(),
            description: "Return one running GUI application by PID without launching, quitting, focusing, hiding, or sending input."
                .to_string(),
            input_schema: computer_get_app_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_APP_WINDOWS_TOOL.to_string(),
            description: "List native windows for one running GUI application by PID without focusing, moving, resizing, or capturing screenshots."
                .to_string(),
            input_schema: computer_list_app_windows_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_GET_FRONTMOST_APP_TOOL.to_string(),
            description: "Return the last tracked non-Script-Kit frontmost app from the frontmost app tracker cache without refreshing, focusing, activating, or requesting permissions."
                .to_string(),
            input_schema: computer_get_frontmost_app_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_MENUS_TOOL.to_string(),
            description: "List cached menu items for the last tracked real application without refreshing, focusing, clicking, or requesting permissions."
                .to_string(),
            input_schema: computer_list_menus_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_TRAY_MENU_TOOL.to_string(),
            description: "List Script Kit's own tray menu model without opening the menu, clicking status items, invoking actions, or requesting permissions."
                .to_string(),
            input_schema: computer_list_tray_menu_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_SCREENS_TOOL.to_string(),
            description: "List attached screens/displays without moving windows, changing screen placement, or requesting permissions."
                .to_string(),
            input_schema: computer_list_screens_input_schema(),
        },
        ToolDefinition {
            name: COMPUTER_LIST_PERMISSIONS_TOOL.to_string(),
            description: "List read-only macOS permission status for Script Kit computer-use features without requesting permissions."
                .to_string(),
            input_schema: computer_list_permissions_input_schema(),
        },
    ]
}

pub fn is_computer_use_tool(name: &str) -> bool {
    name.starts_with(COMPUTER_USE_NAMESPACE)
}

pub fn handle_computer_use_tool_call(
    name: &str,
    arguments: &Value,
    runtime: Option<&dyn ComputerUseRuntimeBridge>,
) -> ToolResult {
    match name {
        COMPUTER_SEE_TOOL => handle_see(arguments, runtime),
        COMPUTER_LIST_WINDOWS_TOOL => handle_list_windows(arguments),
        COMPUTER_GET_WINDOW_TOOL => handle_get_window(arguments),
        COMPUTER_GET_FOCUSED_WINDOW_TOOL => handle_get_focused_window(arguments),
        COMPUTER_LIST_APPS_TOOL => handle_list_apps(arguments, runtime),
        COMPUTER_GET_APP_TOOL => handle_get_app(arguments, runtime),
        COMPUTER_LIST_APP_WINDOWS_TOOL => handle_list_app_windows(arguments, runtime),
        COMPUTER_GET_FRONTMOST_APP_TOOL => handle_get_frontmost_app(arguments),
        COMPUTER_LIST_MENUS_TOOL => handle_list_menus(arguments),
        COMPUTER_LIST_TRAY_MENU_TOOL => handle_list_tray_menu(arguments),
        COMPUTER_LIST_SCREENS_TOOL => handle_list_screens(arguments),
        COMPUTER_LIST_PERMISSIONS_TOOL => handle_list_permissions(arguments),
        _ => error_result(
            "unknown_tool",
            &format!("Unknown computer-use tool: {name}"),
        ),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListWindowsArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListWindowsResult {
    schema_version: u32,
    windows: Vec<AutomationWindowInfo>,
    focused_window_id: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseGetWindowArgs {
    id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseGetWindowResult {
    schema_version: u32,
    source: &'static str,
    status: &'static str,
    window: Option<AutomationWindowInfo>,
    warnings: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseGetFocusedWindowArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseGetFocusedWindowResult {
    schema_version: u32,
    source: &'static str,
    scope: &'static str,
    status: &'static str,
    focused_window_id: Option<String>,
    window: Option<AutomationWindowInfo>,
    warnings: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ComputerUseListAppsArgs {
    #[serde(default)]
    include_hidden: bool,
    #[serde(default)]
    include_background: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListAppsResult {
    schema_version: u32,
    apps: Vec<ComputerUseRunningAppInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frontmost_pid: Option<i32>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseGetAppArgs {
    pid: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseGetAppResult {
    schema_version: u32,
    source: &'static str,
    scope: &'static str,
    status: &'static str,
    app: Option<ComputerUseRunningAppInfo>,
    warnings: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListAppWindowsArgs {
    pid: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListAppWindowsResult {
    schema_version: u32,
    source: &'static str,
    scope: &'static str,
    status: &'static str,
    app: Option<ComputerUseRunningAppInfo>,
    windows: Vec<ComputerUseAppWindowInfo>,
    warnings: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseGetFrontmostAppArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseGetFrontmostAppResult {
    schema_version: u32,
    source: &'static str,
    scope: &'static str,
    status: &'static str,
    app: Option<ComputerUseFrontmostApp>,
    warnings: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseFrontmostApp {
    pid: i32,
    bundle_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListMenusArgs {}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListTrayMenuArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListMenusResult {
    schema_version: u32,
    source: &'static str,
    app: Option<ComputerUseMenuApp>,
    cache: ComputerUseMenuCache,
    menus: Vec<ComputerUseMenuItem>,
    warnings: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseMenuApp {
    pid: i32,
    bundle_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseMenuCache {
    status: &'static str,
    is_fetching: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseMenuItem {
    title: String,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    shortcut: Option<String>,
    children: Vec<ComputerUseMenuItem>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListScreensArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListScreensResult {
    schema_version: u32,
    screens: Vec<DisplayInfo>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComputerUseListPermissionsArgs {}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseListPermissionsResult {
    schema_version: u32,
    permissions: Vec<ComputerUsePermissionStatus>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUsePermissionStatus {
    id: &'static str,
    name: &'static str,
    granted: Option<bool>,
    status: &'static str,
}

fn handle_see(arguments: &Value, runtime: Option<&dyn ComputerUseRuntimeBridge>) -> ToolResult {
    let args: ComputerUseSeeArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let Some(runtime) = runtime else {
        return runtime_error_result(&args, ComputerUseRuntimeError::Unavailable);
    };

    let request = ComputerUseInspectRequest {
        target: args.target.clone(),
        hi_dpi: args.hi_dpi,
        probes: args.probes.clone(),
    };

    match runtime.inspect_automation_window(request) {
        Ok(snapshot) => json_tool_result(&snapshot),
        Err(error) => runtime_error_result(&args, error),
    }
}

fn handle_list_windows(arguments: &Value) -> ToolResult {
    let _args: ComputerUseListWindowsArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    json_tool_result(&ComputerUseListWindowsResult {
        schema_version: AUTOMATION_WINDOW_SCHEMA_VERSION,
        windows: crate::windows::list_automation_windows(),
        focused_window_id: crate::windows::focused_automation_window_id(),
    })
}

fn handle_get_window(arguments: &Value) -> ToolResult {
    let args: ComputerUseGetWindowArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let window = crate::windows::automation_window_by_id(&args.id);

    json_tool_result(&ComputerUseGetWindowResult {
        schema_version: AUTOMATION_WINDOW_SCHEMA_VERSION,
        source: "automationWindowRegistry",
        status: if window.is_some() {
            "found"
        } else {
            "notFound"
        },
        window,
        warnings: Vec::new(),
    })
}

fn handle_get_focused_window(arguments: &Value) -> ToolResult {
    let _args: ComputerUseGetFocusedWindowArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let window = crate::windows::focused_automation_window();
    let focused_window_id = window.as_ref().map(|window| window.id.clone());

    json_tool_result(&ComputerUseGetFocusedWindowResult {
        schema_version: AUTOMATION_WINDOW_SCHEMA_VERSION,
        source: "automationWindowRegistry",
        scope: "focusedAutomationWindow",
        status: if window.is_some() {
            "focused"
        } else {
            "noFocusedWindow"
        },
        focused_window_id,
        window,
        warnings: Vec::new(),
    })
}

fn handle_list_apps(
    arguments: &Value,
    runtime: Option<&dyn ComputerUseRuntimeBridge>,
) -> ToolResult {
    let args: ComputerUseListAppsArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let Some(runtime) = runtime else {
        return error_result(
            "runtime_unavailable",
            "computer/list_apps requires the live GPUI runtime bridge to enumerate running applications safely",
        );
    };

    let request = ComputerUseListAppsRequest {
        include_hidden: args.include_hidden,
        include_background: args.include_background,
    };

    match runtime.list_running_apps(request) {
        Ok(snapshot) => json_tool_result(&ComputerUseListAppsResult {
            schema_version: COMPUTER_APPS_SCHEMA_VERSION,
            apps: snapshot.apps,
            frontmost_pid: snapshot.frontmost_pid,
        }),
        Err(error) => error_result(error.error_code(), &error.message()),
    }
}

fn handle_get_app(arguments: &Value, runtime: Option<&dyn ComputerUseRuntimeBridge>) -> ToolResult {
    let args: ComputerUseGetAppArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let Some(runtime) = runtime else {
        return error_result(
            "runtime_unavailable",
            "computer/get_app requires the live GPUI runtime bridge to enumerate running applications safely",
        );
    };

    let request = ComputerUseListAppsRequest {
        include_hidden: true,
        include_background: true,
    };

    match runtime.list_running_apps(request) {
        Ok(snapshot) => {
            let app = snapshot.apps.into_iter().find(|app| app.pid == args.pid);
            json_tool_result(&ComputerUseGetAppResult {
                schema_version: COMPUTER_APPS_SCHEMA_VERSION,
                source: "nsWorkspaceRunningApplications",
                scope: "runningAppPid",
                status: if app.is_some() { "found" } else { "notFound" },
                app,
                warnings: Vec::new(),
            })
        }
        Err(error) => error_result(error.error_code(), &error.message()),
    }
}

fn handle_list_app_windows(
    arguments: &Value,
    runtime: Option<&dyn ComputerUseRuntimeBridge>,
) -> ToolResult {
    let args: ComputerUseListAppWindowsArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let Some(runtime) = runtime else {
        return error_result(
            "runtime_unavailable",
            "computer/list_app_windows requires the live GPUI runtime bridge to enumerate app windows safely",
        );
    };

    let request = ComputerUseListAppWindowsRequest { pid: args.pid };

    match runtime.list_app_windows(request) {
        Ok(snapshot) => json_tool_result(&ComputerUseListAppWindowsResult {
            schema_version: COMPUTER_APP_WINDOWS_SCHEMA_VERSION,
            source: "coreGraphicsWindowList",
            scope: "runningAppPid",
            status: if snapshot.app.is_some() {
                "found"
            } else {
                "notFound"
            },
            app: snapshot.app,
            windows: snapshot.windows,
            warnings: snapshot.warnings,
        }),
        Err(error) => error_result(error.error_code(), &error.message()),
    }
}

fn handle_get_frontmost_app(arguments: &Value) -> ToolResult {
    let _args: ComputerUseGetFrontmostAppArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let app = get_last_real_app().map(|app| ComputerUseFrontmostApp {
        pid: app.pid,
        bundle_id: app.bundle_id,
        name: app.name,
        window_title: app.window_title,
    });

    json_tool_result(&ComputerUseGetFrontmostAppResult {
        schema_version: COMPUTER_FRONTMOST_APP_SCHEMA_VERSION,
        source: "frontmostAppTrackerCache",
        scope: "lastNonScriptKitApp",
        status: if app.is_some() {
            "tracked"
        } else {
            "noTrackedApp"
        },
        app,
        warnings: Vec::new(),
    })
}

fn handle_list_menus(arguments: &Value) -> ToolResult {
    let _args: ComputerUseListMenusArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let snapshot = get_cached_menu_snapshot();
    let app = snapshot.app.map(|app| ComputerUseMenuApp {
        pid: app.pid,
        bundle_id: app.bundle_id,
        name: app.name,
        window_title: app.window_title,
    });

    json_tool_result(&ComputerUseListMenusResult {
        schema_version: COMPUTER_MENUS_SCHEMA_VERSION,
        source: "frontmostAppTrackerCache",
        app,
        cache: ComputerUseMenuCache {
            status: snapshot.status.as_str(),
            is_fetching: snapshot.is_fetching,
        },
        menus: snapshot.items.iter().map(computer_use_menu_item).collect(),
        warnings: Vec::new(),
    })
}

fn handle_list_tray_menu(arguments: &Value) -> ToolResult {
    let _args: ComputerUseListTrayMenuArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    json_tool_result(&crate::tray::current_tray_menu_observation_snapshot())
}

fn computer_use_menu_item(item: &MenuBarItem) -> ComputerUseMenuItem {
    ComputerUseMenuItem {
        title: item.title.clone(),
        enabled: item.enabled,
        shortcut: item
            .shortcut
            .as_ref()
            .map(|shortcut| shortcut.to_display_string()),
        children: item.children.iter().map(computer_use_menu_item).collect(),
    }
}

fn handle_list_screens(arguments: &Value) -> ToolResult {
    let _args: ComputerUseListScreensArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    match list_screens() {
        Ok(screens) => json_tool_result(&ComputerUseListScreensResult {
            schema_version: COMPUTER_SCREENS_SCHEMA_VERSION,
            screens,
        }),
        Err(error) => error_result("screen_list_failed", &error),
    }
}

fn handle_list_permissions(arguments: &Value) -> ToolResult {
    let _args: ComputerUseListPermissionsArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    json_tool_result(&ComputerUseListPermissionsResult {
        schema_version: COMPUTER_PERMISSIONS_SCHEMA_VERSION,
        permissions: vec![
            permission_status(
                "accessibility",
                "Accessibility",
                Some(crate::permissions_wizard::check_accessibility_permission()),
            ),
            permission_status(
                "screenRecording",
                "Screen Recording",
                crate::platform::screen_capture_access_preflight(),
            ),
        ],
    })
}

fn permission_status(
    id: &'static str,
    name: &'static str,
    granted: Option<bool>,
) -> ComputerUsePermissionStatus {
    let status = match granted {
        Some(true) => "granted",
        Some(false) => "notGranted",
        None => "unknown",
    };

    ComputerUsePermissionStatus {
        id,
        name,
        granted,
        status,
    }
}

#[cfg(target_os = "macos")]
fn list_screens() -> Result<Vec<DisplayInfo>, String> {
    use core_graphics::display::CGDisplay;

    const MACOS_MENU_BAR_HEIGHT: i32 = 24;

    let display_ids =
        CGDisplay::active_displays().map_err(|_| "Failed to get active displays".to_string())?;
    let main_display_id = CGDisplay::main().id;
    let mut screens = Vec::with_capacity(display_ids.len());

    for (index, display_id) in display_ids.iter().copied().enumerate() {
        let display = CGDisplay::new(display_id);
        let bounds = display.bounds();
        let visible_y = bounds.origin.y as i32 + MACOS_MENU_BAR_HEIGHT;
        let visible_height =
            (bounds.size.height as u32).saturating_sub(MACOS_MENU_BAR_HEIGHT as u32);

        screens.push(DisplayInfo {
            display_id,
            name: format!("Display {}", index + 1),
            is_primary: display_id == main_display_id,
            bounds: TargetWindowBounds {
                x: bounds.origin.x as i32,
                y: bounds.origin.y as i32,
                width: bounds.size.width as u32,
                height: bounds.size.height as u32,
            },
            visible_bounds: TargetWindowBounds {
                x: bounds.origin.x as i32,
                y: visible_y,
                width: bounds.size.width as u32,
                height: visible_height,
            },
            scale_factor: None,
        });
    }

    Ok(screens)
}

#[cfg(not(target_os = "macos"))]
fn list_screens() -> Result<Vec<DisplayInfo>, String> {
    Ok(vec![DisplayInfo {
        display_id: 0,
        name: "Primary Display".to_string(),
        is_primary: true,
        bounds: TargetWindowBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        visible_bounds: TargetWindowBounds {
            x: 0,
            y: 24,
            width: 1920,
            height: 1056,
        },
        scale_factor: Some(1.0),
    }])
}

fn runtime_error_result(args: &ComputerUseSeeArgs, error: ComputerUseRuntimeError) -> ToolResult {
    let target = args
        .target
        .as_ref()
        .map(|target| serde_json::to_value(target).unwrap_or(Value::Null));

    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::json!({
                "schemaVersion": 1,
                "errorCode": error.error_code(),
                "message": error.message(),
                "target": target,
            })
            .to_string(),
        }],
        is_error: Some(true),
    }
}

fn json_tool_result<T: serde::Serialize>(value: &T) -> ToolResult {
    match serde_json::to_string(value) {
        Ok(text) => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: None,
        },
        Err(error) => error_result("serialization_failed", &error.to_string()),
    }
}

fn error_result(code: &str, message: &str) -> ToolResult {
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::json!({
                "schemaVersion": 1,
                "errorCode": code,
                "message": message,
            })
            .to_string(),
        }],
        is_error: Some(true),
    }
}

fn computer_see_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "target": {
                "description": "AutomationWindowTarget. Omit to use the focused automation window.",
                "oneOf": [
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": { "type": { "const": "main" } },
                        "required": ["type"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": { "type": { "const": "focused" } },
                        "required": ["type"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "id" },
                            "id": { "type": "string" }
                        },
                        "required": ["type", "id"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "kind" },
                            "kind": {
                                "type": "string",
                                "enum": ["main", "notes", "ai", "miniAi", "acpDetached", "actionsDialog", "promptPopup"]
                            },
                            "index": { "type": "integer", "minimum": 0 }
                        },
                        "required": ["type", "kind"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "titleContains" },
                            "text": { "type": "string" }
                        },
                        "required": ["type", "text"]
                    }
                ]
            },
            "hiDpi": { "type": "boolean", "default": false },
            "probes": {
                "type": "array",
                "default": [],
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 }
                    },
                    "required": ["x", "y"]
                }
            }
        }
    })
}

fn computer_list_windows_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_get_window_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "id": { "type": "string" }
        },
        "required": ["id"]
    })
}

fn computer_get_focused_window_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_list_apps_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "includeHidden": {
                "type": "boolean",
                "default": false,
                "description": "Include hidden running GUI applications."
            },
            "includeBackground": {
                "type": "boolean",
                "default": false,
                "description": "Include accessory, prohibited, and unknown background applications in addition to regular GUI apps."
            }
        }
    })
}

fn computer_get_app_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "pid": { "type": "integer" }
        },
        "required": ["pid"]
    })
}

fn computer_list_app_windows_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "pid": { "type": "integer" }
        },
        "required": ["pid"]
    })
}

fn computer_get_frontmost_app_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_list_menus_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_list_tray_menu_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_list_screens_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

fn computer_list_permissions_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{
        AutomationInspectSnapshot, AutomationWindowBounds, AutomationWindowInfo,
        AutomationWindowKind, AutomationWindowTarget, SemanticQuality, TargetWindowBounds,
        AUTOMATION_INSPECT_SCHEMA_VERSION, AUTOMATION_WINDOW_SCHEMA_VERSION,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeComputerUseRuntime;

    impl ComputerUseRuntimeBridge for FakeComputerUseRuntime {
        fn inspect_automation_window(
            &self,
            request: ComputerUseInspectRequest,
        ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError> {
            assert_eq!(request.target, Some(AutomationWindowTarget::Focused));
            assert_eq!(request.hi_dpi, Some(true));
            assert_eq!(
                request.probes,
                vec![
                    crate::protocol::PixelProbe { x: 10, y: 20 },
                    crate::protocol::PixelProbe { x: 30, y: 40 },
                ]
            );

            Ok(AutomationInspectSnapshot {
                schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
                window_id: "main:0".to_string(),
                window_kind: "Main".to_string(),
                title: Some("Script Kit".to_string()),
                resolved_bounds: None,
                target_bounds_in_screenshot: None,
                surface_hit_point: None,
                suggested_hit_points: Vec::new(),
                elements: Vec::new(),
                total_count: 0,
                focused_semantic_id: None,
                selected_semantic_id: None,
                screenshot_width: Some(800),
                screenshot_height: Some(600),
                pixel_probes: Vec::new(),
                os_window_id: Some(123),
                semantic_quality: Some(SemanticQuality::Full),
                warnings: Vec::new(),
            })
        }

        fn list_running_apps(
            &self,
            request: ComputerUseListAppsRequest,
        ) -> Result<
            crate::computer_use::runtime_bridge::ComputerUseListAppsSnapshot,
            ComputerUseRuntimeError,
        > {
            assert!(request.include_hidden);
            assert!(request.include_background);

            Ok(
                crate::computer_use::runtime_bridge::ComputerUseListAppsSnapshot {
                    apps: vec![
                        ComputerUseRunningAppInfo {
                            pid: 101,
                            bundle_id: Some("com.apple.Terminal".to_string()),
                            name: "Terminal".to_string(),
                            is_active: true,
                            is_hidden: false,
                            activation_policy: "regular".to_string(),
                        },
                        ComputerUseRunningAppInfo {
                            pid: 202,
                            bundle_id: None,
                            name: "Background Utility".to_string(),
                            is_active: false,
                            is_hidden: true,
                            activation_policy: "accessory".to_string(),
                        },
                    ],
                    frontmost_pid: Some(101),
                },
            )
        }

        fn list_app_windows(
            &self,
            request: ComputerUseListAppWindowsRequest,
        ) -> Result<
            crate::computer_use::runtime_bridge::ComputerUseListAppWindowsSnapshot,
            ComputerUseRuntimeError,
        > {
            assert_eq!(request.pid, 101);

            Ok(
                crate::computer_use::runtime_bridge::ComputerUseListAppWindowsSnapshot {
                    app: Some(ComputerUseRunningAppInfo {
                        pid: 101,
                        bundle_id: Some("com.apple.Terminal".to_string()),
                        name: "Terminal".to_string(),
                        is_active: true,
                        is_hidden: false,
                        activation_policy: "regular".to_string(),
                    }),
                    windows: vec![ComputerUseAppWindowInfo {
                        native_window_id: 98765,
                        title: Some("Terminal".to_string()),
                        bounds: TargetWindowBounds {
                            x: 10,
                            y: 20,
                            width: 300,
                            height: 200,
                        },
                        is_on_screen: true,
                        layer: 0,
                        z_order: 0,
                    }],
                    warnings: Vec::new(),
                },
            )
        }
    }

    struct PanickingComputerUseRuntime;

    impl ComputerUseRuntimeBridge for PanickingComputerUseRuntime {
        fn inspect_automation_window(
            &self,
            _request: ComputerUseInspectRequest,
        ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError> {
            panic!("computer/list_tray_menu must not inspect automation windows")
        }

        fn list_running_apps(
            &self,
            _request: ComputerUseListAppsRequest,
        ) -> Result<
            crate::computer_use::runtime_bridge::ComputerUseListAppsSnapshot,
            ComputerUseRuntimeError,
        > {
            panic!("computer/list_tray_menu must not list running apps")
        }

        fn list_app_windows(
            &self,
            _request: ComputerUseListAppWindowsRequest,
        ) -> Result<
            crate::computer_use::runtime_bridge::ComputerUseListAppWindowsSnapshot,
            ComputerUseRuntimeError,
        > {
            panic!("computer/list_tray_menu must not list app windows")
        }
    }

    #[test]
    fn computer_use_tool_definitions_are_registered() {
        let names: Vec<String> = get_computer_use_tool_definitions()
            .into_iter()
            .map(|tool| tool.name)
            .collect();

        assert_eq!(
            names,
            vec![
                COMPUTER_SEE_TOOL.to_string(),
                COMPUTER_LIST_WINDOWS_TOOL.to_string(),
                COMPUTER_GET_WINDOW_TOOL.to_string(),
                COMPUTER_GET_FOCUSED_WINDOW_TOOL.to_string(),
                COMPUTER_LIST_APPS_TOOL.to_string(),
                COMPUTER_GET_APP_TOOL.to_string(),
                COMPUTER_LIST_APP_WINDOWS_TOOL.to_string(),
                COMPUTER_GET_FRONTMOST_APP_TOOL.to_string(),
                COMPUTER_LIST_MENUS_TOOL.to_string(),
                COMPUTER_LIST_TRAY_MENU_TOOL.to_string(),
                COMPUTER_LIST_SCREENS_TOOL.to_string(),
                COMPUTER_LIST_PERMISSIONS_TOOL.to_string()
            ]
        );
    }

    #[test]
    fn computer_see_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_SEE_TOOL)
            .expect("computer/see tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn computer_list_windows_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_WINDOWS_TOOL)
            .expect("computer/list_windows tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_get_focused_window_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_GET_FOCUSED_WINDOW_TOOL)
            .expect("computer/get_focused_window tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_get_window_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_GET_WINDOW_TOOL)
            .expect("computer/get_window tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            properties
                .get("id")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str),
            Some("string")
        );
        assert_eq!(
            tool.input_schema.get("required"),
            Some(&serde_json::json!(["id"]))
        );
    }

    #[test]
    fn computer_list_apps_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_APPS_TOOL)
            .expect("computer/list_apps tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert!(properties.contains_key("includeHidden"));
        assert!(properties.contains_key("includeBackground"));
    }

    #[test]
    fn computer_get_app_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_GET_APP_TOOL)
            .expect("computer/get_app tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(properties.len(), 1);
        assert_eq!(
            properties
                .get("pid")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str),
            Some("integer")
        );
        assert_eq!(
            tool.input_schema.get("required"),
            Some(&serde_json::json!(["pid"]))
        );
    }

    #[test]
    fn computer_list_app_windows_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_APP_WINDOWS_TOOL)
            .expect("computer/list_app_windows tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(properties.len(), 1);
        assert_eq!(
            properties
                .get("pid")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str),
            Some("integer")
        );
        assert_eq!(
            tool.input_schema.get("required"),
            Some(&serde_json::json!(["pid"]))
        );
    }

    #[test]
    fn computer_get_frontmost_app_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_GET_FRONTMOST_APP_TOOL)
            .expect("computer/get_frontmost_app tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_list_menus_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_MENUS_TOOL)
            .expect("computer/list_menus tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_list_tray_menu_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_TRAY_MENU_TOOL)
            .expect("computer/list_tray_menu tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_list_permissions_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_PERMISSIONS_TOOL)
            .expect("computer/list_permissions tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn computer_list_screens_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_LIST_SCREENS_TOOL)
            .expect("computer/list_screens tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            tool.input_schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|properties| properties.is_empty()),
            Some(true)
        );
    }

    #[test]
    fn is_computer_use_tool_matches_only_computer_namespace() {
        assert!(is_computer_use_tool("computer/see"));
        assert!(!is_computer_use_tool("computer-use/see"));
        assert!(!is_computer_use_tool("kit/state"));
    }

    #[test]
    fn computer_see_without_runtime_returns_tool_error() {
        let result = handle_computer_use_tool_call(COMPUTER_SEE_TOOL, &serde_json::json!({}), None);

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("runtime_unavailable"));
    }

    #[test]
    fn computer_list_apps_without_runtime_returns_tool_error() {
        let result =
            handle_computer_use_tool_call(COMPUTER_LIST_APPS_TOOL, &serde_json::json!({}), None);

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("runtime_unavailable"));
    }

    #[test]
    fn computer_get_app_without_runtime_returns_tool_error() {
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_APP_TOOL,
            &serde_json::json!({ "pid": 101 }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("runtime_unavailable"));
    }

    #[test]
    fn computer_list_tray_menu_without_runtime_returns_snapshot() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_TRAY_MENU_TOOL,
            &serde_json::json!({}),
            None,
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_tray_menu json");
        assert_eq!(value["schemaVersion"], serde_json::json!(1));
        assert_eq!(value["source"], "scriptKitTrayMenuModel");
        assert_eq!(value["owner"]["scope"], "ownTrayMenuOnly");
        assert!(value["sections"].is_array());
        assert!(value["warnings"].is_array());
    }

    #[test]
    fn computer_see_with_runtime_returns_raw_snapshot() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({
                "target": { "type": "focused" },
                "hiDpi": true,
                "probes": [
                    { "x": 10, "y": 20 },
                    { "x": 30, "y": 40 }
                ]
            }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);

        let snapshot: AutomationInspectSnapshot =
            serde_json::from_str(&result.content[0].text).expect("automation inspect snapshot");
        assert_eq!(snapshot.schema_version, AUTOMATION_INSPECT_SCHEMA_VERSION);
        assert_eq!(snapshot.window_id, "main:0");
        assert!(!result.content[0].text.contains("\"action\""));
    }

    #[test]
    fn computer_list_apps_with_runtime_returns_running_app_snapshot() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_APPS_TOOL,
            &serde_json::json!({
                "includeHidden": true,
                "includeBackground": true
            }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_apps json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_APPS_SCHEMA_VERSION)
        );
        assert_eq!(value["frontmostPid"], 101);

        let apps = value["apps"].as_array().expect("apps array");
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0]["pid"], 101);
        assert_eq!(apps[0]["bundleId"], "com.apple.Terminal");
        assert_eq!(apps[0]["name"], "Terminal");
        assert_eq!(apps[0]["isActive"], true);
        assert_eq!(apps[0]["isHidden"], false);
        assert_eq!(apps[0]["activationPolicy"], "regular");
        assert_eq!(apps[1]["bundleId"], serde_json::Value::Null);
        assert!(!result.content[0].text.contains("\"launch\""));
        assert!(!result.content[0].text.contains("\"quit\""));
        assert!(!result.content[0].text.contains("\"focus\""));
    }

    #[test]
    fn computer_get_app_returns_running_app_by_pid() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_APP_TOOL,
            &serde_json::json!({ "pid": 101 }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_app json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_APPS_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "nsWorkspaceRunningApplications");
        assert_eq!(value["scope"], "runningAppPid");
        assert_eq!(value["status"], "found");
        assert_eq!(value["app"]["pid"], 101);
        assert_eq!(value["app"]["bundleId"], "com.apple.Terminal");
        assert_eq!(value["app"]["name"], "Terminal");
        assert_eq!(value["app"]["isActive"], true);
        assert_eq!(value["app"]["isHidden"], false);
        assert_eq!(value["app"]["activationPolicy"], "regular");
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"action\"",
            "\"click\"",
            "\"execute\"",
            "\"focus\"",
            "\"activate\"",
            "\"launch\"",
            "\"quit\"",
            "\"hide\"",
            "\"terminate\"",
            "\"forceTerminate\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/get_app result must not expose executable fields; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_get_app_returns_not_found_for_unknown_pid() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_APP_TOOL,
            &serde_json::json!({ "pid": 999 }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_app json");
        assert_eq!(value["source"], "nsWorkspaceRunningApplications");
        assert_eq!(value["scope"], "runningAppPid");
        assert_eq!(value["status"], "notFound");
        assert!(value["app"].is_null());
        assert!(value["warnings"]
            .as_array()
            .is_some_and(|warnings| warnings.is_empty()));
    }

    #[test]
    fn computer_see_rejects_max_elements_instead_of_truncating() {
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({ "maxElements": 1 }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_see_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({ "unknown": true }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_list_windows_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_WINDOWS_TOOL,
            &serde_json::json!({ "target": { "type": "focused" } }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_get_focused_window_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!({ "target": { "type": "focused" } }),
            serde_json::json!({ "focus": true }),
            serde_json::json!({ "activate": true }),
            serde_json::json!({ "refresh": true }),
            serde_json::json!({ "click": true }),
            serde_json::json!({ "id": "main" }),
        ] {
            let result =
                handle_computer_use_tool_call(COMPUTER_GET_FOCUSED_WINDOW_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_get_window_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!(null),
            serde_json::json!([]),
            serde_json::json!({}),
            serde_json::json!({ "id": 123 }),
            serde_json::json!({ "id": null }),
            serde_json::json!({ "target": { "type": "focused" } }),
            serde_json::json!({ "id": "main", "focus": true }),
            serde_json::json!({ "id": "main", "activate": true }),
            serde_json::json!({ "id": "main", "refresh": true }),
            serde_json::json!({ "id": "main", "click": true }),
            serde_json::json!({ "id": "main", "includeElements": true }),
            serde_json::json!({ "id": "main", "screenshot": true }),
        ] {
            let result = handle_computer_use_tool_call(COMPUTER_GET_WINDOW_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_get_window_ignores_supplied_runtime() {
        let runtime = PanickingComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_WINDOW_TOOL,
            &serde_json::json!({ "id": "missing-window-id-for-runtime-test" }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_window json");
        assert_eq!(value["source"], "automationWindowRegistry");
        assert_eq!(value["status"], "notFound");
        assert!(value["window"].is_null());
    }

    #[test]
    fn computer_list_apps_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_APPS_TOOL,
            &serde_json::json!({ "launch": true }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_get_app_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!(null),
            serde_json::json!([]),
            serde_json::json!({}),
            serde_json::json!({ "pid": "101" }),
            serde_json::json!({ "pid": 101, "focus": true }),
            serde_json::json!({ "pid": 101, "activate": true }),
            serde_json::json!({ "pid": 101, "launch": true }),
            serde_json::json!({ "pid": 101, "quit": true }),
            serde_json::json!({ "pid": 101, "hide": true }),
            serde_json::json!({ "pid": 101, "includeWindows": true }),
        ] {
            let result = handle_computer_use_tool_call(COMPUTER_GET_APP_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_list_app_windows_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!(null),
            serde_json::json!([]),
            serde_json::json!({}),
            serde_json::json!({ "pid": "101" }),
            serde_json::json!({ "pid": 101, "focus": true }),
            serde_json::json!({ "pid": 101, "activate": true }),
            serde_json::json!({ "pid": 101, "move": true }),
            serde_json::json!({ "pid": 101, "resize": true }),
            serde_json::json!({ "pid": 101, "screenshot": true }),
        ] {
            let result =
                handle_computer_use_tool_call(COMPUTER_LIST_APP_WINDOWS_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_get_frontmost_app_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!({ "refresh": true }),
            serde_json::json!({ "focus": true }),
            serde_json::json!({ "activate": true }),
            serde_json::json!({ "pid": 123 }),
            serde_json::json!({ "bundleId": "com.apple.Safari" }),
            serde_json::json!({ "includeMenus": true }),
            serde_json::json!({ "click": true }),
        ] {
            let result =
                handle_computer_use_tool_call(COMPUTER_GET_FRONTMOST_APP_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_list_menus_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!({ "pid": 101 }),
            serde_json::json!({ "bundleId": "com.apple.Terminal" }),
            serde_json::json!({ "refresh": true }),
            serde_json::json!({ "target": "frontmost" }),
            serde_json::json!({ "click": true }),
            serde_json::json!({ "path": [0, 1] }),
            serde_json::json!({ "includeDisabled": true }),
        ] {
            let result = handle_computer_use_tool_call(COMPUTER_LIST_MENUS_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_list_tray_menu_rejects_bad_arguments() {
        for arguments in [
            serde_json::json!({ "click": true }),
            serde_json::json!({ "execute": true }),
            serde_json::json!({ "index": 0 }),
            serde_json::json!({ "itemName": "GitHub" }),
            serde_json::json!({ "actionId": "tray.open_github" }),
            serde_json::json!({ "open": true }),
            serde_json::json!({ "target": "menubar" }),
            serde_json::json!({ "includeGlobalStatusItems": true }),
        ] {
            let result =
                handle_computer_use_tool_call(COMPUTER_LIST_TRAY_MENU_TOOL, &arguments, None);

            assert_eq!(result.is_error, Some(true));
            assert!(result.content[0].text.contains("invalid_arguments"));
        }
    }

    #[test]
    fn computer_list_permissions_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_PERMISSIONS_TOOL,
            &serde_json::json!({ "request": true }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_list_screens_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_SCREENS_TOOL,
            &serde_json::json!({ "move": true }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_list_windows_returns_registry_snapshot_without_runtime() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let id = format!("mcp-list-windows-test-{nonce}");

        crate::windows::upsert_automation_window(AutomationWindowInfo {
            id: id.clone(),
            kind: AutomationWindowKind::Notes,
            title: Some("MCP List Windows Test".to_string()),
            focused: false,
            visible: true,
            semantic_surface: Some("notes".to_string()),
            bounds: Some(AutomationWindowBounds {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }),
            parent_window_id: None,
            parent_kind: None,
        });

        let result =
            handle_computer_use_tool_call(COMPUTER_LIST_WINDOWS_TOOL, &serde_json::json!({}), None);

        crate::windows::remove_automation_window(&id);

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_windows json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(AUTOMATION_WINDOW_SCHEMA_VERSION)
        );
        assert!(value["focusedWindowId"].is_null() || value["focusedWindowId"].is_string());

        let windows = value["windows"].as_array().expect("windows array");
        let window = windows
            .iter()
            .find(|window| window["id"] == id)
            .expect("registered test window should be listed");
        assert_eq!(window["kind"], "notes");
        assert_eq!(window["visible"], true);
        assert_eq!(window["semanticSurface"], "notes");
    }

    #[test]
    fn computer_get_focused_window_returns_registry_snapshot_without_runtime() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let id = format!("mcp-focused-window-test-{nonce}");

        crate::windows::upsert_automation_window(AutomationWindowInfo {
            id: id.clone(),
            kind: AutomationWindowKind::Notes,
            title: Some("MCP Focused Window Test".to_string()),
            focused: false,
            visible: true,
            semantic_surface: Some("notes".to_string()),
            bounds: Some(AutomationWindowBounds {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }),
            parent_window_id: None,
            parent_kind: None,
        });
        assert!(crate::windows::set_automation_focus(&id));

        let result = handle_computer_use_tool_call(
            COMPUTER_GET_FOCUSED_WINDOW_TOOL,
            &serde_json::json!({}),
            None,
        );

        crate::windows::remove_automation_window(&id);

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_focused_window json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(AUTOMATION_WINDOW_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "automationWindowRegistry");
        assert_eq!(value["scope"], "focusedAutomationWindow");
        assert_eq!(value["status"], "focused");
        assert_eq!(value["focusedWindowId"], id);
        assert_eq!(value["window"]["id"], id);
        assert_eq!(value["window"]["kind"], "notes");
        assert_eq!(value["window"]["focused"], true);
        assert_eq!(value["window"]["visible"], true);
        assert_eq!(value["window"]["semanticSurface"], "notes");
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"action\"",
            "\"click\"",
            "\"execute\"",
            "\"focus\"",
            "\"activate\"",
            "\"launch\"",
            "\"quit\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/get_focused_window result must not expose executable fields; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_get_window_returns_registry_snapshot_by_id_without_runtime() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let id = format!("mcp-get-window-test-{nonce}");

        crate::windows::upsert_automation_window(AutomationWindowInfo {
            id: id.clone(),
            kind: AutomationWindowKind::Notes,
            title: Some("MCP Get Window Test".to_string()),
            focused: false,
            visible: true,
            semantic_surface: Some("notes".to_string()),
            bounds: Some(AutomationWindowBounds {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }),
            parent_window_id: None,
            parent_kind: None,
        });

        let result = handle_computer_use_tool_call(
            COMPUTER_GET_WINDOW_TOOL,
            &serde_json::json!({ "id": id.clone() }),
            None,
        );

        crate::windows::remove_automation_window(&id);

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_window json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(AUTOMATION_WINDOW_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "automationWindowRegistry");
        assert_eq!(value["status"], "found");
        assert_eq!(value["window"]["id"], id);
        assert_eq!(value["window"]["kind"], "notes");
        assert_eq!(value["window"]["visible"], true);
        assert_eq!(value["window"]["semanticSurface"], "notes");
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"action\"",
            "\"click\"",
            "\"execute\"",
            "\"focus\"",
            "\"activate\"",
            "\"launch\"",
            "\"quit\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/get_window result must not expose executable fields; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_get_window_returns_not_found_for_unknown_id_without_runtime() {
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_WINDOW_TOOL,
            &serde_json::json!({ "id": "missing-window-id-for-test" }),
            None,
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_window json");
        assert_eq!(value["source"], "automationWindowRegistry");
        assert_eq!(value["status"], "notFound");
        assert!(value["window"].is_null());
        assert!(value["warnings"]
            .as_array()
            .is_some_and(|warnings| warnings.is_empty()));
    }

    #[test]
    fn computer_get_frontmost_app_returns_cached_snapshot_without_runtime() {
        let result = handle_computer_use_tool_call(
            COMPUTER_GET_FRONTMOST_APP_TOOL,
            &serde_json::json!({}),
            None,
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid get_frontmost_app json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_FRONTMOST_APP_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "frontmostAppTrackerCache");
        assert_eq!(value["scope"], "lastNonScriptKitApp");
        assert!(value["status"] == "tracked" || value["status"] == "noTrackedApp");
        assert!(value["app"].is_null() || value["app"].is_object());
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"click\"",
            "\"execute\"",
            "\"focus\"",
            "\"activate\"",
            "\"launch\"",
            "\"quit\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/get_frontmost_app result must not expose executable fields; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_list_app_windows_returns_runtime_snapshot() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_APP_WINDOWS_TOOL,
            &serde_json::json!({ "pid": 101 }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_app_windows json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_APP_WINDOWS_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "coreGraphicsWindowList");
        assert_eq!(value["scope"], "runningAppPid");
        assert_eq!(value["status"], "found");
        assert_eq!(value["app"]["pid"], 101);
        assert_eq!(value["windows"][0]["nativeWindowId"], 98765);
        assert_eq!(value["windows"][0]["title"], "Terminal");
        assert_eq!(value["windows"][0]["bounds"]["width"], 300);
        assert_eq!(value["windows"][0]["isOnScreen"], true);
        assert_eq!(value["windows"][0]["layer"], 0);
        assert_eq!(value["windows"][0]["zOrder"], 0);
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"action\"",
            "\"click\"",
            "\"execute\"",
            "\"focus\"",
            "\"activate\"",
            "\"launch\"",
            "\"quit\"",
            "\"move\"",
            "\"resize\"",
            "\"setBounds\"",
            "\"axElementPath\"",
            "\"AXPress\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/list_app_windows result must not expose executable fields; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_list_app_windows_requires_runtime() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_APP_WINDOWS_TOOL,
            &serde_json::json!({ "pid": 101 }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("runtime_unavailable"));
    }

    #[test]
    fn computer_list_menus_returns_cached_snapshot_without_runtime() {
        let result =
            handle_computer_use_tool_call(COMPUTER_LIST_MENUS_TOOL, &serde_json::json!({}), None);

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_menus json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_MENUS_SCHEMA_VERSION)
        );
        assert_eq!(value["source"], "frontmostAppTrackerCache");
        assert!(value["cache"]["status"].is_string());
        assert!(value["cache"]["isFetching"].is_boolean());
        assert!(value["menus"].is_array());
        assert!(value["warnings"].is_array());

        for forbidden in [
            "\"action\"",
            "\"click\"",
            "\"press\"",
            "\"execute\"",
            "\"axElementPath\"",
            "\"AXPress\"",
        ] {
            assert!(
                !result.content[0].text.contains(forbidden),
                "computer/list_menus result must not expose menu action handles; found {forbidden}"
            );
        }
    }

    #[test]
    fn computer_list_tray_menu_with_runtime_returns_snapshot() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_TRAY_MENU_TOOL,
            &serde_json::json!({}),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_tray_menu json");
        assert_eq!(value["schemaVersion"], serde_json::json!(1));
        assert_eq!(value["source"], "scriptKitTrayMenuModel");
        assert_eq!(value["owner"]["scope"], "ownTrayMenuOnly");
        assert!(value["sections"].is_array());
        assert!(!result.content[0].text.contains("\"click\""));
        assert!(!result.content[0].text.contains("\"execute\""));
    }

    #[test]
    fn computer_list_tray_menu_ignores_supplied_runtime() {
        let runtime = PanickingComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_TRAY_MENU_TOOL,
            &serde_json::json!({}),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);
    }

    #[test]
    fn computer_list_permissions_returns_status_without_runtime() {
        let result = handle_computer_use_tool_call(
            COMPUTER_LIST_PERMISSIONS_TOOL,
            &serde_json::json!({}),
            None,
        );

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid permissions json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_PERMISSIONS_SCHEMA_VERSION)
        );

        let permissions = value["permissions"].as_array().expect("permissions array");
        let accessibility = permissions
            .iter()
            .find(|permission| permission["id"] == "accessibility")
            .expect("accessibility status");
        assert_eq!(accessibility["name"], "Accessibility");
        assert!(accessibility["granted"].is_boolean());
        assert!(accessibility["status"] == "granted" || accessibility["status"] == "notGranted");

        let screen_recording = permissions
            .iter()
            .find(|permission| permission["id"] == "screenRecording")
            .expect("screen recording status");
        assert_eq!(screen_recording["name"], "Screen Recording");
        assert!(
            screen_recording["status"] == "granted"
                || screen_recording["status"] == "notGranted"
                || screen_recording["status"] == "unknown"
        );
        assert!(!result.content[0].text.contains("requestAccessibility"));
        assert!(!result.content[0].text.contains("openSettings"));
    }

    #[test]
    fn computer_list_screens_returns_screen_snapshot_without_runtime() {
        let result =
            handle_computer_use_tool_call(COMPUTER_LIST_SCREENS_TOOL, &serde_json::json!({}), None);

        assert_eq!(result.is_error, None);
        let value: serde_json::Value =
            serde_json::from_str(&result.content[0].text).expect("valid list_screens json");
        assert_eq!(
            value["schemaVersion"],
            serde_json::json!(COMPUTER_SCREENS_SCHEMA_VERSION)
        );

        let screens = value["screens"].as_array().expect("screens array");
        for screen in screens {
            assert!(screen["displayId"].is_number());
            assert!(screen["name"].is_string());
            assert!(screen["isPrimary"].is_boolean());
            assert!(screen["bounds"]["width"].is_number());
            assert!(screen["bounds"]["height"].is_number());
            assert!(screen["visibleBounds"]["width"].is_number());
            assert!(screen["visibleBounds"]["height"].is_number());
        }
    }
}
