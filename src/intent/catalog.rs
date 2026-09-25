//! What Jev may choose from: the launcher's commands, window arrangements, and text tools.

use serde::{Deserialize, Serialize};

use crate::window_control::TilePosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandKind {
    Builtin,
    Script,
    Scriptlet,
    App,
}

impl CommandKind {
    /// Near-ties between commands go to apps, then built-ins, then scripts.
    pub fn tie_rank(self) -> u8 {
        match self {
            Self::App => 0,
            Self::Builtin => 1,
            Self::Script | Self::Scriptlet => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCode {
    pub lang: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    /// `builtin/<id>`, `script/<plugin>/<slug>`, `scriptlet/<plugin>/<slug>`, or `app/<name>`.
    pub id: String,
    pub kind: CommandKind,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capture_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<CommandCode>,
}

/// What a window arrangement does, in `window_control` terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    Tile(TilePosition),
    NextDisplay,
    PreviousDisplay,
    Minimize,
    Close,
}

impl WindowAction {
    /// The SDK's name for a tile, for generated scripts.
    pub fn sdk_position(self) -> Option<&'static str> {
        use TilePosition as T;
        let WindowAction::Tile(position) = self else {
            return None;
        };
        Some(match position {
            T::LeftHalf => "left",
            T::RightHalf => "right",
            T::TopHalf => "top",
            T::BottomHalf => "bottom",
            T::TopLeft => "top-left",
            T::TopRight => "top-right",
            T::BottomLeft => "bottom-left",
            T::BottomRight => "bottom-right",
            T::LeftThird => "left-third",
            T::CenterThird => "center-third",
            T::RightThird => "right-third",
            T::TopThird => "top-third",
            T::MiddleThird => "middle-third",
            T::BottomThird => "bottom-third",
            T::FirstTwoThirds => "first-two-thirds",
            T::LastTwoThirds => "last-two-thirds",
            T::TopTwoThirds => "top-two-thirds",
            T::BottomTwoThirds => "bottom-two-thirds",
            T::Center => "center",
            T::AlmostMaximize => "almost-maximize",
            T::Fullscreen => "maximize",
            _ => return None,
        })
    }
}

/// A window arrangement such as "Tile {{app}} Left Half", with the app left as a slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowOp {
    pub id: String,
    pub label: String,
    pub template: String,
    pub description: String,
    #[serde(default, skip_serializing)]
    pub code: String,
}

impl WindowOp {
    pub fn action(&self) -> Option<WindowAction> {
        WINDOW_OPS
            .iter()
            .find(|(label, ..)| *label == self.label)
            .map(|(.., action)| *action)
    }

    pub fn title_for(&self, app: &str) -> String {
        self.template.replacen("{{app}}", app, 1)
    }
}

