use gpui::Hsla;

#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineDropdownColors {
    pub(crate) foreground: Hsla,
    pub(crate) muted_foreground: Hsla,
    pub(crate) accent: Hsla,
}
