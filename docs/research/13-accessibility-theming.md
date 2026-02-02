# AI Chat Accessibility and Theming Research

Date: 2026-02-01

## Scope
Research notes for the AI chat window: accessibility best practices, theme support, contrast, focus management, and concrete suggestions.

## A11y best practices for chat UIs
- Use a live region for the conversation stream. The WAI-ARIA technique for sequential updates recommends `role="log"` for chat history so new messages are announced without moving focus; `role="log"` has implicit `aria-live="polite"` and `aria-atomic="false"`. The technique explicitly uses chat as an example. (W3C ARIA23)
  - MDN notes `role="log"` requires an accessible name (use `aria-label` or `aria-labelledby`) and confirms the implicit `aria-live`/`aria-atomic` defaults. (MDN log role)
  - For broader AT compatibility, MDN recommends adding a redundant `aria-live="polite"` when using `role="log"`. (MDN live regions)
- Use status messages for transient system updates that should be announced without stealing focus (e.g., "Assistant is typing", "Message sent"). WCAG 4.1.3 defines status messages as results/progress/waiting/error updates that do not change context. (W3C Understanding 4.1.3)
  - `role="status"` is a polite live region with implicit `aria-atomic="true"`, and MDN explicitly says not to move focus when it updates. (MDN status role)

## Theme support and system accessibility settings
- Support light/dark themes and allow system-driven toggles. Apple notes that dark interfaces often reduce contrast; test dark mode together with the system Increase Contrast setting. (Apple Dark Interface evaluation criteria)
- Respect system accessibility settings that improve readability on macOS: Increase Contrast (adds borders/definition) and Reduce Transparency (replaces transparency with solid backgrounds). (Apple Support - Display accessibility settings)
- For apps with translucent surfaces, Apple recommends considering reduced translucency when Increase Contrast or Reduce Transparency are enabled. (Apple Dark Interface evaluation criteria)
- Provide non-color affordances for state changes and status (e.g., icon shape, outline, underline), aligning with "Differentiate without color" in macOS accessibility settings. (Apple Support - Display accessibility settings)

## Contrast requirements (text + UI)
- Text contrast: WCAG SC 1.4.3 requires at least 4.5:1 for normal text and 3:1 for large text (14pt bold / 18pt regular or larger). (W3C Understanding 1.4.3)
  - Apply this to message text, timestamps, placeholders, and any text shown on hover/focus. (W3C Understanding 1.4.3)
- Non-text contrast: WCAG SC 1.4.11 requires a 3:1 contrast ratio for UI components and focus indicators against adjacent colors. (W3C Understanding 1.4.11)
  - This includes focus rings, selection highlights, and icon-only controls that convey state. (W3C Understanding 1.4.11)
- Apple notes that "Sufficient Contrast" typically means 4.5:1 for most text, and recommends supporting Increase Contrast if relying on system UI, or providing alternate color schemes if using custom UI. (Apple Sufficient Contrast evaluation criteria)

## Focus management (modal vs non-modal)
- If the chat window is modal:
  - Move focus into the dialog on open, typically to the first focusable element; if needed for long content, focus a static element at the top via `tabindex="-1"`. (WAI-ARIA APG Dialog Modal Pattern)
  - Keep focus trapped inside the dialog; Tab/Shift+Tab should cycle within it. (WAI-ARIA APG Dialog Modal Pattern)
  - On close, return focus to the element that opened the dialog unless there is a logical workflow reason to choose a different target. (WAI-ARIA APG Dialog Modal Pattern)
  - Provide a visible close button in the tab order. (WAI-ARIA APG Dialog Modal Pattern)
  - Use `aria-modal="true"` only when the background is actually inert and visually obscured. (WAI-ARIA APG Dialog Modal Pattern)

## Suggestions for our AI chat window (Script Kit GPUI)
1. Conversation stream semantics
   - Mark the message list with `role="log"` and an accessible name ("Conversation", "Chat history"). Add `aria-live="polite"` explicitly for compatibility.
   - Append messages to the end of the log so AT announces new content predictably (matches ARIA23 chat example).

2. Status updates without focus theft
   - Use a separate `role="status"` region for ephemeral updates (typing, streaming start/finish, errors) and ensure focus stays in the input.
   - Avoid re-focusing the message list when new content arrives; provide a shortcut/button to jump to latest if desired.

3. Contrast and focus indicators
   - Validate all text against 4.5:1 (3:1 for large text), including timestamps and placeholders.
   - Ensure selection, focus rings, and icon-only controls hit 3:1 against adjacent colors.
   - When theming via transparency/vibrancy, verify contrast in both default and Increase Contrast / Reduce Transparency modes.

4. Theme + accessibility settings
   - Respect OS-level Reduce Transparency by switching to solid backgrounds and stronger borders.
   - If Increase Contrast is enabled, bump border/outline contrast and ensure icons remain distinct.
   - Provide a high-contrast theme variant if custom colors are used (avoid relying solely on system frameworks).

5. Dialog behavior (if chat is modal)
   - Trap focus, support Escape to close, and restore focus to the invoker on close.
   - If initial focus on the input would scroll important content out of view, focus a static heading first.

## Quick verification checklist
- Screen reader announces new messages in the log without stealing focus.
- Status messages announce politely (typing/progress/error) and do not move focus.
- Keyboard-only: input, actions, and close button are reachable; focus stays in modal.
- Contrast checks: text >= 4.5:1 (or 3:1 large), UI/focus indicators >= 3:1.
- macOS accessibility settings: Increase Contrast and Reduce Transparency produce usable visuals.

## References
- W3C ARIA23: Using `role=log` for sequential updates (chat example): https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA23
- MDN: `role=log` (implicit `aria-live`, accessible name): https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/log_role
- MDN: ARIA live regions (compatibility note for `aria-live`): https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Live_Regions
- MDN: `role=status` (polite live region, do not move focus): https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/status_role
- W3C Understanding 4.1.3 Status Messages: https://w3c.github.io/wcag21/understanding/status-messages
- W3C Understanding 1.4.3 Contrast (Minimum): https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum
- WAI-ARIA Authoring Practices: Dialog (Modal) Pattern: https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/
- W3C Understanding 1.4.11 Non-text Contrast: https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast
- Apple Support: Display accessibility settings (Increase Contrast / Reduce Transparency / Differentiate without color): https://support.apple.com/en-bw/guide/mac-help/-unac089/mac
- Apple Developer: Dark Interface accessibility evaluation criteria: https://developer.apple.com/help/app-store-connect/manage-app-accessibility/dark-interface-accessibility-evaluation-criteria
- Apple Developer: Sufficient Contrast evaluation criteria: https://developer.apple.com/help/app-store-connect/manage-app-accessibility/sufficient-contrast-evaluation-criteria/
