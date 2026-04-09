//! Storybook - Component Preview Tool for script-kit-gpui
//!
//! A standalone binary for previewing and testing Script Kit components.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin storybook
//! cargo run --bin storybook -- --story "main-menu"
//! cargo run --bin storybook -- --story "main-menu" --screenshot
//! cargo run --bin storybook -- --catalog-json
//! ```
//!
//! # Exit Codes
//!
//! - 0: success
//! - 1: invalid `--story` or `--variant` ID (interactive mode)
//! - 2: structured error (JSON on stderr for `--catalog-json`, `--adopt`, `--screenshot`)

use gpui::*;
use script_kit_gpui::storybook::{
    all_stories, save_selected_story_variant, StoryBrowser, StorybookJsonError,
    StorybookJsonErrorBody,
};
use script_kit_gpui::theme;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Uniform JSON success envelope matching the error envelope schema.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StorybookJsonSuccess<T: serde::Serialize> {
    schema_version: u8,
    ok: bool,
    data: T,
}

/// Screenshot-specific success payload.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotSuccess {
    path: String,
}

fn print_available_stories() {
    eprintln!("Available stories:");
    for entry in all_stories() {
        let compare_ready = if entry.story.variants().len() > 1 {
            " (compare-ready)"
        } else {
            ""
        };
        eprintln!(
            "  {:<28} - {}{}",
            entry.story.id(),
            entry.story.name(),
            compare_ready
        );
    }
}

fn print_json_error_and_exit(
    kind: &'static str,
    message: String,
    hint: &'static str,
    code: i32,
) -> ! {
    let payload = StorybookJsonError {
        schema_version: 1,
        ok: false,
        error: StorybookJsonErrorBody {
            kind,
            message,
            hint,
        },
    };

    eprintln!(
        "{}",
        serde_json::to_string_pretty(&payload).expect("serialize storybook json error payload"),
    );
    std::process::exit(code);
}

fn print_adopt_result_and_exit(story_id: Option<&str>, variant_id: Option<&str>) -> ! {
    let Some(story_id) = story_id else {
        print_json_error_and_exit(
            "missing_story",
            "Missing required --story <ID> for --adopt.".to_string(),
            "Pass --adopt --story <ID> --variant <ID>.",
            2,
        );
    };

    let Some(variant_id) = variant_id else {
        print_json_error_and_exit(
            "missing_variant",
            "Missing required --variant <ID> for --adopt.".to_string(),
            "Pass --adopt --story <ID> --variant <ID>.",
            2,
        );
    };

    let Some(entry) = all_stories().find(|entry| entry.story.id() == story_id) else {
        let available: Vec<String> = all_stories()
            .map(|entry| entry.story.id().to_string())
            .collect();

        print_json_error_and_exit(
            "unknown_story",
            format!(
                "Unknown story '{}'. Available stories: {}",
                story_id,
                available.join(", ")
            ),
            "Run `cargo run --bin storybook -- --catalog-json` to inspect available stories.",
            2,
        );
    };

    let available_variants: Vec<String> = entry
        .story
        .variants()
        .into_iter()
        .map(|variant| variant.stable_id())
        .collect();

    if !available_variants.iter().any(|id| id == variant_id) {
        print_json_error_and_exit(
            "unknown_variant",
            format!(
                "Unknown variant '{}' for story '{}'. Available variants: {}",
                variant_id,
                story_id,
                available_variants.join(", ")
            ),
            "Run `cargo run --bin storybook -- --catalog-json` and inspect the story's variants.",
            2,
        );
    }

    tracing::info!(
        event = "storybook_adopt_requested",
        story_id = story_id,
        variant_id = variant_id,
        "Requested story adoption"
    );

    match save_selected_story_variant(story_id, variant_id) {
        Ok(result) => {
            let payload = StorybookJsonSuccess {
                schema_version: 1,
                ok: true,
                data: result,
            };

            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("serialize storybook adopt success payload"),
            );
            std::process::exit(0);
        }
        Err(error) => {
            print_json_error_and_exit(
                "adopt_failed",
                format!("{error:#}"),
                "Verify the selection store path is writable and the existing JSON is valid.",
                2,
            );
        }
    }
}

fn print_screenshot_result_and_exit(result: Result<String, Box<dyn std::error::Error>>) -> ! {
    match result {
        Ok(path) => {
            let payload = StorybookJsonSuccess {
                schema_version: 1,
                ok: true,
                data: ScreenshotSuccess { path },
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("serialize screenshot success payload"),
            );
            std::process::exit(0);
        }
        Err(error) => {
            print_json_error_and_exit(
                "screenshot_failed",
                error.to_string(),
                "Verify a Storybook window is visible before requesting --screenshot.",
                2,
            );
        }
    }
}

fn main() {
    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    let mut initial_story: Option<String> = None;
    let mut initial_variant: Option<String> = None;
    let mut auto_screenshot = false;
    let mut start_compare = false;
    let mut adopt_selection = false;

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
            "--adopt" => {
                adopt_selection = true;
            }
            "--screenshot" | "-c" => {
                auto_screenshot = true;
            }
            "--catalog-json" => {
                print_story_catalog_json_and_exit();
            }
            "--help" | "-h" => {
                tracing::info!(
                    event = "storybook_help_rendered",
                    story_count = all_stories().count(),
                    "Rendered storybook help"
                );
                eprintln!("Script Kit Storybook - Component Preview Tool");
                eprintln!();
                eprintln!("Usage: storybook [OPTIONS]");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  -s, --story <ID>     Open a specific story by ID");
                eprintln!("  -v, --variant <ID>   Pre-select a variant id");
                eprintln!("  --compare            Open in side-by-side compare mode");
                eprintln!(
                    "  --adopt              Persist the requested story/variant and exit as JSON"
                );
                eprintln!("  -c, --screenshot     Capture screenshot and exit");
                eprintln!("  --catalog-json       Print compare-ready story catalog as JSON");
                eprintln!("  -h, --help           Show this help message");
                eprintln!();
                print_available_stories();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    // Non-interactive adopt: validate, persist, print JSON, exit — no GPUI window.
    if adopt_selection {
        print_adopt_result_and_exit(initial_story.as_deref(), initial_variant.as_deref());
    }

    let should_screenshot = Arc::new(AtomicBool::new(auto_screenshot));

    let initial_story_for_window = initial_story.clone();
    let initial_variant_for_window = initial_variant.clone();
    let start_compare_for_window = start_compare;

    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        theme::init_theme_cache();
        theme::sync_gpui_component_theme(cx);

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

                let result = capture_storybook_screenshot();
                print_screenshot_result_and_exit(result);
            });
        }
    });
}

/// Capture screenshot of storybook window using xcap, returning the saved path.
fn capture_storybook_screenshot() -> Result<String, Box<dyn std::error::Error>> {
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

            let saved_path = filepath.display().to_string();
            tracing::info!(
                event = "storybook_screenshot_saved",
                path = %saved_path,
                "Captured storybook screenshot"
            );

            return Ok(saved_path);
        }
    }

    Err("Storybook window not found".into())
}

fn build_story_catalog_json() -> Result<String, Box<dyn std::error::Error>> {
    tracing::info!(
        event = "storybook_catalog_json_requested",
        "Building story catalog json"
    );
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
            print_json_error_and_exit(
                "catalog_load_failed",
                format!("{error:#}"),
                "Run `cargo check` and verify story registration compiles before requesting --catalog-json.",
                2,
            );
        }
    }
}
