//! Typed chrome style for the Notes window.
//!
//! This module extracts the Notes window's layout constants into a typed struct
//! that can be driven by storybook adoption. The real render path consumes
//! `adopted_style()` which resolves from on-disk storybook selections when the
//! `storybook` feature is active, otherwise returns `NotesWindowStyle::current()`.

/// Typed chrome style governing Notes window layout dimensions and opacity.
///
/// Each field corresponds to a layout constant previously defined inline in
/// `window.rs`. The struct is `Copy` so it can be cheaply threaded through
/// render closures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotesWindowStyle {
    /// Titlebar height in px (macOS traffic light clearance).
    pub titlebar_height: f32,
    /// Footer / status-bar height in px.
    pub footer_height: f32,
    /// Horizontal padding inside the editor area in px.
    pub editor_padding_x: f32,
    /// Vertical padding inside the editor area in px.
    pub editor_padding_y: f32,
    /// Overall chrome opacity multiplier (1.0 = fully opaque).
    pub chrome_opacity: f32,
}

impl NotesWindowStyle {
    /// The current production defaults — matches the inline constants in `window.rs`.
    pub const fn current() -> Self {
        Self {
            titlebar_height: 36.0,
            footer_height: 28.0,
            editor_padding_x: 16.0, // px_4 = 16px
            editor_padding_y: 12.0, // py_3 = 12px
            chrome_opacity: 1.0,
        }
    }

    /// Compact variant — tighter spacing for smaller windows.
    pub const fn compact() -> Self {
        Self {
            titlebar_height: 28.0,
            footer_height: 22.0,
            editor_padding_x: 8.0,
            editor_padding_y: 6.0,
            chrome_opacity: 1.0,
        }
    }

    /// Airy variant — more breathing room, relaxed layout.
    pub const fn airy() -> Self {
        Self {
            titlebar_height: 44.0,
            footer_height: 32.0,
            editor_padding_x: 24.0,
            editor_padding_y: 16.0,
            chrome_opacity: 1.0,
        }
    }
}

/// Resolve the adopted style from storybook selection (storybook feature gate).
#[cfg(feature = "storybook")]
pub(crate) fn adopted_style() -> NotesWindowStyle {
    crate::storybook::adopted_notes_window_style()
}

/// Production default — always returns `current()`.
#[cfg(not(feature = "storybook"))]
pub(crate) fn adopted_style() -> NotesWindowStyle {
    NotesWindowStyle::current()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_style_matches_production_constants() {
        let style = NotesWindowStyle::current();
        assert_eq!(style.titlebar_height, 36.0);
        assert_eq!(style.footer_height, 28.0);
        assert_eq!(style.editor_padding_x, 16.0);
        assert_eq!(style.editor_padding_y, 12.0);
        assert_eq!(style.chrome_opacity, 1.0);
    }

    #[test]
    fn compact_is_tighter_than_current() {
        let current = NotesWindowStyle::current();
        let compact = NotesWindowStyle::compact();
        assert!(compact.titlebar_height < current.titlebar_height);
        assert!(compact.footer_height < current.footer_height);
        assert!(compact.editor_padding_x < current.editor_padding_x);
    }

    #[test]
    fn airy_is_larger_than_current() {
        let current = NotesWindowStyle::current();
        let airy = NotesWindowStyle::airy();
        assert!(airy.titlebar_height > current.titlebar_height);
        assert!(airy.footer_height > current.footer_height);
        assert!(airy.editor_padding_x > current.editor_padding_x);
    }

    #[test]
    fn all_variants_have_positive_dimensions() {
        for style in [
            NotesWindowStyle::current(),
            NotesWindowStyle::compact(),
            NotesWindowStyle::airy(),
        ] {
            assert!(style.titlebar_height > 0.0);
            assert!(style.footer_height > 0.0);
            assert!(style.editor_padding_x >= 0.0);
            assert!(style.editor_padding_y >= 0.0);
            assert!(style.chrome_opacity > 0.0);
        }
    }
}
