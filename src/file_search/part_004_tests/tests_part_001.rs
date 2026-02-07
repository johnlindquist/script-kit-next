    #[test]
    fn test_list_directory_limit() {
        // limit parameter is deprecated - list_directory no longer truncates
        // Callers should apply their own limit after filtering/scoring
        // We just verify that it doesn't panic and returns reasonable results
        let results = list_directory("/", 3);
        // Should return all entries (up to internal cap) not just 3
        // The "/" directory typically has multiple entries
        assert!(!results.is_empty(), "Root directory should have entries");
        // Verify internal cap works (5000)
        assert!(results.len() <= 5000, "Should respect internal cap of 5000");
    }
    #[test]
    fn test_list_directory_hides_dotfiles() {
        // Hidden files (starting with .) should be excluded
        let results = list_directory("~", 100);

        for result in &results {
            assert!(
                !result.name.starts_with('.'),
                "Should not include hidden files: {}",
                result.name
            );
        }
    }
    #[test]
    fn test_is_directory_path_reexport() {
        // Verify the re-export works
        assert!(is_directory_path("~/dev"));
        assert!(is_directory_path("/usr/local"));
        assert!(is_directory_path("./src"));
        assert!(!is_directory_path("hello world"));
    }
    // ========================================================================
    // Nucleo Filtering Tests
    // ========================================================================

    #[test]
    fn test_filter_results_nucleo_empty_pattern() {
        let results = vec![
            FileResult {
                path: "/test/apple.txt".to_string(),
                name: "apple.txt".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::Document,
            },
            FileResult {
                path: "/test/banana.txt".to_string(),
                name: "banana.txt".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::Document,
            },
        ];

        // Empty pattern with Nucleo matches everything (score 0)
        // This is expected behavior - caller should check for empty pattern before calling
        let filtered = filter_results_nucleo_simple(&results, "");
        assert_eq!(filtered.len(), 2);
    }
    #[test]
    fn test_filter_results_nucleo_empty_pattern_uses_name_tiebreaker() {
        let results = vec![
            FileResult {
                path: "/test/zeta.txt".to_string(),
                name: "zeta.txt".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::Document,
            },
            FileResult {
                path: "/test/alpha.txt".to_string(),
                name: "alpha.txt".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::Document,
            },
        ];

        let filtered = filter_results_nucleo_simple(&results, "");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].1.name, "alpha.txt");
        assert_eq!(filtered[1].1.name, "zeta.txt");
    }
    #[test]
    fn test_filter_results_nucleo_exact_match() {
        let results = vec![
            FileResult {
                path: "/test/mcp-final.txt".to_string(),
                name: "mcp-final".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/definitions.txt".to_string(),
                name: "definitions".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "final" should match "mcp-final" better than "definitions"
        let filtered = filter_results_nucleo_simple(&results, "final");
        assert!(!filtered.is_empty());
        assert_eq!(filtered[0].1.name, "mcp-final");
    }
    #[test]
    fn test_filter_results_nucleo_fuzzy_ordering() {
        let results = vec![
            FileResult {
                path: "/test/define.txt".to_string(),
                name: "define".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/mcp-final.txt".to_string(),
                name: "mcp-final".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/final-test.txt".to_string(),
                name: "final-test".to_string(),
                size: 300,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "fin" should fuzzy match both "mcp-final" and "final-test"
        // Both should rank higher than "define" (which has f, i, n but not consecutive)
        let filtered = filter_results_nucleo_simple(&results, "fin");

        // Should have matches
        assert!(!filtered.is_empty());

        // "final-test" or "mcp-final" should be first (both have "fin" as prefix of "final")
        let first_name = &filtered[0].1.name;
        assert!(
            first_name.contains("final"),
            "Expected 'final' in first result, got: {}",
            first_name
        );
    }
    #[test]
    fn test_filter_results_nucleo_no_matches() {
        let results = vec![
            FileResult {
                path: "/test/apple.txt".to_string(),
                name: "apple".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/banana.txt".to_string(),
                name: "banana".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "xyz" should not match anything
        let filtered = filter_results_nucleo_simple(&results, "xyz");
        assert!(filtered.is_empty());
    }
    #[test]
    fn test_filter_results_nucleo_case_insensitive() {
        let results = vec![FileResult {
            path: "/test/MyDocument.txt".to_string(),
            name: "MyDocument".to_string(),
            size: 100,
            modified: 0,
            file_type: FileType::Document,
        }];

        // Should match regardless of case
        let filtered_lower = filter_results_nucleo_simple(&results, "mydoc");
        let filtered_upper = filter_results_nucleo_simple(&results, "MYDOC");
        let filtered_mixed = filter_results_nucleo_simple(&results, "MyDoc");

        assert!(!filtered_lower.is_empty());
        assert!(!filtered_upper.is_empty());
        assert!(!filtered_mixed.is_empty());
    }
    // ========================================================================
    // FileInfo Tests
    // ========================================================================

    #[test]
    fn test_file_info_from_result() {
        let result = FileResult {
            path: "/test/document.pdf".to_string(),
            name: "document.pdf".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
        };

        let info = FileInfo::from_result(&result);
        assert_eq!(info.path, "/test/document.pdf");
        assert_eq!(info.name, "document.pdf");
        assert_eq!(info.file_type, FileType::Document);
        assert!(!info.is_dir);
    }
    #[test]
    fn test_file_info_from_result_directory() {
        let result = FileResult {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            size: 0,
            modified: 1234567890,
            file_type: FileType::Directory,
        };

        let info = FileInfo::from_result(&result);
        assert_eq!(info.path, "/test/Documents");
        assert_eq!(info.name, "Documents");
        assert_eq!(info.file_type, FileType::Directory);
        assert!(info.is_dir);
    }
    #[test]
    fn test_file_info_from_path() {
        // Test with a path that likely exists
        let info = FileInfo::from_path("/tmp");
        assert_eq!(info.path, "/tmp");
        assert_eq!(info.name, "tmp");
        // /tmp should be a directory on Unix systems
        #[cfg(unix)]
        assert!(info.is_dir);
    }
    // ========================================================================
    // Path Utility Tests (ensure_trailing_slash, parent_dir_display)
    // ========================================================================

    #[test]
    fn test_ensure_trailing_slash_already_has_slash() {
        assert_eq!(ensure_trailing_slash("/foo/bar/"), "/foo/bar/");
        assert_eq!(ensure_trailing_slash("~/dev/"), "~/dev/");
        assert_eq!(ensure_trailing_slash("/"), "/");
        assert_eq!(ensure_trailing_slash("~/"), "~/");
    }
    #[test]
    fn test_ensure_trailing_slash_needs_slash() {
        assert_eq!(ensure_trailing_slash("/foo/bar"), "/foo/bar/");
        assert_eq!(ensure_trailing_slash("~/dev"), "~/dev/");
        assert_eq!(ensure_trailing_slash(".."), "../");
        assert_eq!(ensure_trailing_slash("."), "./");
    }
    #[test]
    fn test_ensure_trailing_slash_edge_cases() {
        // Empty string
        assert_eq!(ensure_trailing_slash(""), "/");
        // Single tilde
        assert_eq!(ensure_trailing_slash("~"), "~/");
    }
    #[test]
    fn test_parent_dir_display_root() {
        // "/" has no parent
        assert_eq!(parent_dir_display("/"), None);
    }
    #[test]
    fn test_parent_dir_display_home_root() {
        // "~/" has no parent (home directory is treated as root)
        assert_eq!(parent_dir_display("~/"), None);
    }
    #[test]
    fn test_parent_dir_display_relative_parent() {
        // "../" -> "../../"
        assert_eq!(parent_dir_display("../"), Some("../../".to_string()));
    }
    #[test]
    fn test_parent_dir_display_relative_current() {
        // "./" -> "../"
        assert_eq!(parent_dir_display("./"), Some("../".to_string()));
    }
    #[test]
    fn test_parent_dir_display_tilde_subdir() {
        // "~/foo/" -> "~/"
        assert_eq!(parent_dir_display("~/foo/"), Some("~/".to_string()));
        // "~/foo/bar/" -> "~/foo/"
        assert_eq!(parent_dir_display("~/foo/bar/"), Some("~/foo/".to_string()));
    }
    #[test]
    fn test_parent_dir_display_absolute_subdir() {
        // "/foo/bar/" -> "/foo/"
        assert_eq!(parent_dir_display("/foo/bar/"), Some("/foo/".to_string()));
        // "/foo/" -> "/"
        assert_eq!(parent_dir_display("/foo/"), Some("/".to_string()));
    }
    #[test]
    fn test_parent_dir_display_multiple_levels() {
        // Deep paths
        assert_eq!(parent_dir_display("/a/b/c/d/"), Some("/a/b/c/".to_string()));
        assert_eq!(
            parent_dir_display("~/projects/rust/kit/"),
            Some("~/projects/rust/".to_string())
        );
    }
    #[test]
    fn test_parent_dir_display_no_trailing_slash() {
        // Paths without trailing slash should still work (normalize first)
        // The function expects trailing slash, but should handle edge cases gracefully
        assert_eq!(parent_dir_display("/foo/bar"), Some("/foo/".to_string()));
        assert_eq!(parent_dir_display("~/foo"), Some("~/".to_string()));
    }
    #[test]
    fn test_terminal_working_directory_uses_directory_path_when_is_dir() {
        let resolved = terminal_working_directory("/tmp/projects", true);
        assert_eq!(resolved, "/tmp/projects");
    }
    #[test]
    fn test_terminal_working_directory_uses_parent_for_file_paths() {
        let resolved = terminal_working_directory("/tmp/projects/readme.md", false);
        assert_eq!(resolved, "/tmp/projects");
    }
    #[test]
    fn test_terminal_working_directory_falls_back_to_original_path_without_parent() {
        let resolved = terminal_working_directory("readme.md", false);
        assert_eq!(resolved, "readme.md");
    }
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_open_in_terminal_returns_explicit_unsupported_error_on_non_macos() {
        let error = open_in_terminal("/tmp/projects/readme.md", false).unwrap_err();
        assert!(
            error.contains("only supported on macOS"),
            "error should explain platform limitation, got: {}",
            error
        );
    }
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_move_to_trash_returns_explicit_unsupported_error_on_non_macos() {
        let error = move_to_trash("/tmp/projects/readme.md").unwrap_err();
        assert!(
            error.contains("only supported on macOS"),
            "error should explain platform limitation, got: {}",
            error
        );
    }
