// Clipboard action handlers for handle_action dispatch.
//
// Contains all `clipboard_*` action handling: pin/unpin, share, paste, copy,
// quick look, attach to AI, open with, CleanShot, OCR, delete, save file,
// and save snippet.

impl ScriptListApp {
    /// Refresh clipboard selection after a delete operation, respecting the active filter.
    ///
    /// After entries are removed from `cached_clipboard_entries`, this method
    /// re-applies the current filter, clamps `selected_index` to the visible
    /// set, scrolls to the new position, and updates `focused_clipboard_entry_id`
    /// to point at a visible entry.
    fn refresh_clipboard_selection_after_delete(&mut self) {
        if let AppView::ClipboardHistoryView {
            filter,
            selected_index,
        } = &mut self.current_view
        {
            let filtered_entries: Vec<_> = if filter.is_empty() {
                self.cached_clipboard_entries.iter().enumerate().collect()
            } else {
                let filter_lower = filter.to_lowercase();
                self.cached_clipboard_entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.text_preview.to_lowercase().contains(&filter_lower))
                    .collect()
            };

            if !filtered_entries.is_empty() {
                *selected_index =
                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                self.clipboard_list_scroll_handle
                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                self.focused_clipboard_entry_id = filtered_entries
                    .get(*selected_index)
                    .map(|(_, entry)| entry.id.clone());
            } else {
                *selected_index = 0;
                self.focused_clipboard_entry_id = None;
            }
        }
    }

    fn spawn_clipboard_paste_simulation(&self) {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = selected_text::simulate_paste_with_cg() {
                    tracing::error!(error = %e, "failed to simulate paste");
                } else {
                    tracing::info!(
                        category = "UI",
                        event = "clipboard_paste_success",
                        "simulated Cmd+V paste"
                    );
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                tracing::warn!("clipboard paste simulation is not yet supported on this platform");
            }
        });
    }

    /// Handle clipboard-specific actions. Returns `true` if handled.
    ///
    /// Clipboard actions manage their own `cx.notify()` calls and early returns;
    /// the caller should **not** call `cx.notify()` when this returns a handled outcome.
    fn handle_clipboard_action(
        &mut self,
        action_id: &str,
        selected_clipboard_entry: Option<clipboard_history::ClipboardEntryMeta>,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let trace_id = &dctx.trace_id;
        match action_id {
            "clipboard_pin" | "clipboard_unpin" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                let result = if action_id == "clipboard_pin" {
                    clipboard_history::pin_entry(&entry.id)
                } else {
                    clipboard_history::unpin_entry(&entry.id)
                };

                match result {
                    Ok(()) => {
                        // Refresh cached entries (pin/unpin updates cache ordering)
                        self.cached_clipboard_entries =
                            clipboard_history::get_cached_entries(CLIPBOARD_CACHE_SIZE);

                        // Keep selection on the same entry when possible
                        if let AppView::ClipboardHistoryView {
                            filter,
                            selected_index,
                        } = &mut self.current_view
                        {
                            let filtered_entries: Vec<_> = if filter.is_empty() {
                                self.cached_clipboard_entries.iter().enumerate().collect()
                            } else {
                                let filter_lower = filter.to_lowercase();
                                self.cached_clipboard_entries
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, e)| {
                                        e.text_preview.to_lowercase().contains(&filter_lower)
                                    })
                                    .collect()
                            };

                            if let Some(new_index) =
                                filtered_entries.iter().position(|(_, e)| e.id == entry.id)
                            {
                                *selected_index = new_index;
                            } else if !filtered_entries.is_empty() {
                                *selected_index =
                                    (*selected_index).min(filtered_entries.len().saturating_sub(1));
                            } else {
                                *selected_index = 0;
                            }

                            if !filtered_entries.is_empty() {
                                self.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            }
                            self.focused_clipboard_entry_id = filtered_entries
                                .get(*selected_index)
                                .map(|(_, entry)| entry.id.clone());
                        }

                        if let Some(message) = clipboard_pin_action_success_hud(action_id) {
                            self.show_hud(message.to_string(), Some(HUD_SHORT_MS), cx);
                        }
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to toggle clipboard pin");
                        self.show_error_toast(format!("Failed to update pin: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            "clipboard_share" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_error_toast("Clipboard entry content unavailable", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(entry_id = %entry.id, content_type = ?entry.content_type, "opening share sheet");

                let share_result = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => {
                        crate::platform::show_share_sheet(
                            crate::platform::ShareSheetItem::Text(content),
                        );
                        Ok(())
                    }
                    clipboard_history::ContentType::Image => {
                        if let Some(png_bytes) =
                            clipboard_history::content_to_png_bytes(&content)
                        {
                            crate::platform::show_share_sheet(
                                crate::platform::ShareSheetItem::ImagePng(png_bytes),
                            );
                            Ok(())
                        } else {
                            Err("Failed to decode clipboard image".to_string())
                        }
                    }
                };

                match share_result {
                    Ok(()) => self.show_hud(
                        "Share sheet opened".to_string(),
                        Some(HUD_SHORT_MS),
                        cx,
                    ),
                    Err(message) => {
                        self.show_error_toast(message, cx);
                    }
                }
                DispatchOutcome::success()
            }
            // Paste to active app and close window (Enter)
            "clipboard_paste" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(entry_id = %entry.id, "paste entry");
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(
                            category = "UI",
                            event = "clipboard_paste_start",
                            "entry copied, hiding window before simulated paste"
                        );
                        self.hide_main_and_reset(cx);
                        self.spawn_clipboard_paste_simulation();
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to paste entry");
                        self.show_error_toast(format!("Failed to paste: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            "clipboard_attach_to_ai" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_error_toast("Clipboard entry content unavailable", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(
                    category = "AI",
                    entry_id = %entry.id,
                    content_type = ?entry.content_type,
                    "Attaching clipboard entry to AI chat"
                );

                let deferred_action = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::Color => {
                        DeferredAiWindowAction::SetInput { text: content, submit: false }
                    }
                    clipboard_history::ContentType::File => {
                        let attachment_path =
                            shellexpand::tilde(content.trim()).into_owned();
                        if attachment_path.is_empty() {
                            self.show_error_toast("Clipboard file path is empty", cx);
                            return DispatchOutcome::success();
                        }

                        DeferredAiWindowAction::AddAttachment {
                            path: attachment_path,
                        }
                    }
                    clipboard_history::ContentType::Image => {
                        // Offload PNG decode + base64 encode to a background thread
                        // to avoid blocking the UI during large image processing.
                        let (result_tx, result_rx) =
                            async_channel::bounded::<Result<String, String>>(1);
                        let trace_id = dctx.trace_id.clone();
                        let action_id_owned = action_id.to_string();

                        std::thread::spawn(move || {
                            let started_at = std::time::Instant::now();
                            let result = (|| {
                                let png_bytes =
                                    clipboard_history::content_to_png_bytes(&content)
                                        .ok_or_else(|| {
                                            "Failed to decode clipboard image".to_string()
                                        })?;
                                use base64::Engine;
                                Ok(base64::engine::general_purpose::STANDARD
                                    .encode(&png_bytes))
                            })();
                            tracing::info!(
                                category = "AI",
                                event = "clipboard_image_prep_done",
                                trace_id = %trace_id,
                                duration_ms = started_at.elapsed().as_millis() as u64,
                                success = result.is_ok(),
                                "Clipboard image preparation finished"
                            );
                            let _ = result_tx.send_blocking(result);
                        });

                        let trace_id = dctx.trace_id.clone();
                        let action_id_owned2 = action_id.to_string();
                        cx.spawn(async move |this, cx| {
                            let Ok(result) = result_rx.recv().await else {
                                return;
                            };
                            let _ = this.update(cx, |this, cx| match result {
                                Ok(image_base64) => {
                                    this.open_ai_window_after_main_hide(
                                        &action_id_owned2,
                                        &trace_id,
                                        DeferredAiWindowAction::SetInputWithImage {
                                            text: String::new(),
                                            image_base64,
                                            submit: false,
                                        },
                                        cx,
                                    );
                                }
                                Err(message) => {
                                    tracing::error!(
                                        category = "AI",
                                        action = %action_id_owned,
                                        error = %message,
                                        "Clipboard image decode failed"
                                    );
                                    this.show_error_toast(message, cx);
                                }
                            });
                        })
                        .detach();

                        return DispatchOutcome::success();
                    }
                };

                self.open_ai_window_after_main_hide(
                    action_id,
                    &dctx.trace_id,
                    deferred_action,
                    cx,
                );
                DispatchOutcome::success()
            }
            // Copy to clipboard without pasting (Cmd+Enter)
            "clipboard_copy" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(entry_id = %entry.id, "copying entry to clipboard");
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(category = "UI", event = "clipboard_copy_success", "entry copied to clipboard");
                        self.show_hud(
                            "Copied to clipboard".to_string(),
                            Some(HUD_SHORT_MS),
                            cx,
                        );
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to copy entry");
                        self.show_error_toast(format!("Failed to copy: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            // Paste and keep window open (Opt+Enter)
            "clipboard_paste_keep_open" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(entry_id = %entry.id, "paste and keep open");
                match clipboard_history::copy_entry_to_clipboard(&entry.id) {
                    Ok(()) => {
                        tracing::info!(
                            category = "UI",
                            event = "clipboard_paste_start",
                            "entry copied, simulating paste"
                        );
                        self.spawn_clipboard_paste_simulation();
                        // Keep the window open - do NOT call hide_main_and_reset
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to copy entry");
                        self.show_error_toast(format!("Failed to paste: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            "clipboard_quick_look" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                    tracing::error!(error = %e, "failed to Quick Look");
                    self.show_error_toast(format!("Failed to Quick Look: {}", e), cx);
                }
                DispatchOutcome::success()
            }
            "clipboard_open_with" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_error_toast("Failed to load clipboard content", cx);
                    return DispatchOutcome::success();
                };

                let full_entry = clipboard_history::ClipboardEntry {
                    id: entry.id.clone(),
                    content,
                    content_type: entry.content_type,
                    timestamp: entry.timestamp,
                    pinned: entry.pinned,
                    ocr_text: entry.ocr_text.clone(),
                    source_app_name: None,
                    source_app_bundle_id: None,
                };

                let temp_path = match clipboard_history::save_entry_to_temp_file(&full_entry) {
                    Ok(path) => path,
                    Err(e) => {
                        tracing::error!(error = %e, "failed to save temp file");
                        self.show_error_toast("Failed to save temp file", cx);
                        return DispatchOutcome::success();
                    }
                };

                #[cfg(target_os = "macos")]
                {
                    let path_str = temp_path.to_string_lossy().to_string();
                    if let Err(e) = crate::file_search::open_with(&path_str) {
                        tracing::error!(error = %e, "failed to Open With");
                        self.show_error_toast("Failed to Open With", cx);
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let _ = temp_path;
                    self.show_unsupported_platform_toast("Open With", cx);
                }
                DispatchOutcome::success()
            }
            "clipboard_annotate_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_error_toast(
                        "CleanShot actions are only available for images",
                        cx,
                    );
                    return DispatchOutcome::success();
                }

                #[cfg(target_os = "macos")]
                {
                    if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                        tracing::error!(error = %e, "failed to copy image");
                        self.show_error_toast("Failed to copy image", cx);
                        return DispatchOutcome::success();
                    }

                    let url = "cleanshot://open-from-clipboard";
                    match std::process::Command::new("open").arg(url).spawn() {
                        Ok(_) => {
                            self.show_hud(
                                "Opening CleanShot X…".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "failed to open CleanShot X");
                            self.show_error_toast("Failed to open CleanShot X", cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_unsupported_platform_toast("CleanShot", cx);
                }
                DispatchOutcome::success()
            }
            "clipboard_upload_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_error_toast(
                        "CleanShot actions are only available for images",
                        cx,
                    );
                    return DispatchOutcome::success();
                }

                #[cfg(target_os = "macos")]
                {
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_error_toast("Failed to load image content", cx);
                        return DispatchOutcome::success();
                    };

                    let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                    else {
                        self.show_error_toast("Failed to decode image", cx);
                        return DispatchOutcome::success();
                    };

                    let temp_path = std::env::temp_dir()
                        .join(format!("script-kit-clipboard-{}.png", uuid::Uuid::new_v4()));

                    if let Err(e) = std::fs::write(&temp_path, png_bytes) {
                        tracing::error!(error = %e, "failed to write temp image");
                        self.show_error_toast("Failed to save image", cx);
                        return DispatchOutcome::success();
                    }

                    let path_str = temp_path.to_string_lossy();
                    let encoded_path = self.percent_encode_for_url(&path_str);
                    let url = format!(
                        "cleanshot://open-annotate?filepath={}&action=upload",
                        encoded_path
                    );

                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => {
                            self.show_hud(
                                "Opening CleanShot X upload…".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "failed to open CleanShot X");
                            self.show_error_toast("Failed to open CleanShot X", cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_unsupported_platform_toast("CleanShot", cx);
                }
                DispatchOutcome::success()
            }
            "clipboard_ocr" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_error_toast("OCR is only available for images", cx);
                    return DispatchOutcome::success();
                }

                // Check if we already have cached OCR text
                if let Some(ref cached_text) = entry.ocr_text {
                    if !cached_text.trim().is_empty() {
                        tracing::debug!(category = "UI", event = "clipboard_ocr_cached", "using cached OCR text");
                        self.copy_to_clipboard_with_feedback(
                            cached_text,
                            "Copied text from image".to_string(),
                            true,
                            cx,
                        );
                        return DispatchOutcome::success();
                    }
                }

                #[cfg(all(target_os = "macos", feature = "ocr"))]
                {
                    // Get image content
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_error_toast("Failed to load image content", cx);
                        return DispatchOutcome::success();
                    };

                    // Decode to RGBA bytes for OCR
                    let Some((width, height, rgba_bytes)) =
                        clipboard_history::decode_to_rgba_bytes(&content)
                    else {
                        self.show_error_toast("Failed to decode image", cx);
                        return DispatchOutcome::success();
                    };

                    tracing::debug!(width, height, "starting OCR on image");

                    let entry_id = entry.id.clone();
                    match script_kit_gpui::ocr::extract_text_from_rgba(
                        width,
                        height,
                        &rgba_bytes,
                    ) {
                        Ok(text) => {
                            if text.trim().is_empty() {
                                tracing::debug!(category = "UI", event = "clipboard_ocr_empty", "no text found in image");
                                self.show_error_toast("No text found in image", cx);
                            } else {
                                tracing::debug!(chars = text.len(), "extracted OCR text");

                                // Cache the OCR result
                                let _ = clipboard_history::update_ocr_text(&entry_id, &text);

                                // Copy to clipboard
                                self.copy_to_clipboard_with_feedback(
                                    &text,
                                    "Copied text from image".to_string(),
                                    true,
                                    cx,
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "OCR failed");
                            self.show_error_toast(format!("Failed to extract text: {}", e), cx);
                        }
                    }
                }

                #[cfg(not(all(target_os = "macos", feature = "ocr")))]
                {
                    self.show_unsupported_platform_toast("OCR", cx);
                }
                DispatchOutcome::success()
            }
            // Clipboard delete actions
            "clipboard_delete_multiple" => {
                let filter_text = match &self.current_view {
                    AppView::ClipboardHistoryView { filter, .. } => filter.trim().to_string(),
                    _ => String::new(),
                };

                if filter_text.is_empty() {
                    self.show_error_toast(
                        "Type in search first, then use Delete Entries...",
                        cx,
                    );
                    return DispatchOutcome::success();
                }

                let filter_lower = filter_text.to_lowercase();
                let ids_to_delete: Vec<String> = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|entry| entry.text_preview.to_lowercase().contains(&filter_lower))
                    .map(|entry| entry.id.clone())
                    .collect();

                if ids_to_delete.is_empty() {
                    self.show_error_toast("No matching entries to delete", cx);
                    return DispatchOutcome::success();
                }

                let delete_count = ids_to_delete.len();
                let confirm_options = crate::confirm::ParentConfirmOptions::destructive(
                    "Delete Clipboard Entries",
                    format!(
                        "Are you sure you want to delete these {} matching clipboard entries?",
                        delete_count
                    ),
                    "Delete",
                );
                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();

                cx.spawn(async move |this, cx| {
                    match crate::confirm::confirm_with_parent_dialog(cx, confirm_options, &trace_id).await {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::info!(
                                trace_id = %trace_id,
                                status = "cancelled",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action cancelled: clipboard_delete_multiple"
                            );
                            return;
                        }
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    error = %e,
                                    "failed to open confirmation modal"
                                );
                                this.show_error_toast_with_code(
                                    "Failed to open confirmation dialog",
                                    Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                    cx,
                                );
                            });
                            return;
                        }
                    }

                    let _ = this.update(cx, move |this, cx| {
                        let mut deleted = 0usize;
                        let mut failed = 0usize;
                        for id in ids_to_delete {
                            match clipboard_history::remove_entry(&id) {
                                Ok(()) => deleted += 1,
                                Err(e) => {
                                    failed += 1;
                                    tracing::error!(entry_id = %id, error = %e, "failed to delete clipboard entry");
                                }
                            }
                        }

                        this.cached_clipboard_entries =
                            clipboard_history::get_cached_entries(CLIPBOARD_CACHE_SIZE);
                        this.refresh_clipboard_selection_after_delete();
                        cx.notify();

                        if failed == 0 {
                            this.show_hud(
                                format!("Deleted {} entries", deleted),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        } else {
                            this.show_error_toast(
                                format!("Deleted {}, failed {}", deleted, failed),
                                cx,
                            );
                        }
                    });
                })
                .detach();
                DispatchOutcome::success()
            }
            "clipboard_delete" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                match clipboard_history::remove_entry(&entry.id) {
                    Ok(()) => {
                        tracing::info!(entry_id = %entry.id, "deleted clipboard entry");
                        // Refresh cached entries
                        self.cached_clipboard_entries =
                            clipboard_history::get_cached_entries(CLIPBOARD_CACHE_SIZE);

                        // Update selection respecting active filter
                        self.refresh_clipboard_selection_after_delete();

                        self.show_hud("Entry deleted".to_string(), Some(HUD_SHORT_MS), cx);
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to delete clipboard entry");
                        self.show_error_toast(format!("Failed to delete: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            "clipboard_delete_all" => {
                // Delete all unpinned entries
                let unpinned_count = self
                    .cached_clipboard_entries
                    .iter()
                    .filter(|e| !e.pinned)
                    .count();

                if unpinned_count == 0 {
                    self.show_error_toast("No unpinned entries to delete", cx);
                    return DispatchOutcome::success();
                }

                let confirm_options = crate::confirm::ParentConfirmOptions::destructive(
                    "Delete All Clipboard Entries",
                    format!(
                        "Are you sure you want to delete all {} unpinned clipboard entries?",
                        unpinned_count
                    ),
                    "Delete All",
                );
                let trace_id = trace_id.to_string();
                let start = std::time::Instant::now();

                cx.spawn(async move |this, cx| {
                    match crate::confirm::confirm_with_parent_dialog(cx, confirm_options, &trace_id).await {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::info!(
                                trace_id = %trace_id,
                                status = "cancelled",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action cancelled: clipboard_delete_all"
                            );
                            return;
                        }
                        Err(e) => {
                            let _ = this.update(cx, |this, cx| {
                                tracing::error!(
                                    trace_id = %trace_id,
                                    status = "failed",
                                    duration_ms = start.elapsed().as_millis() as u64,
                                    error = %e,
                                    "failed to open confirmation modal"
                                );
                                this.show_error_toast_with_code(
                                    "Failed to open confirmation dialog",
                                    Some(crate::action_helpers::ERROR_MODAL_FAILED),
                                    cx,
                                );
                            });
                            return;
                        }
                    }

                    let _ = this.update(cx, move |this, cx| {
                        match clipboard_history::clear_unpinned_history() {
                            Ok(()) => {
                                tracing::info!(count = unpinned_count, "deleted unpinned clipboard entries");
                                this.cached_clipboard_entries =
                                    clipboard_history::get_cached_entries(CLIPBOARD_CACHE_SIZE);

                                // Reset selection respecting active filter
                                this.refresh_clipboard_selection_after_delete();

                                this.show_hud(
                                    format!(
                                        "Deleted {} entries (pinned preserved)",
                                        unpinned_count
                                    ),
                                    Some(HUD_2500_MS),
                                    cx,
                                );
                                cx.notify();
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to clear unpinned history");
                                this.show_error_toast(format!("Failed to delete: {}", e), cx);
                            }
                        }
                    });
                })
                .detach();
                DispatchOutcome::success()
            }

            "clipboard_save_file" => {
                let Some(entry) = selected_clipboard_entry.clone() else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_error_toast("Clipboard content unavailable", cx);
                    return DispatchOutcome::success();
                };

                // Determine filename and content based on type
                let (file_content, extension) = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => (content.into_bytes(), "txt"),
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) =
                            clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_error_toast("Failed to decode image", cx);
                            return DispatchOutcome::success();
                        };
                        (png_bytes, "png")
                    }
                };

                // Get save location (Desktop or home)
                let home =
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                let desktop = home.join("Desktop");
                let save_dir = if desktop.exists() { desktop } else { home };

                // Generate unique filename with timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("clipboard-{}.{}", timestamp, extension);
                let save_path = save_dir.join(&filename);

                match std::fs::write(&save_path, &file_content) {
                    Ok(()) => {
                        tracing::info!(path = ?save_path, "saved clipboard to file");
                        let reveal_result_rx =
                            self.reveal_in_finder_with_feedback_async(&save_path, trace_id);
                        let trace_id = trace_id.to_string();
                        let start = std::time::Instant::now();
                        cx.spawn(async move |this, cx| {
                            let Ok(reveal_result) = reveal_result_rx.recv().await else {
                                return;
                            };

                            let _ = this.update(cx, |this, cx| match reveal_result {
                                Ok(()) => {
                                    tracing::info!(
                                        trace_id = %trace_id,
                                        status = "completed",
                                        duration_ms = start.elapsed().as_millis() as u64,
                                        "Async action completed: clipboard_save_file"
                                    );
                                    this.show_hud(
                                        format!("Saved to: {}", save_path.display()),
                                        Some(HUD_LONG_MS),
                                        cx,
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                                Err(message) => {
                                    // File was saved but reveal failed — log the reveal error,
                                    // show only HUD for the successful save (no dual feedback).
                                    tracing::warn!(
                                        trace_id = %trace_id,
                                        status = "completed",
                                        duration_ms = start.elapsed().as_millis() as u64,
                                        error = %message,
                                        "Async action completed with warning: clipboard_save_file reveal failed"
                                    );
                                    this.show_hud(
                                        format!("Saved to: {}", save_path.display()),
                                        Some(HUD_LONG_MS),
                                        cx,
                                    );
                                    this.hide_main_and_reset(cx);
                                }
                            });
                        })
                        .detach();
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to save file");
                        self.show_error_toast(format!("Failed to save: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }
            "clipboard_save_snippet" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_error_toast("No clipboard entry selected", cx);
                    return DispatchOutcome::success();
                };

                if entry.content_type != clipboard_history::ContentType::Text {
                    self.show_error_toast("Only text can be saved as snippet", cx);
                    return DispatchOutcome::success();
                }

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_error_toast("Clipboard content unavailable", cx);
                    return DispatchOutcome::success();
                };

                // Generate a default keyword from the first few words
                let default_keyword: String = content
                    .chars()
                    .take(20)
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                let default_keyword = if default_keyword.is_empty() {
                    "snippet".to_string()
                } else {
                    default_keyword
                };

                // Create snippet file in extensions directory
                let kenv = dirs::home_dir()
                    .map(|h| h.join(".kenv"))
                    .unwrap_or_else(|| std::path::PathBuf::from("/"));
                let extensions_dir = kenv.join("extensions");
                let snippets_file = extensions_dir.join("clipboard-snippets.md");

                // Ensure extensions directory exists
                if !extensions_dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&extensions_dir) {
                        tracing::error!(error = %e, "failed to create extensions dir");
                        self.show_error_toast(
                            format!("Failed to create snippets: {}", e),
                            cx,
                        );
                        return DispatchOutcome::success();
                    }
                }

                // Generate unique keyword with timestamp suffix
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() % 10000)
                    .unwrap_or(0);
                let keyword = format!("{}-{}", default_keyword, timestamp);

                // Create snippet entry with proper fence handling
                let fence = if content.contains("```") {
                    "~~~~"
                } else {
                    "```"
                };
                let snippet_entry = format!(
                    "\n## {}\n\n{}\nname: {}\ntool: paste\nkeyword: {}\n{}\n\n{}paste\n{}\n{}\n",
                    keyword, fence, keyword, keyword, fence, fence, content, fence
                );

                // Append to snippets file
                let result = if snippets_file.exists() {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&snippets_file)
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(snippet_entry.as_bytes())
                        })
                } else {
                    let header = "# Clipboard Snippets\n\nSnippets created from clipboard history.\n";
                    std::fs::write(
                        &snippets_file,
                        format!("{}{}", header, snippet_entry),
                    )
                };

                match result {
                    Ok(()) => {
                        tracing::info!(keyword = %keyword, "created snippet");
                        self.show_hud(
                            format!("Snippet created: type '{}' to paste", keyword),
                            Some(HUD_LONG_MS),
                            cx,
                        );
                        // Refresh scripts to pick up new snippet
                        self.refresh_scripts(cx);
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to save snippet");
                        self.show_error_toast(format!("Failed to save: {}", e), cx);
                    }
                }
                DispatchOutcome::success()
            }

            _ => DispatchOutcome::not_handled(),
        }
    }
}
