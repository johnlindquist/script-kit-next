Implemented the fastest P0 slice from `plans/missing-features.md` (“Process Manager”): added user-facing built-ins to inspect and stop running script processes.

**Changed files**
- `src/builtins.rs:176`  
  - Added `UtilityCommandType::ProcessManager` and `UtilityCommandType::StopAllProcesses`.
  - Added built-ins:
    - `builtin-process-manager` (`src/builtins.rs:1092`)
    - `builtin-stop-all-processes` (`src/builtins.rs:1103`)
  - Added tests:
    - `test_get_builtin_entries_includes_process_manager_command` (`src/builtins.rs:1736`)
    - `test_get_builtin_entries_includes_stop_all_processes_command` (`src/builtins.rs:1757`)
- `src/app_execute.rs:1098`  
  - Implemented utility command execution for both new built-ins.
  - `ProcessManager`: copies formatted process report to clipboard + HUD message.
  - `StopAllProcesses`: kills all tracked child processes + HUD message.
  - Added structured log lines with correlation IDs:
    - `correlation_id=process-manager-inspect`
    - `correlation_id=process-manager-stop-all`
- `src/process_manager.rs:186`  
  - Added `get_active_processes_sorted()` (newest first).
  - Added `format_active_process_report(max_entries)` for clipboard/UI reporting.
  - Added tests:
    - `test_get_active_processes_sorted_newest_first` (`src/process_manager.rs:526`)
    - `test_format_active_process_report_includes_summary_and_limit` (`src/process_manager.rs:539`)
    - `test_format_active_process_report_empty_state` (`src/process_manager.rs:551`)

**Verification run**
- Ran `cargo check` (fresh run).
- Result: failed due pre-existing unrelated branch errors:
  - `src/watcher.rs:23`/`:25`/`:27`/`:29`/`:31` (`config::defaults` private module access)
  - `src/hotkeys.rs:367`/`:368`/`:374`/`:375`/`:1169`/`:1173` (`Option<HotkeyConfig>` mismatch)
- No compile errors were reported in the new process-manager code paths above.

**How to test (after branch compile is restored)**
1. `cargo test --lib test_get_builtin_entries_includes_process_manager_command`
2. `cargo test --lib test_get_builtin_entries_includes_stop_all_processes_command`
3. `cargo test --lib test_format_active_process_report_includes_summary_and_limit`
4. Launch app with stdin protocol:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - Run built-ins `Process Manager` and `Stop All Running Scripts` from main menu.
5. Check logs:
   - `grep -i "process-manager-inspect\\|process-manager-stop-all" ~/.scriptkit/logs/script-kit-gpui.jsonl`

**Risks / known gaps**
- This is a high-impact quick win, not a full Process Manager view yet.
- Per-process actions (restart/kill single process), rich status UI, and runtime duration/status columns are still missing.
- Full verification is currently blocked by unrelated compile failures listed above.

**Commits**
- No commits were created in this task.