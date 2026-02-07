//! Markdown rendering for chat messages
//!
//! Uses pulldown-cmark for parsing and syntect for fenced code highlighting.
//! Supports: headings, lists, blockquotes, bold/italic, inline code, code blocks, links.
//!
//! Performance: The markdown is parsed once and cached in a global HashMap keyed
//! by content hash + dark-mode flag. On subsequent render frames (e.g. during
//! scrolling at 60fps) we skip pulldown-cmark parsing and syntect highlighting
//! entirely, and only build cheap GPUI elements from the cached representation.

use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClipboardItem, FontWeight, IntoElement,
    SharedString,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::notes::code_highlight::{highlight_code_lines, CodeLine, CodeSpan};
use crate::theme::PromptColors;

mod api;
mod code_table;
mod helpers;
mod inline_render;
mod parse;
mod render_blocks;
mod scope;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod types;

pub use api::{render_markdown, render_markdown_with_scope};
use code_table::{build_code_block_element, build_table_element};
pub(super) use helpers::*;
use inline_render::{render_hr, render_inline_spans};
use parse::parse_markdown;
use render_blocks::build_markdown_elements;
pub(super) use scope::*;
#[cfg(test)]
pub(super) use test_support::*;
pub(super) use types::*;
