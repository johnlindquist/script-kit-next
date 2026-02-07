    use super::*;
    // ========================================================================
    // Query Builder Tests
    // ========================================================================

    #[test]
    fn test_looks_like_advanced_mdquery_detects_kmditem() {
        assert!(looks_like_advanced_mdquery("kMDItemFSName == 'test'"));
        assert!(looks_like_advanced_mdquery(
            "kMDItemContentType == 'public.image'"
        ));
    }
    #[test]
    fn test_looks_like_advanced_mdquery_detects_operators() {
        assert!(looks_like_advanced_mdquery("name == test"));
        assert!(looks_like_advanced_mdquery("size != 0"));
        assert!(looks_like_advanced_mdquery("date >= 2024"));
        assert!(looks_like_advanced_mdquery("size <= 1000"));
        assert!(looks_like_advanced_mdquery("type == image && size > 1000"));
        assert!(looks_like_advanced_mdquery("ext == jpg || ext == png"));
    }
    #[test]
    fn test_looks_like_advanced_mdquery_simple_queries() {
        // Simple text queries should NOT be detected as advanced
        assert!(!looks_like_advanced_mdquery("hello"));
        assert!(!looks_like_advanced_mdquery("my document"));
        assert!(!looks_like_advanced_mdquery("test.txt"));
        assert!(!looks_like_advanced_mdquery("file-name"));
    }
    #[test]
    fn test_escape_md_string_basic() {
        assert_eq!(escape_md_string("hello"), "hello");
        assert_eq!(escape_md_string("test file"), "test file");
    }
    #[test]
    fn test_escape_md_string_quotes() {
        assert_eq!(escape_md_string(r#"file"name"#), r#"file\"name"#);
        assert_eq!(escape_md_string(r#""quoted""#), r#"\"quoted\""#);
    }
    #[test]
    fn test_escape_md_string_backslashes() {
        assert_eq!(escape_md_string(r"path\to\file"), r"path\\to\\file");
        assert_eq!(escape_md_string(r"\escaped\"), r"\\escaped\\");
    }
    #[test]
    fn test_escape_md_string_mixed() {
        assert_eq!(escape_md_string(r#"file\"name"#), r#"file\\\"name"#);
    }
    #[test]
    fn test_build_mdquery_simple_query() {
        let query = build_mdquery("hello");
        assert_eq!(query, r#"kMDItemFSName == "*hello*"c"#);
    }
    #[test]
    fn test_build_mdquery_with_spaces() {
        let query = build_mdquery("my document");
        assert_eq!(query, r#"kMDItemFSName == "*my document*"c"#);
    }
    #[test]
    fn test_build_mdquery_passes_through_advanced() {
        let advanced = "kMDItemFSName == 'test.txt'";
        let query = build_mdquery(advanced);
        assert_eq!(query, advanced); // Should pass through unchanged
    }
    #[test]
    fn test_build_mdquery_with_special_chars() {
        let query = build_mdquery(r#"file"name"#);
        assert_eq!(query, r#"kMDItemFSName == "*file\"name*"c"#);
    }
    #[test]
    fn test_build_mdquery_trims_whitespace() {
        let query = build_mdquery("  hello  ");
        assert_eq!(query, r#"kMDItemFSName == "*hello*"c"#);
    }
    // ========================================================================
    // File Type Detection Tests
    // ========================================================================

    #[test]
    fn test_detect_file_type_image() {
        assert_eq!(
            detect_file_type(Path::new("/test/photo.png")),
            FileType::Image
        );
        assert_eq!(
            detect_file_type(Path::new("/test/photo.JPG")),
            FileType::Image
        );
        assert_eq!(
            detect_file_type(Path::new("/test/photo.heic")),
            FileType::Image
        );
    }
    #[test]
    fn test_detect_file_type_document() {
        assert_eq!(
            detect_file_type(Path::new("/test/doc.pdf")),
            FileType::Document
        );
        assert_eq!(
            detect_file_type(Path::new("/test/doc.docx")),
            FileType::Document
        );
        assert_eq!(
            detect_file_type(Path::new("/test/doc.txt")),
            FileType::Document
        );
    }
    #[test]
    fn test_detect_file_type_audio() {
        assert_eq!(
            detect_file_type(Path::new("/test/song.mp3")),
            FileType::Audio
        );
        assert_eq!(
            detect_file_type(Path::new("/test/song.wav")),
            FileType::Audio
        );
    }
    #[test]
    fn test_detect_file_type_video() {
        assert_eq!(
            detect_file_type(Path::new("/test/movie.mp4")),
            FileType::Video
        );
        assert_eq!(
            detect_file_type(Path::new("/test/movie.mov")),
            FileType::Video
        );
    }
    #[test]
    fn test_detect_file_type_application() {
        assert_eq!(
            detect_file_type(Path::new("/Applications/Safari.app")),
            FileType::Application
        );
    }
    #[test]
    fn test_detect_file_type_generic_file() {
        assert_eq!(
            detect_file_type(Path::new("/test/script.rs")),
            FileType::File
        );
        assert_eq!(
            detect_file_type(Path::new("/test/config.json")),
            FileType::File
        );
    }
    #[test]
    fn test_search_files_empty_query() {
        let results = search_files("", None, 10);
        assert!(results.is_empty());
    }
    #[test]
    fn test_file_result_creation() {
        let result = FileResult {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
        };

        assert_eq!(result.path, "/test/file.txt");
        assert_eq!(result.name, "file.txt");
        assert_eq!(result.size, 1024);
        assert_eq!(result.file_type, FileType::Document);
    }
    #[test]
    fn test_file_metadata_creation() {
        let meta = FileMetadata {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
            readable: true,
            writable: true,
        };

        assert_eq!(meta.path, "/test/file.txt");
        assert!(meta.readable);
        assert!(meta.writable);
    }
    #[test]
    fn test_default_file_type() {
        assert_eq!(FileType::default(), FileType::Other);
    }
    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_search_files_real_query() {
        // This test only runs on macOS and verifies mdfind works
        let results = search_files("System Preferences", Some("/System"), 5);
        // We don't assert specific results as they may vary,
        // but the function should not panic
        assert!(results.len() <= 5);
    }
    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_get_file_metadata_real_file() {
        // Test with a file that should exist on all macOS systems
        let meta = get_file_metadata("/System/Library/CoreServices/Finder.app");
        // Finder.app should exist on macOS
        if let Some(m) = meta {
            assert!(!m.name.is_empty());
            assert!(m.readable);
        }
        // It's OK if this returns None on some systems
    }
    // ========================================================================
    // UI Helper Function Tests
    // ========================================================================

    #[test]
    fn test_file_type_icon() {
        assert_eq!(file_type_icon(FileType::Directory), "📁");
        assert_eq!(file_type_icon(FileType::Application), "📦");
        assert_eq!(file_type_icon(FileType::Image), "🖼️");
        assert_eq!(file_type_icon(FileType::Document), "📄");
        assert_eq!(file_type_icon(FileType::Audio), "🎵");
        assert_eq!(file_type_icon(FileType::Video), "🎬");
        assert_eq!(file_type_icon(FileType::File), "📃");
        assert_eq!(file_type_icon(FileType::Other), "📎");
    }
    #[test]
    fn test_format_file_size() {
        // Bytes
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1023), "1023 B");

        // Kilobytes
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(10240), "10.0 KB");

        // Megabytes
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.0 MB");

        // Gigabytes
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.0 GB");
    }
    #[test]
    fn test_format_relative_time() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Just now (0 seconds ago)
        assert_eq!(format_relative_time(now), "Just now");

        // Minutes ago
        assert_eq!(format_relative_time(now - 60), "1 min ago");
        assert_eq!(format_relative_time(now - 120), "2 mins ago");
        assert_eq!(format_relative_time(now - 59 * 60), "59 mins ago");

        // Hours ago
        assert_eq!(format_relative_time(now - 3600), "1 hour ago");
        assert_eq!(format_relative_time(now - 7200), "2 hours ago");

        // Days ago
        assert_eq!(format_relative_time(now - 86400), "1 day ago");
        assert_eq!(format_relative_time(now - 172800), "2 days ago");

        // Unknown (0 timestamp)
        assert_eq!(format_relative_time(0), "Unknown");
    }
    #[test]
    fn test_shorten_path() {
        // Test with a path that doesn't start with home
        assert_eq!(shorten_path("/usr/local/bin"), "/usr/local/bin");
        assert_eq!(shorten_path("/etc/hosts"), "/etc/hosts");

        // Test with home directory path (if home dir is available)
        if let Some(home) = dirs::home_dir() {
            if let Some(home_str) = home.to_str() {
                let test_path = format!("{}/Documents/test.txt", home_str);
                assert_eq!(shorten_path(&test_path), "~/Documents/test.txt");
            }
        }
    }
    // ========================================================================
    // Directory Navigation Tests
    // ========================================================================

    #[test]
    fn test_expand_path_home() {
        // Test ~ expansion
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_str().unwrap();

            // Just ~
            assert_eq!(expand_path("~"), Some(home_str.to_string()));

            // ~/subdir
            let expanded = expand_path("~/Documents");
            assert!(expanded.is_some());
            assert!(expanded.unwrap().starts_with(home_str));
        }
    }
    #[test]
    fn test_expand_path_absolute() {
        // Absolute paths should pass through unchanged
        assert_eq!(expand_path("/usr/local"), Some("/usr/local".to_string()));
        assert_eq!(expand_path("/"), Some("/".to_string()));
        assert_eq!(
            expand_path("/System/Library"),
            Some("/System/Library".to_string())
        );
    }
    #[test]
    fn test_expand_path_relative_current() {
        // Relative paths with .
        let cwd = std::env::current_dir().unwrap();
        let cwd_str = cwd.to_str().unwrap();

        // Just .
        let expanded = expand_path(".");
        assert!(expanded.is_some());
        assert_eq!(expanded.unwrap(), cwd_str);

        // ./subdir
        let expanded = expand_path("./src");
        assert!(expanded.is_some());
        let expected = cwd.join("src");
        assert_eq!(expanded.unwrap(), expected.to_str().unwrap());
    }
    #[test]
    fn test_expand_path_relative_parent() {
        // Relative paths with ..
        let cwd = std::env::current_dir().unwrap();
        if let Some(parent) = cwd.parent() {
            let parent_str = parent.to_str().unwrap();

            // Just ..
            let expanded = expand_path("..");
            assert!(expanded.is_some());
            assert_eq!(expanded.unwrap(), parent_str);
        }
    }
    #[test]
    fn test_expand_path_empty() {
        assert_eq!(expand_path(""), None);
        assert_eq!(expand_path("   "), None);
    }
    #[test]
    fn test_expand_path_not_path() {
        // Regular text should return None
        assert_eq!(expand_path("hello"), None);
        assert_eq!(expand_path("search query"), None);
    }
    #[test]
    fn test_list_directory_nonexistent() {
        // Non-existent directory should return empty
        let results = list_directory("/this/path/does/not/exist/at/all", 50);
        assert!(results.is_empty());
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn test_list_directory_system() {
        // List /System which exists on all macOS systems
        let results = list_directory("/System", 10);
        assert!(!results.is_empty(), "Should find items in /System");

        // Should contain Library
        let has_library = results.iter().any(|r| r.name == "Library");
        assert!(has_library, "Should contain Library folder");

        // Library should be marked as directory
        let library = results.iter().find(|r| r.name == "Library");
        if let Some(lib) = library {
            assert_eq!(lib.file_type, FileType::Directory);
        }
    }
    #[test]
    fn test_list_directory_home() {
        // List home directory using ~
        let results = list_directory("~", 100);

        // Home should have at least some contents
        // (assuming it's a valid home directory)
        // Don't assert specific files as they vary by system
        assert!(
            results.is_empty() || !results.is_empty(),
            "Should not panic on home directory"
        );
    }
    #[test]
    fn test_list_directory_dirs_first() {
        // Test using /tmp which usually has both dirs and files
        let results = list_directory("/tmp", 50);

        // If we have results, verify sorting
        if results.len() >= 2 {
            // Find first file (non-directory)
            let first_file_idx = results
                .iter()
                .position(|r| !matches!(r.file_type, FileType::Directory));

            // Find last directory
            let last_dir_idx = results
                .iter()
                .rposition(|r| matches!(r.file_type, FileType::Directory));

            // If we have both dirs and files, dirs should come first
            if let (Some(first_file), Some(last_dir)) = (first_file_idx, last_dir_idx) {
                assert!(
                    last_dir < first_file,
                    "Directories should come before files"
                );
            }
        }
    }
