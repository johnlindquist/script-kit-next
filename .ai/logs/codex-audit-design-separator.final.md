**Findings**
1. Fixed: Separator style presets encoded raw hex values instead of semantic color roles, which made token-level consistency hard to enforce.  
`src/designs/separator_variations/part_01.rs:150`, `src/designs/separator_variations/part_03.rs:120`

2. Fixed: Recommendation coverage and redundancy were not directly inspectable, so catalog-only and potentially redundant variants were easy to miss in audits.  
`src/designs/separator_variations/part_04.rs:109`, `src/designs/separator_variations/part_04.rs:123`, `src/designs/separator_variations/part_05.rs:202`

3. Open gap (not changed in this task scope): design gallery counts separator variants but its renderer currently builds only group-header and icon items, so separator variations are not visibly audited in the gallery UI.  
`src/app_impl/ui_window.rs:107`, `src/prompt_handler/part_001.rs:929`, `src/render_builtins/design_gallery.rs:9`

**Changed Files**
- `src/designs/separator_variations/part_01.rs`
- `src/designs/separator_variations/part_03.rs`
- `src/designs/separator_variations/part_04.rs`
- `src/designs/separator_variations/part_05.rs`

**How To Test**
1. `cargo test designs::separator_variations::parts::tests`
2. `cargo check --lib`
3. `cargo clippy --lib -- -D warnings`

**Commits**
- `369c0c0` `refactor(separator-variations): model color presets as roles` (pushed to `main`)

**Risks / Known Gaps**
- `SeparatorColorRole` currently provides fallback hex values; actual per-theme runtime mapping still depends on future renderer integration.
- Separator variants remain partially “catalog-only” unless gallery rendering is extended to include them (outside this scope).
