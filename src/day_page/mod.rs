//! Day Page document binding — substrate-backed file session for today's page.

mod document;
mod render;
mod sediment;
pub(crate) mod telemetry;
#[cfg(test)]
mod tests;

pub use document::{DayPageBinding, DayPageDocumentSession};
pub use render::render_fragment_back_bar;
pub use sediment::{
    context_parts_from_day_page_markdown_links, day_page_markdown_reference_for_context_part,
    format_provenance_hint, load_fragment_provenance, normalize_day_page_markdown_references,
    parse_day_page_segments, resolve_fragment_path, DayPageSegment, FragmentProvenance,
    FRAGMENT_BACK_ID,
};
