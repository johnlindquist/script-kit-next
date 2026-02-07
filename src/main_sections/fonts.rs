/// Register bundled JetBrains Mono font with GPUI's text system
///
/// This embeds the font files directly in the binary and registers them
/// at application startup, making "JetBrains Mono" available as a font family.
fn register_bundled_fonts(cx: &mut App) {
    use std::borrow::Cow;

    // Embed font files at compile time
    static JETBRAINS_MONO_REGULAR: &[u8] =
        include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
    static JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf");
    static JETBRAINS_MONO_ITALIC: &[u8] =
        include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf");
    static JETBRAINS_MONO_BOLD_ITALIC: &[u8] =
        include_bytes!("../../assets/fonts/JetBrainsMono-BoldItalic.ttf");
    static JETBRAINS_MONO_MEDIUM: &[u8] =
        include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf");
    static JETBRAINS_MONO_SEMIBOLD: &[u8] =
        include_bytes!("../../assets/fonts/JetBrainsMono-SemiBold.ttf");

    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(JETBRAINS_MONO_REGULAR),
        Cow::Borrowed(JETBRAINS_MONO_BOLD),
        Cow::Borrowed(JETBRAINS_MONO_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_BOLD_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_MEDIUM),
        Cow::Borrowed(JETBRAINS_MONO_SEMIBOLD),
    ];

    match cx.text_system().add_fonts(fonts) {
        Ok(()) => {
            logging::log("FONT", "Registered JetBrains Mono font family (6 styles)");
        }
        Err(e) => {
            logging::log(
                "FONT",
                &format!(
                    "Failed to register JetBrains Mono: {}. Falling back to system font.",
                    e
                ),
            );
        }
    }
}
