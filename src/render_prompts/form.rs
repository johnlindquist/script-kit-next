mod __render_prompts_form_docs {
    //! Form prompt rendering plus submit-time validation helpers for field values.
    //! Core entrypoints are `ScriptListApp::render_form_prompt` and validator utilities for email/number fields.
    //! It depends on `form_prompt` entities, shared footer helpers, and `ui_foundation` key semantics.
}

// Form prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

include!("form/helpers.rs");
include!("form/render.rs");

#[cfg(test)]
include!("form/tests.rs");
