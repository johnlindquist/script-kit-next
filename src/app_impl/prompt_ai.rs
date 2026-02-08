use super::*;
use anyhow::{anyhow, Context as AnyhowContext, Result};

#[derive(Debug, Clone, Copy)]
enum AiScriptGenerationStage {
    SelectModel,
    ResolveProvider,
    RequestCompletion,
    ExtractScript,
    CreateScriptFile,
    WriteScriptFile,
    OpenEditor,
}

impl AiScriptGenerationStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::SelectModel => "select_model",
            Self::ResolveProvider => "resolve_provider",
            Self::RequestCompletion => "request_completion",
            Self::ExtractScript => "extract_script",
            Self::CreateScriptFile => "create_script_file",
            Self::WriteScriptFile => "write_script_file",
            Self::OpenEditor => "open_editor",
        }
    }
}

const AI_SCRIPT_GENERATION_SYSTEM_PROMPT: &str = r#"You are ScriptKitScriptGenerator.
Generate a complete Script Kit TypeScript script and return code only.
Requirements:
- Include metadata comments at the top:
  // Name: <script name>
  // Description: <short summary>
- Include: import "@scriptkit/sdk";
- Use idiomatic Script Kit APIs (for example await arg(), await div(), await editor()) when useful.
- Prefer clear async/await flow and practical defaults.
- Return a full runnable script with no extra explanation."#;

fn build_ai_script_generation_user_prompt(description: &str) -> String {
    format!(
        "Generate a complete Script Kit script for this request:\n\n{}\n\nReturn only the TypeScript script source.",
        description.trim()
    )
}

fn select_default_ai_script_model(
    registry: &crate::ai::ProviderRegistry,
) -> Option<crate::ai::ModelInfo> {
    let all_models = registry.get_all_models();
    all_models
        .iter()
        .find(|model| model.provider.eq_ignore_ascii_case("vercel"))
        .cloned()
        .or_else(|| all_models.first().cloned())
}

fn extract_generated_script_source(raw_response: &str) -> Option<String> {
    let trimmed = raw_response.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Prefer typed code fences first, then generic fences.
    for fence in ["```typescript", "```ts", "```javascript", "```js", "```"] {
        if let Some(content) = extract_first_fenced_block(trimmed, fence) {
            return normalize_generated_script(content);
        }
    }

    normalize_generated_script(trimmed.to_string())
}

fn extract_first_fenced_block(response: &str, fence_start: &str) -> Option<String> {
    let start_index = response.find(fence_start)?;
    let after_fence = &response[start_index + fence_start.len()..];
    let after_newline = after_fence
        .strip_prefix("\r\n")
        .or_else(|| after_fence.strip_prefix('\n'))
        .unwrap_or(after_fence);
    let end_index = after_newline.find("```")?;
    Some(after_newline[..end_index].trim().to_string())
}

fn normalize_generated_script(content: String) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("{trimmed}\n"))
    }
}

fn derive_script_name_from_description(description: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in description.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }

        if slug.len() >= 48 {
            break;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "ai-generated-script".to_string()
    } else {
        slug
    }
}

fn generate_script_via_ai_backend(
    registry: &crate::ai::ProviderRegistry,
    model_id: &str,
    prompt_description: &str,
    config: &crate::config::Config,
) -> Result<std::path::PathBuf> {
    let provider = registry
        .find_provider_for_model(model_id)
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "state={} attempted=find_provider_for_model model_id={} failure=provider_not_found",
                AiScriptGenerationStage::ResolveProvider.as_str(),
                model_id
            )
        })?;

    let request_messages = vec![
        crate::ai::ProviderMessage::system(AI_SCRIPT_GENERATION_SYSTEM_PROMPT),
        crate::ai::ProviderMessage::user(build_ai_script_generation_user_prompt(prompt_description)),
    ];

    let ai_response = provider
        .send_message(&request_messages, model_id)
        .with_context(|| {
            format!(
                "state={} attempted=send_message model_id={} provider={} failure=provider_call_failed",
                AiScriptGenerationStage::RequestCompletion.as_str(),
                model_id,
                provider.provider_id()
            )
        })?;

    let generated_script = extract_generated_script_source(&ai_response).ok_or_else(|| {
        anyhow!(
            "state={} attempted=extract_generated_script model_id={} failure=empty_script_response",
            AiScriptGenerationStage::ExtractScript.as_str(),
            model_id
        )
    })?;

    let script_name = derive_script_name_from_description(prompt_description);
    let script_path = crate::script_creation::create_new_script(&script_name).with_context(|| {
        format!(
            "state={} attempted=create_new_script name={} failure=create_script_failed",
            AiScriptGenerationStage::CreateScriptFile.as_str(),
            script_name
        )
    })?;

    std::fs::write(&script_path, generated_script).with_context(|| {
        format!(
            "state={} attempted=write_script path={} failure=write_failed",
            AiScriptGenerationStage::WriteScriptFile.as_str(),
            script_path.display()
        )
    })?;

    crate::script_creation::open_in_editor(&script_path, config).with_context(|| {
        format!(
            "state={} attempted=open_in_editor path={} failure=open_editor_failed",
            AiScriptGenerationStage::OpenEditor.as_str(),
            script_path.display()
        )
    })?;

    Ok(script_path)
}

