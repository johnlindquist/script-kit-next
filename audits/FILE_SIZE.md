# File Size Audit

**Generated:** 2026-01-03  
**Threshold:** 1000 lines

## Summary

Found **40 files** over 1000 lines. Total: ~68,000+ lines in oversized files.

| Category | Count | Lines |
|----------|-------|-------|
| Test files (*_tests.rs) | 4 | ~12,500 |
| Core app files | 8 | ~17,500 |
| Feature modules | 15 | ~24,000 |
| Utility/infrastructure | 7 | ~10,500 |
| Protocol/data | 3 | ~4,000 |
| UI/rendering | 3 | ~4,500 |

---

## Priority 1: Critical Splits (>2000 lines, high complexity)

### 1. `src/scripts_tests.rs` - 4,941 lines

**Current:** Monolithic test file for script loading/parsing

**Recommended split:**
```
src/scripts/
  mod.rs              # Re-exports
  types.rs            # Script, Scriptlet, SearchResult types (~200 lines)
  metadata.rs         # extract_script_metadata, parse_metadata_line (~150 lines)
  scriptlet_parser.rs # parse_scriptlet_section, read_scriptlets (~300 lines)
  search.rs           # fuzzy_search_* functions (~600 lines)
  grouping.rs         # get_grouped_results, TimeGroup (~200 lines)
  scheduling.rs       # register_scheduled_scripts (~100 lines)
  tests/
    mod.rs            # Test utilities
    metadata_tests.rs
    scriptlet_tests.rs
    search_tests.rs
    grouping_tests.rs
```

### 2. `src/app_impl.rs` - 3,289 lines

**Current:** Single impl block for ScriptListApp

**Recommended split:**
```
src/app/
  mod.rs              # Re-exports
  state.rs            # App state management, fields
  event_handlers.rs   # Keyboard, mouse, focus events
  message_handlers.rs # Protocol message handling
  script_execution.rs # Script launching logic
  ui_state.rs         # View switching, prompts
```

### 3. `src/render_prompts.rs` - 2,465 lines

**Current:** Rendering logic for all prompt types

**Recommended split:**
```
src/prompts/
  mod.rs              # Re-exports, PromptType enum
  arg.rs              # ArgPrompt rendering
  div.rs              # Already exists (1,059 lines) - may need further split
  editor.rs           # EditorPrompt rendering  
  form.rs             # Form/fields rendering
  term.rs             # Terminal prompt rendering
  shared.rs           # Shared rendering utilities
```

### 4. `src/main.rs` - 2,357 lines

**Current:** Entry point with many concerns mixed

**Recommended split:**
```
src/
  main.rs             # Entry point only (~100 lines)
  app/
    mod.rs            # App setup, window creation
    focus.rs          # Focusable impl
    view.rs           # AppView enum
    messages.rs       # PromptMessage, FocusedInput enums
```

### 5. `src/executor.rs` - 2,264 lines

**Current:** Script execution with multiple distinct sections

**Has clear section markers! Recommended split:**
```
src/executor/
  mod.rs              # Re-exports
  auto_submit.rs      # AutoSubmitConfig, is_auto_submit_enabled (~250 lines)
  session.rs          # ScriptSession, SplitSession, ProcessHandle (~400 lines)
  runner.rs           # execute_script_interactive, spawn_script (~350 lines)
  errors.rs           # CrashInfo, parse_stack_trace, generate_suggestions (~350 lines)
  scriptlet.rs        # ScriptletExecOptions, run_scriptlet, tool executors (~700 lines)
  selected_text.rs    # SelectedTextHandleResult, handle_selected_text_message (~200 lines)
```

---

## Priority 2: High Value Splits (1500-2200 lines)

### 6. `src/scripts.rs` - 2,036 lines

**Current:** Script loading + fuzzy search mixed

**Recommended split:**
```
src/scripts/
  mod.rs              # Re-exports  
  types.rs            # Script, Scriptlet structs
  loader.rs           # read_scripts, read_scripts_from_dir
  scriptlet_loader.rs # read_scriptlets, load_scriptlets
  metadata.rs         # extract_script_metadata, ScheduleMetadata
  search/
    mod.rs            # fuzzy_search_unified_* functions
    scripts.rs        # fuzzy_search_scripts
    scriptlets.rs     # fuzzy_search_scriptlets
    builtins.rs       # fuzzy_search_builtins
    apps.rs           # fuzzy_search_apps
    windows.rs        # fuzzy_search_windows
    indices.rs        # compute_match_indices_for_result
  grouping.rs         # get_grouped_results
```

### 7. `src/notes/window.rs` - 2,041 lines

**Current:** Full Notes window implementation

**Recommended split:**
```
src/notes/
  mod.rs              # Re-exports
  window.rs           # NotesApp struct, window management (~300 lines)
  render.rs           # Render impl (~500 lines)
  events.rs           # Event handlers (~400 lines)
  sidebar.rs          # Sidebar rendering (~300 lines)
  editor.rs           # Note editor rendering (~300 lines)
  dialogs.rs          # Export, confirmation dialogs
```

### 8. `src/config.rs` - 2,024 lines

