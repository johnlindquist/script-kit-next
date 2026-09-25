//! The launcher's commands and the live screen, as the intent engine sees them. Lives in
//! the binary so it reads the binary's own tracker, window, and command state.

use std::hash::{Hash, Hasher};
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

/// What's on screen now. Selected text and clipboard contents are never read here:
/// steps that act on them read them when the plan runs.
#[cfg(target_os = "macos")]
pub fn live_screen(last_action: Option<LastAction>) -> ScreenContext {
    let front = crate::frontmost_app_tracker::get_last_real_app();
    let menus = menus_from_bar(&crate::frontmost_app_tracker::get_cached_menu_items());
    let mut open_windows: Vec<OpenWindow> = crate::window_control::list_windows()
        .unwrap_or_default()
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

/// Cheap fingerprint of the screen, so a plan recalled for the same words is only
/// reused on the same app.
pub fn screen_key() -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    #[cfg(target_os = "macos")]
    if let Some(app) = crate::frontmost_app_tracker::get_last_real_app() {
        app.bundle_id.hash(&mut hasher);
        app.window_title.hash(&mut hasher);
    }
    hasher.finish()
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

/// Apply a window step to the frontmost window of `app`.
#[cfg(target_os = "macos")]
pub fn run_window_step(op: &WindowOp, app: &str) -> anyhow::Result<()> {
    use crate::window_control as wc;
    let action = op
        .action()
        .ok_or_else(|| anyhow::anyhow!("unknown window operation {}", op.label))?;
    let window = wc::list_windows()?
        .into_iter()
        .filter(|w| w.app.eq_ignore_ascii_case(app))
        .min_by_key(|w| w.global_order)
        .ok_or_else(|| anyhow::anyhow!("no open window for {app}"))?;
    match action {
        WindowAction::Tile(position) => wc::tile_window(window.id, binary_tile(position)),
        WindowAction::NextDisplay => wc::move_to_next_display(window.id),
        WindowAction::PreviousDisplay => wc::move_to_previous_display(window.id),
        WindowAction::Minimize => wc::minimize_window(window.id),
        WindowAction::Close => wc::close_window(window.id),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn run_window_step(_op: &WindowOp, _app: &str) -> anyhow::Result<()> {
    anyhow::bail!("window steps need macOS")
}
