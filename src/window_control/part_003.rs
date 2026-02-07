/// Calculate the bounds for a tiling position within a display.
fn calculate_tile_bounds(display: &Bounds, position: TilePosition) -> Bounds {
    let half_width = display.width / 2;
    let half_height = display.height / 2;
    let third_width = display.width / 3;
    let third_height = display.height / 3;
    let two_thirds_width = (display.width * 2) / 3;
    let two_thirds_height = (display.height * 2) / 3;

    match position {
        // Half positions
        TilePosition::LeftHalf => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::RightHalf => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: display.height,
        },
        TilePosition::TopHalf => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: half_height,
        },
        TilePosition::BottomHalf => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: display.width,
            height: half_height,
        },

        // Quadrant positions
        TilePosition::TopLeft => Bounds {
            x: display.x,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::TopRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomLeft => Bounds {
            x: display.x,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },
        TilePosition::BottomRight => Bounds {
            x: display.x + half_width as i32,
            y: display.y + half_height as i32,
            width: half_width,
            height: half_height,
        },

        // Horizontal thirds positions
        TilePosition::LeftThird => Bounds {
            x: display.x,
            y: display.y,
            width: third_width,
            height: display.height,
        },
        TilePosition::CenterThird => Bounds {
            x: display.x + third_width as i32,
            y: display.y,
            width: third_width,
            height: display.height,
        },
        TilePosition::RightThird => Bounds {
            x: display.x + (two_thirds_width) as i32,
            y: display.y,
            width: third_width,
            height: display.height,
        },

        // Vertical thirds positions
        TilePosition::TopThird => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: third_height,
        },
        TilePosition::MiddleThird => Bounds {
            x: display.x,
            y: display.y + third_height as i32,
            width: display.width,
            height: third_height,
        },
        TilePosition::BottomThird => Bounds {
            x: display.x,
            y: display.y + two_thirds_height as i32,
            width: display.width,
            height: third_height,
        },

        // Horizontal two-thirds positions
        TilePosition::FirstTwoThirds => Bounds {
            x: display.x,
            y: display.y,
            width: two_thirds_width,
            height: display.height,
        },
        TilePosition::LastTwoThirds => Bounds {
            x: display.x + third_width as i32,
            y: display.y,
            width: two_thirds_width,
            height: display.height,
        },

        // Vertical two-thirds positions
        TilePosition::TopTwoThirds => Bounds {
            x: display.x,
            y: display.y,
            width: display.width,
            height: two_thirds_height,
        },
        TilePosition::BottomTwoThirds => Bounds {
            x: display.x,
            y: display.y + third_height as i32,
            width: display.width,
            height: two_thirds_height,
        },

        // Centered positions
        TilePosition::Center => {
            // 60% of screen, centered
            let width = (display.width * 60) / 100;
            let height = (display.height * 60) / 100;
            let x_offset = (display.width - width) / 2;
            let y_offset = (display.height - height) / 2;
            Bounds {
                x: display.x + x_offset as i32,
                y: display.y + y_offset as i32,
                width,
                height,
            }
        }
        TilePosition::AlmostMaximize => {
            // 90% of screen with margins
            let margin_x = (display.width * 5) / 100; // 5% margin on each side
            let margin_y = (display.height * 5) / 100;
            Bounds {
                x: display.x + margin_x as i32,
                y: display.y + margin_y as i32,
                width: display.width - (margin_x * 2),
                height: display.height - (margin_y * 2),
            }
        }

        TilePosition::Fullscreen => *display,
    }
}
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    fn cf_get_retain_count(cf: CFTypeRef) -> isize {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFGetRetainCount(cf: CFTypeRef) -> isize;
        }

        unsafe { CFGetRetainCount(cf) }
    }

    #[test]
    fn test_bounds_new() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 100);
        assert_eq!(bounds.height, 200);
    }

    #[test]
    fn test_calculate_tile_bounds_left_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::LeftHalf);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_right_half() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::RightHalf);

        assert_eq!(bounds.x, 960);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 1055);
    }

    #[test]
    fn test_calculate_tile_bounds_top_left() {
        let display = Bounds::new(0, 25, 1920, 1080);
        let bounds = calculate_tile_bounds(&display, TilePosition::TopLeft);

        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 25);
        assert_eq!(bounds.width, 960);
        assert_eq!(bounds.height, 540);
    }

    #[test]
    fn test_calculate_tile_bounds_fullscreen() {
        let display = Bounds::new(0, 25, 1920, 1055);
        let bounds = calculate_tile_bounds(&display, TilePosition::Fullscreen);

        assert_eq!(bounds, display);
    }

    #[test]
    fn test_tile_position_equality() {
        assert_eq!(TilePosition::LeftHalf, TilePosition::LeftHalf);
        assert_ne!(TilePosition::LeftHalf, TilePosition::RightHalf);
    }

    #[test]
    fn test_try_create_cf_string_rejects_interior_nul() {
        let error = try_create_cf_string("AX\0Title").expect_err("interior NUL should fail");
        assert!(
            error.to_string().contains("interior NUL"),
            "error should describe invalid CFString input: {error}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_window_cache_releases_previous_pointer_on_overwrite() {
        clear_window_cache();

        let window_id = 0xCAFE_1000;
        let first_window =
            try_create_cf_string("window-cache-overwrite-first").expect("valid CFString literal");
        let second_window =
            try_create_cf_string("window-cache-overwrite-second").expect("valid CFString literal");

        cache_window(window_id, cf_retain(first_window) as AXUIElementRef);
        let first_after_insert = cf_get_retain_count(first_window);

        cache_window(window_id, cf_retain(second_window) as AXUIElementRef);
        let first_after_overwrite = cf_get_retain_count(first_window);

        assert_eq!(
            first_after_overwrite + 1,
            first_after_insert,
            "cache overwrite should release old retained window pointer"
        );

        clear_window_cache();
        cf_release(first_window);
        cf_release(second_window);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_window_cache_get_returns_owned_reference_and_releases_on_drop() {
        clear_window_cache();

        let window_id = 0xCAFE_2000;
        let window =
            try_create_cf_string("window-cache-owned-get").expect("valid CFString literal");
        cache_window(window_id, cf_retain(window) as AXUIElementRef);

        let before_get = cf_get_retain_count(window);
        let owned = get_cached_window(window_id).expect("window should exist in cache");
        assert_eq!(owned.as_ptr(), window as AXUIElementRef);

        let during_get = cf_get_retain_count(window);
        assert_eq!(
            during_get,
            before_get + 1,
            "get_cached_window should retain before returning"
        );

        drop(owned);
        let after_drop = cf_get_retain_count(window);
        assert_eq!(
            after_drop, before_get,
            "dropping owned cached window should release retained reference"
        );

        clear_window_cache();
        cf_release(window);
    }

    #[test]
    fn test_permission_check_does_not_panic() {
        // This test verifies the permission check doesn't panic
        let _has_permission = has_accessibility_permission();
    }

    #[test]
    #[ignore] // Requires accessibility permission
    fn test_list_windows() {
        let windows = list_windows().expect("Should list windows");
        println!("Found {} windows:", windows.len());
        for window in &windows {
            println!(
                "  [{:08x}] {}: {} ({:?})",
                window.id, window.app, window.title, window.bounds
            );
        }
    }

    #[test]
    #[ignore] // Requires accessibility permission and a visible window
    fn test_tile_window_left_half() {
        let windows = list_windows().expect("Should list windows");
        if let Some(window) = windows.first() {
            tile_window(window.id, TilePosition::LeftHalf).expect("Should tile window");
            println!("Tiled '{}' to left half", window.title);
        } else {
            panic!("No windows found to test with");
        }
    }
}
