use script_kit_gpui::window_resize::{height_for_view, initial_window_height, ViewType};

#[test]
fn test_height_for_view_uses_single_row_baseline_when_arg_choices_are_empty() {
    let empty_choices_height = height_for_view(ViewType::ArgPromptWithChoices, 0);
    let single_choice_height = height_for_view(ViewType::ArgPromptWithChoices, 1);

    assert_eq!(
        empty_choices_height, single_choice_height,
        "empty choice lists should use the same baseline height as one visible row"
    );
    assert_eq!(
        initial_window_height(),
        height_for_view(ViewType::ScriptList, 0),
        "initial height should match script list default height"
    );
}
