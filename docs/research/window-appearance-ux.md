# Window Appearance and Visual Design Research for Launcher Apps

This document consolidates research on window appearance, visual design patterns, and UX best practices for launcher applications like Script Kit, with references to industry leaders like Raycast, Alfred, and macOS Spotlight.

---

## Table of Contents

1. [Window Shadows, Borders, and Blur Effects](#1-window-shadows-borders-and-blur-effects)
2. [Dark Mode and Light Mode Considerations](#2-dark-mode-and-light-mode-considerations)
3. [Typography and Spacing Best Practices](#3-typography-and-spacing-best-practices)
4. [Visual Hierarchy Patterns](#4-visual-hierarchy-patterns)
5. [Recommendations for Script Kit](#5-recommendations-for-script-kit)

---

## 1. Window Shadows, Borders, and Blur Effects

### Glassmorphism: The Dominant Trend (2024-2026)

[Glassmorphism](https://yellowslice.in/bed/glassmorphism-in-user-interfaces/) is the current dominant visual style for launcher apps, combining:

- **Transparency**: Semi-transparent backgrounds that reveal content beneath
- **Background blur**: Frosted glass effect (typically 4-6px blur radius)
- **Soft layering**: Multiple depth levels creating dimensional interfaces
- **Subtle borders**: Semi-transparent white borders to define edges

Key implementations:
- Apple Vision Pro (2024) heavily utilizes glassmorphism throughout its UI
- Samsung One UI 7 features frosted glass textures with gradient effects
- Apple's "Liquid Glass" design language (WWDC 2025) introduces glossy textures and dynamic lighting

### Shadow Best Practices

According to [Smashing Magazine](https://www.smashingmagazine.com/2017/02/shadows-blur-effects-user-interface-design/) and [LogRocket](https://blog.logrocket.com/ux-design/shadows-ui-design-tips-best-practices/):

**General Principles:**
- Softer shadows look more elegant and polished
- Lower opacity + higher blur = more realistic shadows
- Consistency is key: align shadow direction using consistent X, Y, blur, and spread values
- Shadows establish visual hierarchy and convey depth

**macOS-Style Window Shadow (CSS):**
```css
/* Filter approach */
filter: drop-shadow(0 25px 45px rgba(0, 0, 0, 0.40))
        drop-shadow(0 0 2px rgba(0, 0, 0, 0.50));

/* Box-shadow approach */
box-shadow:
  inset 0 1px 0 rgba(255, 255, 255, 0.6),
  0 22px 70px 4px rgba(0, 0, 0, 0.56),
  0 0 0 1px rgba(0, 0, 0, 0.0);
```

**Elevation-Based Shadows:**
```css
/* Navbar - low elevation */
.navbar { box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1); }

/* Dropdown - medium elevation */
.dropdown { box-shadow: 0 5px 20px rgba(0, 0, 0, 0.15); }

/* Modal/Launcher - high elevation */
.modal { box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3); }
```

### Background Blur and Vibrancy

From [Apple's NSVisualEffectView documentation](https://developer.apple.com/documentation/appkit/nsvisualeffectview) and [CreateWithPlay](https://createwithplay.com/blog/best-practices-for-using-materials-properties):

**Material Thickness Options:**
- **Ultra Thin**: Maximum background visibility, useful when context is helpful
- **Thin**: Light blur, good for secondary surfaces
- **Regular**: Balanced blur for most use cases
- **Thick**: Strong blur, less background visibility
- **Chrome/Ultra Thick**: Minimal background visibility

**Vibrancy Levels:**
- **Primary**: Most colors brought through (use for main text)
- **Secondary**: Moderate vibrancy
- **Tertiary**: Less vibrancy
- **Divider**: Minimal vibrancy

**Best Practices:**
- Start with low blur values (4-6px) - larger values look muddy and tax GPU
- Use glass effects sparingly on key elements only (navigation, primary buttons, important cards)
- Colorful images/icons should remain unmodified (no vibrancy)
- Performance consideration: heavy blur effects can cause lag on older hardware

### Border Radius and Rounded Corners

From [Medium](https://medium.com/design-bootcamp/building-a-consistent-corner-radius-system-in-ui-1f86eed56dd3) and [Microsoft Fluent 2](https://fluent2.microsoft.design/shapes):

**Semantic Corner Radius Scale:**
```
XS: 2px  - Small components (<32px)
S:  4px  - Default controls (WinUI default)
M:  8px  - Overlays, cards (WinUI OverlayCornerRadius)
L:  12px - Large panels
XL: 16px - Major containers
```

**Guidelines:**
- Small radii = formal, enterprise, business-oriented
- Large radii = friendly, cozy, informal
- Use bigger radii for bigger shapes (maintain visual consistency)
- Google Play icons: 30% of icon size for corner radius

---

## 2. Dark Mode and Light Mode Considerations

### Color Selection

From [DubBot](https://dubbot.com/dubblog/2023/dark-mode-a11y.html) and [Graphic Eagle](https://www.graphiceagle.com/dark-mode-ui/):

**Avoid Pure Black and White:**
- Pure black (#000000) causes eye strain and "halation effect"
- Pure white (#FFFFFF) is too harsh in dark interfaces
- Use softer shades: dark grays instead of pure black

**Recommended Dark Mode Palette:**
```
Background:     #1A1A1A to #2D2D2D (soft black)
Surface:        #2D2D2D to #3D3D3D (elevated surfaces)
Primary Text:   #E8E8E8 to #F0F0F0 (soft white)
Secondary Text: #A0A0A0 to #B0B0B0 (muted)
Accent:         Brand color at reduced saturation
```

**Avoid Saturated Colors:**
- High saturation is harsh on eyes in dark mode
- Saturated colors often fail WCAG contrast requirements
- Desaturate accent colors for dark mode variants

### WCAG Contrast Requirements

From [W3C WCAG](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html) and [WebAIM](https://webaim.org/articles/contrast/):

**Minimum Contrast Ratios (Level AA):**
| Element Type | Minimum Ratio |
|-------------|---------------|
| Normal text (<18pt) | 4.5:1 |
| Large text (>=18pt or >=14pt bold) | 3:1 |
| UI components & graphics | 3:1 |

**Enhanced Contrast (Level AAA):**
| Element Type | Minimum Ratio |
|-------------|---------------|
| Normal text | 7:1 |
| Large text | 4.5:1 |

### Raycast vs Alfred: Theme Approaches

From [Medium comparison](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89):

**Raycast:**
- Modern, sleek interface with smooth animations
- Signature purple accent chosen for perceived speed correlation
- Supports light/dark themes with system appearance adoption
- Hundreds of community themes available
- Animations provide visual feedback without feeling slow

**Alfred:**
- Utilitarian design (looks like 2012)
- Clean but dated interface
- Supports theming and light/dark switching
- Raycast-style themes available for Alfred

---

## 3. Typography and Spacing Best Practices

### Font Size Guidelines

From [Learn UI Design](https://www.learnui.design/blog/ultimate-guide-font-sizes-ui-design.html) and [b13](https://b13.com/blog/designing-with-type-a-guide-to-ui-font-size-guidelines):

**Recommended Sizes:**
| Element | iOS/macOS | Android | Web |
|---------|-----------|---------|-----|
| Body text | 17pt min | 16sp | 16px min |
| Titles | 20-24pt | 20sp | 20-24px |
| Captions | 12-14pt | 12-14sp | 12-14px |
| Headlines | 28-34pt | 24-34sp | 28-34px |

**Key Principles:**
- Limit to ~4 font sizes per screen (avoid size proliferation)
- Use predictable scale ratios (1.125, 1.25, 1.333)
- Test on actual devices, not just design tools
- System fonts (San Francisco) optimize legibility automatically

### San Francisco Font System

From [Apple Developer](https://developer.apple.com/fonts/):

**Font Variants:**
- **SF Pro**: macOS, iOS, iPadOS - nine weights with italics
- **SF Compact**: watchOS - optimized for small screens
- **SF Mono**: Terminal, Xcode - six weights with italics

**Optical Sizes:**
- **SF Text**: Below 20pt (tighter tracking, larger x-height)
- **SF Display**: 20pt and above (refined details for large sizes)

**Usage:**
```css
/* Web */
font-family: -apple-system, BlinkMacSystemFont, "San Francisco", system-ui, sans-serif;
```

### Spacing System

From [UX Planet](https://uxplanet.org/principles-of-spacing-in-ui-design-a-beginners-guide-to-the-4-point-spacing-system-6e88233b527a) and [Canva](https://www.canva.dev/docs/apps/design-guidelines/spacing/):

**4-Point/8-Point Grid:**
```
4px  - Tight spacing (related elements)
8px  - Default small spacing
12px - Medium spacing
16px - Section spacing, default padding
24px - Large section spacing
32px - Major separations
48px - Touch target minimum
```

**List Item Recommendations:**
- Item padding: 12-16px
- Between items: 8px
- Card spacing: 16-24px
- Edge padding: 16px minimum

**Search Bar Specific:**
- Above/below search bar: 16px
- Within sections: 8px
- Major sections: 16px

---

## 4. Visual Hierarchy Patterns

### Command Palette Design

From [Philip C. Davis](https://philipcdavis.com/writing/command-palette-interfaces) and [Destiner](https://destiner.io/blog/post/designing-a-command-palette/):

**Core Characteristics:**
- Keyboard-first interaction (mouse/trackpad optional)
- Fuzzy search for forgiving input matching
- Grouped and nested navigation
- Fast, memorable activation shortcut (Cmd+K, Cmd+Shift+P)

**Design Best Practices:**
- Don't be too strict about exact command names
- Use fuzzy search to handle typos
- Group related commands logically
- Don't include every setting (link to preferences instead)
- Consistent shortcut display in results

### Keyboard Shortcut Display

From [Microsoft](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/dnacc/guidelines-for-keyboard-user-interface-design) and [Medium UX Patterns](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1):

**Keycap Design Principles:**
- Visual affordance: make shortcuts look "pressable"
- Consistent styling across the interface
- Include in tooltips and help content
- Follow platform conventions (Cmd on Mac, Ctrl on Windows)

**Accessibility:**
- Keyboard is primary navigation for some users
- Provide discoverable shortcut documentation
- Don't override system-wide shortcuts
- Follow standard conventions (Ctrl+C, Ctrl+V, etc.)

### Animation and Transitions

From [Raycast Animated Window Manager](https://www.raycast.com/matheuschein/animated-window-manager):

**Animation Guidelines:**
- Use `ease-out` for most cases (feels fast and natural)
- Use `ease-in-out` for already-visible elements
- Never use `linear` except for infinite loops
- Prefer transforms and opacity over layout changes
- Respect reduced motion preferences
- Keep animations purposeful, not gratuitous

**Raycast Animation Philosophy:**
- Smooth, polished transitions
- Native-feeling macOS animations
- No jarring jumps
- Visual feedback that feels responsive

---

## 5. Recommendations for Script Kit

Based on the research above, here are specific recommendations for Script Kit's window appearance and visual design:

### Window Container

```
Background: NSVisualEffectMaterial::HudWindow or similar
Blur radius: 6-8px (balance between effect and performance)
Border: 1px rgba(255, 255, 255, 0.1) in dark mode
        1px rgba(0, 0, 0, 0.1) in light mode
Corner radius: 12px (M-L scale, friendly but professional)
Shadow: 0 20px 60px rgba(0, 0, 0, 0.35) (high elevation)
```

### Color System

**Dark Mode:**
```
Window background:  rgba(30, 30, 30, 0.8) + blur
Surface elevated:   rgba(45, 45, 45, 0.9)
Primary text:       #E8E8E8 (contrast 12:1)
Secondary text:     #A8A8A8 (contrast 7:1)
Selected item:      rgba(88, 86, 214, 0.3) (purple tint)
Border subtle:      rgba(255, 255, 255, 0.08)
```

**Light Mode:**
```
Window background:  rgba(255, 255, 255, 0.85) + blur
Surface elevated:   rgba(245, 245, 245, 0.95)
Primary text:       #1A1A1A (contrast 14:1)
Secondary text:     #6B6B6B (contrast 5:1)
Selected item:      rgba(88, 86, 214, 0.15)
Border subtle:      rgba(0, 0, 0, 0.06)
```

### Typography Scale

```
Search input:       17px / 1.4 line-height
List item primary:  15px / 1.3 line-height
List item secondary: 13px / 1.3 line-height
Section headers:    12px / 1.2 line-height (uppercase, tracked)
Keyboard shortcuts: 11px / monospace
```

### Spacing System

```
Window padding:     16px
Search bar height:  48px (touch-friendly)
List item height:   44px minimum
List item padding:  12px horizontal, 8px vertical
Between items:      0px (use borders or backgrounds for separation)
Section spacing:    16px
Icon size:          24px with 12px padding
```

### Animation Recommendations

```
Window appear:      scale(0.96) -> scale(1), opacity 0 -> 1
                    duration: 150ms, ease-out
Window dismiss:     scale(1) -> scale(0.96), opacity 1 -> 0
                    duration: 100ms, ease-in
Selection change:   background-color transition
                    duration: 80ms, ease-out
List scroll:        native smooth scrolling
Search typing:      instant, no delay
```

### Accessibility Checklist

- [ ] All text meets 4.5:1 contrast ratio minimum
- [ ] UI components meet 3:1 contrast ratio
- [ ] Keyboard navigation fully functional
- [ ] Focus states clearly visible
- [ ] Reduced motion preference respected
- [ ] Screen reader labels for all interactive elements
- [ ] Shortcut hints discoverable

### Performance Considerations

- Limit blur effects to window background only
- Use GPU-accelerated transforms for animations
- Avoid animating layout properties (width, height, top, left)
- Cache rendered list items for smooth scrolling
- Debounce search input (50-100ms)
- Lazy load extension icons

---

## Sources

### Glassmorphism and Visual Effects
- [Yellowslice - Glassmorphism in User Interfaces](https://yellowslice.in/bed/glassmorphism-in-user-interfaces/)
- [EverydayUX - Glassmorphism Apple Liquid Glass](https://www.everydayux.net/glassmorphism-apple-liquid-glass-interface-design/)
- [UXPilot - Glassmorphism UI Features](https://uxpilot.ai/blogs/glassmorphism-ui)
- [Clay - Glassmorphism in UX](https://clay.global/blog/glassmorphism-ui)

### Shadows and Blur
- [Smashing Magazine - Shadows and Blur Effects](https://www.smashingmagazine.com/2017/02/shadows-blur-effects-user-interface-design/)
- [LogRocket - Shadows in UI Design](https://blog.logrocket.com/ux-design/shadows-ui-design-tips-best-practices/)
- [Josh W. Comeau - Designing Beautiful Shadows](https://www.joshwcomeau.com/css/designing-shadows/)
- [CodePen - macOS Window Drop Shadow](https://codepen.io/joeyhoer/pen/beXJzj)

### macOS Vibrancy
- [Apple Developer - NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview)
- [CreateWithPlay - Apple Materials Blur and Vibrancy](https://createwithplay.com/blog/best-practices-for-using-materials-properties)
- [Mackuba - Dark Side of the Mac](https://mackuba.eu/2018/07/04/dark-side-mac-1/)
- [Tauri - Window Vibrancy](https://github.com/tauri-apps/window-vibrancy)

### Launcher App Comparisons
- [Medium - Alfred vs Raycast](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89)
- [Raycast Official](https://www.raycast.com/)
- [Raycast Manual - Settings](https://manual.raycast.com/preferences)
- [Evan Travers - Raycast Review](https://evantravers.com/articles/2023/02/16/raycast-review-as-an-longtime-alfred-user/)

### Typography
- [Learn UI Design - Font Sizes](https://www.learnui.design/blog/ultimate-guide-font-sizes-ui-design.html)
- [b13 - UI Font Size Guidelines](https://b13.com/blog/designing-with-type-a-guide-to-ui-font-size-guidelines)
- [Apple Developer - Fonts](https://developer.apple.com/fonts/)
- [Apple WWDC20 - Details of UI Typography](https://developer.apple.com/videos/play/wwdc2020/10175/)

### Spacing and Layout
- [UX Planet - 4-Point Spacing System](https://uxplanet.org/principles-of-spacing-in-ui-design-a-beginners-guide-to-the-4-point-spacing-system-6e88233b527a)
- [Canva - Spacing Guidelines](https://www.canva.dev/docs/apps/design-guidelines/spacing/)
- [Material UI - Spacing](https://mui.com/material-ui/customization/spacing/)

### Corner Radius
- [Medium - Building Consistent Corner Radius](https://medium.com/design-bootcamp/building-a-consistent-corner-radius-system-in-ui-1f86eed56dd3)
- [Microsoft - Fluent 2 Shapes](https://fluent2.microsoft.design/shapes)
- [Microsoft - WinUI Corner Radius](https://learn.microsoft.com/en-us/windows/apps/design/style/rounded-corner)

### Command Palette Design
- [Philip C. Davis - Command Palette Interfaces](https://philipcdavis.com/writing/command-palette-interfaces)
- [Destiner - Designing a Command Palette](https://destiner.io/blog/post/designing-a-command-palette/)
- [Mobbin - Command Palette UI Design](https://mobbin.com/glossary/command-palette)
- [Medium - Command Palette UX Patterns](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)

### Accessibility
- [W3C WCAG - Contrast Minimum](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)
- [WebAIM - Contrast and Color](https://webaim.org/articles/contrast/)
- [DubBot - Dark Mode Accessibility](https://dubbot.com/dubblog/2023/dark-mode-a11y.html)
- [Microsoft - Keyboard Accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility)

### Animation
- [Raycast Store - Animated Window Manager](https://www.raycast.com/matheuschein/animated-window-manager)
- [Raycast Store - Easings](https://www.raycast.com/madebyankur/easings)

---

*Last updated: January 2026*
