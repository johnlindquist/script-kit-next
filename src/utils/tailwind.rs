//! Tailwind CSS class mapping for Script Kit GPUI

/// Tailwind-style value that can be applied to a GPUI div
#[derive(Debug, Clone, Default)]
pub struct TailwindStyles {
    // Layout
    pub flex: bool,
    pub flex_col: bool,
    pub flex_row: bool,
    pub flex_1: bool,
    pub items_center: bool,
    pub items_start: bool,
    pub items_end: bool,
    pub justify_center: bool,
    pub justify_between: bool,
    pub justify_start: bool,
    pub justify_end: bool,

    // Sizing
    pub w_full: bool,
    pub h_full: bool,
    pub min_w_0: bool,
    pub min_h_0: bool,

    // Spacing (in pixels)
    pub padding: Option<f32>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub margin: Option<f32>,
    pub margin_x: Option<f32>,
    pub margin_y: Option<f32>,
    pub margin_top: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub gap: Option<f32>,

    // Colors (as 0xRRGGBB)
    pub text_color: Option<u32>,
    pub bg_color: Option<u32>,
    pub border_color: Option<u32>,

    // Typography
    pub font_size: Option<f32>,
    pub font_bold: bool,
    pub font_medium: bool,
    pub font_normal: bool,

    // Borders
    pub rounded: Option<f32>,
    pub border: bool,
    pub border_width: Option<f32>,
}

impl TailwindStyles {
    /// Parse a space-separated class string into TailwindStyles
    pub fn parse(class_string: &str) -> Self {
        let mut styles = TailwindStyles::default();

        for class in class_string.split_whitespace() {
            styles.apply_class(class);
        }

        styles
    }

    /// Apply a single Tailwind class to this style struct
    fn apply_class(&mut self, class: &str) {
        match class {
            // Layout
            "flex" => self.flex = true,
            "flex-col" => self.flex_col = true,
            "flex-row" => self.flex_row = true,
            "flex-1" => self.flex_1 = true,
            "items-center" => self.items_center = true,
            "items-start" => self.items_start = true,
            "items-end" => self.items_end = true,
            "justify-center" => self.justify_center = true,
            "justify-between" => self.justify_between = true,
            "justify-start" => self.justify_start = true,
            "justify-end" => self.justify_end = true,

            // Sizing
            "w-full" => self.w_full = true,
            "h-full" => self.h_full = true,
            "min-w-0" => self.min_w_0 = true,
            "min-h-0" => self.min_h_0 = true,

            // Typography
            "font-bold" => self.font_bold = true,
            "font-medium" => self.font_medium = true,
            "font-normal" => self.font_normal = true,
            "text-sm" => self.font_size = Some(14.0),
            "text-base" => self.font_size = Some(16.0),
            "text-lg" => self.font_size = Some(18.0),
            "text-xl" => self.font_size = Some(20.0),
            "text-2xl" => self.font_size = Some(24.0),
            "text-3xl" => self.font_size = Some(30.0),
            "text-4xl" => self.font_size = Some(36.0),

            // Border radius
            "rounded" => self.rounded = Some(4.0),
            "rounded-sm" => self.rounded = Some(2.0),
            "rounded-md" => self.rounded = Some(6.0),
            "rounded-lg" => self.rounded = Some(8.0),
            "rounded-xl" => self.rounded = Some(12.0),
            "rounded-2xl" => self.rounded = Some(16.0),
            "rounded-full" => self.rounded = Some(9999.0),
            "rounded-none" => self.rounded = Some(0.0),

            // Border
            "border" => self.border = true,
            "border-0" => self.border_width = Some(0.0),
            "border-2" => self.border_width = Some(2.0),
            "border-4" => self.border_width = Some(4.0),
            "border-8" => self.border_width = Some(8.0),

            // Otherwise, try pattern matching
            _ => self.apply_pattern_class(class),
        }
    }

