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

    #[cfg(target_os = "macos")]
    {
        let styles = [
            JETBRAINS_MONO_REGULAR,
            JETBRAINS_MONO_BOLD,
            JETBRAINS_MONO_ITALIC,
            JETBRAINS_MONO_BOLD_ITALIC,
            JETBRAINS_MONO_MEDIUM,
            JETBRAINS_MONO_SEMIBOLD,
        ];
        let mut registered_count = 0;
        for font_data in styles {
            if unsafe { register_font_process_wide(font_data) } {
                registered_count += 1;
            }
        }
        logging::log(
            "FONT",
            &format!(
                "Registered {} of 6 JetBrains Mono styles process-wide in CoreText",
                registered_count
            ),
        );
    }
}

#[cfg(target_os = "macos")]
unsafe fn register_font_process_wide(font_data: &'static [u8]) -> bool {
    use std::ptr;
    use std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGDataProviderCreateWithData(
            info: *mut c_void,
            data: *const c_void,
            size: usize,
            releaseData: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize)>,
        ) -> *mut c_void;
        fn CGFontCreateWithDataProvider(
            provider: *mut c_void,
        ) -> *mut c_void;
    }

    #[link(name = "CoreText", kind = "framework")]
    extern "C" {
        fn CTFontManagerRegisterGraphicsFont(
            font: *mut c_void,
            error: *mut *mut c_void,
        ) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *mut c_void);
    }

    let provider = CGDataProviderCreateWithData(
        ptr::null_mut(),
        font_data.as_ptr() as *const c_void,
        font_data.len(),
        None,
    );
    if provider.is_null() {
        return false;
    }

    let cg_font = CGFontCreateWithDataProvider(provider);
    CFRelease(provider);
    if cg_font.is_null() {
        return false;
    }

    let mut error = ptr::null_mut();
    let success = CTFontManagerRegisterGraphicsFont(cg_font, &mut error);
    CFRelease(cg_font);

    if !success && !error.is_null() {
        CFRelease(error);
    }

    success
}
