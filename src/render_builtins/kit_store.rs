use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

const KIT_STORE_GITHUB_API_BASE: &str = "https://api.github.com";
const KIT_STORE_GITHUB_ACCEPT: &str = "application/vnd.github+json";
const KIT_STORE_GITHUB_VERSION: &str = "2022-11-28";
const KIT_STORE_GITHUB_USER_AGENT: &str = "script-kit-gpui-kit-store-view";
const KIT_STORE_GITHUB_TOPICS: [&str; 2] = ["scriptkit-kit", "script-kit"];

#[derive(Debug, Default, Deserialize)]
struct KitStoreGithubSearchResponse {
    #[serde(default)]
    items: Vec<KitStoreGithubRepo>,
}

#[derive(Debug, Default, Deserialize)]
struct KitStoreGithubRepo {
    #[serde(default)]
    name: String,
    #[serde(default)]
    full_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    stargazers_count: u64,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    clone_url: String,
}

impl ScriptListApp {
    fn kit_store_search_results(query: &str) -> Vec<KitStoreSearchResult> {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .https_only(true)
            .build()
            .new_agent();

        let normalized_query = query.split_whitespace().collect::<Vec<_>>().join("+");
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        for topic in KIT_STORE_GITHUB_TOPICS {
            let url = if normalized_query.is_empty() {
                format!(
                    "{KIT_STORE_GITHUB_API_BASE}/search/repositories?q=topic:{topic}&sort=stars&order=desc"
                )
            } else {
                format!(
                    "{KIT_STORE_GITHUB_API_BASE}/search/repositories?q=topic:{topic}+{normalized_query}&sort=stars&order=desc"
                )
            };

            let response = match agent
                .get(&url)
                .header("Accept", KIT_STORE_GITHUB_ACCEPT)
                .header("X-GitHub-Api-Version", KIT_STORE_GITHUB_VERSION)
                .header("User-Agent", KIT_STORE_GITHUB_USER_AGENT)
                .call()
            {
                Ok(response) => response,
                Err(error) => {
                    tracing::warn!("Kit Store GitHub search request failed: {}", error);
                    continue;
                }
            };

            let status = response.status().as_u16();
            if !(200..300).contains(&status) {
                tracing::warn!("Kit Store GitHub search status: {}", status);
                continue;
            }

            let mut body = response.into_body();
            let parsed = match body.read_json::<KitStoreGithubSearchResponse>() {
                Ok(parsed) => parsed,
                Err(error) => {
                    tracing::warn!("Kit Store GitHub JSON parse failed: {}", error);
                    continue;
                }
            };

            for item in parsed.items {
                if item.full_name.is_empty() || !seen.insert(item.full_name.clone()) {
                    continue;
                }
                results.push(KitStoreSearchResult {
                    name: item.name,
                    full_name: item.full_name,
                    description: item.description.unwrap_or_default(),
                    stars: item.stargazers_count,
                    html_url: item.html_url,
                    clone_url: item.clone_url,
                });
            }
        }

        results.sort_by(|a, b| b.stars.cmp(&a.stars).then_with(|| a.name.cmp(&b.name)));
        results
    }

    fn kit_store_list_installed() -> Vec<script_kit_gpui::kit_store::InstalledKit> {
        script_kit_gpui::kit_store::storage::list_installed_kits().unwrap_or_else(|error| {
            tracing::warn!("Kit Store list installed kits failed: {}", error);
            Vec::new()
        })
    }

