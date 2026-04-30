//! Source-level structural contract: `ActionsDialog`'s Enter-key
//! routing MUST preserve the "drill-down-before-execute" ordering
//! AND the three-outcome contract of `ActionsDialogActivation`.
//!
//! # Why this matters (UX)
//!
//! Enter on the shared `ActionsDialog` at `src/actions/dialog.rs` is
//! the third-most-fired user gesture on every Cmd+K host (after
//! typing and arrow navigation). The routing inside
//! `activate_selected` (line 1175) + `activate_action_id` (line 1189)
//! must return EXACTLY ONE of three outcomes per press:
//!
//!   - `NoSelection` — header row or empty filter. `get_selected_action_id`
//!     returns `None`, caller decides what to do (typically no-op).
//!   - `DrillDownPushed { action_id, route_id }` — the selected action
//!     has a registered drill-down route. `push_route(route, cx)` is
//!     called to open the child sub-menu; the `on_select` callback is
//!     NOT fired. This is the ACP agent-switch path.
//!   - `Executed { action_id, should_close }` — the selected action is
//!     terminal. `on_select(action_id)` fires exactly once; caller
//!     decides whether to close based on `should_close`.
//!
//! The regression this contract blocks is SUBTLE: a contributor
//! "simplifying" `activate_action_id` to always call
//! `(self.on_select)(action_id.clone())` first, then check
//! `drill_down_routes` afterward, double-fires on drill-down actions
//! — the parent's `on_select` callback runs AND the child route
//! pushes. The ACP agent-switch handler, for example, would persist
//! the partial "switch agent" selection AND open the agent-list
//! child, leaving the user in an inconsistent state where a selection
//! has been committed to the upstream store but the UI still expects
//! them to pick an agent. Even worse, the `Executed { should_close }`
//! return value would then evaluate on a mutation that shouldn't
//! have happened — the caller might close the dialog entirely and
//! the child route never renders.
//!
//! A mirror-image regression at the enum level: dropping the
//! `DrillDownPushed` variant (or merging it into `Executed` via a
//! `bool is_drill_down` flag) — compiles if every match-arm is
//! updated in one commit, but the Pass #11 escape-filter-agnostic
//! contract's `PoppedRoute` analog has already demonstrated that
//! enum-level simplifications of Cmd+K routing state are a real
//! refactor threat.
//!
//! Pass #11 pinned escape outcomes (2 variants). Pass #15 pinned
//! route-stack push/pop. Pass #16 pins the Enter-key half of the
//! activation round-trip — together with #11, every user-visible
//! terminal gesture on the shared `ActionsDialog` (Enter, Escape)
//! now has a structural contract that blocks the same class of
//! "simplification" refactor.
//!
//! # Anchors
//!
//!   (1) `activate_selected` body MUST call `get_selected_action_id(`
//!       AND early-return `NoSelection` — header rows and empty
//!       filters must NOT silently fall through into
//!       `activate_action_id` with an invented id.
//!   (2) `activate_action_id` body MUST check
//!       `drill_down_routes.get(&action_id)` BEFORE calling
//!       `(self.on_select)(` — the byte-offset ordering anchor.
//!       Reversing the order double-fires on drill-down actions.
//!   (3) `activate_action_id` body MUST call `self.push_route(`
//!       inside the drill-down arm — otherwise the child route
//!       never opens even though `DrillDownPushed` is returned.
//!   (4) `activate_action_id` body MUST return
//!       `ActionsDialogActivation::DrillDownPushed` in the
//!       drill-down arm AND `ActionsDialogActivation::Executed`
//!       in the terminal arm — distinct variants so callers can
//!       decide whether to close.
//!   (5) `ActionsDialogActivation` enum MUST declare exactly three
//!       variants: `DrillDownPushed`, `Executed`, `NoSelection`.
//!       A new variant like `Failed` or `Canceled` is the shape
//!       this regression would take at the enum level — adding
//!       one requires updating this test AND the rustdoc on the
//!       enum so new outcomes are not introduced silently.

const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");

