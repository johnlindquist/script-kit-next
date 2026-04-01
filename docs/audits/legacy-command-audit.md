# Legacy Command Audit

Scope: `AiCommandType::{OpenAi, MiniAi, NewConversation, ClearConversation}` and `AppLauncher` / `builtin-app-launcher`

Method:
- `rg -n 'AiCommandType::(OpenAi|MiniAi|NewConversation|ClearConversation)' src tests`
- `rg -n '\bAppLauncher\b|builtin-app-launcher' src tests`

This document is an accounting pass only. It does not make removal recommendations, because external-command entry points, protocol-adjacent `promptType` strings, and test/audit fixtures are still present.

## Summary

| Family | Exact `rg` hits | User-visible | Compatibility-only | Removal-blocking | Notes |
| --- | ---: | --- | --- | --- | --- |
| Legacy AI Commands | 20 | No direct builtin registration for these enum variants; current builtin is `builtin-ai-chat` / `Open AI Harness` at `src/builtins/mod.rs:459-460` | Enum variants and unregistered executor/helper match arms remain in `src/builtins/mod.rs:113-117`, `src/app_execute/builtin_execution.rs:205-208`, and `src/app_execute/builtin_execution.rs:2094-2097` | Tests and adjacent legacy AI entry points remain in `tests/source_audits/mini_ai_window.rs:206-207`, `tests/source_audits/execution_helpers.rs:250,269,288,303`, `tests/tab_ai_routing.rs:1841`, `src/main_entry/runtime_stdin_match_tail.rs:7-45`, `src/main_entry/app_run_setup.rs:2031-2068`, `src/tray/mod.rs:74-77`, `src/tray/mod.rs:240-242`, `src/actions/builders/chat.rs:123-132` | Exact `AiCommandType` hits are not registered in `get_builtin_entries()`, but the broader legacy AI family is still externally reachable. |
| AppLauncher | 52 | No current `get_builtin_entries()` or tray/menu registration; removal is documented at `src/builtins/mod.rs:409-410` and `src/builtins/mod.rs:439-442` | Legacy builtin enum/comments remain in `src/builtins/mod.rs:248`, `src/builtins/mod.rs:409-410`, `src/builtins/mod.rs:439-442`, plus non-behavioral comment mentions in `src/app_actions/helpers.rs:25` and `src/app_impl/shortcuts_hud_grid.rs:67` | Runtime dispatch, live view plumbing, protocol/persistence strings, and tests remain in `src/app_execute/builtin_execution.rs:1677`, `src/main_entry/runtime_stdin_match_core.rs:184-185`, `src/main_entry/runtime_stdin.rs:272-273`, `src/main_entry/app_run_setup.rs:1601-1602`, `src/app_impl/tab_ai_mode.rs:1165,1184,1623`, `src/ai/tab_context.rs:3220,3238,3486`, `tests/tab_ai_execution.rs:26,64,212,241`, `tests/tab_ai_memory.rs:24,289,299,330`, `tests/fixtures/tab_ai_execution_record_v1.json:7`, `tests/tab_ai_input_coverage.rs:403,527,530,532,1010`, `tests/tab_ai_context.rs:238`, `tests/tab_ai_prompt.rs:182,186,211`, and additional audit/unit-test references listed below | AppLauncher is no longer main-search registered, but it is still wired through direct dispatch aliases and `Tab AI` context/persistence semantics. |

## Legacy AI Commands

### User-visible

No exact `AiCommandType::{OpenAi, MiniAi, NewConversation, ClearConversation}` reference appears in `get_builtin_entries()`. The current builtin registration is `builtin-ai-chat` with the user-visible label `Open AI Harness` at `src/builtins/mod.rs:459-460`.

Adjacent user-visible legacy AI surfaces still exist outside the exact `rg` hits:
- `src/tray/mod.rs:74-77` defines `TrayMenuAction::OpenAiChat`.
- `src/tray/mod.rs:240-242` creates the tray item labeled `Open AI Chat`.
- `src/actions/builders/chat.rs:123-132` still exposes a user-visible `Clear Conversation` action in chat context actions.

### Compatibility-only

