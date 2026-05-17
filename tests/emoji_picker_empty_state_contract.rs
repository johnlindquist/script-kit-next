const EMOJI_PICKER: &str = include_str!("../src/render_builtins/emoji_picker.rs");

#[test]
fn emoji_picker_empty_state_copy_is_modeled() {
    assert!(
        EMOJI_PICKER.contains("enum EmojiPickerEmptyState")
            && EMOJI_PICKER.contains("NoEmojisFound")
            && EMOJI_PICKER.contains("NoFilteredMatches"),
        "Emoji Picker empty-state copy should use named states"
    );
    assert!(
        EMOJI_PICKER.contains("fn from_filter(filter: &str) -> Self")
            && EMOJI_PICKER.contains("fn message(self) -> &'static str"),
        "Emoji Picker empty states should own filter classification and visible copy"
    );
    assert!(
        EMOJI_PICKER.contains("EmojiPickerEmptyState::from_filter(&filter).message()"),
        "Emoji Picker renderer should derive empty-state copy from the model"
    );
    assert!(
        !EMOJI_PICKER.contains("child(if filter.is_empty()"),
        "Emoji Picker empty-state copy must not regress to inline filter-empty branching"
    );
}
