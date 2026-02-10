//! Protocol types for Script Kit GPUI
//!
//! Contains all the helper types used in protocol messages:
//! - Choice, Field for prompts
//! - Clipboard, Keyboard, Mouse action types
//! - ExecOptions, MouseEventData
//! - SubmitValue for JSON-capable submit values
//! - ScriptletData, ProtocolAction
//! - Element types for UI querying
//! - Error data types

mod ai;
mod chat;
mod elements_actions_scriptlets;
mod grid_layout;
mod input;
mod menu_bar;
mod primitives;
mod system;

pub use ai::{AiChatInfo, AiMessageInfo};
pub use chat::{ChatMessagePosition, ChatMessageRole, ChatPromptConfig, ChatPromptMessage};
pub use elements_actions_scriptlets::{
    ElementInfo, ElementType, ProtocolAction, ScriptletData, ScriptletMetadataData,
};
pub use grid_layout::{
    BoxModelSides, ComputedBoxModel, ComputedFlexStyle, GridColorScheme, GridDepthOption,
    GridOptions, LayoutBounds, LayoutComponentInfo, LayoutComponentType, LayoutInfo,
    ScriptErrorData,
};
pub use input::{ExecOptions, MouseData};
pub use menu_bar::MenuBarItemData;
pub use primitives::{
    Choice, ClipboardAction, ClipboardEntryType, ClipboardFormat, ClipboardHistoryAction, Field,
    KeyboardAction, MouseAction, SubmitValue, TilePosition, WindowActionType,
};
pub use system::{
    ClipboardHistoryEntryData, DisplayInfo, FileSearchResultEntry, SystemWindowInfo,
    TargetWindowBounds,
};

#[cfg(test)]
mod tests;
