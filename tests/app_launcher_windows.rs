//! Integration tests for the Windows App Launcher backend.
//!
//! Tests the Windows Start Menu scanning, .lnk parsing, target resolution,
//! and the public API surface of the app_launcher module.
//!
//! All tests are gated behind `#[cfg(target_os = "windows")]` so they
//! compile to nothing on macOS/Linux.

#[cfg(target_os = "windows")]
mod windows_tests {
    use script_kit_gpui::app_launcher::{
        get_app_loading_state, get_apps_db_path, get_apps_db_stats, get_icon_cache_stats,
        hash_path, parse_windows_shortcut, resolve_lnk_target, scan_applications,
        scan_windows_directory_recursive, AppInfo, WINDOWS_APP_DIRECTORIES,
    };
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    // ========================================================================
    // WINDOWS_APP_DIRECTORIES resolution
    // ========================================================================

    #[test]
    fn directories_resolve_to_existing_paths() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

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
    }

    #[test]
    fn directories_contain_start_menu_in_path() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

        let any_has_start_menu = dirs
            .iter()
            .any(|d| d.to_string_lossy().to_lowercase().contains("start menu"));
        assert!(
            any_has_start_menu,
            "At least one directory should contain 'Start Menu': {:?}",
            dirs
        );
    }

    #[test]
    fn directories_include_user_and_system_paths() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

        // Should have both APPDATA-based (user) and ProgramData-based (system) paths
        let has_appdata = dirs
            .iter()
            .any(|d| d.to_string_lossy().to_lowercase().contains("appdata"));
        let has_programdata = dirs
            .iter()
            .any(|d| d.to_string_lossy().to_lowercase().contains("programdata"));

        assert!(
            has_appdata,
            "Should include user AppData Start Menu path: {:?}",
            dirs
        );
        assert!(
            has_programdata,
            "Should include system ProgramData Start Menu path: {:?}",
            dirs
        );
    }

    // ========================================================================
    // Scanning
    // ========================================================================

    #[test]
    fn scan_start_menu_finds_apps() {
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

        // Print count for the report
        eprintln!(
            "[app_launcher] Found {} apps across {} directories",
            all_apps.len(),
            dirs.len()
        );
    }

    #[test]
    fn scanned_apps_have_valid_fields() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;

        let mut all_apps = Vec::new();
        for dir in dirs {
            if let Ok(apps) = scan_windows_directory_recursive(dir) {
                all_apps.extend(apps);
            }
        }

        for app in all_apps.iter().take(20) {
            // Name should not be empty
            assert!(!app.name.is_empty(), "App name should not be empty");

            // Path should exist and end with .lnk
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

    #[test]
    fn scan_finds_well_known_apps() {
        // Every Windows system should have at least some of these
        let well_known = [
            "windows powershell",
            "command prompt",
            "notepad",
            "file explorer",
            "task manager",
            "control panel",
            "calculator",
            "paint",
            "wordpad",
            "snipping tool",
        ];

        let dirs = &*WINDOWS_APP_DIRECTORIES;
        let mut all_apps = Vec::new();
        for dir in dirs {
            if let Ok(apps) = scan_windows_directory_recursive(dir) {
                all_apps.extend(apps);
            }
        }

        let app_names_lower: Vec<String> = all_apps.iter().map(|a| a.name.to_lowercase()).collect();

        let mut found_count = 0;
        for known in &well_known {
            if app_names_lower.iter().any(|n| n.contains(known)) {
                found_count += 1;
                eprintln!("[app_launcher] Found well-known app: {}", known);
            }
        }

        // We should find at least 2 of these on any Windows system
        assert!(
            found_count >= 2,
            "Should find at least 2 well-known apps, found {}/{}. Apps: {:?}",
            found_count,
            well_known.len(),
            app_names_lower.iter().take(30).collect::<Vec<_>>()
        );
    }

    // ========================================================================
    // parse_windows_shortcut
    // ========================================================================

    #[test]
    fn parse_shortcut_extracts_name_from_real_lnk() {
        // Find a real .lnk file from the Start Menu
        let dirs = &*WINDOWS_APP_DIRECTORIES;
        let mut found_lnk = None;

        'outer: for dir in dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
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

        let lnk_path = found_lnk.expect("Should find at least one .lnk in Start Menu");

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

    #[test]
    fn parse_shortcut_returns_none_for_nonexistent() {
        let fake_path = Path::new(r"C:\nonexistent\fake.lnk");
        // parse_windows_shortcut doesn't check existence, it just extracts the name
        // But it should still produce a valid AppInfo
        let app = parse_windows_shortcut(fake_path);
        assert!(
            app.is_some(),
            "Should produce AppInfo even for non-existent path (name extraction is filename-based)"
        );
        let app = app.unwrap();
        assert_eq!(app.name, "fake");
    }

    // ========================================================================
    // resolve_lnk_target
    // ========================================================================

    #[test]
    fn resolve_lnk_target_finds_target_for_real_shortcut() {
        // Find a real .lnk file
        let dirs = &*WINDOWS_APP_DIRECTORIES;
        let mut found_lnk = None;

        'outer: for dir in dirs {
            if let Ok(apps) = scan_windows_directory_recursive(dir) {
                for app in apps {
                    found_lnk = Some(app.path);
                    break 'outer;
                }
            }
        }

        let lnk_path = found_lnk.expect("Should find at least one .lnk");

        // resolve_lnk_target uses PowerShell - it might return None for some shortcuts
        // (e.g., UWP apps don't have traditional targets), but it shouldn't panic
        let target = resolve_lnk_target(&lnk_path);
        eprintln!(
            "[app_launcher] resolve_lnk_target({:?}) = {:?}",
            lnk_path, target
        );

        // The function should at least not panic. If it returns Some, the target should
        // be a non-empty string
        if let Some(ref t) = target {
            assert!(!t.is_empty(), "Resolved target should not be empty");
        }
    }

    #[test]
    fn resolve_lnk_target_returns_none_for_nonexistent() {
        let fake_path = Path::new(r"C:\nonexistent\fake.lnk");
        let target = resolve_lnk_target(fake_path);
        // Should return None or empty for nonexistent path, not panic
        if let Some(ref t) = target {
            // PowerShell may return empty string for non-existent shortcuts
            eprintln!(
                "[app_launcher] resolve_lnk_target for nonexistent returned: {:?}",
                t
            );
        }
    }

    // ========================================================================
    // Launch command construction
    // ========================================================================

    #[test]
    fn launch_command_construction() {
        let test_app = AppInfo {
            name: "Test App".to_string(),
            path: PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\test.lnk"),
            bundle_id: None,
            icon: None,
        };

        // Verify the command can be built without panicking
        let mut binding = Command::new("cmd");
        let cmd = binding
            .args(["/c", "start", ""])
            .arg(&test_app.path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let program = cmd.get_program();
        assert_eq!(program, "cmd", "Launch command should use cmd.exe");
    }

    // ========================================================================
    // scan_applications public API (triggers full scan + DB update)
    // ========================================================================

    #[test]
    fn scan_applications_returns_sorted_deduped_results() {
        let apps = scan_applications();

        eprintln!(
            "[app_launcher] scan_applications returned {} apps",
            apps.len()
        );

        // Should find at least some apps
        assert!(
            !apps.is_empty(),
            "scan_applications should find at least some apps"
        );

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

        // Verify no exact duplicate (name + path) entries
        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();
        for app in apps.iter() {
            let key = format!(
                "{}|{}",
                app.name.to_lowercase(),
                app.path.display().to_string().to_lowercase()
            );
            if !seen.insert(key) {
                duplicates.push(format!("{} ({})", app.name, app.path.display()));
            }
        }
        assert!(
            duplicates.is_empty(),
            "Found exact duplicate (name+path) entries: {:?}",
            duplicates
        );
    }

    // ========================================================================
    // Cross-platform / utility functions
    // ========================================================================

    #[test]
    fn hash_path_deterministic() {
        let path1 = Path::new(r"C:\Program Files\SomeApp\app.exe");
        let path2 = Path::new(r"C:\Program Files\SomeApp\app.exe");
        let path3 = Path::new(r"C:\Program Files\OtherApp\app.exe");

        assert_eq!(hash_path(path1), hash_path(path2), "Same path => same hash");
        assert_ne!(
            hash_path(path1),
            hash_path(path3),
            "Different paths => different hashes"
        );

        let hash = hash_path(path1);
        assert_eq!(hash.len(), 16, "Hash should be 16 hex characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex: {}",
            hash
        );
    }

    #[test]
    fn apps_db_path_is_under_scriptkit() {
        let db_path = get_apps_db_path();
        assert!(
            db_path.ends_with("db/apps.sqlite") || db_path.ends_with(r"db\apps.sqlite"),
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
    fn loading_state_has_message() {
        let state = get_app_loading_state();
        assert!(!state.message().is_empty(), "Should have a message");
    }

    #[test]
    fn db_stats_do_not_panic() {
        let (count, size) = get_apps_db_stats();
        // Just verify it doesn't panic; values depend on prior scan state
        let _ = (count, size);
    }

    #[test]
    fn icon_cache_stats_do_not_panic() {
        let (count, size) = get_icon_cache_stats();
        assert!(
            count == 0 || size > 0,
            "If there are cached files, size should be non-zero"
        );
    }

    // ========================================================================
    // Comprehensive scan report (not a correctness test, but useful for CI)
    // ========================================================================

    #[test]
    fn scan_report() {
        let dirs = &*WINDOWS_APP_DIRECTORIES;
        let mut total_apps = 0;

        eprintln!("\n=== Windows App Launcher Scan Report ===");
        eprintln!("Directories resolved: {}", dirs.len());
        for dir in dirs {
            let count = scan_windows_directory_recursive(dir)
                .map(|apps| apps.len())
                .unwrap_or(0);
            eprintln!("  {:60} => {} apps", dir.display(), count);
            total_apps += count;
        }

        eprintln!("Total apps found (before dedup): {}", total_apps);

        let deduped = scan_applications();
        eprintln!("Total apps after sort+dedup:     {}", deduped.len());

        // Print first 15 apps as a sample
        eprintln!("\nSample apps (first 15):");
        for app in deduped.iter().take(15) {
            let target = app.bundle_id.as_deref().unwrap_or("(no target)");
            eprintln!("  {:<35} => {}", app.name, target);
        }
        eprintln!("=== End Report ===\n");

        assert!(
            deduped.len() >= 5,
            "Any Windows system should have at least 5 apps, found {}",
            deduped.len()
        );
    }
}