    fn kit_store_git_error(operation: &str, status: std::process::ExitStatus, stderr: &[u8]) -> String {
        let stderr = String::from_utf8_lossy(stderr).trim().to_string();
        format!(
            "{} failed with status {}{}",
            operation,
            status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {}", stderr)
            }
        )
    }

    fn kit_store_derive_name(repo_url: &str) -> Result<String, String> {
        let trimmed = repo_url.trim();
        if trimmed.is_empty() {
            return Err("Repository URL is empty".to_string());
        }

        let without_query = trimmed.split('?').next().unwrap_or(trimmed);
        let without_fragment = without_query.split('#').next().unwrap_or(without_query);
        let normalized = match without_fragment.rsplit_once(':') {
            Some((_, remainder)) if without_fragment.contains('@') && remainder.contains('/') => {
                remainder
            }
            _ => without_fragment,
        };
        let normalized = normalized.trim_end_matches('/');
        let normalized = normalized.strip_suffix(".git").unwrap_or(normalized);
        let name = normalized.rsplit('/').next().unwrap_or_default().trim();
        if name.is_empty() {
            return Err(format!(
                "Unable to derive kit name from repository URL '{}'",
                repo_url
            ));
        }
        Ok(name.to_string())
    }

    fn kit_store_git_hash(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .map_err(|error| format!("Failed to run git rev-parse: {}", error))?;
        if !output.status.success() {
            return Err(Self::kit_store_git_error(
                "git rev-parse",
                output.status,
                &output.stderr,
            ));
        }
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if hash.is_empty() {
            return Err("git rev-parse returned empty hash".to_string());
        }
        Ok(hash)
    }

    fn kit_store_install(result: &KitStoreSearchResult) -> Result<script_kit_gpui::kit_store::InstalledKit, String> {
        let repo_url = if result.clone_url.is_empty() {
            return Err("Selected kit is missing clone URL".to_string());
        } else {
            result.clone_url.clone()
        };

        let name = Self::kit_store_derive_name(&repo_url)?;
        let kits_root = script_kit_gpui::setup::get_kit_path().join("kits");
        std::fs::create_dir_all(&kits_root)
            .map_err(|error| format!("Failed to create kits directory: {}", error))?;
        let install_path = kits_root.join(&name);
        if install_path.exists() {
            return Err(format!(
                "Kit '{}' is already installed at {}",
                name,
                install_path.display()
            ));
        }

        let clone_output = Command::new("git")
            .arg("clone")
            .arg(&repo_url)
            .arg(&install_path)
            .output()
            .map_err(|error| format!("Failed to run git clone: {}", error))?;
        if !clone_output.status.success() {
            return Err(Self::kit_store_git_error(
                "git clone",
                clone_output.status,
                &clone_output.stderr,
            ));
        }

        let git_hash = Self::kit_store_git_hash(&install_path)?;
        let mut kits = Self::kit_store_list_installed();
        kits.retain(|kit| kit.name != name);

        let installed = script_kit_gpui::kit_store::InstalledKit {
            name,
            path: install_path,
            repo_url,
            git_hash,
            installed_at: chrono::Utc::now().to_rfc3339(),
        };
        kits.push(installed.clone());

        script_kit_gpui::kit_store::storage::save_kit_registry(&kits)
            .map_err(|error| format!("Failed to update kit registry: {}", error))?;

        Ok(installed)
    }

    fn kit_store_update(kit: &script_kit_gpui::kit_store::InstalledKit) -> Result<(), String> {
        let pull_output = Command::new("git")
            .arg("-C")
            .arg(&kit.path)
            .arg("pull")
            .arg("--ff-only")
            .output()
            .map_err(|error| format!("Failed to run git pull: {}", error))?;
        if !pull_output.status.success() {
            return Err(Self::kit_store_git_error(
                "git pull",
                pull_output.status,
                &pull_output.stderr,
            ));
        }

        let latest_hash = Self::kit_store_git_hash(&kit.path)?;
        let mut kits = Self::kit_store_list_installed();
        if let Some(existing) = kits.iter_mut().find(|existing| existing.name == kit.name) {
            existing.git_hash = latest_hash;
        }

        script_kit_gpui::kit_store::storage::save_kit_registry(&kits)
            .map_err(|error| format!("Failed to save updated kit registry: {}", error))
    }

    fn kit_store_remove(kit: &script_kit_gpui::kit_store::InstalledKit) -> Result<(), String> {
        if kit.path.exists() {
            std::fs::remove_dir_all(&kit.path)
                .map_err(|error| format!("Failed to remove kit directory: {}", error))?;
        }
        script_kit_gpui::kit_store::storage::remove_kit(&kit.name)
            .map_err(|error| format!("Failed to update kit registry: {}", error))
    }

    fn kit_store_refresh_installed_view(&mut self, cx: &mut Context<Self>) {
        if let AppView::InstalledKitsView {
            selected_index,
            kits,
        } = &mut self.current_view
        {
            *kits = Self::kit_store_list_installed();
            if kits.is_empty() {
                *selected_index = 0;
            } else {
                *selected_index = (*selected_index).min(kits.len().saturating_sub(1));
            }
            cx.notify();
        }
    }

    fn kit_store_install_selected_result(&mut self, selected: &KitStoreSearchResult, cx: &mut Context<Self>) {
        match Self::kit_store_install(selected) {
            Ok(installed) => {
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!("Installed kit '{}'", installed.name),
                        &self.theme,
                    )
                    .duration_ms(Some(2500)),
                );
                cx.notify();
            }
            Err(error) => {
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to install kit: {}", error),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    fn kit_store_update_selected_kit(&mut self, kit: &script_kit_gpui::kit_store::InstalledKit, cx: &mut Context<Self>) {
        match Self::kit_store_update(kit) {
            Ok(()) => {
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!("Updated kit '{}'", kit.name),
                        &self.theme,
                    )
                    .duration_ms(Some(2500)),
                );
                self.kit_store_refresh_installed_view(cx);
            }
            Err(error) => {
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to update '{}': {}", kit.name, error),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    fn kit_store_remove_selected_kit(&mut self, kit: &script_kit_gpui::kit_store::InstalledKit, cx: &mut Context<Self>) {
        match Self::kit_store_remove(kit) {
            Ok(()) => {
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!("Removed kit '{}'", kit.name),
                        &self.theme,
                    )
                    .duration_ms(Some(2500)),
                );
                self.kit_store_refresh_installed_view(cx);
            }
            Err(error) => {
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to remove '{}': {}", kit.name, error),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    fn kit_store_update_all(&mut self) -> (usize, usize) {
        let kits = Self::kit_store_list_installed();
        let mut updated = 0;
        let mut failed = 0;

        for kit in &kits {
            match Self::kit_store_update(kit) {
                Ok(()) => {
                    updated += 1;
                }
                Err(error) => {
                    failed += 1;
                    tracing::warn!(
                        "Kit Store update-all failed for '{}': {}",
                        kit.name,
                        error
                    );
                }
            }
        }

        (updated, failed)
    }

    fn render_browse_kits(
        &mut self,
        query: &str,
        selected_index: usize,
        results: Vec<KitStoreSearchResult>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let accent_subtle = self.theme.colors.accent.selected_subtle;
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;

        let query_owned = query.to_string();
        let input_display = if query_owned.is_empty() {
            SharedString::from("Search GitHub kits...")
        } else {
            SharedString::from(query_owned.clone())
        };
        let input_is_empty = query_owned.is_empty();
        let total_results = results.len();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                if has_cmd && key == "w" {
                    this.close_and_reset_window(cx);
                    return;
                }

                // Handle browse view state transitions from keyboard.
                let mut next_query: Option<String> = None;
                let mut install_selected = false;

                if let AppView::BrowseKitsView {
                    query,
                    selected_index,
                    results,
                } = &mut this.current_view
                {
                    match key.as_str() {
                        "escape" => {
                            if query.is_empty() {
                                this.go_back_or_close(window, cx);
                            } else {
                                next_query = Some(String::new());
                            }
                        }
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < results.len().saturating_sub(1) {
                                *selected_index += 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            install_selected = true;
                        }
                        "backspace" => {
                            let mut updated = query.clone();
                            if !updated.is_empty() {
                                updated.pop();
                                next_query = Some(updated);
                            }
                        }
                        _ => {
                            if !has_cmd {
                                if let Some(key_char) = &event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            let mut updated = query.clone();
                                            updated.push(ch);
                                            next_query = Some(updated);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(next_query) = next_query {
                    let next_results = Self::kit_store_search_results(&next_query);
                    if let AppView::BrowseKitsView {
                        query,
                        selected_index,
                        results,
                    } = &mut this.current_view
                    {
                        *query = next_query;
                        *selected_index = 0;
                        *results = next_results;
                        cx.notify();
                    }
                } else if install_selected {
                    let selected = if let AppView::BrowseKitsView {
                        selected_index,
                        results,
                        ..
                    } = &this.current_view
                    {
                        results.get(*selected_index).cloned()
                    } else {
                        None
                    };
                    if let Some(selected) = selected {
                        this.kit_store_install_selected_result(&selected, cx);
                    }
                }
            },
        );

        let selected_row = selected_index;
        let hovered_row = self.hovered_index;
        let input_mode = self.input_mode;
        let click_entity = cx.entity().downgrade();
        let install_entity = cx.entity().downgrade();
        let hover_entity = cx.entity().downgrade();
        let results_for_list = results.clone();

        let list: AnyElement = if results_for_list.is_empty() {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .text_color(rgb(text_muted))
                .child("No kits found")
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child("Try a different search query"),
                )
                .into_any_element()
        } else {
            uniform_list(
                "kit-store-browse-list",
                results_for_list.len(),
                move |visible, _window, _cx| {
                    visible
                        .map(|ix| {
                            if let Some(result) = results_for_list.get(ix) {
                                let is_selected = ix == selected_row;
                                let is_hovered =
                                    hovered_row == Some(ix) && input_mode == InputMode::Mouse;
                                let row_bg = rgba((accent_subtle << 8) | selected_alpha);

                                let row_entity = click_entity.clone();
                                let row_hover_entity = hover_entity.clone();
                                let install_btn_entity = install_entity.clone();
                                let result_for_install = result.clone();

                                div()
                                    .id(ElementId::NamedInteger("kit-store-browse-row".into(), ix as u64))
                                    .w_full()
                                    .h(px(72.0))
                                    .px(px(12.0))
                                    .py(px(8.0))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(12.0))
                                    .when(is_selected || is_hovered, |row| row.bg(row_bg))
                                    .cursor_pointer()
                                    .on_hover(move |is_hovered, _window, cx| {
                                        if let Some(entity) = row_hover_entity.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                if *is_hovered {
                                                    this.input_mode = InputMode::Mouse;
                                                    if this.hovered_index != Some(ix) {
                                                        this.hovered_index = Some(ix);
                                                        cx.notify();
                                                    }
                                                } else if this.hovered_index == Some(ix) {
                                                    this.hovered_index = None;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                    })
                                    .on_click(move |_event, _window, cx| {
                                        if let Some(entity) = row_entity.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                if let AppView::BrowseKitsView { selected_index, .. } =
                                                    &mut this.current_view
                                                {
                                                    *selected_index = ix;
                                                }
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(text_primary))
                                                    .child(format!(
                                                        "{}  •  ★ {}",
                                                        result.name, result.stars
                                                    )),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(result.full_name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_muted))
                                                    .child(if result.description.is_empty() {
                                                        "No description".to_string()
                                                    } else {
                                                        result.description.clone()
                                                    }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px(px(10.0))
                                            .py(px(6.0))
                                            .rounded(px(6.0))
                                            .bg(rgba((accent_subtle << 8) | 0x80))
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(text_primary))
                                            .cursor_pointer()
                                            .on_click(move |_event, _window, cx| {
                                                if let Some(entity) = install_btn_entity.upgrade() {
                                                    let result_for_install =
                                                        result_for_install.clone();
                                                    entity.update(cx, |this, cx| {
                                                        this.kit_store_install_selected_result(
                                                            &result_for_install,
                                                            cx,
                                                        );
                                                    });
                                                }
                                            })
                                            .child("Install"),
                                    )
                            } else {
                                div().id(ix).h(px(72.0))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .rounded(px(design_visual.radius_lg))
            .font_family(design_typography.font_family)
            .text_color(rgb(text_primary))
            .key_context("kit_store_browse")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("🧰 Browse Kit Store"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} kits", total_results)),
                    ),
            )
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .py(px(design_spacing.padding_xs))
                    .child(list),
            )
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Install")
                    .primary_shortcut("↵")
                    .show_secondary(true)
                    .secondary_label("Back")
                    .secondary_shortcut("esc"),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }

    fn render_installed_kits(
        &mut self,
        selected_index: usize,
        kits: Vec<script_kit_gpui::kit_store::InstalledKit>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let accent_subtle = self.theme.colors.accent.selected_subtle;
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let total_kits = kits.len();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                if key == "escape" {
                    this.go_back_or_close(window, cx);
                    return;
                }
                if has_cmd && key == "w" {
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::InstalledKitsView {
                    selected_index,
                    kits,
                } = &mut this.current_view
                {
                    match key.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < kits.len().saturating_sub(1) {
                                *selected_index += 1;
                                this.list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            let selected = kits.get(*selected_index).cloned();
                            if let Some(selected) = selected {
                                this.kit_store_update_selected_kit(&selected, cx);
                            }
                        }
                        "delete" | "backspace" => {
                            let selected = kits.get(*selected_index).cloned();
                            if let Some(selected) = selected {
                                this.kit_store_remove_selected_kit(&selected, cx);
                            }
                        }
                        _ => {}
                    }
                }
            },
        );

        let selected_row = selected_index;
        let hovered_row = self.hovered_index;
        let input_mode = self.input_mode;
        let click_entity = cx.entity().downgrade();
        let update_entity = cx.entity().downgrade();
        let remove_entity = cx.entity().downgrade();
        let hover_entity = cx.entity().downgrade();
        let kits_for_list = kits.clone();

        let list: AnyElement = if kits_for_list.is_empty() {
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .text_color(rgb(text_muted))
                .child("No installed kits")
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child("Use \"Browse Kit Store\" to install one"),
                )
                .into_any_element()
        } else {
            uniform_list(
                "kit-store-installed-list",
                kits_for_list.len(),
                move |visible, _window, _cx| {
                    visible
                        .map(|ix| {
                            if let Some(kit) = kits_for_list.get(ix) {
                                let is_selected = ix == selected_row;
                                let is_hovered =
                                    hovered_row == Some(ix) && input_mode == InputMode::Mouse;
                                let row_bg = rgba((accent_subtle << 8) | selected_alpha);

                                let row_entity = click_entity.clone();
                                let row_hover_entity = hover_entity.clone();
                                let update_btn_entity = update_entity.clone();
                                let remove_btn_entity = remove_entity.clone();
                                let kit_for_update = kit.clone();
                                let kit_for_remove = kit.clone();

                                div()
                                    .id(ElementId::NamedInteger("kit-store-installed-row".into(), ix as u64))
                                    .w_full()
                                    .h(px(76.0))
                                    .px(px(12.0))
                                    .py(px(8.0))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(12.0))
                                    .when(is_selected || is_hovered, |row| row.bg(row_bg))
                                    .cursor_pointer()
                                    .on_hover(move |is_hovered, _window, cx| {
                                        if let Some(entity) = row_hover_entity.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                if *is_hovered {
                                                    this.input_mode = InputMode::Mouse;
                                                    if this.hovered_index != Some(ix) {
                                                        this.hovered_index = Some(ix);
                                                        cx.notify();
                                                    }
                                                } else if this.hovered_index == Some(ix) {
                                                    this.hovered_index = None;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                    })
                                    .on_click(move |_event, _window, cx| {
                                        if let Some(entity) = row_entity.upgrade() {
                                            entity.update(cx, |this, cx| {
                                                if let AppView::InstalledKitsView { selected_index, .. } =
                                                    &mut this.current_view
                                                {
                                                    *selected_index = ix;
                                                }
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(text_primary))
                                                    .child(kit.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(kit.repo_url.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_muted))
                                                    .child(format!("commit {}", kit.git_hash)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .px(px(10.0))
                                                    .py(px(6.0))
                                                    .rounded(px(6.0))
                                                    .bg(rgba((accent_subtle << 8) | 0x80))
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(text_primary))
                                                    .cursor_pointer()
                                                    .on_click(move |_event, _window, cx| {
                                                        if let Some(entity) = update_btn_entity.upgrade() {
                                                            let kit_for_update = kit_for_update.clone();
                                                            entity.update(cx, |this, cx| {
                                                                this.kit_store_update_selected_kit(
                                                                    &kit_for_update,
                                                                    cx,
                                                                );
                                                            });
                                                        }
                                                    })
                                                    .child("Update"),
                                            )
                                            .child(
                                                div()
                                                    .px(px(10.0))
                                                    .py(px(6.0))
                                                    .rounded(px(6.0))
                                                    .bg(rgba((ui_border << 8) | 0xA0))
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(text_primary))
                                                    .cursor_pointer()
                                                    .on_click(move |_event, _window, cx| {
                                                        if let Some(entity) = remove_btn_entity.upgrade() {
                                                            let kit_for_remove = kit_for_remove.clone();
                                                            entity.update(cx, |this, cx| {
                                                                this.kit_store_remove_selected_kit(
                                                                    &kit_for_remove,
                                                                    cx,
                                                                );
                                                            });
                                                        }
                                                    })
                                                    .child("Remove"),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(76.0))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .rounded(px(design_visual.radius_lg))
            .font_family(design_typography.font_family)
            .text_color(rgb(text_primary))
            .key_context("kit_store_installed")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("📦 Installed Kits"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} installed", total_kits)),
                    ),
            )
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .py(px(design_spacing.padding_xs))
                    .child(list),
            )
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Update")
                    .primary_shortcut("↵")
                    .show_secondary(true)
                    .secondary_label("Remove")
                    .secondary_shortcut("⌫"),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
