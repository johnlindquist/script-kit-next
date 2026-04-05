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

pub(crate) mod acp_state;
mod ai;
pub mod automation_surface;
pub mod automation_window;
pub(crate) mod batch_wait;
mod chat;
mod elements_actions_scriptlets;
mod grid_layout;
mod input;
mod menu_bar;
mod primitives;
pub mod simulated_gpui_event;
mod system;

pub use acp_state::{
    AcpAcceptedItem, AcpInputLayoutMetrics, AcpInputLayoutTelemetry, AcpKeyRoute,
    AcpKeyRouteTelemetry, AcpPickerItemAcceptedTelemetry, AcpPickerState, AcpResolvedTarget,
    AcpSetupActionKind, AcpSetupSnapshot, AcpStateSnapshot, AcpTestProbeSnapshot,
    AcpWaitCondition, ACP_STATE_SCHEMA_VERSION, ACP_TEST_PROBE_MAX_EVENTS,
    ACP_TEST_PROBE_SCHEMA_VERSION,
};
pub use ai::{AiChatInfo, AiContextPartInput, AiMessageInfo};
pub use automation_surface::{AutomationSurfaceSnapshot, AUTOMATION_SURFACE_SCHEMA_VERSION};
pub use automation_window::{
    AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
    AUTOMATION_WINDOW_SCHEMA_VERSION,
};
pub use batch_wait::{
    BatchCommand, BatchOptions, BatchResultEntry, StateMatchSpec, TransactionCommandTrace,
    TransactionError, TransactionErrorCode, TransactionTrace, TransactionTraceMode,
    TransactionTraceStatus, UiStateSnapshot, WaitCondition, WaitDetailedCondition,
    WaitNamedCondition, WaitPollObservation,
};
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
pub use simulated_gpui_event::SimulatedGpuiEvent;
pub use system::{
    ClipboardHistoryEntryData, DisplayInfo, FileSearchResultEntry, SystemWindowInfo,
    TargetWindowBounds,
};

#[cfg(test)]
mod tests;
