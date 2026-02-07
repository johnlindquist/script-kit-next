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

pub(crate) mod chrome;
pub(crate) mod focus;
pub(crate) mod keymap;
pub(crate) mod shell;
pub(crate) mod spec;
pub(crate) mod style;

// Re-export primary types
pub use self::chrome::{ChromeMode, ChromeSpec, DividerSpec};
pub use self::focus::{FocusPolicy, ShellFocus};
pub use self::keymap::{KeymapSpec, ShellAction};
pub use self::shell::AppShell;
pub use self::spec::{ButtonSpec, FooterSpec, HeaderSpec, InputSpec, ShellSpec};
pub use self::style::ShellStyleCache;

#[cfg(test)]
mod tests;
