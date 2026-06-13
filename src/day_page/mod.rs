//! Day Page document binding — substrate-backed file session for today's page.

mod document;
mod render;
mod sediment;
#[cfg(test)]
mod tests;

pub use document::{DayPageBinding, DayPageDocumentSession};
pub use render::render_fragment_back_bar;
pub use sediment::{
    format_provenance_hint, load_fragment_provenance, parse_day_page_segments,
    resolve_fragment_path, DayPageSegment, FragmentProvenance, FRAGMENT_BACK_ID,
};
