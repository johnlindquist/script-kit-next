//! App Shell Module - Unified frame and chrome for all prompts
//!
//! This module provides a presentational shell that wraps prompt content with
//! consistent styling, focus management, and keyboard routing.
//!
//! # Architecture
//!
//! The shell is a *frame + chrome* layer, not another "app". State and lifecycle
//! remain in the window root (ScriptListApp). The shell:
//!
//! - Takes stable focus handles by reference (doesn't own them)
//! - Renders frame (background, shadow, radius, padding)
//! - Renders optional header/footer based on spec
//! - Routes keyboard events to actions
//! - Never allocates per-render (uses SmallVec, SharedString, action enums)
//!
//! # Usage
//!
//! Each view returns a `ShellSpec` that describes what it needs:
//!
//! ```rust,ignore
//! impl ScriptListApp {
//!     fn script_list_spec(&self, cx: &Context<Self>) -> ShellSpec {
//!         ShellSpec::new()
//!             .header(HeaderSpec::search("Type to search...").button("Run", ""))
//!             .footer(FooterSpec::new().primary("Run Script", ""))
//!             .content(self.render_list_content(cx))
//!             .chrome(ChromeSpec::full_frame())
//!             .focus_policy(FocusPolicy::HeaderInput)
//!     }
//! }
//! ```
//!
//! Then render via AppShell:
//!
//! ```rust,ignore
//! AppShell::render(spec, &self.shell_runtime, window, cx)
//! ```

pub mod chrome;
pub mod focus;
pub mod keymap;
pub mod shell;
pub mod spec;
pub mod style;

// Re-export primary types
pub use chrome::{ChromeMode, ChromeSpec, DividerSpec};
pub use focus::{FocusPolicy, ShellFocus};
pub use keymap::{KeymapSpec, ShellAction};
pub use shell::AppShell;
pub use spec::{ButtonSpec, FooterSpec, HeaderSpec, InputSpec, ShellSpec};
pub use style::ShellStyleCache;

#[cfg(test)]
mod tests;
