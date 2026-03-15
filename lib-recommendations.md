# Library Recommendations

Rust crates this project should consider adopting, ranked by impact.
Each section lists the crate, what it replaces, and the specific files where it would help most.

---

## Tier 1 — High ROI, Low Risk

### 1. `itertools` — Iterator combinators

**Add:** `itertools = "0.14"`

Eliminates `.collect::<Vec<_>>()` intermediaries across 50+ call sites.

#### `.join()` directly on iterators (20+ sites)

| File | Current pattern |
|------|-----------------|
| `src/app_impl/filtering_cache.rs` | `.collect::<Vec<_>>().join("\n")` |
| `src/render_builtins/kit_store.rs` | `.collect::<Vec<_>>().join("+")` |
| `src/ai/script_generation.rs` | `.collect::<Vec<_>>().join(" ")` |
| `src/executor/stderr_buffer.rs` | `.collect::<Vec<_>>().join("\n")` |
| `src/app_actions/handle_action/emoji.rs` | `.collect::<Vec<_>>().join(" ")` and `.join("")` |
| `src/actions/builders/shared.rs` | `.collect::<Vec<_>>().join("-")` |
| `src/extension_types/mod.rs` | `.collect::<Vec<_>>().join("-")` |
| `src/kit_store/browser.rs` | `.collect::<Vec<_>>().join("+")` |
| `src/scripts/scriptlet_loader/parsing.rs` | `.collect::<Vec<_>>().join("-")` |
| `src/app_execute/builtin_execution.rs` | `.collect::<Vec<_>>().join(", ")` |
| `src/actions/command_bar.rs` | `.collect::<Vec<_>>().join(", ")` |
| `src/notes/window/navigation.rs` | `.collect::<Vec<_>>().join(" ")` (x2) |
| `src/designs/core/metadata.rs` | `.collect::<Vec<_>>().join(" · ")` (x3) |
| `src/calculator.rs` | `.collect::<Vec<_>>().join(" ")` (x3) |
| `src/ai/providers.rs` | `.collect::<Vec<_>>().join("")` (x2) |
| `src/notes/code_highlight.rs` | `.collect::<Vec<_>>().join("\n")` |

All become: `.join("...")` with `use itertools::Itertools;`

#### `.sorted()` / `.sorted_by()` / `.dedup()` on iterators (30+ sites)

| File | Current pattern |
|------|-----------------|
| `src/storybook/registry.rs` | `.collect()` then `sort()` + `dedup()` |
| `src/stdin_commands/mod.rs` | `.collect()` then `sort()` + `dedup()` |
| `src/scriptlets/mod.rs` | `.collect()` then `sort()` + `dedup()` |
| `src/hud_manager/mod.rs` | `.clone()` then `sort()` + `dedup()` |
| `src/process_manager/mod.rs` | `.collect()` then `sort_by()` |
| `src/scripts/search/windows.rs` | `.collect()` then `sort_by()` |
| `src/kit_store/discover.rs` | `.collect()` then `sort()` |
| `src/scripts/grouping/search_mode.rs` | `.collect()` then `sort_by()` |
| `src/prompts/select/prompt.rs` | `.collect()` then `sort_by()` |
| `src/actions/dialog_*_tests.rs` | `.collect()` then `sort()` + `dedup()` (many test files) |

All become: `.sorted()`, `.sorted_by(...)`, or `.sorted().dedup()` chains.

#### `into_group_map()` for manual grouping

| File | Current pattern |
|------|-----------------|
| `src/clipboard_history/types.rs` | `HashMap::new()` + loop with `.entry(group).or_default().push(entry)` |
| `src/scripts/grouping/grouped_view.rs` | `HashMap<String, Vec<usize>>` + loop with `.entry(kit).or_default().push(idx)` |

Both become: `.into_group_map_by(|e| key_fn(e))`

#### `.chunks()` on iterators

| File | Current pattern |
|------|-----------------|
| `src/ai/storage.rs` | `.collect::<Vec<_>>()` then `.chunks(N)` on the vec |

Becomes: `itertools::Itertools::chunks()` directly on the iterator.

---

### 2. `strum` — Enum derive macros

**Add:** `strum = { version = "0.26", features = ["derive"] }`

Note: Already a transitive dep via `gpui_macos`; adding it directly costs zero extra compile time.

#### `#[derive(strum::Display)]` — replace `as_str()` / `impl Display`

