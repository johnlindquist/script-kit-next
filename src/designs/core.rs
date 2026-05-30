#[allow(unused_imports)]
use super::*;

mod accent_variation;
mod metadata;
pub mod registry;
mod render;
mod tokens;
mod variant;

#[cfg(test)]
mod match_reason;
#[cfg(test)]
mod tests;

pub use accent_variation::{
    current_accent_variation, set_current_accent_variation, AccentVariation,
};
// `FooterButtonFill` is part of the accent-variation API surface; re-exported for
// callers that name the type even though the footer consumes it field-by-field.
#[allow(unused_imports)]
pub use accent_variation::FooterButtonFill;
pub use render::render_design_item;
pub use tokens::*;
pub use variant::*;

#[cfg(test)]
pub(crate) use match_reason::*;
#[cfg(test)]
pub(crate) use metadata::*;
#[cfg(test)]
pub(crate) use render::{
    extension_default_icon, resolve_search_accessories, resolve_tool_badge, root_file_type_svg_icon,
};
