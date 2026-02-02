# macOS Spotlight UX Patterns Research

This document captures UX patterns from macOS Spotlight and competitor launchers (Alfred, Raycast) to inform Script Kit's design decisions.

---

## Table of Contents

1. [Search Behavior and Result Ranking](#search-behavior-and-result-ranking)
2. [Visual Design and Typography](#visual-design-and-typography)
3. [Content Type Handling](#content-type-handling)
4. [Native macOS Integration Patterns](#native-macos-integration-patterns)
5. [Competitor Analysis: Raycast vs Alfred](#competitor-analysis-raycast-vs-alfred)
6. [Recommendations for Script Kit](#recommendations-for-script-kit)

---

## Search Behavior and Result Ranking

### Real-Time, Adaptive Search

Spotlight provides a **live, interactive search** experience where results modify as you type. Key behaviors:

- **Instant Results**: Results appear immediately as characters are entered
- **Top Hit Priority**: A "Top Hit" is determined using "a combination of relevance and timeliness"
- **Adaptive Learning**: The search algorithm learns from user behavior and presents Siri Suggestions based on common actions and frequent searches
- **Usage-Based Ranking**: Frequently accessed items or contacts appear higher in results

### Relevance Scoring

Spotlight's relevance ranking uses **Search Kit** under the hood, which performs the underlying content search. The relevance scoring is:

- Not directly exposed to users
- Combines multiple signals (recency, frequency of access, file metadata)
- Learns from user interaction patterns over time

### Contextual Results

Modern Spotlight delivers **contextual results** that adapt to what you're doing:

- Type "presentation" and see files, emails, calendar events, notes, and related conversations
- Results change based on current app context and clipboard contents
- Web results, Wikipedia snippets, and Siri Knowledge appear inline

---

## Visual Design and Typography

### System Typography

macOS uses **SF Pro** (San Francisco) as the system font:

- Size-specific outlines and dynamic tracking ensure optimal legibility at every point size
- **Optical sizing**: Text and Display variants merge into a continuous design in macOS 11+
- Use SF Pro Text for text 19pt or smaller, SF Pro Display for 20pt or larger

### Typography Guidelines from Apple HIG

| Style | Usage |
|-------|-------|
| Body | Primary content |
| Footnote | Labels, secondary content |
| Caption | Small labels, metadata |

**Key principles:**
- Minimize number of typefaces in interface
- Use built-in text styles for visual distinction while maintaining legibility
- Custom fonts should only be used for branding or immersive experiences

### Text Field Design

From Apple Human Interface Guidelines:

- **Placeholder text** hints at field purpose (e.g., "Search" or "Type to search")
- Placeholder text uses sentence-style capitalization, no punctuation
- **Match field size to anticipated text quantity** - size communicates expected input length
- Placeholder disappears when typing begins

### Window Appearance

Spotlight's window characteristics:

- **Fixed width** - users cannot resize the Spotlight window
- **Variable height** - expands based on result count
- **Centered positioning** - appears at the top-center of the screen
- **Vibrancy/blur effects** using `NSVisualEffectView` for translucent backgrounds

### Animation Design Philosophy

Apple treats motion as a **functional tool, not decoration**:

- Animations connect moments in the interface
- Communicate hierarchy, continuity, and feedback
- Built around **easing curves** for natural movement
- Transitions resolve quickly (responsive but not abrupt)
- Window resize animations provide smooth appearance
- Can be disabled via "Reduce motion" accessibility setting

---

## Content Type Handling

### Result Categories

Spotlight organizes results into distinct categories:

| Category | Contents |
|----------|----------|
| **Applications** | Installed apps, system apps |
| **Documents** | Files, PDFs, Office documents |
| **Folders** | Directory locations |
| **Images** | Photos, graphics |
| **Mail & Messages** | Email, chat messages |
| **Contacts** | Address Book entries |
| **Events & Reminders** | Calendar appointments, reminders |
| **Music** | Audio files |
| **Movies** | Video files |
| **Bookmarks** | Safari bookmarks |
| **System Preferences** | Settings panels |
| **Developer** | Code files, projects |
| **Definitions** | Dictionary lookups |
| **Calculations** | Math results |
| **Conversions** | Unit conversions |
| **Web Results** | Siri suggestions, web searches |

### Category Filtering

Users can filter by content type:

- In System Settings > Siri & Spotlight, enable/disable categories
- Use `kind:` keyword to filter (e.g., `kind:pdf`, `kind:image`)
- macOS 26 introduces `/pdf`, `/icloud drive` shortcuts

### macOS 26 (Tahoe) Categories

New tab-based organization:

1. **Apps** (Command + 1)
2. **Files** (Command + 2)
3. **Actions** (Command + 3)
4. **Clipboard** (Command + 4)

---

## Native macOS Integration Patterns

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd + Space` | Open Spotlight |
| `Escape` | Clear search / Dismiss |
| `Return` | Open selected item |
| `Cmd + Return` | Reveal in Finder |
| `Cmd + L` | Jump to dictionary definition |
| `Cmd + C` | Copy result (e.g., calculation) |
| `Space` | Quick Look preview |
| `Up/Down` | Navigate results |
| `Cmd + B` | Search web |

### Quick Actions (macOS 26)

Spotlight now exposes **hundreds of actions** directly:

- Send Message
- Start Timer
- Add Reminder
- Change text case
- Remove photo background
- Run Shortcuts

### Quick Keys (macOS 26)

Short character strings that trigger frequent actions:

- `sm` - Send Message
- `ar` - Add Reminder
- Generated automatically based on usage patterns

### App Intents API

Developers can surface app functionality in Spotlight using the **App Intents API**:

- Expose discrete pieces of app functionality to the OS
- Enable context-aware suggestions
- Allow Spotlight to control app features

### Clipboard Integration

- Search recently copied items
- Preview images and text snippets before pasting
- Quick Look support for clipboard content

---

## Competitor Analysis: Raycast vs Alfred

### Visual Design Comparison

| Aspect | Spotlight | Raycast | Alfred |
|--------|-----------|---------|--------|
| **Aesthetic** | Native macOS | Modern, polished (2020s) | Utilitarian (2010s) |
| **Consistency** | System-wide | Unified across extensions | Variable by workflow |
| **Theming** | None | Pro-only themes | Extensive customization |
| **Animations** | System standard | Smooth, custom | Minimal |

### Interaction Model

**Spotlight/Alfred**: Sentence-based, type commands as text
**Raycast**: Menu-based, modal navigation with standardized `Cmd + K` action menu

### Extension Architecture

**Raycast:**
- First-class Extension Store
- Consistent UI across all extensions
- Settings co-located with extensions

**Alfred:**
- Workflows (more DIY approach)
- Greater customization depth
- More tinkering required

### Key Raycast Features

- Keyboard-first navigation (never need mouse)
- Consistent design language across extensions
- Smooth animations and gradient backgrounds
- Careful spacing and typography

### Key Alfred Features

- Extensive theming options (fonts, colors, layout)
- One-time license (vs subscription)
- Powerful workflows for power users
- Classic, familiar interface

---

## Recommendations for Script Kit

Based on this research, here are actionable recommendations:

### 1. Search Behavior

- **Implement adaptive ranking** - Learn from user selections to improve future rankings
- **Real-time results** - Update results as user types (already implemented)
- **Fuzzy matching** - Allow partial matches and typo tolerance
- **Recency weighting** - Prioritize recently-used items

### 2. Visual Design

- **Use SF Pro** - Leverage the system font for native feel
- **Optical sizing** - Use appropriate font weights/sizes for different contexts
- **Vibrancy/blur** - Use `NSVisualEffectView`-style backgrounds for macOS integration
- **Smooth animations** - Add subtle entrance/exit animations with easing curves
- **Consistent spacing** - Follow Apple HIG for padding and margins

### 3. Typography Hierarchy

```
Search Input:     SF Pro Display, 24-28pt, Regular
Item Title:       SF Pro Text, 14-16pt, Medium
Item Subtitle:    SF Pro Text, 12-13pt, Regular, Secondary color
Keyboard Hints:   SF Pro Text, 11pt, Regular, Muted color
Category Headers: SF Pro Text, 11pt, Bold, Uppercase, Secondary color
```

### 4. Result Categories

Implement category grouping with:
- Clear visual separators between categories
- Category headers (collapsible optional)
- Keyboard navigation between categories (Tab key)
- Category filtering (similar to Spotlight's `kind:` syntax)

### 5. Keyboard Shortcuts

Adopt familiar patterns:
| Shortcut | Action |
|----------|--------|
| `Cmd + Space` | Open Script Kit (configurable) |
| `Escape` | Clear / Dismiss |
| `Return` | Execute selected |
| `Cmd + Return` | Secondary action (e.g., reveal file) |
| `Cmd + K` | Open actions menu (Raycast-style) |
| `Tab` | Cycle categories or expand |
| `Space` | Quick Look preview |

### 6. Quick Actions Pattern

Consider implementing:
- **Action bar** at bottom showing available actions for selected item
- **Cmd + K menu** for action discovery
- **Customizable quick keys** for power users

### 7. Clipboard Integration

- Clipboard history search
- Preview before paste
- Type filtering for clipboard items

### 8. Animation Guidelines

- **Duration**: 150-250ms for most transitions
- **Easing**: Use ease-out for entrances, ease-in for exits
- **Respect accessibility**: Check "Reduce motion" preference
- **Purpose**: Animation should communicate state changes, not decorate

### 9. Window Behavior

- **Fixed width** (around 680-720px)
- **Variable height** based on content (max height constraint)
- **Centered at top** of screen
- **Vibrancy background** with blur effect
- **No resize handles** - keep interface simple

### 10. Learning from Raycast

- **Unified extension UX** - All scripts should follow consistent patterns
- **Action menu standardization** - `Cmd + K` for action discovery
- **Keyboard-first** - Every action accessible via keyboard
- **Extension store** - Discovery and installation of community scripts

---

## Sources

### Apple Documentation
- [Apple Human Interface Guidelines - Searching](https://developer.apple.com/design/human-interface-guidelines/searching)
- [Building a search interface for your app](https://developer.apple.com/documentation/corespotlight/building-a-search-interface-for-your-app)
- [Search with Spotlight on Mac](https://support.apple.com/guide/mac-help/search-with-spotlight-mchlp1008/mac)
- [Spotlight keyboard shortcuts](https://support.apple.com/guide/mac-help/spotlight-keyboard-shortcuts-mh26783/mac)
- [Select result categories for Spotlight](https://support.apple.com/guide/mac-help/select-result-categories-for-spotlight-mchl3e00eae9/mac)
- [Text Fields HIG](https://developer.apple.com/design/human-interface-guidelines/text-fields)
- [NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview)
- [Typography HIG](https://developer.apple.com/design/human-interface-guidelines/typography)

### macOS Tahoe (26) Coverage
- [macOS 26: Spotlight gets actions, clipboard manager, shortcuts - 9to5Mac](https://9to5mac.com/2025/06/10/macos-26-spotlight-gets-actions-clipboard-manager-custom-shortcuts/)
- [Apple Supercharges Spotlight in macOS Tahoe - MacRumors](https://www.macrumors.com/2025/06/09/apple-supercharges-spotlight-in-macos-tahoe-with-quick-keys-and-more/)
- [How to use Quick Keys in macOS Tahoe Spotlight - AppleInsider](https://appleinsider.com/inside/macos-tahoe/tips/how-to-use-quick-keys-in-macos-tahoe-spotlight)
- [Take actions and shortcuts in Spotlight - Apple Support](https://support.apple.com/guide/mac-help/take-actions-and-shortcuts-in-spotlight-mchl4953dfeb/mac)

### Analysis and Comparisons
- [Apple's changes to Spotlight signal a radical shift to UX - Medium](https://medium.com/@emil.danielsen/apples-changes-to-spotlight-signal-a-radical-shift-to-ux-and-computing-63c873619acd)
- [Spotlight vs Alfred vs Raycast - Medium](https://medium.com/@andriizolkin/spotlight-vs-alfred-vs-raycast-31bd942ac3b6)
- [Alfred vs Raycast: The Ultimate Launcher Face-Off - Medium](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89)
- [Raycast vs Alfred comparison - Raycast](https://www.raycast.com/raycast-vs-alfred)

### Design and Animation
- [How Apple Designs UI Animations - AppleMagazine](https://applemagazine.com/how-apple-designs-ui-animations/)
- [Spotlight Search Features - SimplyMac](https://www.simplymac.com/macos/spotlight-search-features)
- [Comprehensive Spotlight Analysis - Oreate AI Blog](https://www.oreateai.com/blog/comprehensive-analysis-and-advanced-user-guide-for-the-mac-systems-spotlight-search-function/eaa232e750747fbb708912fc1d96d64a)

---

*Last updated: January 2026*