These references are not currently registered in `get_builtin_entries()`, but they remain in enum/match logic:
- `src/builtins/mod.rs:113-117` defines the four legacy `AiCommandType` variants.
- `src/app_execute/builtin_execution.rs:205-208` keeps all four variants in `ai_command_keeps_main_window_visible(...)`.
- `src/app_execute/builtin_execution.rs:2094-2097` keeps an executor match arm that routes all four variants to `open_tab_ai_chat(cx)`.

### Removal-blocking

These references would fail tests or leave adjacent legacy AI entry points unaccounted for:
- `src/app_execute/builtin_execution.rs:5046,5049,5052,5055,5112` unit tests assert the helper coverage for the legacy variants.
- `tests/source_audits/mini_ai_window.rs:206-207` hard-codes `AiCommandType::MiniAi`.
- `tests/source_audits/execution_helpers.rs:250,269,288,303` hard-code `OpenAi`, `NewConversation`, and `ClearConversation`.
- `tests/tab_ai_routing.rs:1841` hard-codes the grouped `OpenAi | MiniAi` helper branch.
- `src/main_entry/runtime_stdin_match_tail.rs:7-45` still exposes `ExternalCommand::OpenAi`, `ExternalCommand::OpenMiniAi`, and `ExternalCommand::ShowAiCommandBar`.
- `src/main_entry/app_run_setup.rs:2031-2068` duplicates those stdin-dispatch paths.

### Exact `rg` inventory

All direct hits from `rg -n 'AiCommandType::(OpenAi|MiniAi|NewConversation|ClearConversation)' src tests`:

```text
tests/source_audits/mini_ai_window.rs:206:        source.contains("AiCommandType::MiniAi"),
tests/source_audits/mini_ai_window.rs:207:        "builtin_execution.rs must handle AiCommandType::MiniAi"
tests/source_audits/execution_helpers.rs:250:        .find("AiCommandType::OpenAi | AiCommandType::NewConversation")
tests/source_audits/execution_helpers.rs:269:        .find("AiCommandType::ClearConversation")
tests/source_audits/execution_helpers.rs:288:        .find("AiCommandType::ClearConversation")
tests/source_audits/execution_helpers.rs:303:        .find("AiCommandType::ClearConversation")
tests/tab_ai_routing.rs:1841:            .contains("builtins::AiCommandType::OpenAi\n        | builtins::AiCommandType::MiniAi"),
src/app_execute/builtin_execution.rs:205:        | builtins::AiCommandType::OpenAi
src/app_execute/builtin_execution.rs:206:        | builtins::AiCommandType::MiniAi
src/app_execute/builtin_execution.rs:207:        | builtins::AiCommandType::NewConversation
src/app_execute/builtin_execution.rs:208:        | builtins::AiCommandType::ClearConversation
src/app_execute/builtin_execution.rs:2094:                    AiCommandType::OpenAi
src/app_execute/builtin_execution.rs:2095:                    | AiCommandType::MiniAi
src/app_execute/builtin_execution.rs:2096:                    | AiCommandType::NewConversation
src/app_execute/builtin_execution.rs:2097:                    | AiCommandType::ClearConversation => {
src/app_execute/builtin_execution.rs:5046:            &AiCommandType::OpenAi
src/app_execute/builtin_execution.rs:5049:            &AiCommandType::MiniAi
src/app_execute/builtin_execution.rs:5052:            &AiCommandType::NewConversation
src/app_execute/builtin_execution.rs:5055:            &AiCommandType::ClearConversation
src/app_execute/builtin_execution.rs:5112:            &AiCommandType::MiniAi
```

## AppLauncher

### User-visible

No AppLauncher entry is currently registered in `get_builtin_entries()`. The code explicitly documents that removal from main search:
- `src/builtins/mod.rs:409-410`
- `src/builtins/mod.rs:439-442`

There is also no tray/menu registration for AppLauncher in the files reviewed for this audit.

### Compatibility-only

These references document or retain the old builtin identity without registering it in main search:
- `src/builtins/mod.rs:248` keeps `BuiltInFeature::AppLauncher`.
- `src/builtins/mod.rs:409` and `src/builtins/mod.rs:439` document the legacy status.
- `src/app_actions/helpers.rs:25` mentions `AppLauncher` in a feedback-policy comment.
- `src/app_impl/shortcuts_hud_grid.rs:67` mentions `AppLauncher` in a dismissable-view comment.

### Removal-blocking

