
/// 3. 20px container / 12px SVG
fn render_v3(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(20., 4., 12.))
}

/// 4. 21px container / 13px SVG (Golden ratio - baseline)
fn render_v4(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
}

/// 5. 22px container / 14px SVG
fn render_v5(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(22., 4., 14.))
}

// =============================================================================
// LOGO PLACEMENT - RIGHT SIDE (6-10)
// =============================================================================

/// 6. Logo far right (after all elements)
fn render_v6(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(24.)))
        .child(golden_logo())
}

/// 7. Logo before actions
fn render_v7(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 8. Logo after Run (between Run and Actions)
fn render_v8(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(12.)))
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(actions_button(colors))
}

/// 9. Logo with 8px gap from actions
fn render_v9(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(golden_logo())
}

/// 10. Logo with 16px gap from actions
fn render_v10(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
}

// =============================================================================
// LOGO PLACEMENT - LEFT SIDE (11-15)
// =============================================================================

/// 11. Logo before title (logo first)
fn render_v11(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(10.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 12. Logo 8px from title
fn render_v12(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(8.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 13. Logo 12px from title
fn render_v13(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 14. Logo flush left (minimal gap)
fn render_v14(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(6.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 15. Logo with subtle left border/separator
fn render_v15(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(div().w(px(1.)).h(px(20.)).bg(colors.border.to_rgb()))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

// =============================================================================
// CORNER RADIUS & STYLE (16-20)
// =============================================================================

/// 16. Sharp corners (2px radius)
fn render_v16(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 2., 13.))
}

/// 17. Medium corners (4px radius) - same as golden
fn render_v17(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 4., 13.))
}

/// 18. Round corners (6px radius)
fn render_v18(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 6., 13.))
}

/// 19. Circular logo
fn render_v19(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(circular_logo(21., 13.))
}

/// 20. Squircle style (8px radius for smoother corners)
fn render_v20(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 8., 13.))
}