**Current:** All config types + loading

**Recommended split:**
```
src/config/
  mod.rs              # Re-exports, load_config
  types.rs            # Config, BuiltInConfig, ProcessLimits, etc.
  defaults.rs         # Default implementations
  hotkey.rs           # HotkeyConfig
  commands.rs         # CommandConfig
  padding.rs          # ContentPadding
  frecency.rs         # FrecencyConfig
```

### 9. `src/utils.rs` - 1,969 lines

**Current:** HTML parsing + assets + path rendering + Tailwind parsing

**Has clear section markers! Recommended split:**
```
src/utils/
  mod.rs              # Re-exports
  html.rs             # strip_html_tags, HtmlParser, HtmlElement (~550 lines)
  assets.rs           # get_asset_path, get_logo_path (~80 lines)
  paths.rs            # render_path_with_highlights (~100 lines)
  tailwind.rs         # TailwindStyles, parse_color (~900 lines)
```

### 10. `src/ai/window.rs` - 1,968 lines

**Similar to notes/window.rs - split pattern:**
```
src/ai/
  window.rs           # AiApp struct, window management
  render.rs           # Render impl
  events.rs           # Event handlers
  chat_view.rs        # Chat message rendering
  input.rs            # Input area
  model_picker.rs     # Model selection
```

### 11. `src/clipboard_history.rs` - 1,924 lines

**Current:** Database + cache + monitoring + image handling

**Recommended split:**
```
src/clipboard/
  mod.rs              # Re-exports, init_clipboard_history
  types.rs            # ClipboardEntry, ContentType, TimeGroup
  database.rs         # SQLite operations, get_connection
  cache.rs            # LRU cache, get_cached_image, get_cached_entries
  monitor.rs          # clipboard_monitor_loop, stop_clipboard_monitoring
  images.rs           # encode_image_*, decode_*, get_image_dimensions
  time_grouping.rs    # classify_timestamp, group_entries_by_time
```

### 12. `src/theme.rs` - 1,833 lines

**Has clear section markers! Recommended split:**
```
src/theme/
  mod.rs              # Re-exports, load_theme
  types.rs            # Theme, ColorScheme, BackgroundColors, etc.
  defaults.rs         # Default impls
  colors.rs           # ListItemColors, InputFieldColors
  detection.rs        # detect_system_appearance
  gpui_mapping.rs     # hex_to_hsla, map_scriptkit_to_gpui_theme
  serde.rs            # hex_color_serde, hex_color_option_serde modules
```

### 13. `src/actions.rs` - 1,815 lines

**Current:** Actions dialog + script creation mixed

**Has test section markers! Recommended split:**
```
src/actions/
  mod.rs              # Re-exports
  types.rs            # Action, ActionCategory, ScriptInfo
  context.rs          # get_path_context_actions, get_script_context_actions
  global.rs           # get_global_actions
  dialog.rs           # ActionsDialog struct + impl (~600 lines)
  render.rs           # Render impl (~400 lines)
  script_creation.rs  # validate_script_name, create_script_file, generate_script_template
```

---

## Priority 3: Moderate Splits (1200-1500 lines)

### 14. `src/designs/traits.rs` - 1,651 lines

**Recommendation:** Split by design variant types
```
src/designs/
  traits.rs           # Core traits only (~200 lines)
  base.rs             # Base design implementations
  default.rs          # Default variant
  compact.rs          # Compact variant
  minimal.rs          # Minimal variant
```

### 15. `src/terminal/alacritty.rs` - 1,557 lines

**Recommendation:** Terminal emulation is complex - consider:
```
src/terminal/
  alacritty.rs        # Core terminal struct
  grid.rs             # Grid/cell management
  parser.rs           # ANSI escape parsing
  rendering.rs        # Terminal rendering
```

### 16. `src/setup.rs` - 1,534 lines

**Recommendation:** Split setup phases
```
src/setup/
  mod.rs              # ensure_kit_setup main function
  migration.rs        # migrate_from_kenv
  directories.rs      # ensure_dir, directory creation
  files.rs            # write_string_if_*, sample files
  tsconfig.rs         # ensure_tsconfig_paths
  discovery.rs        # bun_is_discoverable
```

### 17. `src/logging.rs` - 1,524 lines

**Has clear section markers! Recommended split:**
```
src/logging/
  mod.rs              # init(), LoggingGuard
  formatter.rs        # CompactAiFormatter
  categories.rs       # category_to_code, infer_category_from_target
  buffer.rs           # log buffer, get_recent_logs
  events/
    mod.rs            # Re-exports
    script.rs         # log_script_event
    ui.rs             # log_ui_event
    key.rs            # log_key_event
    perf.rs           # log_perf
    mouse.rs          # log_mouse_*
    scroll.rs         # log_scroll_*
```

### 18. `src/term_prompt.rs` - 1,456 lines

**Recommendation:**
```
src/prompts/
  term/
    mod.rs            # TermPrompt struct
    render.rs         # Render impl
    input.rs          # Input handling
    pty.rs            # PTY management
```

### 19. `src/mcp_protocol.rs` - 1,455 lines