    /// Apply classes that follow patterns like p-4, bg-blue-500, etc.
    fn apply_pattern_class(&mut self, class: &str) {
        // Spacing: p-*, px-*, py-*, pt-*, pb-*, pl-*, pr-*
        // Tailwind scale: 0=0, 1=4px, 2=8px, 3=12px, 4=16px, 5=20px, 6=24px, 8=32px, 10=40px, 12=48px
        if let Some(value) = class.strip_prefix("p-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("px-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_x = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("py-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_y = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("pt-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_top = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("pb-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_bottom = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("pl-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_left = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("pr-") {
            if let Some(px) = parse_spacing_value(value) {
                self.padding_right = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("m-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("mx-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_x = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("my-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_y = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("mt-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_top = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("mb-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_bottom = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("ml-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_left = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("mr-") {
            if let Some(px) = parse_spacing_value(value) {
                self.margin_right = Some(px);
            }
        } else if let Some(value) = class.strip_prefix("gap-") {
            if let Some(px) = parse_spacing_value(value) {
                self.gap = Some(px);
            }
        }
        // Text colors
        else if let Some(color_name) = class.strip_prefix("text-") {
            if let Some(color) = parse_color(color_name) {
                self.text_color = Some(color);
            }
        }
        // Background colors
        else if let Some(color_name) = class.strip_prefix("bg-") {
            if let Some(color) = parse_color(color_name) {
                self.bg_color = Some(color);
            }
        }
        // Border colors
        else if let Some(color_name) = class.strip_prefix("border-") {
            // Skip border width classes that we already handled
            if !["0", "2", "4", "8"].contains(&color_name) {
                if let Some(color) = parse_color(color_name) {
                    self.border_color = Some(color);
                }
            }
        }
    }
}

/// Parse Tailwind spacing values (0-12 scale) to pixels
fn parse_spacing_value(value: &str) -> Option<f32> {
    match value {
        "0" => Some(0.0),
        "0.5" => Some(2.0),
        "1" => Some(4.0),
        "1.5" => Some(6.0),
        "2" => Some(8.0),
        "2.5" => Some(10.0),
        "3" => Some(12.0),
        "3.5" => Some(14.0),
        "4" => Some(16.0),
        "5" => Some(20.0),
        "6" => Some(24.0),
        "7" => Some(28.0),
        "8" => Some(32.0),
        "9" => Some(36.0),
        "10" => Some(40.0),
        "11" => Some(44.0),
        "12" => Some(48.0),
        "14" => Some(56.0),
        "16" => Some(64.0),
        "20" => Some(80.0),
        "24" => Some(96.0),
        "auto" => None, // Can't represent auto in fixed pixels
        _ => {
            // Try to parse arbitrary value like [20px]
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                if let Some(px_value) = inner.strip_suffix("px") {
                    return px_value.parse().ok();
                }
            }
            None
        }
    }
}

/// Parse Tailwind color names to hex values
pub fn parse_color(color_name: &str) -> Option<u32> {
    // Basic colors
    match color_name {
        "white" => return Some(0xFFFFFF),
        "black" => return Some(0x000000),
        "transparent" => return Some(0x000000), // Note: transparency not fully supported
        "current" => return None,               // Can't resolve current color
        _ => {}
    }

    // Parse color-shade format like "blue-500", "gray-100"
    if let Some((color, shade)) = color_name.rsplit_once('-') {
        let shade: u32 = shade.parse().ok()?;

        // Tailwind color palette (subset of most common colors)
        return match color {
            "slate" => get_slate_color(shade),
            "gray" => get_gray_color(shade),
            "zinc" => get_zinc_color(shade),
            "neutral" => get_neutral_color(shade),
            "stone" => get_stone_color(shade),
            "red" => get_red_color(shade),
            "orange" => get_orange_color(shade),
            "amber" => get_amber_color(shade),
            "yellow" => get_yellow_color(shade),
            "lime" => get_lime_color(shade),
            "green" => get_green_color(shade),
            "emerald" => get_emerald_color(shade),
            "teal" => get_teal_color(shade),
            "cyan" => get_cyan_color(shade),
            "sky" => get_sky_color(shade),
            "blue" => get_blue_color(shade),
            "indigo" => get_indigo_color(shade),
            "violet" => get_violet_color(shade),
            "purple" => get_purple_color(shade),
            "fuchsia" => get_fuchsia_color(shade),
            "pink" => get_pink_color(shade),
            "rose" => get_rose_color(shade),
            _ => None,
        };
    }

    None
}

// Tailwind color palette functions
fn get_slate_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF8FAFC),
        100 => Some(0xF1F5F9),
        200 => Some(0xE2E8F0),
        300 => Some(0xCBD5E1),
        400 => Some(0x94A3B8),
        500 => Some(0x64748B),
        600 => Some(0x475569),
        700 => Some(0x334155),
        800 => Some(0x1E293B),
        900 => Some(0x0F172A),
        950 => Some(0x020617),
        _ => None,
    }
}

fn get_gray_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF9FAFB),
        100 => Some(0xF3F4F6),
        200 => Some(0xE5E7EB),
        300 => Some(0xD1D5DB),
        400 => Some(0x9CA3AF),
        500 => Some(0x6B7280),
        600 => Some(0x4B5563),
        700 => Some(0x374151),
        800 => Some(0x1F2937),
        900 => Some(0x111827),
        950 => Some(0x030712),
        _ => None,
    }
}

fn get_zinc_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFAFAFA),
        100 => Some(0xF4F4F5),
        200 => Some(0xE4E4E7),
        300 => Some(0xD4D4D8),
        400 => Some(0xA1A1AA),
        500 => Some(0x71717A),
        600 => Some(0x52525B),
        700 => Some(0x3F3F46),
        800 => Some(0x27272A),
        900 => Some(0x18181B),
        950 => Some(0x09090B),
        _ => None,
    }
}