impl ScriptListApp {
    pub(crate) fn is_in_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::DivPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
        )
    }

    /// Submit a response to the current prompt
    ///
    /// Uses try_send() to avoid blocking the UI thread if the script's input
    /// channel is full. User-initiated actions should never freeze the UI.
    pub(crate) fn submit_prompt_response(
        &mut self,
        id: String,
        value: Option<String>,
        _cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!("Submitting response for {}: {:?}", id, value),
        );

        let response = Message::Submit { id, value };

        if let Some(ref sender) = self.response_sender {
            // Use try_send to avoid blocking UI thread
            // If channel is full, the script isn't reading - log warning but don't freeze UI
            match sender.try_send(response) {
                Ok(()) => {
                    logging::log("UI", "Response queued for script");
                }
                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                    // Channel is full - script isn't reading stdin fast enough
                    // This shouldn't happen in normal operation, log as warning
                    logging::log(
                        "WARN",
                        "Response channel full - script may be stuck. Response dropped.",
                    );
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    // Channel disconnected - script has exited
                    logging::log("UI", "Response channel disconnected - script exited");
                }
            }
        } else {
            logging::log("UI", "No response sender available");
        }

        // Return to waiting state (script will send next prompt or exit)
        // Don't change view here - wait for next message from script
    }

    /// Get filtered choices for arg prompt
    pub(crate) fn filtered_arg_choices(&self) -> Vec<(usize, &Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices.iter().enumerate().collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// P0: Get filtered choices as owned data for uniform_list closure
    pub(crate) fn get_filtered_arg_choices_owned(&self) -> Vec<(usize, Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Create box shadows from theme configuration
    pub(crate) fn create_box_shadows(&self) -> Vec<BoxShadow> {
        let shadow_config = self.theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        // For black (0x000000), we use h=0, s=0, l=0
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion for shadow color
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0) // achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Show inline AI chat prompt with built-in AI provider support.
    /// This switches to the ChatPrompt view with direct AI integration (no SDK needed).
    /// Prefers Vercel AI Gateway if configured, otherwise uses the first available provider.
    pub fn show_inline_ai_chat(&mut self, initial_query: Option<String>, cx: &mut Context<Self>) {
        use crate::ai::ProviderRegistry;
        use crate::prompts::{ChatEscapeCallback, ChatPrompt, ChatSubmitCallback};

        // Mark as opened from main menu so ESC returns to main menu
        self.opened_from_main_menu = true;

        // Create escape callback that signals via channel
        let escape_sender = self.inline_chat_escape_sender.clone();
        let escape_callback: ChatEscapeCallback = std::sync::Arc::new(move |_id| {
            let _ = escape_sender.try_send(());
        });

        // Use cached registry if available, otherwise build synchronously as fallback
        let registry = self
            .cached_provider_registry
            .clone()
            .unwrap_or_else(|| ProviderRegistry::from_environment_with_config(Some(&self.config)));

        if !registry.has_any_provider() {
            crate::logging::log("CHAT", "No AI providers configured - showing setup card");

            // Create configure callback that signals via channel
            let configure_sender = self.inline_chat_configure_sender.clone();
            let configure_callback: crate::prompts::ChatConfigureCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log("CHAT", "Configure callback triggered - sending signal");
                    let _ = configure_sender.try_send(());
                });

            // Create Claude Code callback that signals via channel
            let claude_code_sender = self.inline_chat_claude_code_sender.clone();
            let claude_code_callback: crate::prompts::ChatClaudeCodeCallback =
                std::sync::Arc::new(move || {
                    crate::logging::log("CHAT", "Claude Code callback triggered - sending signal");
                    let _ = claude_code_sender.try_send(());
                });

            // Create a no-op submit callback since we're in setup mode
            let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
                crate::logging::log("CHAT", "No providers - submission ignored (setup mode)");
            });

            let chat_prompt = ChatPrompt::new(
                "inline-ai-setup".to_string(),
                Some("Configure API key to continue...".to_string()),
                vec![],
                None, // No hint needed - setup card is the UI
                None,
                self.focus_handle.clone(),
                noop_callback,
                std::sync::Arc::clone(&self.theme),
            )
            .with_title("Ask AI")
            .with_save_history(false) // Don't save setup state to history
            .with_escape_callback(escape_callback.clone())
            .with_needs_setup(true)
            .with_configure_callback(configure_callback)
            .with_claude_code_callback(claude_code_callback);

            let entity = cx.new(|_| chat_prompt);
            self.current_view = AppView::ChatPrompt {
                id: "inline-ai-setup".to_string(),
                entity,
            };
            self.focused_input = FocusedInput::None;
            self.pending_focus = Some(FocusTarget::ChatPrompt);
            resize_to_view_sync(ViewType::DivPrompt, 0);
            cx.notify();
            return;
        }

        crate::logging::log(
            "CHAT",
            &format!(
                "Showing inline AI chat with {} providers",
                registry.provider_ids().len()
            ),
        );

        // Create a no-op callback since built-in AI handles submissions internally
        let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
            // Built-in AI mode handles this internally
        });

        let placeholder = Some("Ask anything...".to_string());

        let mut chat_prompt = ChatPrompt::new(
            "inline-ai".to_string(),
            placeholder,
            vec![],
            None,
            None,
            self.focus_handle.clone(),
            noop_callback,
            std::sync::Arc::clone(&self.theme),
        )
        .with_title("Ask AI")
        .with_save_history(true)
        .with_escape_callback(escape_callback)
        .with_builtin_ai(registry, true); // true = prefer Vercel AI Gateway

        // If there's an initial query, set it in the input and auto-submit
        if let Some(query) = initial_query {
            chat_prompt.input.set_text(&query);
            chat_prompt = chat_prompt.with_pending_submit(true);
        }

        let entity = cx.new(|_| chat_prompt);
        self.current_view = AppView::ChatPrompt {
            id: "inline-ai".to_string(),
            entity,
        };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::ChatPrompt);
        resize_to_view_sync(ViewType::DivPrompt, 0);
        cx.notify();
    }

    /// Generate a Script Kit script from a natural-language prompt using the built-in AI backend.
    /// The generated script is saved to disk and opened in the configured editor.
    pub fn generate_script_from_ai_prompt(
        &mut self,
        prompt_description: String,
        cx: &mut Context<Self>,
    ) {
        let prompt_description = prompt_description.trim().to_string();
        if prompt_description.is_empty() {
            return;
        }

        let registry = self.cached_provider_registry.clone().unwrap_or_else(|| {
            crate::ai::ProviderRegistry::from_environment_with_config(Some(&self.config))
        });

        if !registry.has_any_provider() {
            self.toast_manager.push(
                components::toast::Toast::error("No AI providers configured for script generation", &self.theme)
                    .duration_ms(Some(5000)),
            );
            cx.notify();
            return;
        }

        let selected_model = match select_default_ai_script_model(&registry) {
            Some(model) => model,
            None => {
                let stage = AiScriptGenerationStage::SelectModel.as_str();
                logging::log(
                    "AI_SCRIPT_GEN",
                    &format!(
                        "state={} attempted=select_default_model failure=no_available_models",
                        stage
                    ),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        "No AI models available for script generation",
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }
        };

        let model_id = selected_model.id.clone();
        let provider = selected_model.provider.clone();
        let config = self.config.clone();
        let (tx, rx) = async_channel::bounded::<std::result::Result<std::path::PathBuf, String>>(1);

        logging::log(
            "AI_SCRIPT_GEN",
            &format!(
                "state=queued attempted=shift_tab_script_generation model_id={} provider={} prompt_len={}",
                model_id,
                provider,
                prompt_description.len()
            ),
        );
        self.show_hud("Generating script with AI...".to_string(), Some(1500), cx);

        std::thread::spawn(move || {
            logging::log(
                "AI_SCRIPT_GEN",
                &format!(
                    "state=running attempted=generate_script model_id={} provider={}",
                    model_id, provider
                ),
            );

            let generation_result =
                generate_script_via_ai_backend(&registry, &model_id, &prompt_description, &config)
                    .map_err(|error| error.to_string());

            if tx.send_blocking(generation_result).is_err() {
                logging::log(
                    "AI_SCRIPT_GEN",
                    "state=aborted attempted=send_result failure=result_channel_closed",
                );
            }
        });

        cx.spawn(async move |this, cx| {
            let Ok(result) = rx.recv().await else {
                logging::log(
                    "AI_SCRIPT_GEN",
                    "state=aborted attempted=recv_result failure=result_channel_closed",
                );
                return;
            };

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    match result {
                        Ok(script_path) => {
                            let script_name = script_path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("generated script");
                            logging::log(
                                "AI_SCRIPT_GEN",
                                &format!(
                                    "state=completed attempted=generate_script path={}",
                                    script_path.display()
                                ),
                            );
                            app.toast_manager.push(
                                components::toast::Toast::success(
                                    format!("Generated and opened {}", script_name),
                                    &app.theme,
                                )
                                .duration_ms(Some(3500)),
                            );
                            app.close_and_reset_window(cx);
                        }
                        Err(error) => {
                            logging::log(
                                "AI_SCRIPT_GEN",
                                &format!("state=failed attempted=generate_script error={}", error),
                            );
                            app.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to generate script: {}", error),
                                    &app.theme,
                                )
                                .duration_ms(Some(7000)),
                            );
                            cx.notify();
                        }
                    }
                })
            });
        })
        .detach();
    }

    /// Rebuild the cached provider registry in a background thread.
    /// Called after config changes (API key saved, Claude Code enabled, etc.)
    pub fn rebuild_provider_registry_async(&mut self, cx: &mut Context<Self>) {
        self.cached_provider_registry = None;
        let config_clone = self.config.clone();
        let (tx, rx) = async_channel::bounded::<crate::ai::ProviderRegistry>(1);

        std::thread::spawn(move || {
            let registry =
                crate::ai::ProviderRegistry::from_environment_with_config(Some(&config_clone));
            if tx.send_blocking(registry).is_err() {
                logging::log(
                    "APP",
                    "Provider registry rebuild result dropped: receiver unavailable",
                );
            }
        });

        cx.spawn(async move |this, cx| {
            let Ok(registry) = rx.recv().await else {
                logging::log("APP", "Provider registry rebuild failed: channel closed");
                return;
            };

            let provider_count = registry.provider_ids().len();
            let _ = cx.update(|cx| {
                this.update(cx, |app, _cx| {
                    app.cached_provider_registry = Some(registry);
                    logging::log(
                        "APP",
                        &format!("Provider registry rebuilt: {} providers", provider_count),
                    );
                })
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ai_script_generation_user_prompt_includes_description() {
        let prompt = build_ai_script_generation_user_prompt("create a weather script");
        assert!(prompt.contains("create a weather script"));
        assert!(prompt.contains("Script Kit script"));
    }

    #[test]
    fn test_extract_generated_script_source_prefers_typescript_fence() {
        let response = r#"
Here's your script:
```typescript
// Name: Weather
import "@scriptkit/sdk";
```
"#;
        let extracted =
            extract_generated_script_source(response).expect("script should be extracted");
        assert!(extracted.contains("// Name: Weather"));
        assert!(!extracted.contains("```"));
    }

    #[test]
    fn test_extract_generated_script_source_falls_back_to_plain_text() {
        let response = "// Name: Plain\nimport \"@scriptkit/sdk\";";
        let extracted =
            extract_generated_script_source(response).expect("script should be extracted");
        assert_eq!(extracted, "// Name: Plain\nimport \"@scriptkit/sdk\";\n");
    }

    #[test]
    fn test_derive_script_name_from_description_uses_fallback_for_symbols() {
        let name = derive_script_name_from_description("@@@ !!!");
        assert_eq!(name, "ai-generated-script");
    }
}
