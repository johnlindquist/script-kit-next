use std::fs;

fn read_emoji_picker_source() -> String {
    fs::read_to_string("src/render_builtins/emoji_picker.rs")
        .expect("Failed to read src/render_builtins/emoji_picker.rs")
}

#[test]
fn test_render_emoji_picker_builds_category_headers_and_six_cell_rows() {
    let source = read_emoji_picker_source();

    assert!(
        source.contains("enum EmojiGridRow"),
        "render_emoji_picker should define local EmojiGridRow enum"
    );
    assert!(
        source.contains("Header { title: String, count: usize }"),
        "EmojiGridRow should include Header variant with title and count"
    );
    assert!(
        source.contains("Cells { start_index: usize, count: usize }"),
        "EmojiGridRow should include Cells variant with start index and count"
    );
    assert!(
        source.contains("(category_count - row_offset).min(cols)"),
        "emoji grid should chunk category rows using shared GRID_COLS constant"
    );
}

#[test]
fn test_render_emoji_picker_handles_navigation_and_enter_copy() {
    let source = read_emoji_picker_source();

    for helper_call in [
        "is_key_up(key)",
        "is_key_down(key)",
        "is_key_left(key)",
        "is_key_right(key)",
        "is_key_enter(key)",
    ] {
        assert!(
            source.contains(helper_call),
            "Expected key handler helper `{}` in render_emoji_picker",
            helper_call
        );
    }

    assert!(
        source.contains("cx.write_to_clipboard(gpui::ClipboardItem::new_string("),
        "render_emoji_picker should copy selected emoji on Enter/click"
    );
}

#[test]
fn test_render_emoji_picker_wires_horizontal_input_actions_to_grid_navigation() {
    let source = read_emoji_picker_source();

    assert!(
        source.contains("gpui_component::input::MoveLeft"),
        "emoji picker should listen for MoveLeft action from Input"
    );
    assert!(
        source.contains("gpui_component::input::MoveRight"),
        "emoji picker should listen for MoveRight action from Input"
    );
    assert!(
        source.contains(".on_action(handle_move_left_action)")
            && source.contains(".on_action(handle_move_right_action)"),
        "emoji picker container should register left/right action handlers"
    );
}

#[test]
fn test_render_emoji_picker_consumes_horizontal_arrow_keys() {
    let source = read_emoji_picker_source();

    assert!(
        source.contains("_ if is_key_left(key) => {")
            && source.contains("this.navigate_emoji_picker_horizontal(-1, cx);")
            && source.contains("cx.stop_propagation();"),
        "left arrow handling should navigate grid and stop propagation to Input"
    );

    assert!(
        source.contains("_ if is_key_right(key) => {")
            && source.contains("this.navigate_emoji_picker_horizontal(1, cx);")
            && source.contains("cx.stop_propagation();"),
        "right arrow handling should navigate grid and stop propagation to Input"
    );
}

#[test]
fn test_render_emoji_picker_uses_shared_input_focus_and_scroll_handles() {
    let source = read_emoji_picker_source();

    assert!(
        source.contains("Input::new(&self.gpui_input_state)"),
        "emoji picker header should use shared gpui input state"
    );
    assert!(
        source.contains(".track_focus(&self.focus_handle)"),
        "emoji picker should track focus with app focus handle"
    );
    assert!(
        source.contains(".track_scroll(&self.emoji_scroll_handle)"),
        "emoji picker grid should track emoji scroll handle"
    );
}

#[test]
fn test_render_emoji_picker_uses_uniform_row_height_and_lighter_cell_chrome() {
    let source = read_emoji_picker_source();

    assert!(
        source.contains("let row_height = crate::emoji::GRID_ROW_HEIGHT;"),
        "emoji picker should derive row height from shared GRID_ROW_HEIGHT constant"
    );
    assert!(
        source.contains(".h(px(row_height))") && source.contains(".justify_between()"),
        "header and cell rows should be fixed-height rows that satisfy uniform_list"
    );
    assert!(
        source.contains(".gap(px(tile_gap))") && source.contains(".text_size(px(28.0))"),
        "emoji rows should use tile_gap spacing and compact glyph size"
    );
    assert!(
        source.contains("let tile_size = crate::emoji::GRID_TILE_SIZE;")
            && source.contains("let tile_gap = crate::emoji::GRID_TILE_GAP;"),
        "emoji cells should use shared tile constants"
    );
    assert!(
        source.contains(
            "let hover_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x14);"
        ) && source.contains(
            "let selected_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x2a);"
        ),
        "emoji cells should rely on state-based fills"
    );
}