| File | Enum | Current boilerplate |
|------|------|---------------------|
| `src/mcp_streaming/mod.rs` | `SseEventType` | Manual `as_str()` + `impl Display` |
| `src/window_state/mod.rs` | `WindowRole` | Manual `as_str()` + `name()` |
| `src/clipboard_history/types.rs` | `ContentType` | Manual `as_str()` + `from_str()` |
| `src/ai/model.rs` | `MessageRole` | Manual `as_str()`, `parse()`, `FromStr`, `Display` |
| `src/ai/model.rs` | `ChatSource` | Manual `as_str()` + `parse()` |
| `src/mcp_protocol/mod.rs` | `McpMethod` | Manual `from_str()` + `as_str()` |
| `src/app_impl/prompt_ai.rs` | `AiScriptGenerationStage` | Manual `as_str()` |
| `src/prompts/naming/validation.rs` | `NamingTarget` | Manual `as_str()` |
| `src/app_actions/handle_action/paste.rs` | `PasteCloseBehavior` | Manual `as_str()` |
| `src/action_helpers.rs` | `DispatchSurface` | Manual `impl Display` |
| `src/action_helpers.rs` | `ActionOutcomeStatus` | Manual `impl Display` |
| `src/theme/types.rs` | `VibrancyMaterial` | Manual `impl Display` |
| `src/logging/mod.rs` | `LegacyLogLevel` | Manual `as_json_label()` |
| `src/emoji/mod.rs` | `EmojiCategory` | Manual `display_name()` |
| `src/designs/separator_variations/category.rs` | `SeparatorCategory` | Manual Display match |
| `src/designs/core/variant.rs` | `DesignVariant` | Manual Display match |

#### `#[derive(strum::EnumIter)]` — replace manual `ALL_VARIANTS` arrays

| File | Enum | Current boilerplate |
|------|------|---------------------|
| `src/emoji/mod.rs` | `EmojiCategory` | `ALL_CATEGORIES` const array |
| `src/notes/actions_panel.rs` | `NotesAction` | `all()` method listing every variant |

---

### 3. `humantime` + `humansize` — Human-readable formatting

**Add:** `humantime = "2"` and `humansize = "2"`

#### Relative time formatting (5 duplicate implementations)

| File | Function | Pattern |
|------|----------|---------|
| `src/file_search/mod.rs` | `format_relative_time()` | Manual "X min(s) ago", "X hour(s) ago" |
| `src/notes/window/navigation.rs` | `format_relative_time()` | "just now", "Xs ago", "Xm ago", "Xh ago" |
| `src/prompts/env/helpers.rs` | `format_relative_time()` | Same manual implementation |
| `src/render_builtins/clipboard_history_list.rs` | Inline | "just now", `format!("{}m ago", ...)` |
| `src/ai/window/render_message.rs` | Inline | `format!("{}m ago", ...)`, `format!("{}h ago", ...)` |

#### Duration display

| File | Pattern |
|------|---------|
| `src/ai/window/render_message_actions.rs` | `format!("{}ms", dur.as_millis())` / `format!("{}s", secs)` |
| `src/ai/session.rs` | `format!("{:.3}", delay_ms as f64 / 1000.0)` |

#### File size formatting (2 duplicate implementations)

| File | Function | Pattern |
|------|----------|---------|
| `src/file_search/mod.rs` | `format_file_size()` | Manual KB/MB/GB formatting |
| `src/stories/drop_prompt_stories/split.rs` | `format_file_size()` | Same manual implementation |
| `src/render_builtins/file_search.rs` | Inline | `format!("({size_mb:.1} MB)")` |
| `src/render_builtins/file_search_list.rs` | Uses `format_file_size()` | |
| `src/render_builtins/file_search_preview.rs` | Uses `format_file_size()` | |

---

## Tier 2 — Medium ROI, Low-Medium Risk

### 4. `indexmap` — Ordered HashMap

**Add:** `indexmap = "2"`

Best fits where insertion-order iteration AND key lookup are both needed.

| File | Current pattern | Improvement |
|------|-----------------|-------------|
| `src/shortcuts/registry.rs` | `Vec<ShortcutBinding>` + `HashMap<String, usize>` (index) | Single `IndexMap<String, ShortcutBinding>` |
| `src/main_sections/app_state.rs` | `alias_registry`, `action_shortcuts` with "first-registered wins" | `IndexMap` preserves insertion order naturally |
| `src/emoji/mod.rs` | `grouped_emojis() -> Vec<(EmojiCategory, Vec<...>)>` | `IndexMap<EmojiCategory, Vec<...>>` |
| `src/keyword_manager/mod.rs` | `list_triggers() -> Vec<(String, String)>` | `IndexMap<String, String>` |
| `src/snippet/mod.rs` | `BTreeMap<usize, TabstopInfo>` for tabstop ordering | `IndexMap` — faster than BTreeMap for small N |
| `src/protocol/types/input.rs` | `BTreeMap<String, serde_json::Value>` for `extra` | `IndexMap` with `serde` feature for deterministic JSON |

