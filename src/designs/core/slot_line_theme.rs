//! The line above the input as the main menu draws it: the prototype's `.slot` at app scale.

use sk_one_window::contract;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotLineTokens {
    pub height_px: f32,
    pub gap_px: f32,
    pub label_font_size: f32,
    pub title_font_size: f32,
    pub hint_font_size: f32,
    pub icon_size: f32,
    pub progress_height_px: f32,
    /// Tints behind a staged subject (info color) and a waiting message (accent).
    pub staged_tint_alpha: u32,
    pub holding_tint_alpha: u32,
    pub dim_ease_ms: u64,
}

pub const fn slot_line_tokens() -> SlotLineTokens {
    SlotLineTokens {
        height_px: contract::app::SLOT_LINE_PX,
        gap_px: 8.0,
        label_font_size: 10.0,
        title_font_size: 12.0,
        hint_font_size: 11.0,
        icon_size: 14.0,
        progress_height_px: 2.0,
        staged_tint_alpha: 0x17,
        holding_tint_alpha: 0x12,
        dim_ease_ms: contract::SLOT_DIM_EASE_MS,
    }
}
