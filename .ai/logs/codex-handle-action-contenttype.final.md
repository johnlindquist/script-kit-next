Changed files:
- `src/app_actions/handle_action.rs:347`
- `src/app_actions/handle_action.rs:428`
- `src/app_actions/handle_action.rs:1933`

What I changed:
- Updated all 3 `match entry.content_type` sites to handle:
  - `clipboard_history::ContentType::Link`
  - `clipboard_history::ContentType::File`
  - `clipboard_history::ContentType::Color`
- In all three places, those variants now follow the same behavior as `Text` (as requested).

How to test:
1. Run `cargo check`

Verification run:
- Executed `cargo check` in `/Users/johnlindquist/dev/script-kit-gpui`
- Result: passed (warnings only, no errors)

Risks / known gaps:
- Only compile verification was run; no targeted runtime/UI behavior test was executed for these paths.
- Repo has existing unrelated warnings during `cargo check`.

Commits:
- None made.