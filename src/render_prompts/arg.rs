mod __render_prompts_arg_docs {
    //! Arg prompt rendering helpers and `ScriptListApp::render_arg_prompt` implementation.
    //! Key routines include prompt-footer/status helpers and `render_arg_input_text` for visual state.
    //! This fragment depends on `panel`, `components`, and actions-dialog flow and is included by `main.rs`.
}

// Arg prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

include!("arg/helpers.rs");
include!("arg/render.rs");

#[cfg(test)]
include!("arg/tests.rs");