fn get_neutral_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFAFAFA),
        100 => Some(0xF5F5F5),
        200 => Some(0xE5E5E5),
        300 => Some(0xD4D4D4),
        400 => Some(0xA3A3A3),
        500 => Some(0x737373),
        600 => Some(0x525252),
        700 => Some(0x404040),
        800 => Some(0x262626),
        900 => Some(0x171717),
        950 => Some(0x0A0A0A),
        _ => None,
    }
}

fn get_stone_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFAFAF9),
        100 => Some(0xF5F5F4),
        200 => Some(0xE7E5E4),
        300 => Some(0xD6D3D1),
        400 => Some(0xA8A29E),
        500 => Some(0x78716C),
        600 => Some(0x57534E),
        700 => Some(0x44403C),
        800 => Some(0x292524),
        900 => Some(0x1C1917),
        950 => Some(0x0C0A09),
        _ => None,
    }
}

fn get_red_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFEF2F2),
        100 => Some(0xFEE2E2),
        200 => Some(0xFECACA),
        300 => Some(0xFCA5A5),
        400 => Some(0xF87171),
        500 => Some(0xEF4444),
        600 => Some(0xDC2626),
        700 => Some(0xB91C1C),
        800 => Some(0x991B1B),
        900 => Some(0x7F1D1D),
        950 => Some(0x450A0A),
        _ => None,
    }
}

fn get_orange_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFFF7ED),
        100 => Some(0xFFEDD5),
        200 => Some(0xFED7AA),
        300 => Some(0xFDBA74),
        400 => Some(0xFB923C),
        500 => Some(0xF97316),
        600 => Some(0xEA580C),
        700 => Some(0xC2410C),
        800 => Some(0x9A3412),
        900 => Some(0x7C2D12),
        950 => Some(0x431407),
        _ => None,
    }
}

fn get_amber_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFFFBEB),
        100 => Some(0xFEF3C7),
        200 => Some(0xFDE68A),
        300 => Some(0xFCD34D),
        400 => Some(0xFBBF24),
        500 => Some(0xF59E0B),
        600 => Some(0xD97706),
        700 => Some(0xB45309),
        800 => Some(0x92400E),
        900 => Some(0x78350F),
        950 => Some(0x451A03),
        _ => None,
    }
}