The remaining AppLauncher references break down into four blocking clusters.

#### 1. Runtime dispatch and live view entry points

These still instantiate or route to the AppLauncher experience:
- `src/app_execute/builtin_execution.rs:1677`
- `src/main_entry/runtime_stdin_match_core.rs:184-185`
- `src/main_entry/runtime_stdin.rs:272-273`
- `src/main_entry/app_run_setup.rs:1601-1602`
- `src/main_sections/app_view_state.rs:311`
- `src/app_impl/actions_dialog.rs:265,457`
- `src/app_impl/lifecycle_reset.rs:205`
- `src/app_render/group_header_item.rs:26`

Adjacent live-view implementation evidence outside the exact `rg` query still exists and is part of the same blocking runtime path:
- `src/render_builtins/app_launcher.rs:69,72,183,215`
- `src/app_layout/build_component_bounds.rs:47`

#### 2. Protocol-adjacent and persistence semantics

These keep `AppLauncher` as a persisted or emitted `promptType` / context source:
- `src/ai/tab_context.rs:3220,3238,3486`
- `src/app_impl/tab_ai_mode.rs:1165,1184,1623`
- `tests/tab_ai_execution.rs:26,64,212,241`
- `tests/tab_ai_memory.rs:24,289,299,330`
- `tests/fixtures/tab_ai_execution_record_v1.json:7`
- `tests/tab_ai_input_coverage.rs:403,527,530,532,1010`
- `tests/tab_ai_context.rs:238`
- `tests/tab_ai_prompt.rs:182,186,211`

#### 3. Test and audit fixtures for the builtin identity

These assert the old builtin or builtin ID still exists:
- `src/config/config_tests/mod.rs:1477`
- `tests/source_audits/builtin_dispatch_consistency.rs:223`
- `src/builtins/mod.rs:2047,2062,2068,2096,2141`
- `src/scripts/tests/chunk_09.rs:23,32,198`
- `src/scripts/tests/chunk_12.rs:238`
- `src/scripts_tests/chunk_09.rs:23,32,198`
- `src/scripts_tests/chunk_12.rs:238`

#### 4. Additional internal references that would need coordinated cleanup

These are not registration points, but they are executable/internal references that still assume the AppLauncher view family exists:
- `src/main_sections/app_view_state.rs:311`
- `src/app_impl/actions_dialog.rs:265,457`
- `src/app_impl/lifecycle_reset.rs:205`
- `src/app_render/group_header_item.rs:26`

### Exact `rg` inventory

All direct hits from `rg -n '\bAppLauncher\b|builtin-app-launcher' src tests`:

