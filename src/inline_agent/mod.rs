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
pub mod types;
pub mod window;

pub use layout::{place_compact_overlay, InlineAgentLayoutDefaults};
pub use platform_bridge::InlineAgentPlatformBridge;
pub use state::{InlineAgentMode, InlineAgentRunState, InlineAgentState};
pub use types::{InlineAgentOutputAction, InlineAgentSnapshot, INLINE_AGENT_INPUT_PLACEHOLDER};
