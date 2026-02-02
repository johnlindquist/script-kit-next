# List Item UX Research for Launcher Applications

Research on list item design patterns in launcher applications (Raycast, Alfred, Spotlight, VS Code Command Palette) to inform Script Kit's list item design.

---

## 1. List Item Anatomy

### Core Components

Based on research from [Justinmind](https://www.justinmind.com/ui-design/list), [UXPin](https://www.uxpin.com/studio/blog/list-design/), and the [Wise Design System](https://wise.design/components/list-item), list items typically consist of three main zones:

```
+------------------------------------------------------------------+
|  [Icon/Avatar]  |  Title (Primary)         |  [Accessories]      |
|                 |  Subtitle (Secondary)    |  [Keyboard Hint]    |
+------------------------------------------------------------------+
```

#### 1.1 Leading Zone (Supporting Visuals)

- **Icons**: Glyphs representing actions or categories
- **Avatars**: User photos for contact/people lists
- **Thumbnails**: Preview images for files/media
- **Emoji**: Lightweight visual markers
- **Status indicators**: Colored dots or badges

**Best Practice**: Supporting visuals draw attention quickly, make lists scannable, and improve visual alignment.

#### 1.2 Content Zone (Text)

- **Primary Text (Title)**: The most important piece of information
- **Secondary Text (Subtitle)**: Elaboration on the primary text, often muted
- **Tertiary Text**: Additional context (use sparingly)

**Material Design Recommendation**: Do not exceed three lines per list item.

#### 1.3 Trailing Zone (Accessories)

- **Keyboard shortcuts**: Key combinations to trigger the action
- **Metadata**: Dates, counts, prices, ratings
- **Action buttons**: Edit, delete, more options
- **Chevrons**: Indicate drill-down navigation
- **Tags/Badges**: Categorization or status labels

### Launcher-Specific Patterns

From [Raycast API documentation](https://developers.raycast.com/api-reference/user-interface/list):

| Element | Purpose | Example |
|---------|---------|---------|
| Icon | Visual identification | App icon, action glyph |
| Title | Command/item name | "Open Terminal" |
| Subtitle | Context or description | "Developer Tools" |
| Accessories | Metadata + shortcuts | `12 items`, `cmd+T` |
| Keywords | Search optimization | Hidden, aids fuzzy matching |

---

## 2. Selection Highlighting Patterns

### Platform Conventions

Research from [mackuba.eu](https://mackuba.eu/2018/07/04/dark-side-mac-1/) and Apple's [NSVisualEffectView documentation](https://developer.apple.com/documentation/appkit/nsvisualeffectview):

#### macOS Native

- **Active selection**: Accent color background (user-configurable system preference)
- **Vibrancy**: Translucent backgrounds that blur content behind
- **Inactive window**: Muted/gray selection when window loses focus
- **Material-aware**: Selection adapts to light/dark mode and sidebar materials

#### Launcher Apps

| App | Selection Style |
|-----|-----------------|
| **Raycast** | Purple accent (brand color), rounded corners, subtle shadow |
| **Alfred** | Accent color highlight, minimal padding |
| **Spotlight** | System accent color, vibrancy-aware |
| **VS Code** | Blue highlight, high contrast in command palette |

### Visual Hierarchy of States

From [Baymard Institute](https://baymard.com/blog/list-items-hover-and-hit-area) research:

1. **Default**: Normal appearance, clear item boundaries
2. **Hover**: Subtle background change (synchronized across all item elements)
3. **Focused**: Clear focus ring or highlight for keyboard navigation
4. **Selected**: Strong visual distinction (accent color background)
5. **Pressed/Active**: Momentary darker state on click/enter

### Contrast Requirements

From [WCAG guidelines](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html):

- Focus indicator: minimum 3:1 contrast ratio against adjacent colors
- Selection highlight: should be visually distinct from hover
- Text on selection: maintain readable contrast

---

## 3. Hover States and Interactions

### Synchronized Hover Effects

From [Baymard Institute research](https://baymard.com/blog/list-items-hover-and-hit-area):

> "Synchronized hover effects make it instantly clear to the user that all elements within a list item lead to the same destination."

**Key Principle**: When hovering any part of a list item, the entire item should show the hover state. This:
- Clarifies which elements belong together
- Indicates the clickable/interactive area
- Provides logical clues about the action

### Hover Design Patterns

From [UX Planet](https://uxplanet.org/hover-effect-in-ui-design-tips-tricks-9c91d1a2bf22):

| Technique | Use Case |
|-----------|----------|
| Background color change | Most common, low visual noise |
| Subtle shadow/elevation | Indicates actionability |
| Border/outline | Clear boundary definition |
| Text underline (links) | Specific to text links |
| Icon reveal | Show actions on hover |

### Hit Area Considerations

- Entire row should be clickable, not just text
- Generous padding improves touch/click accuracy
- Minimum 44px height for touch targets (Apple HIG)
- Consider "ghost" click areas beyond visible bounds

### Keyboard vs Mouse Hover

From [MDN :focus-visible](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/:focus-visible):

- Show focus rings for keyboard navigation
- Hide focus rings for mouse interactions
- Use `:focus-visible` CSS pseudo-class for this behavior

---

## 4. Information Density Considerations

### Density Modes

Research from [SAP Fiori](https://experience.sap.com/fiori-design-web/cozy-compact/), [Cloudscape](https://cloudscape.design/foundation/visual-foundation/content-density/), and [Material Design](https://m2.material.io/design/layout/applying-density.html):

| Mode | Description | Use Case |
|------|-------------|----------|
| **Comfortable** | Default, optimized for readability | General use, mobile |
| **Compact** | Reduced spacing, more items visible | Power users, data-heavy views |
| **Dense** | Maximum information density | Expert users, dashboards |

### Implementation Guidelines

From [Cloudscape Design System](https://cloudscape.design/foundation/visual-foundation/content-density/):

- **4px base unit** for spacing system
- Compact mode reduces vertical padding inside components
- Reduces both vertical and horizontal spacing between components
- **20% increase** in information density with compact spacing
- **30% increase** with left-aligned form elements

### Best Practices

From [Paul Wallas on Medium](https://paulwallas.medium.com/designing-for-data-density-what-most-ui-tutorials-wont-teach-you-091b3e9b51f4):

1. **Default to comfortable mode**: Better for new users
2. **Allow user preference**: Let users choose their density
3. **Font considerations**: 14px body text, 20px line height for compact
4. **Button height**: 32-36px for compact buttons
5. **Balance rule**: As component density increases, increase layout margins/gutters

### Accessibility Tradeoffs

From [Material Design density guidelines](https://m2.material.io/design/layout/applying-density.html):

- Higher density can decrease accessibility
- **Minimum 48px touch targets** regardless of density mode
- Don't sacrifice readability for density
- Test with screen readers and keyboard-only navigation

---

## 5. Keyboard Navigation Patterns

### Standard Launcher Shortcuts

From [Raycast keyboard shortcuts](https://manual.raycast.com/keyboard-shortcuts) and [VS Code documentation](https://code.visualstudio.com/docs/getstarted/userinterface):

| Shortcut | Action |
|----------|--------|
| `Up/Down Arrow` | Navigate list items |
| `Enter` | Primary action (open, execute) |
| `Cmd+Enter` | Secondary action |
| `Tab` | Move to next section/area |
| `Escape` | Close/dismiss/go back |
| `Cmd+K` or `Cmd+P` | Open command palette |
| `Cmd+[number]` | Quick access to nth item |

### Raycast Action Panel

From [Raycast API](https://developers.raycast.com/api-reference/user-interface/action-panel):

> "Each component can provide interaction via an ActionPanel. The panel has a list of Actions where each one can be associated with a keyboard shortcut."

- First action = primary (Enter)
- Second action = secondary (Cmd+Enter)
- Additional actions shown in expandable panel

---

## 6. Recommendations for Script Kit

### List Item Structure

```rust
struct ListItem {
    // Leading
    icon: Option<Icon>,           // Glyph, emoji, or image

    // Content
    title: String,                // Primary text (bold)
    subtitle: Option<String>,     // Secondary text (muted)

    // Trailing
    accessories: Vec<Accessory>,  // Right-aligned metadata
    shortcut: Option<KeyBinding>, // Keyboard hint
}

enum Accessory {
    Text(String),
    Tag { label: String, color: Color },
    Date(DateTime),
    Icon(Icon),
}
```

### Visual States

| State | Background | Text | Border |
|-------|------------|------|--------|
| Default | Transparent | theme.text | None |
| Hover | theme.surface_hover | theme.text | None |
| Focused | theme.surface_hover | theme.text | theme.focus_ring |
| Selected | theme.accent | theme.on_accent | None |
| Disabled | Transparent | theme.text_muted | None |

### Sizing Recommendations

| Density | Row Height | Icon Size | Font Size | Padding |
|---------|------------|-----------|-----------|---------|
| Comfortable | 44-48px | 24px | 14px | 12px |
| Compact | 32-36px | 20px | 13px | 8px |

### Keyboard Shortcut Display

From [Knock blog on keyboard shortcuts](https://knock.app/blog/how-to-design-great-keyboard-shortcuts):

- Display shortcuts right-aligned in muted text
- Use keycap styling (rounded background, subtle shadow)
- Group modifier keys: `Cmd+Shift+P` not `Cmd Shift P`
- Consider abbreviations: `Cmd` or command symbol

### macOS Integration

- Use system accent color for selection (respect user preference)
- Implement vibrancy for translucent backgrounds
- Match native list behavior (smooth scrolling, momentum)
- Support both light and dark modes

### Accessibility Checklist

- [ ] Focus indicator visible with 3:1 contrast
- [ ] All interactive items keyboard accessible
- [ ] Screen reader announces item content
- [ ] Selection state announced
- [ ] Minimum 44px touch/click targets
- [ ] Color not sole indicator of state

---

## Sources

### Launcher Apps
- [Spotlight vs Alfred vs Raycast - Medium](https://medium.com/@andriizolkin/spotlight-vs-alfred-vs-raycast-31bd942ac3b6)
- [Raycast for Designers - Hack Design](https://www.hackdesign.org/toolkit/raycast/)
- [Raycast API - User Interface](https://developers.raycast.com/api-reference/user-interface)
- [Raycast API - List](https://developers.raycast.com/api-reference/user-interface/list)
- [Command Palette UX Pattern - Bootcamp](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)
- [VS Code User Interface](https://code.visualstudio.com/docs/getstarted/userinterface)

### List Design
- [List UI Design - Justinmind](https://www.justinmind.com/ui-design/list)
- [List Design 101 - UXPin](https://www.uxpin.com/studio/blog/list-design/)
- [30+ List UI Design Examples - Eleken](https://www.eleken.co/blog-posts/list-ui-design)
- [Wise Design - List Item](https://wise.design/components/list-item)
- [UXcel - Anatomy of Lists](https://app.uxcel.com/courses/ui-components-best-practices/anatomy-758)

### Hover and Selection
- [Synchronized Hover Effects - Baymard Institute](https://baymard.com/blog/list-items-hover-and-hit-area)
- [Hover Effect Tips - UX Planet](https://uxplanet.org/hover-effect-in-ui-design-tips-tricks-9c91d1a2bf22)
- [:focus-visible - MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/:focus-visible)
- [Focus Indicators - Deque](https://www.deque.com/blog/give-site-focus-tips-designing-usable-focus-indicators/)
- [Focus Indicators Guide - Sara Soueidan](https://www.sarasoueidan.com/blog/focus-indicators/)
- [WCAG Focus Appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html)

### Information Density
- [Content Density - SAP Fiori](https://experience.sap.com/fiori-design-web/cozy-compact/)
- [Content Density - Cloudscape](https://cloudscape.design/foundation/visual-foundation/content-density/)
- [UI Density - Matt Strom](https://mattstromawn.com/writing/ui-density/)
- [Applying Density - Material Design](https://m2.material.io/design/layout/applying-density.html)
- [Material Density on Web - Google Design](https://medium.com/google-design/using-material-density-on-the-web-59d85f1918f0)

### macOS Native
- [Dark Side of the Mac - mackuba.eu](https://mackuba.eu/2018/07/04/dark-side-mac-1/)
- [NSVisualEffectView - Apple](https://developer.apple.com/documentation/appkit/nsvisualeffectview)
- [Translucent Lists in SwiftUI - Hacking with Swift](https://www.hackingwithswift.com/quick-start/swiftui/how-to-get-translucent-lists-on-macos)

### Keyboard Design
- [Keyboard Shortcuts Design - Knock](https://knock.app/blog/how-to-design-great-keyboard-shortcuts)
- [Raycast Keyboard Shortcuts](https://manual.raycast.com/keyboard-shortcuts)