fn extract_fn_body<'a>(src: &'a str, header: &str) -> &'a str {
    let header_pos = src.find(header).unwrap_or_else(|| {
        panic!(
            "source: `{header}` not found — the method may have been renamed. \
             Update the contract anchors in \
             tests/actions_dialog_enter_routing_contract.rs."
        )
    });
    let open_rel = src[header_pos..]
        .find('{')
        .unwrap_or_else(|| panic!("no `{{` after `{header}` header"));
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in src.as_bytes()[open_abs..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open_abs + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_abs = close_abs.unwrap_or_else(|| panic!("no matching `}}` for `{header}` body"));
    &src[open_abs..=close_abs]
}

#[test]
fn activate_selected_returns_no_selection_on_empty_or_header_row() {
    // `get_selected_action_id()` returns `None` for section headers
    // (Pass #13 / #14 domain) and for empty filtered lists. Without
    // the early-return, `activate_action_id` would be called with
    // an invented empty string id, the `drill_down_routes` lookup
    // would miss, and `on_select("")` would fire — the consumer's
    // `on_select` is usually a match on action IDs, so it'd silently
    // no-op OR panic depending on how strict the consumer is.
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn activate_selected(");
    assert!(
        body.contains("get_selected_action_id("),
        "`activate_selected` MUST call `get_selected_action_id()` \
         to guard against header-row / empty-list activation. Body was:\n{body}"
    );
    assert!(
        body.contains("ActionsDialogActivation::NoSelection"),
        "`activate_selected` MUST return \
         `ActionsDialogActivation::NoSelection` when the selector \
         yields `None`. Without this arm, header-row Enter would \
         fall through to `activate_action_id` with an invented id \
         and fire the consumer's `on_select(\"\")`. Body was:\n{body}"
    );
}

#[test]
fn activate_action_id_checks_drill_down_before_calling_on_select() {
    // The critical precedence anchor. Byte-offset of the
    // `drill_down_routes.get(` lookup MUST be < byte-offset of the
    // `(self.on_select)(` call. If the order is reversed, every
    // drill-down action double-fires — consumer's `on_select`
    // commits a partial selection to upstream state AND the child
    // route pushes. The symmetric negative pin at the return-value
    // level (Executed vs DrillDownPushed) can't catch this because
    // a "naïve" reorder would still return `DrillDownPushed`, it'd
    // just have fired `on_select` as a side effect first.
    let body = extract_fn_body(DIALOG_SOURCE, "pub(crate) fn activate_action_id(");

    let drill_idx = body.find("drill_down_routes.get(").unwrap_or_else(|| {
        panic!(
            "`activate_action_id` MUST look up `drill_down_routes.get(&action_id)` \
             before deciding between drill-down and execute. \
             Body was:\n{body}"
        )
    });
    let on_select_idx = body.find("(self.on_select)(").unwrap_or_else(|| {
        panic!(
            "`activate_action_id` MUST call `(self.on_select)(action_id.clone())` \
             in the terminal (execute) arm. Body was:\n{body}"
        )
    });
    assert!(
        drill_idx < on_select_idx,
        "`activate_action_id`: `drill_down_routes.get(` lookup MUST \
         appear BEFORE `(self.on_select)(` call in source order. \
         Reversing double-fires on every drill-down action — the \
         consumer's `on_select` commits a partial selection (e.g., \
         ACP agent-switch writes to upstream store) AND `push_route` \
         opens the child sub-menu, leaving the user with an already- \
         committed selection under a sub-menu that expects them to \
         choose. Byte-offsets: drill_down_routes.get = {drill_idx}, \
         (self.on_select)( = {on_select_idx}. Body was:\n{body}"
    );
}

#[test]
fn activate_action_id_pushes_route_in_drill_down_arm() {
    // Without `push_route`, the drill-down arm would return
    // `DrillDownPushed { route_id }` to the caller but never
    // actually open the child sub-menu. The caller typically closes
    // the dialog on `DrillDownPushed` (ACP switch flow) or keeps it
    // open awaiting the child render — either way the user sees
    // nothing happen. This anchor also ties `activate_action_id`
    // to the Pass #15 route-stack contract: any regression of
    // `push_route` semantics (snapshot ordering, apply_route_state_*
    // wiring) cascades through here.
    let body = extract_fn_body(DIALOG_SOURCE, "pub(crate) fn activate_action_id(");
    assert!(
        body.contains("self.push_route("),
        "`activate_action_id` MUST call `self.push_route(route, cx)` \
         in the drill-down arm. Returning `DrillDownPushed` without \
         actually pushing the route opens no child sub-menu — the \
         user sees nothing happen on Enter. Body was:\n{body}"
    );
}

#[test]
fn activate_action_id_returns_distinct_drill_down_and_executed_variants() {
    // Both variants must appear in the body so callers can
    // disambiguate. Merging them into a single `Activated { ... }`
    // with a boolean flag is the shape the enum-level regression
    // takes — callers would need to inspect the payload instead of
    // match on the variant, losing the type-level signal that
    // drill-down and execute are distinct outcomes.
    let body = extract_fn_body(DIALOG_SOURCE, "pub(crate) fn activate_action_id(");
    assert!(
        body.contains("ActionsDialogActivation::DrillDownPushed"),
        "`activate_action_id` MUST return \
         `ActionsDialogActivation::DrillDownPushed` in the \
         drill-down arm. Body was:\n{body}"
    );
    assert!(
        body.contains("ActionsDialogActivation::Executed"),
        "`activate_action_id` MUST return \
         `ActionsDialogActivation::Executed` in the terminal arm. \
         Body was:\n{body}"
    );
}

#[test]
fn actions_dialog_activation_has_exactly_three_variants() {
    // Mirror of the Pass #11
    // `actions_dialog_escape_outcome_has_exactly_two_variants` test
    // but for the Enter side. A new variant like `Failed` or
    // `Canceled` or `Deferred` is the shape this regression takes
    // at the enum level even if the body anchors above pass. Adding
    // a fourth outcome requires updating this test AND the rustdoc
    // at `src/actions/dialog.rs:380` so new outcomes are not
    // introduced silently.
    // Struct-like variants (`DrillDownPushed { action_id: String, ... }`)
    // introduce inner `{ ... }` pairs, so a naïve `.find('}')` would
    // match the first inner close-brace instead of the enum's. Use
    // a brace-counter from the enum's opening `{` instead.
    let marker = "pub enum ActionsDialogActivation {";
    let start = DIALOG_SOURCE.find(marker).expect(
        "pub enum ActionsDialogActivation not found in src/actions/dialog.rs — \
         the enum may have been renamed. Update the contract anchors.",
    );
    let after_brace = start + marker.len();
    let mut depth: i32 = 1;
    let mut close_rel: Option<usize> = None;
    for (offset, &b) in DIALOG_SOURCE.as_bytes()[after_brace..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_rel = Some(offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_rel = close_rel.expect("no matching `}` for ActionsDialogActivation enum");
    let enum_body = &DIALOG_SOURCE[after_brace..after_brace + close_rel];

    assert!(
        enum_body.contains("DrillDownPushed"),
        "ActionsDialogActivation must declare `DrillDownPushed`. Body was:\n{enum_body}"
    );
    assert!(
        enum_body.contains("Executed"),
        "ActionsDialogActivation must declare `Executed`. Body was:\n{enum_body}"
    );
    assert!(
        enum_body.contains("NoSelection"),
        "ActionsDialogActivation must declare `NoSelection`. Body was:\n{enum_body}"
    );

    // Count variant headers — lines that start with an uppercase
    // identifier and end with either `{` (struct-like variant),
    // `,` (unit variant), or `(` (tuple variant). Comments and
    // whitespace are skipped. The rustfmt-canonical shape for
    // struct-variants splits across multiple lines, so count
    // opening-line occurrences only.
    let variant_count = enum_body
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                return false;
            }
            // Variant header: an identifier followed by `{`, `(`, or a
            // trailing `,`. Exclude inner struct fields like `action_id: String,`.
            let first_token = trimmed
                .split(|c: char| c == ' ' || c == '{' || c == '(' || c == ',')
                .next()
                .unwrap_or("");
            // Variant names start with an uppercase letter; inner fields are snake_case.
            first_token
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
                && (trimmed.ends_with('{') || trimmed.ends_with('(') || trimmed.ends_with(','))
        })
        .count();

    assert_eq!(
        variant_count, 3,
        "ActionsDialogActivation must have EXACTLY 3 variants \
         (`DrillDownPushed`, `Executed`, `NoSelection`). A new \
         variant like `Failed` / `Canceled` / `Deferred` is the \
         shape the regression this contract blocks would take at \
         the enum level. Adding a fourth outcome requires updating \
         this test AND the documentation at \
         `src/actions/dialog.rs:380`. Found {variant_count} variants. \
         Body was:\n{enum_body}"
    );
}
