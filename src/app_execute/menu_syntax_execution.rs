// Thin adapter that bridges a `MenuSyntaxMode::capture_for(raw)` row to the
// existing script-execution pipeline.
//
// Flow:
//   resolve_capture_dates → build_capture_payload → write_payload_tempfile
//   → payload_env → executor::execute_script_interactive_with_env
//
// Payload files live at `$SK_PATH/menu-syntax/payloads` with a fallback to
// `~/.scriptkit/menu-syntax/payloads` so they are debuggable on disk. Handlers
// read `KIT_MENU_SYNTAX_PAYLOAD_PATH` from the environment.

impl ScriptListApp {
    pub(crate) fn execute_menu_syntax_command_invocation(
        &mut self,
        invocation: crate::menu_syntax::ArgvInvocation,
        cx: &mut Context<Self>,
    ) {
        let head = invocation.head.clone();
        let matching_scripts: Vec<_> = self
            .scripts
            .iter()
            .filter(|script| {
                crate::menu_syntax::command_head_matches(
                    &head,
                    &crate::menu_syntax::script_command_head(script),
                )
            })
            .cloned()
            .collect();
        let matching_scriptlets: Vec<_> = self
            .scriptlets
            .iter()
            .filter(|scriptlet| {
                crate::menu_syntax::command_head_matches(
                    &head,
                    &crate::menu_syntax::scriptlet_command_head(scriptlet),
                )
            })
            .cloned()
            .collect();
        let match_count = matching_scripts.len() + matching_scriptlets.len();

        if match_count > 1 {
            tracing::warn!(
                category = "EXEC",
                event = "menu_syntax_command_ambiguous",
                command_head = %head,
                match_count,
                "Script Kit command head matched multiple registered commands"
            );
            self.show_hud(
                format!("Ambiguous command !{head}"),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }

        if match_count == 1 {
            // Story E slice 2 (Run 14 Pass 15): record argv into the
            // per-command-head history pool so the upcoming `!command`
            // popup surfaces past argv vectors recency-first. Best-effort —
            // a write failure must never block command execution.
            let store = crate::menu_syntax::CommandHistoryStore::from_env();
            if let Err(err) = store.record_argv(&head, &invocation.argv) {
                tracing::warn!(
                    category = "EXEC",
                    event = "menu_syntax_command_argv_history_record_failed",
                    command_head = %head,
                    error = %err,
                    "Failed to record argv into command history pool"
                );
            }
        }

        if let Some(script) = matching_scripts.into_iter().next() {
            let env_pairs = crate::menu_syntax::command_env(&invocation);
            tracing::info!(
                category = "EXEC",
                event = "menu_syntax_command_execute_script",
                command_head = %head,
                script_name = %script.name,
                argv = ?invocation.argv,
                "Launching Script Kit command from ! invocation"
            );
            self.execute_interactive_with_env_and_args(&script, env_pairs, invocation.argv, cx);
            return;
        }

        if let Some(scriptlet) = matching_scriptlets.into_iter().next() {
            let env_pairs = crate::menu_syntax::command_env(&invocation);
            tracing::info!(
                category = "EXEC",
                event = "menu_syntax_command_execute_scriptlet",
                command_head = %head,
                scriptlet_name = %scriptlet.name,
                argv = ?invocation.argv,
                "Launching Script Kit scriptlet command from ! invocation"
            );
            self.execute_scriptlet_with_env_and_args(&scriptlet, env_pairs, invocation.argv, cx);
            return;
        }

        tracing::info!(
            category = "EXEC",
            event = "menu_syntax_command_not_found",
            command_head = %head,
            "No Script Kit command matched ! invocation"
        );
        self.show_hud(
            format!("No command named !{head}"),
            Some(HUD_MEDIUM_MS),
            cx,
        );
    }

    pub(crate) fn execute_menu_syntax_capture_script(
        &mut self,
        script: std::sync::Arc<scripts::Script>,
        invocation: crate::menu_syntax::CaptureInvocation,
        cx: &mut Context<Self>,
    ) {
        // Validation gate: refuse to spawn a handler when the schema we know
        // about (builtin first, dynamic from the script's own spec second)
        // says the payload is incomplete or malformed. No schema → permissive
        // Allow so handlers without declared shape still execute. See
        // [[removed-docs Syntax#Capture Validation Gate]].
        match crate::menu_syntax::decide_capture_gate_for_script(&invocation, &script) {
            crate::menu_syntax::CaptureGateDecision::Allow => {}
            crate::menu_syntax::CaptureGateDecision::BlockMissing { hud_message, .. }
            | crate::menu_syntax::CaptureGateDecision::BlockMalformed { hud_message, .. } => {
                tracing::info!(
                    category = "EXEC",
                    event = "menu_syntax_capture_gate_blocked",
                    target = %invocation.target,
                    script_name = %script.name,
                    hud = %hud_message,
                    "Capture validation gate blocked Enter — no payload written"
                );
                self.show_hud(hud_message, Some(HUD_MEDIUM_MS), cx);
                return;
            }
        }

        // Story D slice 4: record positive #tags and free-form key:value
        // fields to per-target history pools so the autocomplete popup
        // (Pass 9 + Pass 10 surface) reflects what the user has actually
        // captured. Failures are logged but never block execution —
        // history is best-effort and the capture itself is the contract.
        let grammar_payload = crate::menu_syntax::GrammarPayload::from(&invocation);
        let history_store = crate::menu_syntax::HistoryStore::from_env();
        if let Err(err) = history_store.record_payload_tags(&grammar_payload) {
            tracing::warn!(
                category = "EXEC",
                event = "menu_syntax_history_record_tags_failed",
                target = %invocation.target,
                error = %err,
                "history pool tag-record failed; capture proceeds"
            );
        }
        if let Err(err) = history_store.record_payload_fields(&grammar_payload) {
            tracing::warn!(
                category = "EXEC",
                event = "menu_syntax_history_record_fields_failed",
                target = %invocation.target,
                error = %err,
                "history pool field-record failed; capture proceeds"
            );
        }

        let accepts = crate::menu_syntax::script_menu_syntax_specs(&script)
            .into_iter()
            .find(|spec| spec.handles_capture_target(&invocation.target))
            .map(|spec| spec.accepts)
            .unwrap_or_default();
        let clock = crate::menu_syntax::MenuSyntaxClock::local_now();
        let resolved =
            crate::menu_syntax::date::resolve_capture_dates_with_accepts(&invocation, &clock, &accepts);

        let command_id = format!(
            "{}:{}",
            script.plugin_id,
            script
                .path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| script.name.clone())
        );
        let handler_ref = crate::menu_syntax::MenuSyntaxHandlerRef {
            kind: crate::menu_syntax::MenuSyntaxHandlerKind::Script,
            command_id,
            name: script.name.clone(),
            plugin_id: Some(script.plugin_id.clone()),
        };

        let payload = crate::menu_syntax::build_capture_payload(handler_ref, resolved);

        let payload_dir = menu_syntax_payload_dir();
        if let Err(err) = std::fs::create_dir_all(&payload_dir) {
            tracing::error!(
                category = "EXEC",
                event = "menu_syntax_payload_dir_failed",
                error = %err,
                dir = %payload_dir.display(),
                "Failed to create menu-syntax payload directory"
            );
            self.show_hud(
                format!("Menu syntax: failed to create payload dir: {err}"),
                Some(HUD_MEDIUM_MS),
                cx,
            );
            return;
        }

        let payload_path = match crate::menu_syntax::write_payload_tempfile(&payload_dir, &payload)
        {
            Ok(path) => path,
            Err(err) => {
                tracing::error!(
                    category = "EXEC",
                    event = "menu_syntax_write_payload_failed",
                    error = %err,
                    dir = %payload_dir.display(),
                    "Failed to write menu-syntax payload"
                );
                self.show_hud(
                    format!("Menu syntax: failed to write payload: {err}"),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                return;
            }
        };

        let env_pairs = crate::menu_syntax::payload_env(&payload_path, &payload);

        tracing::info!(
            category = "EXEC",
            event = "menu_syntax_capture_execute",
            script_name = %script.name,
            family = %payload.family,
            target = %payload.target,
            payload_path = %payload_path.display(),
            "Launching capture handler with menu-syntax payload"
        );

        // Capture handlers are fire-and-forget file writers. They must NOT use
        // the interactive bun+SDK pipeline: the SDK preload installs a stdin
        // handler that waits on the launcher's JSONL protocol, and the handler
        // script never gets its turn. Spawn bun/node directly with only the
        // script path and the menu-syntax env contract, then detach — the
        // launcher does not need a session handle for a handler it does not
        // speak protocol with.
        match spawn_capture_handler_detached(&script.path, &env_pairs) {
            Ok(pid) => {
                tracing::info!(
                    category = "EXEC",
                    event = "menu_syntax_capture_session_started",
                    script_name = %script.name,
                    pid,
                    "Capture handler spawned detached"
                );
                self.show_hud(
                    format!("Captured to {}", payload.target),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                self.close_and_reset_window(cx);
            }
            Err(err) => {
                tracing::error!(
                    category = "EXEC",
                    event = "menu_syntax_capture_spawn_failed",
                    script_name = %script.name,
                    error = %err,
                    "Failed to spawn capture handler"
                );
                self.show_hud(
                    format!("Capture failed: {err}"),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
            }
        }
    }
}

/// Spawn a capture handler as a detached process.
///
/// Runs `bun run <path>` (or `node <path>` for `.js`) with only the
/// `KIT_MENU_SYNTAX*` env pairs plus the safe allowlist. Does not use the
/// executor's bidirectional JSONL session — capture handlers are
/// fire-and-forget file writers that do not speak the launcher protocol.
fn spawn_capture_handler_detached(
    path: &std::path::Path,
    extra_env: &[(String, String)],
) -> Result<u32, String> {
    let path_str = path.to_string_lossy();
    let is_typescript = path
        .extension()
        .map(|e| matches!(e.to_string_lossy().as_ref(), "ts" | "mts" | "tsx"))
        .unwrap_or(false);
    let is_javascript = path
        .extension()
        .map(|e| matches!(e.to_string_lossy().as_ref(), "js" | "mjs" | "cjs"))
        .unwrap_or(false);

    let (cmd, args): (&str, Vec<String>) = if is_typescript {
        ("bun", vec!["run".to_string(), path_str.to_string()])
    } else if is_javascript {
        ("node", vec![path_str.to_string()])
    } else {
        return Err(format!(
            "Unsupported script extension for capture handler: {}",
            path.display()
        ));
    };

    let executable = executor::runner::find_executable(cmd)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| cmd.to_string());

    let mut command = std::process::Command::new(&executable);
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "USER",
        "LANG",
        "TERM",
        "SHELL",
        "XDG_RUNTIME_DIR",
        "SK_PATH",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    command
        .spawn()
        .map(|child| child.id())
        .map_err(|e| format!("Failed to spawn '{}': {}", executable, e))
}

fn menu_syntax_payload_dir() -> std::path::PathBuf {
    if let Ok(sk_path) = std::env::var("SK_PATH") {
        let root = std::path::PathBuf::from(sk_path);
        if !root.as_os_str().is_empty() {
            return root.join("menu-syntax").join("payloads");
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("menu-syntax")
        .join("payloads")
}
