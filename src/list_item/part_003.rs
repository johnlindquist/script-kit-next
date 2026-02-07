fn decode_png_to_render_image_internal(
    png_data: &[u8],
    convert_to_bgra: bool,
) -> Result<Arc<RenderImage>, image::ImageError> {
    use image::GenericImageView;
    use smallvec::SmallVec;

    // Decode PNG
    let img = image::load_from_memory(png_data)?;

    // Convert to RGBA8
    let mut rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    // Convert RGBA to BGRA for Metal/GPUI rendering
    // GPUI's internal image loading does this swap (see gpui/src/platform.rs)
    // We must do the same when creating RenderImage directly from image::Frame
    if convert_to_bgra {
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2); // Swap R and B: RGBA -> BGRA
        }
    }

    // Create Frame from buffer (now in BGRA order if converted)
    let buffer = image::RgbaImage::from_raw(width, height, rgba.into_raw())
        .expect("Failed to create image buffer");
    let frame = image::Frame::new(buffer);

    // Create RenderImage
    let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

    Ok(Arc::new(render_image))
}
/// Create an IconKind from PNG bytes by pre-decoding them
///
/// Returns None if decoding fails. This should be called once when loading
/// icons, not during rendering.
pub fn icon_from_png(png_data: &[u8]) -> Option<IconKind> {
    decode_png_to_render_image(png_data)
        .ok()
        .map(IconKind::Image)
}
/// Render a section header for grouped lists (e.g., "Recent", "Main")
///
/// Visual design for section headers:
/// - Standard casing (not uppercase)
/// - 12px font (meets desktop minimum)
/// - Semi-bold weight (SEMIBOLD for subtlety)
/// - Dimmed color (subtle but readable)
/// - 32px height (8px grid aligned)
/// - Left-aligned with list item padding
/// - Subtle background tint for visual grouping
///
/// ## Technical Note: list() Height
/// Uses GPUI's `list()` component which supports variable-height items.
/// Section headers render at 32px, regular items at 40px.
///
/// # Arguments
/// * `label` - The section label (displayed as-is, standard casing)
/// * `icon` - Optional icon name (lucide icon, e.g., "settings")
/// * `colors` - ListItemColors for theme-aware styling
/// * `is_first` - Whether this is the first header in the list (suppresses top border)
///
pub fn render_section_header(
    label: &str,
    icon: Option<&str>,
    colors: ListItemColors,
    is_first: bool,
) -> impl IntoElement {
    // Section header at 32px (8px grid aligned, SECTION_HEADER_HEIGHT)
    // Used with GPUI's list() component which supports variable-height items.
    //
    // Layout: 32px total height
    // - pt(12px) top padding for visual separation from above item
    // - ~12px text height
    // - pb(4px) bottom padding for visual separation from below item

    // Parse label to separate name from count (e.g., "SUGGESTED · 5" → "SUGGESTED", "5")
    let (section_name, count_text) = if let Some(dot_pos) = label.find(" · ") {
        (&label[..dot_pos], Some(&label[dot_pos + " · ".len()..]))
    } else {
        (label, None)
    };

    // Build the inner content row: icon (optional) → section name → count (optional)
    // Headers should whisper — subtle orientation labels, not attention-grabbers
    let header_text_color = rgba((colors.text_secondary << 8) | ALPHA_ICON_QUIET);
    let mut content = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(SECTION_GAP))
        .text_size(px(SECTION_HEADER_FONT_SIZE))
        .font_weight(FontWeight::NORMAL) // Lightest weight — headers recede behind items
        .text_color(header_text_color);

    // Add icon before section name if provided — very quiet to avoid visual noise
    if let Some(name) = icon {
        if let Some(icon_name) = icon_name_from_str(name) {
            content = content.child(
                svg()
                    .external_path(icon_name.external_path())
                    .size(px(SECTION_HEADER_ICON_SIZE))
                    .text_color(rgba((colors.text_secondary << 8) | ALPHA_DESC_QUIET)),
            );
        }
    }

    content = content.child(section_name.to_string());

    // Add count badge if present - rendered as a very subtle separate element
    if let Some(count) = count_text {
        content = content.child(
            div()
                .text_xs()
                .font_weight(FontWeight::NORMAL)
                .text_color(rgba((colors.text_secondary << 8) | ALPHA_DESC_QUIET))
                .child(count.to_string()),
        );
    }

    // Clean section headers — no background tint for a calmer list appearance
    let header = div()
        .w_full()
        .h(px(SECTION_HEADER_HEIGHT))
        .px(px(SECTION_PADDING_X))
        .pt(px(SECTION_PADDING_TOP))
        .pb(px(SECTION_PADDING_BOTTOM))
        .flex()
        .flex_col()
        .justify_end(); // Align content to bottom for better visual anchoring

    // Only show top separator on non-first headers — very subtle
    let header = if is_first {
        header
    } else {
        header
            .border_t_1()
            .border_color(rgba((colors.text_secondary << 8) | ALPHA_SEPARATOR))
    };

    header.child(content)
}
// Note: GPUI rendering tests omitted due to GPUI macro recursion limit issues.
// The LIST_ITEM_HEIGHT constant is 40.0 and the component is integration-tested
// via the main application's script list and arg prompt rendering.
// Unit tests for format_shortcut_display are in src/list_item_tests.rs.
