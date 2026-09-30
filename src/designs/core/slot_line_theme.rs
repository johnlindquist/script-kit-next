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
    pub chip: SubjectChipTokens,
}

/// The subject chip in the input: solid (info color) at its start once attached, dashed at its
/// end while only offered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubjectChipTokens {
    pub height_px: f32,
    pub radius_px: f32,
    pub gap_px: f32,
    pub pad_left_px: f32,
    pub pad_right_px: f32,
    /// Between an attached chip and the words after it.
    pub attached_margin_px: f32,
    /// Between two attached chips.
    pub between_px: f32,
    /// Between the words' end and an offered chip.
    pub offered_margin_px: f32,
    /// How far a chip in the input sits below the input's centre. The input's words sit low in
    /// their line (room for descenders), so a centred chip reads high beside them; this drop puts
    /// the chip's text on the words' baseline.
    pub input_drop_px: f32,
    pub font_size: f32,
    pub attached_weight: f32,
    pub offered_weight: f32,
    pub icon_size: f32,
    pub max_title_px: f32,
    pub fill_alpha: u32,
    pub edge_alpha: u32,
    pub offered_edge_alpha: u32,
    pub offered_text_alpha: u32,
    /// The chip ← highlighted, which ⌫ deletes.
    pub highlight_fill_alpha: u32,
    pub highlight_edge_alpha: u32,
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
        chip: SubjectChipTokens {
            height_px: 26.0,
            radius_px: 8.0,
            gap_px: 6.0,
            pad_left_px: 8.0,
            pad_right_px: 4.0,
            attached_margin_px: 10.0,
            between_px: 6.0,
            offered_margin_px: 6.0,
            input_drop_px: 1.5,
            font_size: 13.0,
            attached_weight: 550.0,
            offered_weight: 450.0,
            icon_size: 14.0,
            max_title_px: 260.0,
            fill_alpha: 0x21,
            edge_alpha: 0x4D,
            offered_edge_alpha: 0x38,
            offered_text_alpha: 0x9E,
            highlight_fill_alpha: 0xB3,
            highlight_edge_alpha: 0xFF,
        },
    }
}