fn get_yellow_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFEFCE8),
        100 => Some(0xFEF9C3),
        200 => Some(0xFEF08A),
        300 => Some(0xFDE047),
        400 => Some(0xFACC15),
        500 => Some(0xEAB308),
        600 => Some(0xCA8A04),
        700 => Some(0xA16207),
        800 => Some(0x854D0E),
        900 => Some(0x713F12),
        950 => Some(0x422006),
        _ => None,
    }
}

fn get_lime_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF7FEE7),
        100 => Some(0xECFCCB),
        200 => Some(0xD9F99D),
        300 => Some(0xBEF264),
        400 => Some(0xA3E635),
        500 => Some(0x84CC16),
        600 => Some(0x65A30D),
        700 => Some(0x4D7C0F),
        800 => Some(0x3F6212),
        900 => Some(0x365314),
        950 => Some(0x1A2E05),
        _ => None,
    }
}

fn get_green_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF0FDF4),
        100 => Some(0xDCFCE7),
        200 => Some(0xBBF7D0),
        300 => Some(0x86EFAC),
        400 => Some(0x4ADE80),
        500 => Some(0x22C55E),
        600 => Some(0x16A34A),
        700 => Some(0x15803D),
        800 => Some(0x166534),
        900 => Some(0x14532D),
        950 => Some(0x052E16),
        _ => None,
    }
}

fn get_emerald_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xECFDF5),
        100 => Some(0xD1FAE5),
        200 => Some(0xA7F3D0),
        300 => Some(0x6EE7B7),
        400 => Some(0x34D399),
        500 => Some(0x10B981),
        600 => Some(0x059669),
        700 => Some(0x047857),
        800 => Some(0x065F46),
        900 => Some(0x064E3B),
        950 => Some(0x022C22),
        _ => None,
    }
}

fn get_teal_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF0FDFA),
        100 => Some(0xCCFBF1),
        200 => Some(0x99F6E4),
        300 => Some(0x5EEAD4),
        400 => Some(0x2DD4BF),
        500 => Some(0x14B8A6),
        600 => Some(0x0D9488),
        700 => Some(0x0F766E),
        800 => Some(0x115E59),
        900 => Some(0x134E4A),
        950 => Some(0x042F2E),
        _ => None,
    }
}

fn get_cyan_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xECFEFF),
        100 => Some(0xCFFAFE),
        200 => Some(0xA5F3FC),
        300 => Some(0x67E8F9),
        400 => Some(0x22D3EE),
        500 => Some(0x06B6D4),
        600 => Some(0x0891B2),
        700 => Some(0x0E7490),
        800 => Some(0x155E75),
        900 => Some(0x164E63),
        950 => Some(0x083344),
        _ => None,
    }
}

fn get_sky_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF0F9FF),
        100 => Some(0xE0F2FE),
        200 => Some(0xBAE6FD),
        300 => Some(0x7DD3FC),
        400 => Some(0x38BDF8),
        500 => Some(0x0EA5E9),
        600 => Some(0x0284C7),
        700 => Some(0x0369A1),
        800 => Some(0x075985),
        900 => Some(0x0C4A6E),
        950 => Some(0x082F49),
        _ => None,
    }
}

fn get_blue_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xEFF6FF),
        100 => Some(0xDBEAFE),
        200 => Some(0xBFDBFE),
        300 => Some(0x93C5FD),
        400 => Some(0x60A5FA),
        500 => Some(0x3B82F6),
        600 => Some(0x2563EB),
        700 => Some(0x1D4ED8),
        800 => Some(0x1E40AF),
        900 => Some(0x1E3A8A),
        950 => Some(0x172554),
        _ => None,
    }
}

fn get_indigo_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xEEF2FF),
        100 => Some(0xE0E7FF),
        200 => Some(0xC7D2FE),
        300 => Some(0xA5B4FC),
        400 => Some(0x818CF8),
        500 => Some(0x6366F1),
        600 => Some(0x4F46E5),
        700 => Some(0x4338CA),
        800 => Some(0x3730A3),
        900 => Some(0x312E81),
        950 => Some(0x1E1B4B),
        _ => None,
    }
}

