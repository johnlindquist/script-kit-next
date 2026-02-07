#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_scan_applications_returns_apps() {
        let apps = scan_applications();

        // Should find at least some apps on any macOS system
        assert!(
            !apps.is_empty(),
            "Should find at least some applications on macOS"
        );

        // Check that Calculator exists (it's always present in /System/Applications on macOS)
        let calculator = apps.iter().find(|a| a.name == "Calculator");
        assert!(calculator.is_some(), "Calculator.app should be found");

        if let Some(calculator) = calculator {
            assert!(
                calculator.path.exists(),
                "Calculator path should exist: {:?}",
                calculator.path
            );
            assert!(
                calculator.bundle_id.is_some(),
                "Calculator should have a bundle ID"
            );
            assert_eq!(
                calculator.bundle_id.as_deref(),
                Some("com.apple.calculator"),
                "Calculator bundle ID should be com.apple.calculator"
            );
        }
    }

    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_app_info_has_required_fields() {
        let apps = scan_applications();

        for app in apps.iter().take(10) {
            // Name should not be empty
            assert!(!app.name.is_empty(), "App name should not be empty");

            // Path should end with .app
            assert!(
                app.path.extension().map(|e| e == "app").unwrap_or(false),
                "App path should end with .app: {:?}",
                app.path
            );

            // Path should exist
            assert!(app.path.exists(), "App path should exist: {:?}", app.path);
        }
    }

    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_apps_sorted_alphabetically() {
        let apps = scan_applications();

        // Verify apps are sorted by lowercase name
        for window in apps.windows(2) {
            let a = &window[0];
            let b = &window[1];
            assert!(
                a.name.to_lowercase() <= b.name.to_lowercase(),
                "Apps should be sorted: {} should come before {}",
                a.name,
                b.name
            );
        }
    }

    #[test]
    fn test_extract_bundle_id_finder() {
        let finder_path = Path::new("/System/Applications/Finder.app");
        if finder_path.exists() {
            let bundle_id = extract_bundle_id(finder_path);
            assert_eq!(
                bundle_id,
                Some("com.apple.finder".to_string()),
                "Should extract Finder bundle ID"
            );
        }
    }

    #[test]
    fn test_extract_bundle_id_nonexistent() {
        let fake_path = Path::new("/nonexistent/Fake.app");
        let bundle_id = extract_bundle_id(fake_path);
        assert!(
            bundle_id.is_none(),
            "Should return None for nonexistent app"
        );
    }

    #[test]
    fn test_parse_app_bundle() {
        let finder_path = Path::new("/System/Applications/Finder.app");
        if finder_path.exists() {
            let app_info = parse_app_bundle(finder_path);
            assert!(app_info.is_some(), "Should parse Finder.app");

            let app = app_info.unwrap();
            assert_eq!(app.name, "Finder");
            assert_eq!(app.path, finder_path);
            assert!(app.bundle_id.is_some());
        }
    }

    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_no_duplicate_apps() {
        let apps = scan_applications();
        // Use a set to check for true duplicates
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for app in apps.iter() {
            let lower_name = app.name.to_lowercase();
            if !seen.insert(lower_name.clone()) {
                duplicates.push(app.name.clone());
            }
        }

        // Allow a small number of duplicates (some systems have app variants)
        // e.g., same app name in different locations
        assert!(
            duplicates.len() <= 5,
            "Too many duplicate app names ({}): {:?}",
            duplicates.len(),
            duplicates
        );
    }

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_extract_app_icon() {
        // Test icon extraction for Calculator (always present on macOS)
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if calculator_path.exists() {
            let icon = extract_app_icon(calculator_path);
            assert!(icon.is_some(), "Should extract Calculator icon");

            if let Some(icon_data) = icon {
                // PNG magic bytes: 0x89 0x50 0x4E 0x47
                assert!(icon_data.len() > 8, "Icon data should have content");
                assert_eq!(
                    &icon_data[0..4],
                    &[0x89, 0x50, 0x4E, 0x47],
                    "Icon should be valid PNG"
                );
            }
        }
    }

    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_app_has_icon() {
        // Use fresh scan which does synchronous icon loading
        // (scan_applications() defers icon decoding to background for performance)
        let apps = scan_applications_fresh();

        // Check that at least some apps have icons (most should)
        let apps_with_icons = apps.iter().filter(|a| a.icon.is_some()).count();

        // Most apps should have icons - at least 50%
        assert!(
            apps_with_icons > apps.len() / 2,
            "At least half of apps should have icons, got {}/{}",
            apps_with_icons,
            apps.len()
        );
    }

    // Note: launch_application is not tested automatically to avoid
    // actually launching apps during test runs. It can be tested manually.

    /// Test that load_apps_from_db returns apps WITH icons decoded synchronously.
    ///
    /// The bug was that a previous version deferred icon decoding to a background
    /// thread that updated a LOCAL Arc, then returned a clone of the Vec without icons.
    /// The fix is to decode icons synchronously in load_apps_from_db.
    #[cfg(feature = "slow-tests")]
    #[test]
    fn test_load_apps_from_db_returns_apps_with_icons() {
        // First, ensure we have some apps in the database by doing a fresh scan
        // This populates the SQLite DB with apps including icon blobs
        let fresh_apps = scan_applications_fresh();
        assert!(!fresh_apps.is_empty(), "Should have apps after fresh scan");

        // Count how many apps have icons after fresh scan
        let fresh_with_icons = fresh_apps.iter().filter(|a| a.icon.is_some()).count();
        assert!(
            fresh_with_icons > 0,
            "Fresh scan should produce some apps with icons"
        );

        // Now test that load_apps_from_db returns apps WITH icons decoded
        let cached_apps = load_apps_from_db();

        // Verify we got apps
        assert!(!cached_apps.is_empty(), "Should load apps from DB");

        // Count apps with icons from cache - should match or be close to fresh scan
        let cached_with_icons = cached_apps.iter().filter(|a| a.icon.is_some()).count();

        // The fix ensures icons are decoded synchronously, so cached apps should have icons
        assert!(
            cached_with_icons > 0,
            "Cached apps should have icons decoded. Found {} apps but {} with icons",
            cached_apps.len(),
            cached_with_icons
        );
    }

    #[test]
    fn test_hash_path() {
        let path1 = Path::new("/Applications/Safari.app");
        let path2 = Path::new("/Applications/Safari.app");
        let path3 = Path::new("/Applications/Finder.app");

        // Same path should produce same hash
        assert_eq!(
            hash_path(path1),
            hash_path(path2),
            "Same path should produce same hash"
        );

        // Different paths should produce different hashes
        assert_ne!(
            hash_path(path1),
            hash_path(path3),
            "Different paths should produce different hashes"
        );

        // Hash should be 16 hex characters
        let hash = hash_path(path1);
        assert_eq!(hash.len(), 16, "Hash should be 16 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex characters: {}",
            hash
        );
    }

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_get_or_extract_icon_caches() {
        // Test that get_or_extract_icon properly caches icons
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if !calculator_path.exists() {
            return;
        }

        // First call - may or may not hit cache
        let icon1 = get_or_extract_icon(calculator_path);
        assert!(icon1.is_some(), "Should extract Calculator icon");

        // Second call should hit cache
        let icon2 = get_or_extract_icon(calculator_path);
        assert!(icon2.is_some(), "Should load Calculator icon from cache");

        // Both should have the same content
        let bytes1 = icon1.unwrap();
        let bytes2 = icon2.unwrap();
        assert_eq!(bytes1, bytes2, "Cached icon should match extracted icon");

        // Verify cache file exists
        let cache_dir = get_icon_cache_dir().unwrap();
        let cache_key = hash_path(calculator_path);
        let cache_file = cache_dir.join(format!("{}.png", cache_key));
        assert!(
            cache_file.exists(),
            "Cache file should exist: {:?}",
            cache_file
        );
    }

    #[test]
    fn test_decode_with_rb_swap() {
        use image::ImageEncoder;

        // Create a simple 2x2 PNG with known colors
        // Pixel at (0,0) = Red (255, 0, 0, 255)
        // Pixel at (1,0) = Blue (0, 0, 255, 255)
        // Pixel at (0,1) = Green (0, 255, 0, 255)
        // Pixel at (1,1) = White (255, 255, 255, 255)
        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255])); // Red
        img.put_pixel(1, 0, image::Rgba([0, 0, 255, 255])); // Blue
        img.put_pixel(0, 1, image::Rgba([0, 255, 0, 255])); // Green
        img.put_pixel(1, 1, image::Rgba([255, 255, 255, 255])); // White

        // Encode to PNG
        let mut original_png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut original_png);
        encoder
            .write_image(&img, 2, 2, image::ExtendedColorType::Rgba8)
            .expect("Failed to encode PNG");

        // Use the decode function with BGRA conversion
        let render_image =
            crate::list_item::decode_png_to_render_image_with_bgra_conversion(&original_png)
                .expect("Should decode with BGRA conversion");

        // Verify we got a RenderImage (we can't easily inspect pixels in RenderImage,
        // but we can verify it was created successfully)
        assert!(
            std::sync::Arc::strong_count(&render_image) >= 1,
            "Should create valid RenderImage"
        );
    }

    #[test]
    fn test_get_icon_cache_stats() {
        let (count, size) = get_icon_cache_stats();
        // We can't make strong assertions about exact counts since
        // other tests may have populated the cache, but we can check types
        assert!(
            count == 0 || size > 0,
            "If there are cached files, size should be non-zero"
        );
    }

    #[test]
    fn test_get_apps_db_path() {
        let db_path = get_apps_db_path();
        assert!(
            db_path.ends_with("db/apps.sqlite"),
            "DB path should end with db/apps.sqlite: {:?}",
            db_path
        );
        assert!(
            db_path.to_string_lossy().contains(".scriptkit"),
            "DB path should be under .scriptkit: {:?}",
            db_path
        );
    }

    #[test]
    fn test_loading_state() {
        // Initial state should be Ready (default)
        let state = get_app_loading_state();
        // Note: state may vary if other tests are running

        // Test message generation
        assert!(!state.message().is_empty(), "Should have a message");
    }

    #[test]
    fn test_get_apps_db_stats() {
        let (count, size) = get_apps_db_stats();
        // Stats should be valid - size is usize so always >= 0
        // Just verify the function returns without error
        let _ = (count, size);
    }
}
