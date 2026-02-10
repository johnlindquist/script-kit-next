Changed files:
- `src/app_execute/builtin_execution.rs:1201`
- `src/app_render/group_header_item.rs:39`

What I changed:
- Added `BuiltInFeature::KitStoreCommand(cmd_type)` arm in `execute_builtin` with logging + toast placeholder handling for:
  - `BrowseKits`
  - `InstalledKits`
  - `UpdateAllKits`
- Added `BuiltInFeature::KitStoreCommand(_)` arm in `builtin_feature_annotation` returning `"Kit Store"`.

How to test:
1. Run `cargo check --bin script-kit-gpui`.
2. Confirm there are no non-exhaustive-match errors for:
   - `src/app_execute/builtin_execution.rs`
   - `src/app_render/group_header_item.rs`

Verification result:
- `cargo check --bin script-kit-gpui` fails due unrelated pre-existing errors in `src/app_actions/handle_action.rs:347`, `src/app_actions/handle_action.rs:425`, and `src/app_actions/handle_action.rs:1927` (non-exhaustive `ContentType` matches).
- The targeted `KitStoreCommand` exhaustiveness errors are resolved.

Risks / known gaps:
- `KitStoreCommand` execution is currently placeholder toast behavior; it does not yet open the Kit Store UI flows.
- No commit was made in this run.