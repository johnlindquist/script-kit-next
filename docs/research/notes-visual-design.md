# Notes Window Visual Design Research

Date: 2026-02-01
Scope: Visual styling patterns for notes windows (transparency and vibrancy, color, typography), plus 2024 trend signals.

## 1) Trend signals (2024) relevant to notes windows
- Bold typography, dark mode, and glassmorphism are highlighted as 2024 UI trends. This implies notes windows can lean on strong type hierarchy, optional dark themes, and subtle frosted layers for depth. (ANODA 2024 UI trends)
- Minimalist design is still called out as a dominant approach in 2024. This aligns with content-first notes windows that reduce chrome and prioritize writing space. (ANODA 2024 UI trends)

## 2) Transparency and vibrancy patterns (desktop materials)
- Fluent materials define Acrylic (semi-transparent, frosted glass) and recommend it for transient, light-dismiss surfaces like popovers and menus. This suggests using translucency mainly for overlays, headers, or panels, not for full-page backgrounds. (Fluent Material)
- Acrylic guidance stresses:
  - Use acrylic for transient UI and supporting surfaces that overlap content.
  - Avoid large background surfaces and avoid placing multiple acrylic panes edge-to-edge (visible seams).
  - Avoid accent-colored text on acrylic and keep contrast in mind.
  - Provide fallbacks: transparency can be disabled by the system (high contrast mode, battery saver, or user settings). (Microsoft Learn Acrylic)
- Apple’s design guidance updates in 2025 emphasize Liquid Glass across multiple UI areas, indicating ongoing platform momentum toward glass-like materials and translucency. (Apple Design What’s New)

## 3) Color and contrast basics
- WCAG 2.1 contrast minimum: 4.5:1 for normal text and 3:1 for large text. Notes windows should meet these ratios for body text, metadata, and placeholder text. (W3C Contrast Minimum)
- Apple design tips emphasize legible text size (at least 11 pt), ample contrast, and improved legibility via line height or letter spacing. (Apple UI Design Tips)

## 4) Typography for readable notes
- San Francisco (SF Pro) is the system font for Apple platforms and is designed for legibility with size-specific outlines and dynamic tracking. Using SF Pro aligns with macOS native typography. (Apple Fonts)
- USWDS recommends line length around 45 to 90 characters (target ~66) for long-form readability, and line height of at least 1.5 for longer text. (USWDS Typography)

## 5) Layout and density patterns for notes windows
- Content-first layout: keep the editor dominant and use lightweight UI chrome (thin dividers, subtle icons, minimal toolbars).
- Use whitespace to separate sections (title, metadata, body) rather than heavy borders.
- Keep navigation elements (list of notes, tags, search) visually secondary to the editor.
- Use typography (size, weight, spacing) for hierarchy before color.

## 6) Practical styling heuristics for Script Kit notes
- Use one translucent layer at a time (ex: header or sidebar). Avoid stacking multiple blurred layers.
- Keep translucency subtle and tinted; ensure text contrast still meets WCAG ratios.
- Provide an opaque fallback when transparency is disabled or when content has low contrast.
- Default to neutral base colors; reserve accent colors for primary actions and selection.
- Body text targets: 14-16px (or 11-12pt) with line height 1.4-1.6; keep text measure near 60-75 characters per line.
- Prefer SF Pro for body and UI text; use bold or semibold weights for headings.

## Sources
- Apple UI Design Tips (UI Design Dos and Don’ts)
  https://developer.apple.com/design/tips/
- Apple Fonts (San Francisco / SF Pro)
  https://developer.apple.com/fonts/
- USWDS Typography (line length, line height)
  https://designsystem.digital.gov/components/typography/
- W3C WCAG 2.1 Contrast Minimum
  https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html
- Fluent 2 Design System - Material
  https://fluent2.microsoft.design/material
- Microsoft Learn - Acrylic Material
  https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic
- Apple Design What’s New (Liquid Glass guidance updates)
  https://developer.apple.com/design/whats-new/
- ANODA UI Design Trends 2024
  https://www.anoda.mobi/ux-blog/top-ui-design-trends-for-2024-standout-user-interface-practices
