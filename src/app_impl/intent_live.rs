//! The launcher's commands and the live screen, as the intent engine sees them. Lives in
//! the binary so it reads the binary's own tracker, window, and command state.

use std::sync::Arc;

use crate::app_launcher::AppInfo;
use crate::builtins::BuiltInEntry;
use crate::menu_bar::MenuBarItem;
use crate::scripts::{Script, Scriptlet};

use crate::intent::{
    native_window_ops, Catalog, Command, CommandKind, FrontApp, LastAction, MenuCommand,
    OpenWindow, ScreenContext, WindowAction, WindowOp,
};

/// Scriptlets from this plugin rewrite the selected text.
pub const TEXT_TOOL_PLUGIN: &str = "ai-text-tools";
const OPEN_WINDOW_LIMIT: usize = 24;

pub fn script_command_id(script: &Script) -> String {
    format!("script:{}", script.path.display())
}

pub fn scriptlet_command_id(scriptlet: &Scriptlet) -> String {
    format!("scriptlet:{}:{}", scriptlet.plugin_id, scriptlet.name)
}

pub fn builtin_command_id(entry: &BuiltInEntry) -> String {
    format!("builtin:{}", entry.id)
}

pub fn app_command_id(app: &AppInfo) -> String {
    format!("app:{}", app.path.display())
}

/// Every command the launcher can run, with ids that map back to its rows.
pub fn catalog_from_launcher(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
) -> Catalog {
    let mut commands = Vec::new();
    let mut text_tools = Vec::new();
    for entry in builtins {
        // The app_command pathway already reaches the front app's menus.
        if entry.id == "builtin/do-in-current-app" {
            continue;
        }
        commands.push(Command {
            id: builtin_command_id(entry),
            kind: CommandKind::Builtin,
            name: entry.name.clone(),
            description: entry.description.clone(),
            plugin: None,
            capture_targets: Vec::new(),
            code: None,
        });
    }
    for script in scripts {
        commands.push(Command {
            id: script_command_id(script),
            kind: CommandKind::Script,
            name: script.name.clone(),
            description: script.description.clone().unwrap_or_default(),
            plugin: Some(script.plugin_id.clone()),
            capture_targets: crate::menu_syntax::first_concrete_capture_target_for_script(script)
                .into_iter()
                .collect(),
            code: None,
        });
    }
    for scriptlet in scriptlets {
        if scriptlet.name.contains("{{") {
            continue;
        }
        let command = Command {
            id: scriptlet_command_id(scriptlet),
            kind: CommandKind::Scriptlet,
            name: scriptlet.name.clone(),
            description: scriptlet
                .description
                .clone()
                .or_else(|| scriptlet.plugin_title.clone())
                .unwrap_or_else(|| scriptlet.plugin_id.clone()),
            plugin: Some(scriptlet.plugin_id.clone()),
            capture_targets: Vec::new(),
            code: None,
        };
        if scriptlet.plugin_id == TEXT_TOOL_PLUGIN {
            text_tools.push(command.clone());
        }
        commands.push(command);
    }
    for app in apps {
        commands.push(Command {
            id: app_command_id(app),
            kind: CommandKind::App,
            name: app.name.clone(),
            description: "Application".into(),
            plugin: None,
            capture_targets: Vec::new(),
            code: None,
        });
    }
    Catalog {
        commands,
        window_ops: native_window_ops(),
        text_tools,
    }
}

/// Clickable leaf items of the front app's menu bar, skipping the Apple menu.
pub fn menus_from_bar(items: &[MenuBarItem]) -> Vec<MenuCommand> {
    fn walk(item: &MenuBarItem, path: &mut Vec<String>, out: &mut Vec<MenuCommand>) {
        if item.title.trim().is_empty() || item.title == "-" || item.is_separator() || !item.enabled
        {
            return;
        }
        path.push(item.title.clone());
        if item.children.is_empty() {
            if path.len() >= 2 {
                out.push(MenuCommand {
                    menu: path[0].clone(),
                    label: path[1..].join(" › "),
                    shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
                });
            }
        } else {
            for child in &item.children {
                walk(child, path, out);
            }
        }
        path.pop();
    }
    let mut out = Vec::new();
    for item in items.iter().skip(1) {
        walk(item, &mut Vec::new(), &mut out);
    }
    out
}

