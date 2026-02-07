use super::*;

impl ScriptListApp {
    fn webcam_photo_directory() -> std::path::PathBuf {
        if let Some(home) = dirs::home_dir() {
            let desktop = home.join("Desktop");
            if desktop.exists() {
                return desktop;
            }
        }

        let temp = std::env::temp_dir();
        if temp.exists() {
            temp
        } else {
            std::path::PathBuf::from("/tmp")
        }
    }

    #[cfg(target_os = "macos")]
    fn encode_webcam_frame_to_png(
        pixel_buffer: &core_video::pixel_buffer::CVPixelBuffer,
    ) -> Result<Vec<u8>, String> {
        use image::ImageEncoder;

        let lock_flags = core_video::pixel_buffer::kCVPixelBufferLock_ReadOnly;
        let lock_status = pixel_buffer.lock_base_address(lock_flags);
        if lock_status != core_video::r#return::kCVReturnSuccess {
            return Err(format!(
                "Failed to lock webcam frame (status={})",
                lock_status
            ));
        }

        let result = (|| -> Result<Vec<u8>, String> {
            if !pixel_buffer.is_planar() || pixel_buffer.get_plane_count() < 2 {
                return Err("Webcam frame format is not NV12".to_string());
            }

            let width = pixel_buffer.get_width_of_plane(0);
            let height = pixel_buffer.get_height_of_plane(0);
            if width == 0 || height == 0 {
                return Err("Webcam frame is empty".to_string());
            }

            let y_stride = pixel_buffer.get_bytes_per_row_of_plane(0);
            let uv_stride = pixel_buffer.get_bytes_per_row_of_plane(1);
            let uv_height = pixel_buffer.get_height_of_plane(1);

            let y_plane_ptr = unsafe { pixel_buffer.get_base_address_of_plane(0) as *const u8 };
            let uv_plane_ptr = unsafe { pixel_buffer.get_base_address_of_plane(1) as *const u8 };
            if y_plane_ptr.is_null() || uv_plane_ptr.is_null() {
                return Err("Webcam frame memory is unavailable".to_string());
            }

            let y_plane = unsafe { std::slice::from_raw_parts(y_plane_ptr, y_stride * height) };
            let uv_plane =
                unsafe { std::slice::from_raw_parts(uv_plane_ptr, uv_stride * uv_height) };

            let mut rgb = vec![0u8; width * height * 3];

            for y in 0..height {
                let y_row = y * y_stride;
                let uv_row = (y / 2) * uv_stride;

                for x in 0..width {
                    let y_val = y_plane[y_row + x] as f32;
                    let uv_idx = uv_row + (x / 2) * 2;
                    if uv_idx + 1 >= uv_plane.len() {
                        continue;
                    }

                    let u = uv_plane[uv_idx] as f32 - 128.0;
                    let v = uv_plane[uv_idx + 1] as f32 - 128.0;

                    let r = (y_val + 1.402 * v).clamp(0.0, 255.0) as u8;
                    let g = (y_val - 0.344_136 * u - 0.714_136 * v).clamp(0.0, 255.0) as u8;
                    let b = (y_val + 1.772 * u).clamp(0.0, 255.0) as u8;

                    let idx = (y * width + x) * 3;
                    rgb[idx] = r;
                    rgb[idx + 1] = g;
                    rgb[idx + 2] = b;
                }
            }

            let mut png_data = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            encoder
                .write_image(
                    &rgb,
                    width as u32,
                    height as u32,
                    image::ColorType::Rgb8.into(),
                )
                .map_err(|e| format!("Failed to encode webcam frame: {}", e))?;

            Ok(png_data)
        })();

