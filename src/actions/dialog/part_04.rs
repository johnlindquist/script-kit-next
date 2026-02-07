impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        include!("part_04/body_part_01.rs");
        include!("part_04/body_part_02.rs");
        include!("part_04/body_part_03.rs")
    }
}
