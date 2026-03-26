//! Storybook - Component Preview Tool for script-kit-gpui
//!
//! A standalone binary for previewing and testing Script Kit components.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin storybook
//! cargo run --bin storybook -- --story "button"
//! cargo run --bin storybook -- --story "header-variations" --screenshot
//! cargo run --bin storybook -- --story "footer-layout-variations" --compare --variant scriptkit-branded
//! ```
//!
//! # Exit Codes
//!
//! - 0: success
//! - 1: invalid `--story` or `--variant` ID
//! - 2: `--catalog-json` failure (structured JSON error on stderr)

use gpui::*;
use script_kit_gpui::storybook::{StorybookJsonError, StorybookJsonErrorBody, StoryBrowser};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    let mut initial_story: Option<String> = None;
    let mut initial_variant: Option<String> = None;
    let mut auto_screenshot = false;
    let mut start_compare = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--story" | "-s" => {
                if i + 1 < args.len() {
                    initial_story = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--variant" | "-v" => {
                if i + 1 < args.len() {
                    initial_variant = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--compare" => {
                start_compare = true;
            }
            "--screenshot" | "-c" => {
                auto_screenshot = true;
            }
            "--catalog-json" => {
                print_story_catalog_json_and_exit();
            }
            "--help" | "-h" => {
                eprintln!("Script Kit Storybook - Component Preview Tool");
                eprintln!();
                eprintln!("Usage: storybook [OPTIONS]");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  -s, --story <ID>     Open a specific story by ID");
                eprintln!("  -v, --variant <ID>   Pre-select a variant id");
                eprintln!("  --compare            Open in side-by-side compare mode");
                eprintln!("  -c, --screenshot     Capture screenshot and exit");
                eprintln!("  --catalog-json       Print compare-ready story catalog as JSON");
                eprintln!("  -h, --help           Show this help message");
                eprintln!();
                eprintln!("Available stories:");
                eprintln!("  button           - Button component variants");
                eprintln!("  toast            - Toast notification component");
                eprintln!("  form-fields      - Form input components");
                eprintln!("  list-item        - List item component");
                eprintln!("  scrollbar        - Scrollbar component");
                eprintln!("  design-tokens    - Design system tokens");
                eprintln!("  header-variations - Header component variants");
                eprintln!("  footer-layout-variations - Footer layout variants (compare-ready)");
                eprintln!("  header-design-variations - Header layout variants (compare-ready)");
                eprintln!("  actions-window           - Actions dialog variants (compare-ready)");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    let should_screenshot = Arc::new(AtomicBool::new(auto_screenshot));

    let initial_story_for_window = initial_story.clone();
    let initial_variant_for_window = initial_variant.clone();
    let start_compare_for_window = start_compare;

    gpui_platform::application().run(move |cx| {
        // Create window options
        let window_size = size(px(1200.), px(800.));
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                window_size,
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Script Kit Storybook".into()),
                appears_transparent: false,
                ..Default::default()
            }),
            window_min_size: Some(size(px(800.), px(600.))),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            ..Default::default()
        };

        let should_screenshot_clone = should_screenshot.clone();
        let initial_story = initial_story_for_window.clone();
        let initial_variant = initial_variant_for_window.clone();

        let _window_handle = cx
            .open_window(options, move |_window, cx| {
                let initial_story = initial_story.clone();
                let initial_variant = initial_variant.clone();

                cx.new(move |cx| {
                    let mut browser = StoryBrowser::new(cx);

                    if let Some(ref story_id) = initial_story {
                        if !browser.select_story(story_id) {
                            let known = browser.story_ids();
                            eprintln!(
                                "{{\"error\":\"unknown_story\",\"story\":{:?},\"available\":{:?}}}",
                                story_id, known
                            );
                            std::process::exit(1);
                        }
                    }

                    if start_compare_for_window {
                        browser.open_compare_mode();
                    }

                    if let Some(ref variant_id) = initial_variant {
                        if !browser.select_variant_id(variant_id) {
                            let known = browser.variant_ids();
                            eprintln!(
                                "{{\"error\":\"unknown_variant\",\"variant\":{:?},\"available\":{:?}}}",
                                variant_id, known
                            );
                            std::process::exit(1);
                        }
                    }

                    browser
                })
            })
            .expect("Failed to open storybook window");

        // If auto-screenshot mode, wait for render then capture and exit
        if should_screenshot_clone.load(Ordering::SeqCst) {
            // Use a thread to wait and then capture
            std::thread::spawn(move || {
                // Wait for window to fully render
                std::thread::sleep(std::time::Duration::from_millis(1500));

                // Capture screenshot using xcap
                if let Err(e) = capture_storybook_screenshot() {
                    eprintln!("[SCREENSHOT ERROR] {}", e);
                }

                // Exit the process
                std::process::exit(0);
            });
        }
    });
}

/// Capture screenshot of storybook window using xcap
fn capture_storybook_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use std::fs;
    use std::path::PathBuf;
    use xcap::Window;

    let windows = Window::all()?;

    for win in windows {
        let title = win.title().unwrap_or_default();

        if title.contains("Storybook") {
            let img = win.capture_image()?;
            let width = img.width();
            let height = img.height();
            let rgba_data = img.into_raw();

            // Encode as PNG
            let mut png_data = Vec::new();
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(&rgba_data, width, height, image::ExtendedColorType::Rgba8)?;

            // Save to file
            let screenshot_dir = PathBuf::from("test-screenshots");
            fs::create_dir_all(&screenshot_dir)?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);

            let filepath = screenshot_dir.join(format!("storybook-{}.png", timestamp));
            fs::write(&filepath, &png_data)?;

            eprintln!("[SCREENSHOT] Saved: {}", filepath.display());
            return Ok(());
        }
    }

    Err("Storybook window not found".into())
}

fn build_story_catalog_json() -> Result<String, Box<dyn std::error::Error>> {
    let snapshot = script_kit_gpui::storybook::load_story_catalog_snapshot()?;
    Ok(serde_json::to_string_pretty(&snapshot)?)
}

fn print_story_catalog_json_and_exit() -> ! {
    match build_story_catalog_json() {
        Ok(json) => {
            println!("{json}");
            std::process::exit(0);
        }
        Err(error) => {
            let payload = StorybookJsonError {
                schema_version: 1,
                ok: false,
                error: StorybookJsonErrorBody {
                    kind: "catalog_load_failed",
                    message: format!("{error:#}"),
                    hint: "Run `cargo check` and verify story registration compiles before requesting --catalog-json.",
                },
            };

            eprintln!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("serialize storybook catalog error payload"),
            );
            std::process::exit(2);
        }
    }
}
