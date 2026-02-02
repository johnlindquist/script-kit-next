# Notes App Accessibility Research

Date: 2026-02-01
Scope: Accessibility features for notes apps with emphasis on keyboard navigation, screen reader support, and high contrast/visual accessibility. Includes OS-level settings that affect notes apps.

## Key findings
- Many notes apps document keyboard navigation via shortcut guides (Apple Notes, OneNote, Google Keep, Evernote, Notion, Joplin).
- Explicit screen reader guidance is available for OneNote and Joplin; iOS VoiceOver is documented to work with all built-in apps (including Notes).
- High contrast and visibility settings are largely OS-level (Windows contrast themes, macOS Increase Contrast/Reduce Transparency, iOS Increase Contrast/Reduce Transparency) and can affect app UI colors and readability.

## Keyboard navigation (documented)
- Apple Notes (macOS): Apple documents keyboard shortcuts for switching between sidebar, note list, and search, plus navigation and formatting shortcuts. Source: Apple Notes keyboard shortcuts. (https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac)
- Microsoft OneNote: Screen reader navigation docs specify keyboard-driven movement through major UI regions (for example, cycling focus across panes). Source: OneNote screen reader navigation guide. (https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-onenote-4097a3f7-067d-4a81-a0c1-1afa4a15dffb)
- Google Keep (web/desktop): Google Keep provides a full keyboard shortcuts list for navigating notes, searching, and creating notes/lists. Source: Google Keep keyboard shortcuts. (https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- Evernote: Evernote provides a comprehensive keyboard shortcuts page. Source: Evernote keyboard shortcuts. (https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)
- Notion: Notion documents extensive keyboard shortcuts, including block navigation and editing actions. Source: Notion keyboard shortcuts. (https://www.notion.com/help/keyboard-shortcuts)
- Joplin: The accessibility guide describes keyboard focus regions and shortcuts for jumping between UI areas (for example, through menu items or region-focused commands). Source: Joplin screen reader accessibility. (https://joplinapp.org/help/apps/screen_reader_accessibility/)

## Screen reader support (documented)
- Microsoft OneNote: Microsoft provides screen reader-specific guidance and lists supported screen readers by platform. Source: OneNote screen reader basics. (https://support.microsoft.com/en-us/office/basic-tasks-using-a-screen-reader-with-onenote-32cd532b-d5d4-442b-bc13-1d0ad2016377)
- Microsoft OneNote: The navigation guide includes techniques for moving across panes with a screen reader and simplifying reading order. Source: OneNote screen reader navigation guide. (https://support.microsoft.com/en-us/office/use-a-screen-reader-to-explore-and-navigate-onenote-4097a3f7-067d-4a81-a0c1-1afa4a15dffb)
- Joplin: Joplin provides a dedicated screen reader accessibility guide covering region-based navigation and keyboard focus behavior. Source: Joplin screen reader accessibility. (https://joplinapp.org/help/apps/screen_reader_accessibility/)
- Apple Notes (iOS): Apple states VoiceOver works with all built-in apps on iPhone; Notes is a built-in app. Source: VoiceOver in iPhone apps. (https://support.apple.com/en-kw/guide/iphone/iphe4ee74be8/ios)

## High contrast and visual accessibility (OS-level)
- Windows: High contrast (contrast themes) is configured in Windows Accessibility settings; it changes OS colors and can affect many apps. Sources: Windows contrast themes settings and Microsoft 365 accessibility note about contrast themes affecting Windows and most apps. (https://support.microsoft.com/en-us/windows/change-contrast-themes-in-windows-fedc744c-90ac-69fd-330c-4bb423633bed) (https://support.microsoft.com/en-us/office/use-color-and-contrast-for-accessibility-in-microsoft-365-353cd0f4-d76d-4c66-8474-cb90a22dcd49)
- macOS: Accessibility Display settings include Increase Contrast and Reduce Transparency, which affect app visuals system-wide. Source: macOS Display accessibility settings. (https://support.apple.com/my-mm/guide/mac-help/unac089/mac)
- iOS/iPadOS: Display & Text Size accessibility settings include Increase Contrast and Reduce Transparency. Source: iOS Display & Text Size preferences. (https://support.apple.com/en-mide/111773)

Implication for notes apps: If the app uses system colors and standard UI components, it should respond to these OS-level settings; this is especially relevant for contrast themes on Windows and Increase Contrast/Reduce Transparency on Apple platforms.

## Accessible text editor considerations (developer guidance)
- Joplin's development accessibility guidance emphasizes labeled controls, keyboard accessibility, screen reader usability, and sufficient contrast (aligned with WCAG 2.2 and common accessibility practices). Source: Joplin development accessibility. (https://joplinapp.org/help/dev/accessibility/)

## Notes for follow-up (if deeper coverage is needed)
- Expand research to app-specific screen reader documentation for other popular notes apps (for example, Evernote, Google Keep, Notion) if available.
- Validate in-app behavior with platform screen readers (VoiceOver, Narrator, NVDA, TalkBack) and OS contrast modes to confirm real-world support.
