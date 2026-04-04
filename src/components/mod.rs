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
#[cfg(test)]
mod error_handling_audit_tests;
pub mod focusable_prompt_wrapper;
pub mod form_fields;
#[cfg(test)]
mod form_fields_tests;
pub mod hint_strip;
pub mod inline_prompt_input;
pub mod minimal_prompt_shell;
pub(crate) mod overlay_modal;
pub mod prompt_container;
pub mod prompt_footer;
pub mod prompt_header;
pub mod prompt_layout_shell;
pub mod scrollbar;
pub mod section_divider;
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
pub use focusable_prompt_wrapper::{
    match_focusable_prompt_intercepted_key, FocusablePrompt, FocusablePromptInterceptedKey,
};
#[allow(unused_imports)]
pub use form_fields::{FormCheckbox, FormFieldColors, FormFieldState, FormTextArea, FormTextField};
#[allow(unused_imports)]
pub use hint_strip::{
    render_hint_icons, render_hint_icons_clickable, render_hint_icons_hsla, ClickableHint,
    HintStrip,
};
#[allow(unused_imports)]
pub use inline_prompt_input::InlinePromptInput;
#[allow(unused_imports)]
pub use minimal_prompt_shell::MinimalPromptShell;
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
pub(crate) use prompt_layout_shell::{
    emit_prompt_chrome_audit, emit_prompt_hint_audit, is_universal_prompt_hints,
    render_universal_prompt_hint_strip, render_universal_prompt_hint_strip_clickable,
    universal_prompt_hints, PromptChromeAudit, PromptHintAudit, UNIVERSAL_PROMPT_HINT_COUNT,
};
#[allow(unused_imports)]
pub(crate) use prompt_layout_shell::{
    prompt_detail_card, prompt_field_style, prompt_form_help, prompt_form_intro,
    prompt_form_section, prompt_scroll_value, prompt_scroll_value_with_id, prompt_surface,
    prompt_text_field, PromptFieldState, PromptFieldStyle,
};
#[allow(unused_imports)]
pub use prompt_layout_shell::{prompt_shell_container, prompt_shell_content};
#[allow(unused_imports)]
pub(crate) use prompt_layout_shell::{
    render_expanded_view_prompt_shell, render_expanded_view_scaffold,
    render_expanded_view_scaffold_with_hints, render_hint_strip_leading_text,
    render_minimal_list_prompt_scaffold, render_minimal_list_prompt_shell,
    render_simple_hint_strip, render_simple_prompt_shell,
};
#[allow(unused_imports)]
pub use section_divider::SectionDivider;
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
