# Task: Audit Button Colors for Light Theme

Check all button implementations for proper light theme support:

1. Find the theme system (likely `src/theme/` or `theme.rs`)
2. Check how button colors are defined for light vs dark themes
3. Ensure buttons have sufficient contrast in light theme
4. Verify hover and active states work in light theme

Button colors should:
- Use theme tokens, not hardcoded colors
- Have proper contrast ratios for accessibility
- Have distinct hover/active/disabled states

Fix any buttons using hardcoded dark-theme-only colors.