### 5. `dashmap` — Concurrent HashMap

**Add:** `dashmap = "6"`

Best fits for read-heavy caches currently behind `Mutex<HashMap<...>>`.

| File | Current pattern | Improvement |
|------|-----------------|-------------|
| `src/keyword_manager/mod.rs` | `Arc<Mutex<HashMap<String, KeywordScriptlet>>>` | `DashMap` — concurrent reads without locking |
| `src/scheduler/mod.rs` | `Arc<Mutex<HashMap<PathBuf, ScheduledScript>>>` | `DashMap` for concurrent scheduler access |
| `src/ai/session.rs` | `Mutex<HashMap<String, Arc<Mutex<ClaudeSession>>>>` | `DashMap` for concurrent session lookup |
| `src/notes/code_highlight.rs` | `OnceLock<Mutex<HashMap<u64, Vec<CodeLine>>>>` | `OnceLock<DashMap<...>>` — lock-free cache reads |
| `src/prompts/markdown/scope.rs` | `OnceLock<Mutex<HashMap<...>>>` | `OnceLock<DashMap<...>>` |
| `src/window_control/cache.rs` | `OnceLock<Mutex<HashMap<u32, usize>>>` | `OnceLock<DashMap<...>>` |
| `src/process_manager/mod.rs` | `RwLock<HashMap<u32, ProcessInfo>>` | `DashMap` — simpler API, similar performance |
| `src/secrets.rs` | `OnceLock<Mutex<Option<HashMap<...>>>>` | `OnceLock<DashMap<...>>` |
| `src/shortcuts/persistence.rs` | `OnceLock<Mutex<Option<HashMap<...>>>>` | `OnceLock<DashMap<...>>` |
| `src/aliases/persistence.rs` | `OnceLock<Mutex<Option<HashMap<...>>>>` | `OnceLock<DashMap<...>>` |

### 6. `bytes` — Zero-copy byte buffers

**Add:** `bytes = "1"`

Replaces `Vec<u8>` cloning in image/clipboard/protocol paths.

| File | Current pattern | Improvement |
|------|-----------------|-------------|
| `src/clipboard_history/image.rs` | `image.bytes.to_vec()` (arboard image data) | `Bytes::from(...)` — ref-counted, no copy on share |
| `src/clipboard_history/blob_store.rs` | `load_blob() -> Option<Vec<u8>>` | `-> Option<Bytes>` for ref-counted blob sharing |
| `src/platform/ai_commands.rs` | `FocusedWindowCapture { png_data: Vec<u8> }` | `Bytes` — screenshot data shared without cloning |
| `src/platform/screen_capture_overlay.rs` | `png_data: Vec<u8>` + `std::fs::read()` | `Bytes` for loaded PNG data |
| `src/app_launcher/scanning.rs` | `slice::from_raw_parts(...).to_vec()` (ObjC NSData) | `Bytes::copy_from_slice(...)` for ref-counted sharing |
| `src/execute_script/mod.rs` | `img.bytes.to_vec()` (clipboard to base64) | `Bytes` to avoid copy |
| `src/terminal/alacritty/handle_creation.rs` | `buffer[..n].to_vec()` per PTY read | `Bytes::copy_from_slice(...)` for per-read efficiency |
| `src/ocr.rs` | `extract_text_async(..., rgba_data: Vec<u8>)` | `Bytes` for RGBA data passed to async |
| `src/tray/mod.rs` | `render_svg_to_rgba() -> Vec<u8>` | `Bytes` if result is shared |

---

## Tier 3 — Nice to Have

### 7. `notify-debouncer-full` — Built-in file watch debouncing

**Add:** `notify-debouncer-full = "0.5"` (replaces manual debounce)

| File | Current pattern |
|------|-----------------|
| `src/watcher/mod.rs` | Custom debounce via `HashMap<PathBuf, (Event, Instant)>`, `next_deadline()`, `flush_expired()` |
| `src/watcher/generic.rs` | Manual `notify::recommended_watcher` with custom debounce loop |
| `src/config/types.rs` | `WatcherConfig` with `debounce_ms`, `storm_threshold` |

The custom debouncer works but is ~200 lines that `notify-debouncer-full` handles out of the box.
**Caveat:** The existing debouncer has project-specific storm detection; migrating requires mapping those semantics.