use crate::frontmost_app_tracker::TrackedApp;
use crate::intent::window_target::{
    ObservedWindow, WindowIdentity, WindowScope, WindowSnapshot, WindowTarget,
};

#[cfg(target_os = "macos")]
fn window_snapshot(
    windows: &[crate::window_control::WindowInfo],
    front: Option<&TrackedApp>,
) -> WindowSnapshot {
    let own = std::process::id() as i32;
    let active = crate::platform::accessibility::app_identity::current_frontmost_app_identity();
    let front = front.filter(|app| {
        active.process_id == Some(own)
            || (active.process_id == Some(app.pid)
                && active.bundle_id.as_deref() == Some(&app.bundle_id))
    });
    WindowSnapshot {
        origin: front
            .and_then(|app| Some((app.pid, app.bundle_id.clone(), app.window_title.clone()?))),
        foreign_front: windows
            .iter()
            .find(|w| w.is_frontmost_app && w.pid != own)
            .map(|w| w.pid),
        // Don't truncate this set to the classifier's label limit; a later duplicate counts.
        rows: windows
            .iter()
            .filter(|w| w.pid != own)
            .map(|w| ObservedWindow {
                identity: WindowIdentity {
                    pid: w.pid,
                    bundle: w.bundle_id.clone().unwrap_or_default(),
                    app: w.app.clone(),
                    title: w.title.clone(),
                    native_id: w.handle.native_window_id.map_or(0, |id| id.0),
                    generation: w.handle.registry_generation,
                    nonce: w.handle.nonce,
                },
                focused: w.is_focused,
                minimized: w.is_minimized,
            })
            .collect(),
        scope: WindowScope::Screen,
    }
}

/// Populate a window subject on the background executor, not when its action is clicked.
#[cfg(target_os = "macos")]
pub(crate) fn observe_front(app: &TrackedApp) -> Option<WindowTarget> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery).ok()?;
    let windows = crate::window_control::list_windows().ok()?;
    let current = crate::frontmost_app_tracker::get_last_real_app()?;
    if current != *app {
        return None;
    }
    window_snapshot(&windows, Some(app)).bind(&app.name, true)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn observe_front(_: &TrackedApp) -> Option<WindowTarget> {
    None
}

