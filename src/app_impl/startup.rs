use super::*;

pub(super) fn calculate_fallback_error_message(expression: &str) -> String {
    format!(
        "Could not evaluate expression \"{}\". Check the syntax and try again.",
        expression
    )
}

impl ScriptListApp {
    fn new(
        config: config::Config,
        bun_available: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        include!("startup_new_prelude.rs");
        include!("startup_new_state.rs");
        include!("startup_new_tab.rs");
        include!("startup_new_arrow.rs");
        include!("startup_new_actions.rs");
    }
}