### 8. `compact_str` (or `smol_str`) — Inline small strings

**Add:** `compact_str = "0.8"` or `smol_str = "0.3"`

Inlines strings ≤ 24 bytes on the stack (no heap allocation). Most useful in hot search/render paths.

| File | Pattern | Why it helps |
|------|---------|--------------|
| `src/scripts/search/scripts.rs` | `query.to_lowercase()`, `extract_filename()` per item | Filenames/queries usually < 24 bytes |
| `src/scripts/search/apps.rs` | `nucleo.score(&app.name)` per app | App names are short |
| `src/scripts/search/builtins.rs` | `nucleo.score(&entry.name)` per entry | Builtin names are short |
| `src/list_item/mod.rs` | `self.name.to_string()`, `shortcut.to_string()` in render | Per-frame string allocs |
| `src/render_builtins/clipboard_history_list.rs` | `"just now".to_string()` per list item | Static strings cloned per frame |
| `src/keyword_manager/mod.rs` | `trigger.to_string()`, `name.to_string()` | Keywords/triggers are short |
| `src/actions/command_bar.rs` | `id`, `title` fields in `GroupedActionItem` | Action IDs/titles are short |

**Caveat:** Requires touching struct definitions; highest effort of all recommendations.

### 9. `std::sync::LazyLock` (or `once_cell::sync::Lazy`) — Simpler statics

**No new dependency needed** — `std::sync::LazyLock` is stable since Rust 1.80.

40+ `OnceLock` + `.get_or_init(...)` patterns could become `LazyLock::new(|| ...)`:

| File | Static |
|------|--------|
| `src/notes/code_highlight.rs` | `SYNTAX_SET`, `DARK_THEME`, `LIGHT_THEME`, `HIGHLIGHT_CACHE` |
| `src/theme/types.rs` | `APPEARANCE_CACHE`, `THEME_CACHE` |
| `src/theme/presets.rs` | `PRESETS_CACHE` |
| `src/lib.rs` | `SHOW_WINDOW_CHANNEL` |
| `src/keystroke_logger.rs` | `KEYSTROKE_LOGGER` |
| `src/hud_manager/mod.rs` | `HUD_MANAGER` |
| `src/windows/registry.rs` | `REGISTRY` |
| `src/window_manager/mod.rs` | `WINDOW_MANAGER` |
| `src/hotkeys/mod.rs` | `HOTKEY_ROUTES`, `MAIN_MANAGER`, `SCRIPT_HOTKEY_MANAGER` |
| `src/keyword_manager/mod.rs` | `KEYWORD_MANAGER` |
| `src/window_control/cache.rs` | `WINDOW_CACHE` |
| `src/secrets.rs` | `SECRETS_CACHE` |
| `src/shortcuts/persistence.rs` | `SHORTCUT_OVERRIDES_CACHE` |
| `src/aliases/persistence.rs` | `ALIAS_OVERRIDES_CACHE` |
| `src/menu_cache/mod.rs` | `MENU_CACHE_DB` |
| `src/stories/mod.rs` | `ALL_STORIES` |
| `src/clipboard_history/db_worker/mod.rs` | `DB_SENDER`, `WORKER_STARTED` |
| `src/prompts/markdown/scope.rs` | `MARKDOWN_CACHE` |
| `src/app_launcher/scanning.rs` | `APP_CACHE` |
| `src/keyboard_monitor/mod.rs` | `DEBOUNCED_LOG` |

---

## Not Recommended

| Crate | Why not |
|-------|---------|
| `tokio` | GPUI has its own async executor; tokio would conflict |
| `reqwest` | `ureq` is the right choice for blocking HTTP in this architecture |
| `clap` | Uses stdin JSON protocol, not CLI args |
| `egui` / `iced` | Committed to GPUI |
| `time` | `chrono` works fine and is deeply integrated; switching adds churn |

---

## Summary

| Crate | Effort | Sites affected | Risk |
|-------|--------|----------------|------|
| `itertools` | Low | 50+ | Negligible |
| `strum` | Low | 16 enums | Negligible |
| `humantime` + `humansize` | Low | 15+ | Negligible |
| `indexmap` | Low-Med | 6-10 | Low |
| `dashmap` | Medium | 10+ | Low |
| `bytes` | Medium | 10+ | Low |
| `notify-debouncer-full` | Medium | 3 files | Medium |
| `compact_str` | High | 20+ structs | Medium |
| `LazyLock` (std) | Low | 40+ statics | Negligible |