fn get_violet_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xF5F3FF),
        100 => Some(0xEDE9FE),
        200 => Some(0xDDD6FE),
        300 => Some(0xC4B5FD),
        400 => Some(0xA78BFA),
        500 => Some(0x8B5CF6),
        600 => Some(0x7C3AED),
        700 => Some(0x6D28D9),
        800 => Some(0x5B21B6),
        900 => Some(0x4C1D95),
        950 => Some(0x2E1065),
        _ => None,
    }
}

fn get_purple_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFAF5FF),
        100 => Some(0xF3E8FF),
        200 => Some(0xE9D5FF),
        300 => Some(0xD8B4FE),
        400 => Some(0xC084FC),
        500 => Some(0xA855F7),
        600 => Some(0x9333EA),
        700 => Some(0x7E22CE),
        800 => Some(0x6B21A8),
        900 => Some(0x581C87),
        950 => Some(0x3B0764),
        _ => None,
    }
}

fn get_fuchsia_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFDF4FF),
        100 => Some(0xFAE8FF),
        200 => Some(0xF5D0FE),
        300 => Some(0xF0ABFC),
        400 => Some(0xE879F9),
        500 => Some(0xD946EF),
        600 => Some(0xC026D3),
        700 => Some(0xA21CAF),
        800 => Some(0x86198F),
        900 => Some(0x701A75),
        950 => Some(0x4A044E),
        _ => None,
    }
}

fn get_pink_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFDF2F8),
        100 => Some(0xFCE7F3),
        200 => Some(0xFBCFE8),
        300 => Some(0xF9A8D4),
        400 => Some(0xF472B6),
        500 => Some(0xEC4899),
        600 => Some(0xDB2777),
        700 => Some(0xBE185D),
        800 => Some(0x9D174D),
        900 => Some(0x831843),
        950 => Some(0x500724),
        _ => None,
    }
}

fn get_rose_color(shade: u32) -> Option<u32> {
    match shade {
        50 => Some(0xFFF1F2),
        100 => Some(0xFFE4E6),
        200 => Some(0xFECDD3),
        300 => Some(0xFDA4AF),
        400 => Some(0xFB7185),
        500 => Some(0xF43F5E),
        600 => Some(0xE11D48),
        700 => Some(0xBE123C),
        800 => Some(0x9F1239),
        900 => Some(0x881337),
        950 => Some(0x4C0519),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tailwind_flex_classes() {
        let styles = TailwindStyles::parse("flex flex-col items-center justify-between");
        assert!(styles.flex);
        assert!(styles.flex_col);
        assert!(styles.items_center);
        assert!(styles.justify_between);
    }

    #[test]
    fn test_tailwind_spacing_classes() {
        let styles = TailwindStyles::parse("p-4 px-2 mt-8 gap-2");
        assert_eq!(styles.padding, Some(16.0));
        assert_eq!(styles.padding_x, Some(8.0));
        assert_eq!(styles.margin_top, Some(32.0));
        assert_eq!(styles.gap, Some(8.0));
    }

    #[test]
    fn test_tailwind_color_classes() {
        let styles = TailwindStyles::parse("text-white bg-blue-500 border-gray-300");
        assert_eq!(styles.text_color, Some(0xFFFFFF));
        assert_eq!(styles.bg_color, Some(0x3B82F6));
        assert_eq!(styles.border_color, Some(0xD1D5DB));
    }

    #[test]
    fn test_tailwind_typography_classes() {
        let styles = TailwindStyles::parse("text-2xl font-bold");
        assert_eq!(styles.font_size, Some(24.0));
        assert!(styles.font_bold);
    }

    #[test]
    fn test_tailwind_border_radius_classes() {
        let styles = TailwindStyles::parse("rounded-lg");
        assert_eq!(styles.rounded, Some(8.0));

        let styles = TailwindStyles::parse("rounded-full");
        assert_eq!(styles.rounded, Some(9999.0));
    }

    #[test]
    fn test_tailwind_sizing_classes() {
        let styles = TailwindStyles::parse("w-full h-full min-w-0");
        assert!(styles.w_full);
        assert!(styles.h_full);
        assert!(styles.min_w_0);
    }
}