        let unlock_status = pixel_buffer.unlock_base_address(lock_flags);
        if unlock_status != core_video::r#return::kCVReturnSuccess {
            logging::log(
                "ERROR",
                &format!(
                    "Failed to unlock webcam frame (status={}) after capture",
                    unlock_status
                ),
            );
        }

        result
    }

    #[cfg(target_os = "macos")]
    fn capture_webcam_photo(&mut self, cx: &mut Context<Self>) -> bool {
        let pixel_buffer = match &self.current_view {
            AppView::WebcamView { entity } => entity.read(cx).pixel_buffer.clone(),
            _ => None,
        };

        let Some(pixel_buffer) = pixel_buffer else {
            cx.notify();
            self.show_hud("No camera frame available yet".to_string(), Some(2000), cx);
            return false;
        };

        let png_data = match Self::encode_webcam_frame_to_png(&pixel_buffer) {
            Ok(data) => data,
            Err(e) => {
                logging::log("ERROR", &format!("Failed to capture webcam photo: {}", e));
                cx.notify();
                self.show_hud(format!("Failed to capture photo: {}", e), Some(3000), cx);
                return false;
            }
        };

        let save_dir = Self::webcam_photo_directory();
        if let Err(e) = std::fs::create_dir_all(&save_dir) {
            logging::log(
                "ERROR",
                &format!("Failed to create webcam photo directory: {}", e),
            );
            cx.notify();
            self.show_hud(
                format!("Failed to create photo directory: {}", e),
                Some(3000),
                cx,
            );
            return false;
        }

        let filename = format!(
            "webcam-photo-{}.png",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        let save_path = save_dir.join(filename);

        match std::fs::write(&save_path, png_data) {
            Ok(()) => {
                logging::log(
                    "ACTIONS",
                    &format!("Webcam photo saved: {}", save_path.display()),
                );
                cx.notify();
                self.show_hud(
                    format!("Photo saved to {}", save_path.display()),
                    Some(3500),
                    cx,
                );
                self.reveal_in_finder(&save_path);
                true
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to save webcam photo: {}", e));
                cx.notify();
                self.show_hud(format!("Failed to save photo: {}", e), Some(3000), cx);
                false
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn capture_webcam_photo(&mut self, cx: &mut Context<Self>) -> bool {
        logging::log(
            "ACTIONS",
            "capture_webcam_photo requested on unsupported platform",
        );
        cx.notify();
        self.show_hud(
            "Webcam capture is only supported on macOS".to_string(),
            Some(2500),
            cx,
        );
        false
    }

    fn webcam_actions_for_dialog() -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};

        vec![
            Action::new(
                "capture",
                "Capture Photo",
                Some("Take a photo".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
            Action::new(
                "close",
                "Close",
                Some("Close webcam".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⎋"),
        ]
    }

    fn execute_webcam_action(&mut self, action_id: &str, cx: &mut Context<Self>) {
        match action_id {
            "capture" => {
                logging::log("ACTIONS", "execute_webcam_action: capture");
                if self.capture_webcam_photo(cx) {
                    self.hide_main_and_reset(cx);
                }
            }
            "close" => {
                logging::log("ACTIONS", "execute_webcam_action: close");
                cx.notify();
                self.show_hud("Webcam closed".to_string(), Some(1500), cx);
                self.hide_main_and_reset(cx);
            }
            _ => {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "execute_webcam_action: unknown id '{}', falling back to SDK action routing",
                        action_id
                    ),
                );
                self.trigger_action_by_name(action_id, cx);
            }
        }
    }

    // ========================================================================
    // Actions Dialog Routing - Shared key routing for all prompt types
    // ========================================================================

    /// Route keyboard events to the actions dialog when open.
    ///
    /// This centralizes the duplicated key routing logic from all render_prompts/*.rs
    /// files into a single location, eliminating ~80 lines of duplicated code per prompt.
    ///
    /// # Arguments
    /// * `key` - The key string from the KeyDownEvent (case-insensitive)
    /// * `key_char` - Optional key_char from the event for printable character input
    /// * `host` - Which type of host is routing (determines focus restoration behavior)
    /// * `window` - Window reference for focus operations
    /// * `cx` - Context for entity updates and notifications
    ///
    /// # Returns
    /// * `ActionsRoute::NotHandled` - Actions popup not open, route to normal handlers
    /// * `ActionsRoute::Handled` - Key was consumed by the actions dialog
    /// * `ActionsRoute::Execute { action_id }` - User selected an action, caller should execute it
}