/// What's on screen now. Selected text and clipboard contents are never read here:
/// steps that act on them read them when the plan runs.
#[cfg(target_os = "macos")]
pub fn live_screen(last_action: Option<LastAction>) -> ScreenContext {
    let front = crate::frontmost_app_tracker::get_last_real_app();
    let menus = menus_from_bar(&crate::frontmost_app_tracker::get_cached_menu_items());
    let windows = crate::window_control::list_windows().unwrap_or_default();
    let filename_window = front.as_ref().and_then(|front| {
        let title = front.window_title.as_deref().filter(|t| !t.is_empty())?;
        let mut found = windows.iter().filter(|w| {
            w.pid == front.pid
                && w.bundle_id.as_deref() == Some(&front.bundle_id)
                && w.title == title
                && w.is_focused
                && !w.is_minimized
        });
        let w = found.next()?;
        if found.next().is_some() {
            return None;
        }
        Some((u32::try_from(w.pid).ok()?, w.handle.native_window_id?.0))
    });
    let window_snapshot = window_snapshot(&windows, front.as_ref());
    let mut open_windows: Vec<OpenWindow> = windows
        .into_iter()
        .filter(|w| !w.app.is_empty() && w.app != "Script Kit")
        .map(|w| OpenWindow {
            app: w.app,
            title: w.title,
        })
        .collect();
    open_windows.truncate(OPEN_WINDOW_LIMIT);
    ScreenContext {
        id: "live".into(),
        filename_window,
        window_snapshot,
        front_app: front
            .as_ref()
            .map(|app| FrontApp {
                name: app.name.clone(),
                bundle_id: app.bundle_id.clone(),
            })
            .unwrap_or_default(),
        window_title: front.and_then(|app| app.window_title).unwrap_or_default(),
        open_windows,
        menus,
        last_action,
        subject_read_at_run: true,
        ..ScreenContext::default()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn live_screen(last_action: Option<LastAction>) -> ScreenContext {
    ScreenContext {
        id: "live".into(),
        last_action,
        subject_read_at_run: true,
        ..ScreenContext::default()
    }
}

/// Cached app/menu identity; never enumerates windows on the UI thread.
pub fn screen_key() -> u64 {
    crate::frontmost_app_tracker::intent_context_key()
}

/// The catalog comes from the library crate, whose `TilePosition` is a different type
/// from the binary's copy of the same enum.
#[cfg(target_os = "macos")]
fn binary_tile(
    position: script_kit_gpui::window_control::TilePosition,
) -> crate::window_control::TilePosition {
    macro_rules! same {
        ($($variant:ident),* $(,)?) => {
            match position {
                $(script_kit_gpui::window_control::TilePosition::$variant => {
                    crate::window_control::TilePosition::$variant
                })*
            }
        };
    }
    same!(
        LeftHalf,
        RightHalf,
        TopHalf,
        BottomHalf,
        TopLeft,
        TopRight,
        BottomLeft,
        BottomRight,
        TopLeftSixth,
        TopCenterSixth,
        TopRightSixth,
        BottomLeftSixth,
        BottomCenterSixth,
        BottomRightSixth,
        LeftThird,
        CenterThird,
        RightThird,
        TopThird,
        MiddleThird,
        BottomThird,
        FirstTwoThirds,
        LastTwoThirds,
        TopTwoThirds,
        BottomTwoThirds,
        Center,
        AlmostMaximize,
        Fullscreen,
        NextDisplay,
        PreviousDisplay,
    )
}

/// Re-observe immediately before dispatch. Never retry by app name, title or numeric list ID.
#[cfg(target_os = "macos")]
pub fn run_window_step(
    op: &WindowOp,
    app: &str,
    target: Option<&WindowTarget>,
) -> anyhow::Result<()> {
    use crate::window_control::{self as wc, LegacyWindowAction as Action};
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
    let action = op
        .action()
        .ok_or_else(|| anyhow::anyhow!("unknown window operation {}", op.label))?;
    if action == WindowAction::Quit {
        let script = format!(
            r#"tell application "{}" to quit"#,
            crate::utils::escape_applescript_string(app)
        );
        return crate::platform::run_osascript(&script, "intent_quit_app").map(|_| ());
    }
    let target = target
        .filter(|t| t.identity.app.eq_ignore_ascii_case(app))
        .ok_or_else(|| anyhow::anyhow!("Choose one window and try again"))?;
    let windows = wc::list_windows()?;
    let front = crate::frontmost_app_tracker::get_last_real_app();
    anyhow::ensure!(
        window_snapshot(&windows, front.as_ref()).confirms(target),
        "The window changed"
    );
    let w = &target.identity;
    let handle = wc::WindowHandle {
        pid: w.pid,
        native_window_id: Some(wc::NativeWindowId(w.native_id)),
        registry_generation: w.generation,
        nonce: w.nonce,
    };
    wc::execute_observed_action(handle, |window_id| match action {
        WindowAction::Tile(position) => Action::Tile {
            window_id,
            position: binary_tile(position),
        },
        WindowAction::NextDisplay => Action::MoveToNextDisplay { window_id },
        WindowAction::PreviousDisplay => Action::MoveToPreviousDisplay { window_id },
        WindowAction::Minimize => Action::Minimize { window_id },
        WindowAction::Close => Action::Close { window_id },
        WindowAction::Quit => unreachable!("quit is an application action"),
    })
}

/// Opens `url` in `app`, or in the default browser.
#[cfg(target_os = "macos")]
pub fn open_url(url: &str, app: Option<&str>) -> anyhow::Result<()> {
    let mut command = std::process::Command::new("/usr/bin/open");
    if let Some(app) = app {
        command.arg("-a").arg(app);
    }
    let output = command.arg(url).output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("{}", stderr.trim())
}

#[cfg(not(target_os = "macos"))]
pub fn open_url(url: &str, app: Option<&str>) -> anyhow::Result<()> {
    match app {
        Some(app) => open::with(url, app),
        None => open::that(url),
    }
    .map_err(anyhow::Error::from)
}

#[cfg(not(target_os = "macos"))]
pub fn run_window_step(
    _op: &WindowOp,
    _app: &str,
    _target: Option<&WindowTarget>,
) -> anyhow::Result<()> {
    anyhow::bail!("window steps need macOS")
}