/// The window-management plugin's arrangements, carried out natively so any app's
/// window can be the target, not only the frontmost one.
const WINDOW_OPS: &[(&str, &str, &str, WindowAction)] = &[
    (
        "Tile Left Half",
        "Tile {{app}} Left Half",
        "Tile the app to left half of screen",
        WindowAction::Tile(TilePosition::LeftHalf),
    ),
    (
        "Tile Right Half",
        "Tile {{app}} Right Half",
        "Tile the app to right half of screen",
        WindowAction::Tile(TilePosition::RightHalf),
    ),
    (
        "Tile Top Half",
        "Tile {{app}} Top Half",
        "Tile the app to top half of screen",
        WindowAction::Tile(TilePosition::TopHalf),
    ),
    (
        "Tile Bottom Half",
        "Tile {{app}} Bottom Half",
        "Tile the app to bottom half of screen",
        WindowAction::Tile(TilePosition::BottomHalf),
    ),
    (
        "Tile Top Left",
        "Tile {{app}} Top Left",
        "Tile the app to top-left quadrant",
        WindowAction::Tile(TilePosition::TopLeft),
    ),
    (
        "Tile Top Right",
        "Tile {{app}} Top Right",
        "Tile the app to top-right quadrant",
        WindowAction::Tile(TilePosition::TopRight),
    ),
    (
        "Tile Bottom Left",
        "Tile {{app}} Bottom Left",
        "Tile the app to bottom-left quadrant",
        WindowAction::Tile(TilePosition::BottomLeft),
    ),
    (
        "Tile Bottom Right",
        "Tile {{app}} Bottom Right",
        "Tile the app to bottom-right quadrant",
        WindowAction::Tile(TilePosition::BottomRight),
    ),
    (
        "Tile Left Third",
        "Tile {{app}} Left Third",
        "Tile the app to left third of screen",
        WindowAction::Tile(TilePosition::LeftThird),
    ),
    (
        "Tile Center Third",
        "Tile {{app}} Center Third",
        "Tile the app to center third of screen",
        WindowAction::Tile(TilePosition::CenterThird),
    ),
    (
        "Tile Right Third",
        "Tile {{app}} Right Third",
        "Tile the app to right third of screen",
        WindowAction::Tile(TilePosition::RightThird),
    ),
    (
        "Tile Top Third",
        "Tile {{app}} Top Third",
        "Tile the app to top third of screen",
        WindowAction::Tile(TilePosition::TopThird),
    ),
    (
        "Tile Middle Third",
        "Tile {{app}} Middle Third",
        "Tile the app to middle third of screen",
        WindowAction::Tile(TilePosition::MiddleThird),
    ),
    (
        "Tile Bottom Third",
        "Tile {{app}} Bottom Third",
        "Tile the app to bottom third of screen",
        WindowAction::Tile(TilePosition::BottomThird),
    ),
    (
        "Tile Left Two Thirds",
        "Tile {{app}} Left Two Thirds",
        "Tile the app to left two-thirds of screen",
        WindowAction::Tile(TilePosition::FirstTwoThirds),
    ),
    (
        "Tile Right Two Thirds",
        "Tile {{app}} Right Two Thirds",
        "Tile the app to right two-thirds of screen",
        WindowAction::Tile(TilePosition::LastTwoThirds),
    ),
    (
        "Tile Top Two Thirds",
        "Tile {{app}} Top Two Thirds",
        "Tile the app to top two-thirds of screen",
        WindowAction::Tile(TilePosition::TopTwoThirds),
    ),
    (
        "Tile Bottom Two Thirds",
        "Tile {{app}} Bottom Two Thirds",
        "Tile the app to bottom two-thirds of screen",
        WindowAction::Tile(TilePosition::BottomTwoThirds),
    ),
    (
        "Center",
        "Center {{app}}",
        "Center the app on screen (60% size)",
        WindowAction::Tile(TilePosition::Center),
    ),
    (
        "Almost Maximize",
        "Almost Maximize {{app}}",
        "Expand the app to 90% of screen with margins",
        WindowAction::Tile(TilePosition::AlmostMaximize),
    ),
    (
        "Maximize",
        "Maximize {{app}}",
        "Maximize the app to fill screen",
        WindowAction::Tile(TilePosition::Fullscreen),
    ),
    (
        "Move to Next Display",
        "Move {{app}} to Next Display",
        "Move the app to next display/monitor",
        WindowAction::NextDisplay,
    ),
    (
        "Move to Previous Display",
        "Move {{app}} to Previous Display",
        "Move the app to previous display/monitor",
        WindowAction::PreviousDisplay,
    ),
    (
        "Minimize",
        "Minimize {{app}}",
        "Minimize the app to the Dock",
        WindowAction::Minimize,
    ),
    (
        "Close Window",
        "Close {{app}} Window",
        "Close the frontmost the app window",
        WindowAction::Close,
    ),
];

pub fn native_window_ops() -> Vec<WindowOp> {
    WINDOW_OPS
        .iter()
        .map(|(label, template, description, _)| WindowOp {
            id: format!("window/{}", crate::intent::script::slugify(label)),
            label: label.to_string(),
            template: template.to_string(),
            description: description.to_string(),
            code: String::new(),
        })
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub commands: Vec<Command>,
    #[serde(default)]
    pub window_ops: Vec<WindowOp>,
    #[serde(default)]
    pub text_tools: Vec<Command>,
}

impl Catalog {
    pub fn command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.id == id)
    }
}
