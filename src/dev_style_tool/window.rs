use std::sync::{Mutex, OnceLock};

use gpui::{
    point, px, size, App, AppContext, Bounds, Entity, Pixels, WindowBounds, WindowHandle,
    WindowKind, WindowOptions,
};
use gpui_component::Root;

use crate::protocol::{AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind};
use crate::ScriptListApp;

pub(crate) const DEV_STYLE_TOOL_ENV: &str = "SCRIPT_KIT_STYLE_DEVTOOLS";
pub(crate) const DEV_STYLE_TOOL_AUTOMATION_ID: &str = "dev-style-tool";
pub(crate) const DEV_STYLE_TOOL_TITLE: &str = "Script Kit Dev Style Tool";

static DEV_STYLE_TOOL_HANDLE: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

fn handle_slot() -> &'static Mutex<Option<WindowHandle<Root>>> {
    DEV_STYLE_TOOL_HANDLE.get_or_init(|| Mutex::new(None))
}

pub(crate) fn dev_style_tool_enabled() -> bool {
    cfg!(debug_assertions)
        && matches!(
            std::env::var(DEV_STYLE_TOOL_ENV).ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE")
        )
}

pub(crate) fn is_dev_style_tool_open() -> bool {
    handle_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_some()
}

#[cfg(debug_assertions)]
pub(crate) fn maybe_open_startup_sidecar(
    main_window: WindowHandle<Root>,
    main_app: Entity<ScriptListApp>,
    cx: &mut App,
) {
    if dev_style_tool_enabled() {
        if let Err(error) = open_dev_style_tool_window(main_window, main_app, cx) {
            tracing::warn!(
                target: "script_kit::dev_style_tool",
                ?error,
                "failed to open dev style tool sidecar"
            );
        }
    }
}

pub(crate) fn open_dev_style_tool_window(
    main_window: WindowHandle<Root>,
    main_app: Entity<ScriptListApp>,
    cx: &mut App,
) -> anyhow::Result<WindowHandle<Root>> {
    if let Some(existing) = *handle_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
    {
        if existing
            .update(cx, |_root, _window, cx| {
                cx.notify();
            })
            .is_ok()
        {
            return Ok(existing);
        }
    }

    let bounds = Bounds {
        origin: point(px(620.0), px(120.0)),
        size: size(px(1180.0), px(820.0)),
    };
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: true,
            is_resizable: true,
            window_min_size: Some(size(px(760.0), px(600.0))),
            show: true,
            focus: false,
            kind: WindowKind::Normal,
            ..Default::default()
        },
        move |window, cx| {
            let view = cx.new(|cx| {
                super::render::DevStyleToolApp::new(main_window, main_app.clone(), window, cx)
            });
            window.on_window_should_close(cx, |_window, _cx| {
                clear_dev_style_tool_handle();
                crate::windows::remove_automation_window(DEV_STYLE_TOOL_AUTOMATION_ID);
                true
            });
            cx.new(|cx| Root::new(view, window, cx))
        },
    )?;

    *handle_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(window);
    register_automation_window(Some(automation_bounds_from_gpui(bounds)));
    Ok(window)
}

fn clear_dev_style_tool_handle() {
    *handle_slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = None;
}

fn register_automation_window(bounds: Option<AutomationWindowBounds>) {
    crate::windows::upsert_automation_window(AutomationWindowInfo {
        id: DEV_STYLE_TOOL_AUTOMATION_ID.to_string(),
        kind: AutomationWindowKind::DevStyleTool,
        title: Some(DEV_STYLE_TOOL_TITLE.to_string()),
        focused: false,
        visible: true,
        semantic_surface: Some("devStyleTool".to_string()),
        bounds,
        parent_window_id: Some("main".to_string()),
        parent_kind: Some(AutomationWindowKind::Main),
        pid: Some(std::process::id()),
    });
}

fn automation_bounds_from_gpui(bounds: Bounds<Pixels>) -> AutomationWindowBounds {
    AutomationWindowBounds {
        x: f32::from(bounds.origin.x) as f64,
        y: f32::from(bounds.origin.y) as f64,
        width: f32::from(bounds.size.width) as f64,
        height: f32::from(bounds.size.height) as f64,
    }
}
