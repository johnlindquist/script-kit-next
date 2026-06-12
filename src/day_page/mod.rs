//! Day Page document binding — substrate-backed file session for today's page.

mod document;
mod render;
mod sediment;
#[cfg(test)]
mod tests;

pub use document::{DayPageBinding, DayPageDocumentSession};
pub use render::render_fragment_back_bar;
pub use sediment::{
    format_provenance_hint, fragment_card_id, kept_url_id, load_fragment_provenance,
    parse_day_page_segments, resolve_fragment_path, DayPageSegment, FragmentProvenance,
    FRAGMENT_BACK_ID, FRAGMENT_CARD_ID_PREFIX, KEPT_URL_ID_PREFIX, SEDIMENT_LAYER_ID,
    SEDIMENT_LINE_HEIGHT,
};
