# Task: Audit Icon-Only Buttons

Find and fix icon buttons (buttons with just an icon, no text):

1. Search for icon buttons in the codebase
2. Ensure they have:
   - cursor_pointer()
   - Proper hover background (often circular or rounded)
   - Adequate click target size (at least 32x32 px)
   - Tooltip or aria-label for accessibility

Icon buttons often lack hover states or proper cursors. Find and fix these issues.

Look in toolbars, headers, list items for icon buttons.
