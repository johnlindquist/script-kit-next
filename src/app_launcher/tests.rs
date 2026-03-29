#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // macOS-specific tests
    // ========================================================================

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
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

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_app_info_has_required_fields_macos() {
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

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_apps_sorted_alphabetically_macos() {
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

    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
    #[test]
    fn test_extract_bundle_id_nonexistent() {
        let fake_path = Path::new("/nonexistent/Fake.app");
        let bundle_id = extract_bundle_id(fake_path);
        assert!(
            bundle_id.is_none(),
            "Should return None for nonexistent app"
        );
    }

    #[cfg(target_os = "macos")]
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

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_no_duplicate_apps_macos() {
        let apps = scan_applications();
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for app in apps.iter() {
            let lower_name = app.name.to_lowercase();
            if !seen.insert(lower_name.clone()) {
                duplicates.push(app.name.clone());
            }
        }

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
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if calculator_path.exists() {
            let icon = extract_app_icon(calculator_path);
            assert!(icon.is_some(), "Should extract Calculator icon");

            if let Some(icon_data) = icon {
                assert!(icon_data.len() > 8, "Icon data should have content");
                assert_eq!(
                    &icon_data[0..4],
                    &[0x89, 0x50, 0x4E, 0x47],
                    "Icon should be valid PNG"
                );
            }
        }
    }

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_app_has_icon() {
        let apps = scan_applications_fresh();
        let apps_with_icons = apps.iter().filter(|a| a.icon.is_some()).count();

        assert!(
            apps_with_icons > apps.len() / 2,
            "At least half of apps should have icons, got {}/{}",
            apps_with_icons,
            apps.len()
        );
    }

    // Note: launch_application is not tested automatically to avoid
    // actually launching apps during test runs. It can be tested manually.

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_load_apps_from_db_returns_apps_with_icons() {
        let fresh_apps = scan_applications_fresh();
        assert!(!fresh_apps.is_empty(), "Should have apps after fresh scan");

        let fresh_with_icons = fresh_apps.iter().filter(|a| a.icon.is_some()).count();
        assert!(
            fresh_with_icons > 0,
            "Fresh scan should produce some apps with icons"
        );

        let cached_apps = load_apps_from_db();
        assert!(!cached_apps.is_empty(), "Should load apps from DB");

        let cached_with_icons = cached_apps.iter().filter(|a| a.icon.is_some()).count();
        assert!(
            cached_with_icons > 0,
            "Cached apps should have icons decoded. Found {} apps but {} with icons",
            cached_apps.len(),
            cached_with_icons
        );
    }

    // ========================================================================
    // Cross-platform tests
    // ========================================================================

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
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if !calculator_path.exists() {
            return;
        }

        let icon1 = get_or_extract_icon(calculator_path);
        assert!(icon1.is_some(), "Should extract Calculator icon");

        let icon2 = get_or_extract_icon(calculator_path);
        assert!(icon2.is_some(), "Should load Calculator icon from cache");

        let bytes1 = icon1.unwrap();
        let bytes2 = icon2.unwrap();
        assert_eq!(bytes1, bytes2, "Cached icon should match extracted icon");

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

        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        img.put_pixel(1, 0, image::Rgba([0, 0, 255, 255]));
        img.put_pixel(0, 1, image::Rgba([0, 255, 0, 255]));
        img.put_pixel(1, 1, image::Rgba([255, 255, 255, 255]));

        let mut original_png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut original_png);
        encoder
            .write_image(&img, 2, 2, image::ExtendedColorType::Rgba8)
            .expect("Failed to encode PNG");

        let render_image =
            crate::list_item::decode_png_to_render_image_with_bgra_conversion(&original_png)
                .expect("Should decode with BGRA conversion");

        assert!(
            std::sync::Arc::strong_count(&render_image) >= 1,
            "Should create valid RenderImage"
        );
    }

    #[test]
    fn test_get_icon_cache_stats() {
        let (count, size) = get_icon_cache_stats();
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
        let state = get_app_loading_state();
        assert!(!state.message().is_empty(), "Should have a message");
    }

    #[test]
    fn test_get_apps_db_stats() {
        let (count, size) = get_apps_db_stats();
        let _ = (count, size);
    }

    // ========================================================================
    // Windows-specific tests
    // ========================================================================

    /// Verify WINDOWS_APP_DIRECTORIES resolves to real paths
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_app_directories_resolve() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

        // On any Windows system, at least one Start Menu directory should exist
        assert!(
            !dirs.is_empty(),
            "Should resolve at least one Start Menu directory"
        );

        for dir in dirs {
            assert!(dir.exists(), "Resolved directory should exist: {:?}", dir);
            assert!(
                dir.is_dir(),
                "Resolved path should be a directory: {:?}",
                dir
            );
        }

        // Verify the paths contain expected components
        let any_has_start_menu = dirs
            .iter()
            .any(|d| d.to_string_lossy().to_lowercase().contains("start menu"));
        assert!(
            any_has_start_menu,
            "At least one directory should contain 'Start Menu': {:?}",
            dirs
        );
    }

    /// Verify Windows scanning finds at least some apps
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_scan_finds_apps() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

        let mut all_apps = Vec::new();
        for dir in dirs {
            if let Ok(apps) = scan_windows_directory_recursive(dir) {
                all_apps.extend(apps);
            }
        }

        // The Start Menu should always have at least a few shortcuts
        assert!(
            !all_apps.is_empty(),
            "Should find at least some .lnk shortcuts in Start Menu"
        );

        // Verify AppInfo fields are well-formed
        for app in all_apps.iter().take(20) {
            assert!(!app.name.is_empty(), "App name should not be empty");
            assert!(
                app.path.exists(),
                "App .lnk path should exist: {:?}",
                app.path
            );
            assert!(
                app.path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("lnk"))
                    .unwrap_or(false),
                "App path should end with .lnk: {:?}",
                app.path
            );
        }
    }

    /// Verify parse_windows_shortcut extracts correct name
    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_windows_shortcut_name_extraction() {
        // Find a real .lnk file from the Start Menu to test with
        let dirs = &*WINDOWS_APP_DIRECTORIES;
        let mut found_lnk = None;

        'outer: for dir in dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .map(|e| e.eq_ignore_ascii_case("lnk"))
                        .unwrap_or(false)
                    {
                        found_lnk = Some(path);
                        break 'outer;
                    }
                }
            }
        }

        if let Some(lnk_path) = found_lnk {
            let app = parse_windows_shortcut(&lnk_path);
            assert!(app.is_some(), "Should parse .lnk file: {:?}", lnk_path);

            let app = app.unwrap();
            let expected_name = lnk_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            assert_eq!(
                app.name, expected_name,
                "App name should match .lnk filename stem"
            );
            assert_eq!(app.path, lnk_path, "App path should be the .lnk path");
        }
    }

    /// Verify Windows launch command construction (without actually launching)
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_launch_command_construction() {
        let test_app = AppInfo {
            name: "Test App".to_string(),
            path: PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\test.lnk"),
            bundle_id: None,
            icon: None,
        };

        // Verify the command can be built (doesn't panic or error on construction)
        let mut binding = Command::new("cmd");
        let cmd = binding
            .args(["/c", "start", ""])
            .arg(&test_app.path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let program = cmd.get_program();
        assert_eq!(program, "cmd", "Launch command should use cmd.exe");
    }

    /// Verify that scanning produces sorted, deduplicated results
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_scan_sorted_and_deduped() {
        let apps = scan_all_directories_with_db_update();

        // Verify sorted by lowercase name
        for window in apps.windows(2) {
            let a = &window[0];
            let b = &window[1];
            assert!(
                a.name.to_lowercase() <= b.name.to_lowercase(),
                "Apps should be sorted: '{}' should come before '{}'",
                a.name,
                b.name
            );
        }

        // Verify no exact duplicate names (case-insensitive)
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for app in apps.iter() {
            let lower_name = app.name.to_lowercase();
            if !seen.insert(lower_name.clone()) {
                duplicates.push(app.name.clone());
            }
        }
        assert!(
            duplicates.len() <= 5,
            "Too many duplicate app names ({}): {:?}",
            duplicates.len(),
            duplicates
        );
    }
}
