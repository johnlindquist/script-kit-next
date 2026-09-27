/// The calculator's answer as a list row: the number, and the operation and words it came from.
fn render_inline_calc_list_item(
    calculator: &crate::calculator::CalculatorInlineResult,
    semantic_id: Option<String>,
    ix: usize,
    is_selected: bool,
    colors: ListItemColors,
    main_menu_theme: crate::designs::MainMenuThemeVariant,
) -> AnyElement {
    crate::list_item::ListItem::new(calculator.formatted.clone(), colors)
        .index(ix)
        .selected(is_selected)
        .main_menu_theme(main_menu_theme)
        .semantic_id_opt(semantic_id)
        .description_opt(Some(format!(
            "{} · {}",
            calculator.operation_name, calculator.words
        )))
        .icon_kind(crate::list_item::IconKind::Svg("code".into()))
        .into_any_element()
}
