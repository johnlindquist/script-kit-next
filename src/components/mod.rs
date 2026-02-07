//! Reusable UI Components for GPUI Script Kit
//!
//! This module provides a collection of reusable, theme-aware UI components
//! that follow consistent patterns across the application.
//!
//! # Components
//!
//! - [`Button`] - Interactive button with variants (Primary, Ghost, Icon)
//! - [`Toast`] - Toast notification with variants (Success, Warning, Error, Info)
//! - [`Scrollbar`] - Minimal native-style scrollbar for overlay on lists
//! - [`FormTextField`] - Text input for text/password/email/number types
//! - [`FormTextArea`] - Multi-line text input
//! - [`FormCheckbox`] - Checkbox with label
//! - [`PromptHeader`] - Header component with search input, buttons, and logo
//! - [`PromptFooter`] - Footer component with logo, primary/secondary action buttons
//! - [`PromptContainer`] - Container component for consistent prompt window layout
//! - [`ShortcutRecorder`] - Modal for recording keyboard shortcuts with visual feedback
//! - [`AliasInput`] - Modal for entering command aliases with keyboard focus
//!
//!
//! # Design Patterns
//!
//! All components follow these patterns:
//! - **Colors struct**: Pre-computed colors (Copy/Clone) for efficient closure use
//! - **Builder pattern**: Fluent API with `.method()` chaining
//! - **IntoElement trait**: Compatible with GPUI's element system
//! - **Theme integration**: Use `from_theme()` or `from_design()` for colors

pub mod alias_input;
pub mod button;
pub mod footer_button;
pub mod form_fields;
#[cfg(test)]
mod form_fields_tests;
pub mod input_tokens;
pub mod prompt_container;
pub mod prompt_footer;
pub mod prompt_header;
pub mod prompt_input;
pub mod prompt_layout_shell;
pub mod script_kit_input;
pub mod scrollbar;
pub mod shortcut_recorder;
pub mod text_input;
pub mod toast;
pub mod unified_list_item;
#[cfg(test)]
mod unified_list_item_tests;

// Re-export commonly used types
#[allow(unused_imports)]
pub use alias_input::{AliasInput, AliasInputAction, AliasInputColors};
pub use button::{Button, ButtonColors, ButtonVariant};
#[allow(unused_imports)]
pub use footer_button::FooterButton;
#[allow(unused_imports)]
pub use form_fields::{FormCheckbox, FormFieldColors, FormFieldState, FormTextArea, FormTextField};
#[allow(unused_imports)]
pub use input_tokens::{
    INPUT_FONT_SIZE_ARG, INPUT_FONT_SIZE_CHAT, INPUT_FONT_SIZE_DEFAULT, INPUT_FONT_SIZE_MAIN_MENU,
    INPUT_FONT_SIZE_SEARCH, INPUT_PLACEHOLDER_ARG, INPUT_PLACEHOLDER_CHAT,
    INPUT_PLACEHOLDER_DEFAULT, INPUT_PLACEHOLDER_MAIN_MENU, INPUT_PLACEHOLDER_SEARCH,
};
#[allow(unused_imports)]
pub use script_kit_input::{ScriptKitInput, ScriptKitInputConfig};
#[allow(unused_imports)]
pub use scrollbar::{
    Scrollbar, ScrollbarColors, MIN_THUMB_HEIGHT, SCROLLBAR_PADDING, SCROLLBAR_WIDTH,
};
// These re-exports form the public API - allow unused since not all are used in every crate
#[allow(unused_imports)]
pub use prompt_container::{PromptContainer, PromptContainerColors, PromptContainerConfig};
#[allow(unused_imports)]
pub use prompt_footer::{PromptFooter, PromptFooterColors, PromptFooterConfig};
#[allow(unused_imports)]
pub use prompt_header::{PromptHeader, PromptHeaderColors, PromptHeaderConfig};
#[allow(unused_imports)]
pub use prompt_input::{InputPadding, PromptInput, PromptInputColors, PromptInputConfig};
#[allow(unused_imports)]
pub use prompt_layout_shell::{prompt_shell_container, prompt_shell_content};
#[allow(unused_imports)]
pub use shortcut_recorder::{
    RecordedShortcut, ShortcutConflict, ShortcutRecorder, ShortcutRecorderColors,
};
#[allow(unused_imports)]
pub use text_input::{TextInputState, TextSelection};
#[allow(unused_imports)]
pub use toast::{Toast, ToastAction, ToastColors, ToastVariant};
#[allow(unused_imports)]
pub use unified_list_item::{
    Density, ItemState, LeadingContent, ListItemLayout, SectionHeader, TextContent,
    TrailingContent, UnifiedListItem, UnifiedListItemColors, SECTION_HEADER_HEIGHT,
};
