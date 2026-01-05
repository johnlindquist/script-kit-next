//! Tests for window_state persistence module

#[cfg(test)]
mod tests {
    use crate::platform::DisplayBounds;
    use crate::window_state::*;
    use gpui::{point, px, size, Bounds, WindowBounds};
    use std::env;
    use tempfile::TempDir;

    fn with_temp_state_dir<F: FnOnce()>(f: F) {
        let temp_dir = TempDir::new().unwrap();
        let old_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        let kit_dir = temp_dir.path().join(".sk").join("kit");
        std::fs::create_dir_all(&kit_dir).unwrap();
        f();
        if let Some(home) = old_home {
            env::set_var("HOME", home);
        }
    }

    #[test]
    fn test_persisted_bounds_to_gpui_windowed() {
        let bounds = PersistedWindowBounds {
            mode: PersistedWindowMode::Windowed,
            x: 100.0,
            y: 200.0,
            width: 800.0,
            height: 600.0,
        };
        let gpui_bounds = bounds.to_gpui();
        match gpui_bounds {
            WindowBounds::Windowed(b) => {
                assert_eq!(f64::from(b.origin.x), 100.0);
                assert_eq!(f64::from(b.origin.y), 200.0);
                assert_eq!(f64::from(b.size.width), 800.0);
                assert_eq!(f64::from(b.size.height), 600.0);
            }
            _ => panic!("Expected Windowed bounds"),
        }
    }

    #[test]
    fn test_persisted_bounds_from_gpui() {
        let gpui_bounds = WindowBounds::Windowed(Bounds {
            origin: point(px(150.0), px(250.0)),
            size: size(px(750.0), px(500.0)),
        });
        let persisted = PersistedWindowBounds::from_gpui(gpui_bounds);
        assert_eq!(persisted.mode, PersistedWindowMode::Windowed);
        assert_eq!(persisted.x, 150.0);
        assert_eq!(persisted.y, 250.0);
    }

    #[test]
    fn test_persisted_bounds_roundtrip() {
        let original = PersistedWindowBounds {
            mode: PersistedWindowMode::Maximized,
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let gpui = original.to_gpui();
        let back = PersistedWindowBounds::from_gpui(gpui);
        assert_eq!(back.mode, original.mode);
        assert_eq!(back.x, original.x);
    }

    #[test]
    fn test_is_bounds_visible_fully_on_screen() {
        let displays = vec![DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 1920.0,
            height: 1080.0,
        }];
        let bounds = PersistedWindowBounds::new(100.0, 100.0, 800.0, 600.0);
        assert!(is_bounds_visible(&bounds, &displays));
    }

    #[test]
    fn test_is_bounds_visible_completely_off_screen() {
        let displays = vec![DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 1920.0,
            height: 1080.0,
        }];
        let bounds = PersistedWindowBounds::new(2000.0, 100.0, 800.0, 600.0);
        assert!(!is_bounds_visible(&bounds, &displays));
    }

    #[test]
    fn test_is_bounds_visible_multi_monitor() {
        let displays = vec![
            DisplayBounds {
                origin_x: 0.0,
                origin_y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
            DisplayBounds {
                origin_x: 1920.0,
                origin_y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
        ];
        let bounds = PersistedWindowBounds::new(2000.0, 100.0, 800.0, 600.0);
        assert!(is_bounds_visible(&bounds, &displays));
    }

    #[test]
    fn test_clamp_bounds_off_right() {
        let displays = vec![DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 1920.0,
            height: 1080.0,
        }];
        let bounds = PersistedWindowBounds::new(2500.0, 100.0, 800.0, 600.0);
        let clamped = clamp_bounds_to_displays(&bounds, &displays).unwrap();
        assert!(clamped.x + clamped.width <= 1920.0);
    }

    #[test]
    fn test_save_and_load_state_file() {
        with_temp_state_dir(|| {
            let state = WindowStateFile {
                version: 1,
                main: Some(PersistedWindowBounds::new(100.0, 200.0, 750.0, 475.0)),
                notes: None,
                ai: None,
            };
            assert!(save_state_file(&state));
            let loaded = load_state_file().expect("Should load saved state");
            assert_eq!(loaded.version, 1);
            assert!(loaded.main.is_some());
        });
    }

    #[test]
    fn test_reset_all_positions() {
        with_temp_state_dir(|| {
            save_window_bounds(
                WindowRole::Main,
                PersistedWindowBounds::new(100.0, 100.0, 750.0, 475.0),
            );
            assert!(has_custom_positions());
            reset_all_positions();
            assert!(!has_custom_positions());
        });
    }
}
