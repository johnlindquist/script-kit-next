//! GPUI-facing inline agent overlay state.
//!
//! This layer owns compact/expanded presentation state and talks to platform
//! accessibility through a bridge trait. It does not import AX internals or AI
//! provider implementations.

pub mod automation;
pub mod layout;
pub mod platform_bridge;
pub mod render_actions;
pub mod render_compact;
pub mod render_expanded;
pub mod state;
pub mod telemetry;
pub mod theme;
pub mod types;
pub mod window;

pub use layout::{place_compact_overlay, place_expanded_overlay, InlineAgentLayoutDefaults};
pub use platform_bridge::{InlineAgentPlatformBridge, SystemInlineAgentPlatformBridge};
pub use state::{InlineAgentMode, InlineAgentRunState, InlineAgentState};
pub use theme::{
    InlineAgentColors, InlineAgentContrastSummary, INLINE_AGENT_DISABLED_TEXT_MIN_CONTRAST,
    INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST, INLINE_AGENT_SECONDARY_TEXT_MIN_CONTRAST,
};
pub use types::{
    InlineAgentAnchor, InlineAgentMutationReceipt, InlineAgentOutputAction, InlineAgentSnapshot,
    InlineAgentTextMutation, INLINE_AGENT_INPUT_PLACEHOLDER,
};
pub use window::{
    close_inline_agent_overlay_window, compact_root_automation_id, inline_agent_automation_info,
    inline_agent_window_options, launch_inline_agent_from_focused_text,
    open_inline_agent_mock_fixture, open_inline_agent_pi_fixture,
    plan_compact_inline_agent_overlay, plan_expanded_inline_agent_overlay,
    plan_open_inline_agent_overlay, register_inline_agent_automation_window,
    remove_inline_agent_automation_window, sync_inline_agent_overlay_window,
    update_inline_agent_automation_bounds, InlineAgentOverlayWindow, InlineAgentWindowSnapshot,
    InlineOverlayAttachment, INLINE_AGENT_SEMANTIC_SURFACE, INLINE_AGENT_WINDOW_AUTOMATION_ID,
    INLINE_AGENT_WINDOW_TITLE,
};
