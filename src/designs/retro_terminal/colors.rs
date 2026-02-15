use super::constants::TERMINAL_ITEM_HEIGHT;

/// Phosphor green color (classic CRT green)
pub(crate) const PHOSPHOR_GREEN: u32 = 0x00ff00;

/// CRT black background
pub(crate) const CRT_BLACK: u32 = 0x000000;

/// Dimmed green for less prominent elements
pub(crate) const DIM_GREEN: u32 = 0x00aa00;

/// Very dim green for scanlines/borders
pub(crate) const SCANLINE_GREEN: u32 = 0x003300;

/// Error red for log/error indicators
pub(crate) const ERROR_RED: u32 = 0xff4444;

/// Warning yellow for caution/warn indicators
pub(crate) const WARNING_YELLOW: u32 = 0xffff00;

/// Pre-computed colors for terminal rendering
#[derive(Clone, Copy)]
pub struct TerminalColors {
    pub phosphor: u32,
    pub background: u32,
    pub dim: u32,
    pub scanline: u32,
    pub error: u32,
    pub warning: u32,
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self {
            phosphor: PHOSPHOR_GREEN,
            background: CRT_BLACK,
            dim: DIM_GREEN,
            scanline: SCANLINE_GREEN,
            error: ERROR_RED,
            warning: WARNING_YELLOW,
        }
    }
}

/// Get terminal design constants for external use
pub struct TerminalConstants;

impl TerminalConstants {
    /// Item height for terminal list (dense: 28px)
    pub const fn item_height() -> f32 {
        TERMINAL_ITEM_HEIGHT
    }

    /// Phosphor green color constant
    pub const fn phosphor_green() -> u32 {
        PHOSPHOR_GREEN
    }

    /// CRT black background
    pub const fn crt_black() -> u32 {
        CRT_BLACK
    }

    /// Dim green for secondary elements
    pub const fn dim_green() -> u32 {
        DIM_GREEN
    }

    /// Glow green color (brighter than phosphor for glow effects)
    pub const fn glow_green() -> u32 {
        0x33ff33
    }
}
