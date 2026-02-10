**Findings**
1. **Fixed:** `LayoutComponentType::ListItem` serialized as `"listitem"` in Rust, but TypeScript expects `"listItem"`.  
   - Root cause: `#[serde(rename_all = "lowercase")]` on `LayoutComponentType`.
   - Fix: switched enum serde casing to camelCase and added legacy alias for backward compatibility (`"listitem"` still deserializes).

2. **No other high-confidence type mismatches found** in `src/protocol/types/**/*.rs` for naming, serde attributes, or Option-vs-required field usage against current `scripts/kit-sdk.ts` payload shapes.

**Code Changes**
- Updated enum casing + backward-compatible alias in `src/protocol/types/grid_layout.rs:300`.
- Added protocol compatibility tests in `src/protocol/types/tests/api_consistency.rs:1`.
- Registered new test module in `src/protocol/types/tests/mod.rs:3`.

**How To Test**
1. `cargo test protocol::types::tests::api_consistency`
2. `cargo test protocol::types::tests::`

**Changed Files**
- `src/protocol/types/grid_layout.rs`
- `src/protocol/types/tests/api_consistency.rs`
- `src/protocol/types/tests/mod.rs`

**Risks / Known Gaps**
- Verification was intentionally scoped to protocol-types tests (parallel-agent safe), not full workspace check/clippy/test.
- I did not change files outside the assigned scope.

**Commits**
- `df4bbb7` `fix(protocol-types): align layout component enum casing`