use super::*;

impl ScriptListApp {
    pub(crate) fn record_submit_diagnostic(
        &mut self,
        owner: &'static str,
        route: &'static str,
        prompt_id: Option<&str>,
        value: Option<&str>,
        consumed_enter: bool,
    ) {
        self.submit_diagnostics.generation = self.submit_diagnostics.generation.saturating_add(1);
        let generation = self.submit_diagnostics.generation;
        let surface = format!("{:?}", self.current_view.surface_kind());
        let selected_index = match self.current_view {
            AppView::ScriptList => Some(self.selected_index),
            AppView::ArgPrompt { .. }
            | AppView::MiniPrompt { .. }
            | AppView::MicroPrompt { .. } => Some(self.arg_selected_index),
            _ => None,
        };

        if consumed_enter {
            self.submit_diagnostics.pending_enter_consumed_at = Some(std::time::Instant::now());
        }

        self.submit_diagnostics.last = Some(SubmitDiagnosticEvent {
            generation,
            owner,
            route,
            surface: surface.clone(),
            prompt_id: prompt_id.map(str::to_string),
            value: value.map(str::to_string),
            selected_index,
            consumed_enter,
        });

        tracing::info!(
            target: "script_kit::submit",
            event = "submit_owner_recorded",
            generation,
            owner,
            route,
            surface,
            prompt_id = ?prompt_id,
            value = ?value,
            selected_index = ?selected_index,
            consumed_enter,
            "submit owner recorded"
        );
    }

    pub(crate) fn should_consume_script_list_enter_after_submit(
        &mut self,
        route: &'static str,
    ) -> bool {
        const ENTER_ECHO_GUARD_MS: u128 = 250;
        let Some(consumed_at) = self.submit_diagnostics.pending_enter_consumed_at.take() else {
            return false;
        };

        let age_ms = consumed_at.elapsed().as_millis();
        let consume = age_ms <= ENTER_ECHO_GUARD_MS;
        tracing::info!(
            target: "script_kit::submit",
            event = if consume {
                "script_list_enter_echo_consumed"
            } else {
                "script_list_enter_echo_guard_expired"
            },
            route,
            age_ms,
            guard_ms = ENTER_ECHO_GUARD_MS,
            last_generation = self.submit_diagnostics.last.as_ref().map(|event| event.generation),
            last_owner = self.submit_diagnostics.last.as_ref().map(|event| event.owner),
            last_route = self.submit_diagnostics.last.as_ref().map(|event| event.route),
            "script list enter checked against submit ownership guard"
        );

        consume
    }

    pub(crate) fn submit_diagnostics_snapshot(&self) -> Option<serde_json::Value> {
        let last = self.submit_diagnostics.last.as_ref()?;
        Some(serde_json::json!({
            "generation": last.generation,
            "owner": last.owner,
            "route": last.route,
            "surface": last.surface,
            "promptId": last.prompt_id,
            "value": last.value,
            "selectedIndex": last.selected_index,
            "consumedEnter": last.consumed_enter,
            "pendingEnterGuardActive": self.submit_diagnostics.pending_enter_consumed_at.is_some(),
        }))
    }
}