```text
tests/tab_ai_execution.rs:26:        "AppLauncher".to_string(),
tests/tab_ai_execution.rs:64:    assert_eq!(parsed.prompt_type, "AppLauncher");
tests/tab_ai_execution.rs:212:    assert_eq!(entry.prompt_type, "AppLauncher");
tests/tab_ai_execution.rs:241:        "AppLauncher".to_string(),
src/app_render/group_header_item.rs:26:        builtins::BuiltInFeature::AppLauncher => "Application Launcher".to_string(),
tests/tab_ai_memory.rs:24:        prompt_type: "AppLauncher".to_string(),
tests/tab_ai_memory.rs:289:        prompt_type: "AppLauncher".to_string(),
tests/tab_ai_memory.rs:299:    assert_eq!(json["promptType"], "AppLauncher");
tests/tab_ai_memory.rs:330:        prompt_type: "AppLauncher".to_string(),
tests/fixtures/tab_ai_execution_record_v1.json:7:  "promptType": "AppLauncher",
tests/tab_ai_input_coverage.rs:403:        "AppLauncher",
tests/tab_ai_input_coverage.rs:527:/// AppLauncher: filter + app list → rich
tests/tab_ai_input_coverage.rs:530:    let r = rich_list_receipt("AppLauncher");
tests/tab_ai_input_coverage.rs:532:    assert_eq!(r.prompt_type, "AppLauncher");
tests/tab_ai_input_coverage.rs:1010:        "AppLauncher",
src/config/config_tests/mod.rs:1477:    assert!(!config.requires_confirmation("builtin-app-launcher"));
src/app_actions/helpers.rs:25:// |                      |               | AppLauncher, WindowSwitcher, FileSearch,  |
src/main_sections/app_view_state.rs:311:    AppLauncher,
tests/tab_ai_context.rs:238:            prompt_type: "AppLauncher".to_string(),
tests/source_audits/builtin_dispatch_consistency.rs:223:        "builtins::BuiltInFeature::AppLauncher",
src/render_builtins/app_launcher.rs:69:                logging::log("KEY", &format!("AppLauncher key: '{}'", key));
src/app_impl/shortcuts_hud_grid.rs:67:    /// - Built-in views (ClipboardHistory, AppLauncher, WindowSwitcher, DesignGallery)
src/app_layout/build_component_bounds.rs:47:            AppView::AppLauncherView { .. } => "AppLauncher",
src/scripts/tests/chunk_12.rs:238:            feature: BuiltInFeature::AppLauncher,
src/scripts/tests/chunk_09.rs:23:            id: "builtin-app-launcher".to_string(),
src/scripts/tests/chunk_09.rs:32:            feature: BuiltInFeature::AppLauncher,
src/scripts/tests/chunk_09.rs:198:        feature: BuiltInFeature::AppLauncher,
src/ai/tab_context.rs:3220:            "AppLauncher".to_string(),
src/ai/tab_context.rs:3238:        assert_eq!(record.prompt_type, "AppLauncher");
src/ai/tab_context.rs:3486:            prompt_type: "AppLauncher".to_string(),
src/scripts_tests/chunk_12.rs:238:            feature: BuiltInFeature::AppLauncher,
src/scripts_tests/chunk_09.rs:23:            id: "builtin-app-launcher".to_string(),
src/scripts_tests/chunk_09.rs:32:            feature: BuiltInFeature::AppLauncher,
src/scripts_tests/chunk_09.rs:198:        feature: BuiltInFeature::AppLauncher,
tests/tab_ai_prompt.rs:182:    let prompt = build_tab_ai_user_prompt("force quit", r#"{"ui":{"promptType":"AppLauncher"}}"#);
tests/tab_ai_prompt.rs:186:            prompt_type: "AppLauncher".to_string(),
tests/tab_ai_prompt.rs:211:    assert_eq!(blob.ui.prompt_type, "AppLauncher");
src/app_impl/tab_ai_mode.rs:1165:                        source: "AppLauncher".to_string(),
src/app_impl/tab_ai_mode.rs:1184:                        source: "AppLauncher".to_string(),
src/app_impl/tab_ai_mode.rs:1623:            AppView::AppLauncherView { .. } => "AppLauncher".to_string(),
src/app_impl/lifecycle_reset.rs:205:                Some("AppLauncher filter")
src/builtins/mod.rs:248:    AppLauncher,
src/builtins/mod.rs:409:/// Note: AppLauncher built-in is no longer used since apps now appear directly
src/builtins/mod.rs:439:        // Note: AppLauncher built-in removed - apps now appear directly in main search
src/builtins/mod.rs:2047:        assert_eq!(BuiltInFeature::AppLauncher, BuiltInFeature::AppLauncher);
src/builtins/mod.rs:2062:            BuiltInFeature::AppLauncher
src/builtins/mod.rs:2068:        assert_ne!(BuiltInFeature::AppLauncher, BuiltInFeature::WindowSwitcher);
src/builtins/mod.rs:2096:            BuiltInFeature::AppLauncher
src/builtins/mod.rs:2141:            BuiltInFeature::AppLauncher,
src/app_execute/builtin_execution.rs:1677:            builtins::BuiltInFeature::AppLauncher => {
src/app_impl/actions_dialog.rs:265:            | ActionsDialogHost::AppLauncher => FocusRequest::main_filter(),
src/app_impl/actions_dialog.rs:457:            "ActionsDialogHost::AppLauncher",
```

## Conclusion

No removal recommendation is made in this audit.

Reason:
- The legacy AI enum variants are unregistered as builtins, but tests and adjacent legacy AI window/tray/stdin surfaces still exist.
- AppLauncher is unregistered from main search, but it remains reachable through direct runtime aliases, live view plumbing, `Tab AI` context/persistence strings, and multiple tests/audits.
- A safe removal proposal would require a second pass that updates runtime dispatch, protocol-adjacent `promptType` strings, fixtures, and test expectations together.