**Has test section markers! Recommendation:**
```
src/mcp/
  mod.rs              # Re-exports
  protocol.rs         # JsonRpcRequest/Response, McpMethod
  handlers.rs         # handle_request, handle_initialize
  tools.rs            # handle_tools_list, handle_tools_call
  resources.rs        # handle_resources_list, handle_resources_read
  capabilities.rs     # McpCapabilities, ServerInfo
```

### 20. `src/components/form_fields.rs` - 1,450 lines

**Recommendation:**
```
src/components/forms/
  mod.rs              # Re-exports
  text_field.rs       # Text input field
  select.rs           # Dropdown select
  checkbox.rs         # Checkbox field
  textarea.rs         # Multiline text
  number.rs           # Number input
  validation.rs       # Field validation
```

---

## Priority 4: Lower Priority (1000-1200 lines)

These files should be addressed but are lower priority:

| File | Lines | Suggested Action |
|------|-------|------------------|
| `src/scriptlets.rs` | 1,399 | Split parsing/loading/execution |
| `src/editor.rs` | 1,385 | Split into render/events/snippets |
| `src/app_render.rs` | 1,341 | Break up render helper methods |
| `src/app_launcher.rs` | 1,301 | Split by platform |
| `src/builtins.rs` | 1,279 | Already well-organized by sections |
| `src/prompt_handler.rs` | 1,277 | Split by prompt type |
| `src/execute_script.rs` | 1,266 | Merge into executor/ module |
| `src/expand_manager.rs` | 1,249 | Split types/manager/matchers |
| `src/window_control.rs` | 1,225 | Split by operation type |
| `src/scriptlet_cache.rs` | 1,190 | Split cache/persistence/queries |
| `src/ai/storage.rs` | 1,189 | Split types/queries/migrations |
| `src/designs/separator_variations.rs` | 1,187 | Consider consolidating designs |
| `src/platform.rs` | 1,108 | Split by platform (macos/linux) |
| `src/protocol/types.rs` | 1,088 | Already organized by message type |
| `src/prompts/div.rs` | 1,059 | Split rendering/events/html |
| `src/render_script_list.rs` | 1,006 | Split list/item/grouping |

---

## Test Files Analysis

### Test files over 1000 lines:

| File | Lines | Analysis |
|------|-------|----------|
| `src/scripts_tests.rs` | 4,941 | **CRITICAL** - Should split with main module |
| `src/executor_tests.rs` | 2,171 | Should split with executor/ module |
| `src/scriptlet_tests.rs` | 2,083 | Should split with scriptlets/ module |

**Recommendation:** When splitting modules, keep tests co-located:
```
src/scripts/
  mod.rs
  types.rs
  search.rs
  tests/
    mod.rs           # Test utilities
    types_tests.rs
    search_tests.rs
```

---

## Implementation Strategy

### Phase 1: High-Impact, Low-Risk Splits (Week 1)

1. **Split `src/utils.rs`** - Already has clear section markers
   - Most isolated, lowest risk
   - Creates `src/utils/` module pattern for others

2. **Split `src/logging.rs`** - Already has clear section markers
   - Self-contained
   - Good practice for event categorization

3. **Split `src/theme.rs`** - Already has clear section markers
   - Isolated from core app logic

### Phase 2: Core Module Splits (Week 2-3)

4. **Split `src/executor.rs`** - Clear sections already marked
   - Critical for maintainability
   - High test coverage exists

5. **Split `src/scripts.rs`** → `src/scripts/` module
   - Move tests along with code
   - High value for search maintainability

6. **Split `src/clipboard_history.rs`** → `src/clipboard/`
   - Self-contained feature

### Phase 3: App Core Refactoring (Week 4-5)

7. **Split `src/main.rs`** - Reduce to entry point only
   - Move structs/enums to appropriate modules

8. **Split `src/app_impl.rs`** → `src/app/` module
   - Highest complexity, needs careful planning
   - Consider vertical slices by feature

9. **Split `src/render_prompts.rs`** → consolidate with `src/prompts/`

### Phase 4: Cleanup (Week 6)

10. Address remaining files >1000 lines
11. Consolidate duplicate patterns
12. Update imports across codebase

---

## Module Creation Checklist

When creating a new module directory:

- [ ] Create `mod.rs` with public re-exports
- [ ] Use `pub(crate)` for internal items
- [ ] Keep `impl` blocks with their structs
- [ ] Move tests to `tests/` submodule or `*_tests.rs` file
- [ ] Update `lib.rs` module declarations
- [ ] Run `cargo check && cargo test` after each move
- [ ] Use `pub use` for backwards-compatible exports

---

## Metrics to Track

After splitting:

| Metric | Target |
|--------|--------|
| Max file size | <500 lines (excluding tests) |
| Avg file size | <300 lines |
| Files >1000 lines | 0 (excluding generated code) |
| Module depth | Max 3 levels |

---

## Notes

- Files with `// ===` section markers are easiest to split
- Test files should move with their corresponding modules
- Some duplication exists (e.g., `execute_script.rs` and `executor.rs`)
- Consider using `#[path = "..."]` for gradual migration
