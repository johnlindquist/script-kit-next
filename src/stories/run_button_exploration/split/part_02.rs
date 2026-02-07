
impl Story for RunButtonExplorationStory {
    fn id(&self) -> &'static str {
        "run-button-exploration"
    }

    fn name(&self) -> &'static str {
        "Run Button Exploration (50+)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            // =================================================================
            // SECTION 1: NO RUN BUTTON AT ALL (1-6)
            // =================================================================
            .child(
                story_section("1. NO RUN BUTTON - Just Enter key hint")
                    .child(variation_label(
                        "The simplest option: don't show a button at all",
                    ))
                    .child(variation_item(
                        "1. Minimal - just shortcuts",
                        render_no_run_minimal(colors),
                    ))
                    .child(variation_item(
                        "2. Ask AI only",
                        render_no_run_ask_ai_only(colors),
                    ))
                    .child(variation_item(
                        "3. Just Actions",
                        render_no_run_actions_only(colors),
                    ))
                    .child(variation_item(
                        "4. Keyboard hint in input",
                        render_no_run_hint_in_input(colors),
                    ))
                    .child(variation_item(
                        "5. Enter hint at far right",
                        render_no_run_enter_far_right(colors),
                    ))
                    .child(variation_item(
                        "6. Floating hint below",
                        render_no_run_floating_hint(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 2: ICON-ONLY APPROACHES (7-14)
            // =================================================================
            .child(
                story_section("2. ICON-ONLY - No text, just icon")
                    .child(variation_label("Icons save space and don't change width"))
                    .child(variation_item(
                        "7. Play icon ▶",
                        render_icon_only_play(colors),
                    ))
                    .child(variation_item(
                        "8. Arrow icon →",
                        render_icon_only_arrow(colors),
                    ))
                    .child(variation_item(
                        "9. Check icon ✓",
                        render_icon_only_check(colors),
                    ))
                    .child(variation_item(
                        "10. Return icon ↵",
                        render_icon_only_return(colors),
                    ))
                    .child(variation_item(
                        "11. Filled circle ●",
                        render_icon_only_circle(colors),
                    ))
                    .child(variation_item(
                        "12. Double arrow »",
                        render_icon_only_double_arrow(colors),
                    ))
                    .child(variation_item(
                        "13. Icon in circle",
                        render_icon_in_circle(colors),
                    ))
                    .child(variation_item(
                        "14. Icon with ring",
                        render_icon_with_ring(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 3: FIXED-WIDTH BUTTON (15-22)
            // =================================================================
            .child(
                story_section("3. FIXED-WIDTH - Prevent layout shift")
                    .child(variation_label(
                        "Fixed width prevents jumping when text changes",
                    ))
                    .child(variation_item(
                        "15. 60px fixed 'Run'",
                        render_fixed_width_60(colors, "Run"),
                    ))
                    .child(variation_item(
                        "16. 60px fixed 'Submit'",
                        render_fixed_width_60(colors, "Submit"),
                    ))
                    .child(variation_item(
                        "17. 80px fixed 'Open Chrome'",
                        render_fixed_width_80(colors, "Open Chrome"),
                    ))
                    .child(variation_item(
                        "18. 80px fixed 'Select'",
                        render_fixed_width_80(colors, "Select"),
                    ))
                    .child(variation_item(
                        "19. Truncate long text",
                        render_fixed_truncate(colors),
                    ))
                    .child(variation_item(
                        "20. Fixed with tooltip",
                        render_fixed_with_tooltip(colors),
                    ))
                    .child(variation_item(
                        "21. Fixed pill style",
                        render_fixed_pill(colors),
                    ))
                    .child(variation_item(
                        "22. Fixed ghost style",
                        render_fixed_ghost(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 4: POSITIONED AT EDGES (23-30)
            // =================================================================
            .child(
                story_section("4. EDGE POSITIONING - Always in same spot")
                    .child(variation_label("Pin to edge so other elements don't shift"))
                    .child(variation_item(
                        "23. Far right (after logo)",
                        render_pos_far_right(colors),
                    ))
                    .child(variation_item(
                        "24. Before logo, fixed position",
                        render_pos_before_logo(colors),
                    ))
                    .child(variation_item(
                        "25. In input field right side",
                        render_pos_in_input(colors),
                    ))
                    .child(variation_item(
                        "26. Overlapping input corner",
                        render_pos_overlap_input(colors),
                    ))
                    .child(variation_item(
                        "27. Below header strip",
                        render_pos_below_header(colors),
                    ))
                    .child(variation_item(
                        "28. Floating bottom right",
                        render_pos_floating_br(colors),
                    ))
                    .child(variation_item(
                        "29. As part of list first item",
                        render_pos_in_list(colors),
                    ))
                    .child(variation_item(
                        "30. Sticky footer action",
                        render_pos_sticky_footer(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 5: COMBINE WITH ACTIONS (31-38)
            // =================================================================
            .child(
                story_section("5. MERGE WITH ACTIONS - One unified button")
                    .child(variation_label("What if Run was inside the Actions menu?"))
                    .child(variation_item(
                        "31. Actions dropdown with Run first",
                        render_actions_merged(colors),
                    ))
                    .child(variation_item(
                        "32. Split button: Run | ▼",
                        render_split_button(colors),
                    ))
                    .child(variation_item(
                        "33. Primary action pill + more",
                        render_pill_plus_more(colors),
                    ))
                    .child(variation_item(
                        "34. Contextual - 'Run' + more actions",
                        render_contextual_primary(colors),
                    ))
                    .child(variation_item(
                        "35. Two-part: icon + dropdown",
                        render_two_part(colors),
                    ))
                    .child(variation_item(
                        "36. Expandable on hover",
                        render_expandable_hover(colors),
                    ))
                    .child(variation_item(
                        "37. Cycle through actions",
                        render_cycle_actions(colors),
                    ))
                    .child(variation_item(
                        "38. Quick action + menu",
                        render_quick_plus_menu(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 6: SEMANTIC/CONTEXTUAL ICONS (39-46)
            // =================================================================
            .child(
                story_section("6. CONTEXTUAL ICONS - Icon changes, not text")
                    .child(variation_label(
                        "Icon conveys meaning without text width changes",
                    ))
                    .child(variation_item(
                        "39. Script: terminal icon",
                        render_context_terminal(colors),
                    ))
                    .child(variation_item(
                        "40. Form: send icon",
                        render_context_send(colors),
                    ))
                    .child(variation_item(
                        "41. Choice: check icon",
                        render_context_check(colors),
                    ))
                    .child(variation_item(
                        "42. App: launch icon",
                        render_context_launch(colors),
                    ))
                    .child(variation_item(
                        "43. File: folder icon",
                        render_context_folder(colors),
                    ))
                    .child(variation_item(
                        "44. URL: globe icon",
                        render_context_globe(colors),
                    ))
                    .child(variation_item(
                        "45. Command: gear icon",
                        render_context_gear(colors),
                    ))
                    .child(variation_item(
                        "46. Copy: clipboard icon",
                        render_context_clipboard(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 7: TIGHTER BUTTONS (47-54)
            // =================================================================
            .child(
                story_section("7. TIGHTER BUTTONS - Minimal padding")
                    .child(variation_label(
                        "How small can buttons get while staying clickable?",
                    ))
                    .child(variation_item(
                        "47. Micro: 2px padding",
                        render_tight_micro(colors),
                    ))
                    .child(variation_item(
                        "48. Small: 4px padding",
                        render_tight_small(colors),
                    ))
                    .child(variation_item(
                        "49. Compact: 4px h, 6px w",
                        render_tight_compact(colors),
                    ))
                    .child(variation_item(
                        "50. Text only, no button",
                        render_tight_text_only(colors),
                    ))
                    .child(variation_item(
                        "51. Underline on hover",
                        render_tight_underline(colors),
                    ))
                    .child(variation_item(
                        "52. Badge style",
                        render_tight_badge(colors),
                    ))
                    .child(variation_item(
                        "53. Inline link style",
                        render_tight_link(colors),
                    ))
                    .child(variation_item(
                        "54. Minimal pill",
                        render_tight_pill(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 8: ALTERNATIVE PLACEMENTS (55-62)
            // =================================================================
            .child(
                story_section("8. ALTERNATIVE PLACEMENTS - Outside header")
                    .child(variation_label(
                        "Maybe the action doesn't belong in the header?",
                    ))
                    .child(variation_item(
                        "55. In selected list item",
                        render_alt_in_list_item(colors),
                    ))
                    .child(variation_item(
                        "56. As hover overlay on item",
                        render_alt_hover_overlay(colors),
                    ))
                    .child(variation_item(
                        "57. Keyboard-only (no visual)",
                        render_alt_keyboard_only(colors),
                    ))
                    .child(variation_item(
                        "58. Status bar bottom",
                        render_alt_status_bar(colors),
                    ))
                    .child(variation_item(
                        "59. Context on right-click",
                        render_alt_right_click(colors),
                    ))
                    .child(variation_item(
                        "60. Gesture hint (swipe)",
                        render_alt_gesture(colors),
                    ))
                    .child(variation_item(
                        "61. Double-click to run",
                        render_alt_double_click(colors),
                    ))
                    .child(variation_item(
                        "62. Long-press actions",
                        render_alt_long_press(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 9: VISUAL HIERARCHY (63-70)
            // =================================================================
            .child(
                story_section("9. VISUAL HIERARCHY - De-emphasize or emphasize")
                    .child(variation_label("Control attention via styling"))
                    .child(variation_item(
                        "63. Ghost (barely visible)",
                        render_hier_ghost(colors),
                    ))
                    .child(variation_item(
                        "64. Muted until hover",
                        render_hier_muted(colors),
                    ))
                    .child(variation_item(
                        "65. Primary action (bold)",
                        render_hier_primary(colors),
                    ))
                    .child(variation_item(
                        "66. Accent background",
                        render_hier_accent_bg(colors),
                    ))
                    .child(variation_item(
                        "67. Outline style",
                        render_hier_outline(colors),
                    ))
                    .child(variation_item(
                        "68. Gradient accent",
                        render_hier_gradient(colors),
                    ))
                    .child(variation_item(
                        "69. Glow effect (hover)",
                        render_hier_glow(colors),
                    ))
                    .child(variation_item(
                        "70. Pulsing attention",
                        render_hier_pulse(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 10: RECOMMENDED APPROACHES (71-76)
            // =================================================================
            .child(
                story_section("10. RECOMMENDED - Best combinations")
                    .child(variation_label("Synthesized from above explorations"))
                    .child(variation_item(
                        "71. ★ Icon-only + tooltip",
                        render_rec_icon_tooltip(colors),
                    ))
                    .child(variation_item(
                        "72. ★ Fixed-width ghost",
                        render_rec_fixed_ghost(colors),
                    ))
                    .child(variation_item(
                        "73. ★ No button, Enter hint",
                        render_rec_no_button(colors),
                    ))
                    .child(variation_item(
                        "74. ★ Contextual icon, no text",
                        render_rec_contextual_icon(colors),
                    ))
                    .child(variation_item(
                        "75. ★ Merge into Actions",
                        render_rec_merged(colors),
                    ))
                    .child(variation_item(
                        "76. ★ Split button compact",
                        render_rec_split_compact(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        // Not implementing all 76 as individual variants - the story shows them all
        vec![
            StoryVariant {
                name: "no-run".into(),
                description: Some("No run button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "icon-only".into(),
                description: Some("Icon without text".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-width".into(),
                description: Some("Fixed width button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "recommended".into(),
                description: Some("Recommended approaches".into()),
                ..Default::default()
            },
        ]
    }
}
