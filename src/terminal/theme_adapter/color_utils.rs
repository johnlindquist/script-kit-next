use vte::ansi::Rgb;

/// Converts a u32 hex color (0xRRGGBB) to an Rgb struct.
#[inline]
pub fn hex_to_rgb(hex: u32) -> Rgb {
    Rgb {
        r: ((hex >> 16) & 0xFF) as u8,
        g: ((hex >> 8) & 0xFF) as u8,
        b: (hex & 0xFF) as u8,
    }
}

/// Dims a color by blending it toward mid-gray.
pub(super) fn dim_color(color: Rgb, factor: f32) -> Rgb {
    const GRAY: u8 = 0x80;

    let blend = |c: u8| -> u8 {
        let c = c as f32;
        let gray = GRAY as f32;
        ((c * factor + gray * (1.0 - factor)).clamp(0.0, 255.0)) as u8
    };

    Rgb {
        r: blend(color.r),
        g: blend(color.g),
        b: blend(color.b),
    }
}
