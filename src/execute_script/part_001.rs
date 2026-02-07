impl ScriptListApp {
    fn execute_interactive(&mut self, script: &scripts::Script, cx: &mut Context<Self>) {
        include!("part_001_body/execute_interactive_merged.rs");
    }
}
