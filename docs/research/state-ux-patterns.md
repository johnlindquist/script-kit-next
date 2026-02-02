# State UX Patterns for Launcher Applications

Research findings on empty states, loading states, error states, and onboarding experiences for launcher apps like Script Kit.

---

## Table of Contents

1. [Empty States and No Results](#empty-states-and-no-results)
2. [Loading States and Skeleton Screens](#loading-states-and-skeleton-screens)
3. [Error States and Recovery](#error-states-and-recovery)
4. [First-Run and Onboarding](#first-run-and-onboarding)
5. [Command Palette Accessibility](#command-palette-accessibility)
6. [Recommendations for Script Kit](#recommendations-for-script-kit)

---

## Empty States and No Results

### Types of Empty States

There are four main categories of empty states users encounter:

1. **First use** - New product/service with nothing to show yet
2. **User cleared** - Users completed actions (cleared inbox, finished tasks)
3. **Errors** - Something went wrong (network issues, permissions)
4. **No results/No data** - Search returns nothing or data unavailable

### Best Practices

#### Clear Communication
A useful empty state communicates:
- **What's happening** - Clear status message
- **Why it's happening** - Context for the situation
- **What to do about it** - Actionable next steps

#### Effective Messaging Patterns

| Pattern | Example |
|---------|---------|
| Status + Action | "No scripts yet. Create your first script." |
| Benefit + CTA | "Automate your workflow. Import a script." |
| Question + Solution | "Looking for a command? Try browsing categories." |
| Encouragement + Direction | "All caught up! Explore the script store." |

#### Always Include a Call-to-Action
Never leave users at a dead end. Provide:
- Primary action button
- Alternative paths forward
- Related suggestions

#### "No Results" Search Pages

**Statistics:** 68% of e-commerce sites have "No Results" pages that are essentially dead-ends.

**Recommendations:**
- Display the search query prominently (helps users spot typos)
- Use empathetic language: "Sorry, we couldn't find anything for..."
- Suggest alternative searches or categories
- Show popular/recommended items
- Implement typo-tolerant autocomplete to prevent no-results scenarios
- Never blame the user

**Example approaches:**
- Auto-correct misspellings (like Google/GAP)
- Suggest "Did you mean...?"
- Show recently used or popular commands
- Offer to create a new script with that name

#### Celebratory Empty States
When users clear their tasks, celebrate the achievement:
- "All caught up!" with positive visuals
- Reinforces progress and creates satisfaction

#### Starter Content
For new users, consider providing:
- Sample scripts to explore
- Pre-built templates
- Example workflows users can modify

### Common Mistakes to Avoid

- Generic messaging: "No data available"
- Missing actions (dead ends)
- Overwhelming with too many options
- Negative or blaming language
- Poor visual hierarchy
- Showing column headers for empty tables (accessibility issue)

### Sources
- [Empty State UX Examples - Eleken](https://www.eleken.co/blog-posts/empty-state-ux)
- [Empty States - Toptal](https://www.toptal.com/designers/ux/empty-state-ux-design)
- [Empty States Pattern - UXPin](https://www.uxpin.com/studio/blog/ux-best-practices-designing-the-overlooked-empty-states/)
- [Empty States - Carbon Design System](https://carbondesignsystem.com/patterns/empty-states-pattern/)
- [No Results Page Examples - Prefixbox](https://www.prefixbox.com/blog/no-results-page-examples/)
- [No Results Page UX - UX Booth](https://uxbooth.com/articles/design-no-results-found-pages-that-get-results/)
- [No Results Page Strategies - Baymard](https://baymard.com/blog/no-results-page)

---

## Loading States and Skeleton Screens

### Types of Loading Indicators

| Indicator | Best For | Duration |
|-----------|----------|----------|
| **Spinner** | Single module (video, card) | 2-10 seconds |
| **Skeleton Screen** | Full page/multiple elements | Under 10 seconds |
| **Progress Bar** | Long operations | Over 10 seconds |

### Skeleton Screens

Skeleton screens display a wireframe-like preview of the page layout while content loads.

**Benefits:**
- Creates illusion of shorter wait time
- Reduces cognitive load (gradual transition)
- Helps users develop mental models of page structure
- More engaging than blank screens

**Design Guidelines:**
- Shapes should correspond to content they represent
- Use rounded rectangles for images
- Use simple lines for text
- Match the general layout of the final content

**Animation Best Practices:**
- Use wave/shimmer effect (like Facebook) over pulse
- Keep animations slow and gentle: 1.5-2 second cycle
- Respect `prefers-reduced-motion` setting
- Provide static placeholders for users who disable motion

### Real-World Examples

Major platforms using skeleton screens:
- LinkedIn
- YouTube
- Facebook
- Twitter (popularized the pattern in 2012)
- Google Photos (dominant color-based skeletons)
- Pinterest

### What NOT to Do

Never show skeleton states for:
- Toast notifications
- Overflow menus
- Dropdown items
- Modal containers (contents can have skeletons)
- Other loaders (don't combine spinner + skeleton)

### Accessibility

- Use ARIA attributes to announce loading state to screen readers
- Provide non-moving static placeholders for reduced motion preference
- Ensure sufficient color contrast for skeleton elements

### Raycast-Specific Patterns

Raycast handles loading states with:
- `isLoading` prop on top-level components (`<List>`, `<Detail>`, `<Form>`)
- Loading indicator at the top of the window
- Pagination with placeholder items
- Toast notifications for async operations
- HUD confirmations after actions complete

### Sources
- [Skeleton Screens 101 - Nielsen Norman Group](https://www.nngroup.com/articles/skeleton-screens/)
- [Skeleton Loading Screen Design - LogRocket](https://blog.logrocket.com/ux-design/skeleton-loading-screen-design/)
- [Skeleton vs Loading Screens - OpenReplay](https://blog.openreplay.com/skeleton-screens-vs-loading-screens--a-ux-battle/)
- [Loading Feedback Patterns - Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-loading-feedback)
- [Loading Patterns - Carbon Design System](https://carbondesignsystem.com/patterns/loading-pattern/)
- [Raycast Best Practices](https://developers.raycast.com/information/best-practices)

---

## Error States and Recovery

### Types of Errors

1. **Slips** - User intends one action but does another (autopilot mistakes)
2. **Mistakes** - Mismatch between user's mental model and system behavior
3. **System errors** - Backend failures, crashes
4. **Network errors** - Connectivity issues
5. **Input errors** - Invalid data entry

### Anatomy of a Good Error Message

Every error message should include:
1. **Problem statement** - What went wrong
2. **Cause explanation** - Why it happened (if known)
3. **Solution suggestion** - How to fix it

### Error Display Patterns

| Pattern | Use Case |
|---------|----------|
| **Inline validation** | Form input errors (as user types/leaves field) |
| **Tooltips** | Minor or transient errors |
| **Modals** | Critical or irreversible errors |
| **Alerts/Toast** | Feedback or confirmation |
| **Banners** | Persistent or important errors |
| **Logs** | Debugging, historical record |

### Visual Design Guidelines

- Use bold, high-contrast styling
- Red is conventional but never rely on color alone
- Use redundant indicators (icon + color + text)
- Maintain clear visual hierarchy
- Position errors near their source

### Timing and Prevention

**Avoid premature error display:**
- Don't validate before user finishes input
- Don't show errors on empty required fields until submission
- Early error display feels like "grading before the test"

**Prevention is better than cure:**
- Guide users proactively
- Use autocomplete and suggestions
- Provide input constraints and formatting hints

### Recovery-Focused Design

Transform errors into opportunities:
- Treat errors as guidance, not dead ends
- Explain impact and how to avoid/rectify
- Offer specific next steps
- For known issues (e.g., email already in use), provide relevant actions (login, password recovery)

### Toast Message Cautions

Common usability issues:
- Users don't read message before it disappears
- No way to restore or keep the message visible
- Consider persistent error states for important issues

### Raycast Error Handling

- Handle "expected" errors gracefully
- Show cached data when network fails
- Use Toast for most error feedback
- Show helpful messages for missing runtime dependencies
- Don't disrupt user flow for recoverable errors

### Sources
- [Error Handling UX Patterns - Medium](https://medium.com/design-bootcamp/error-handling-ux-design-patterns-c2a5bbae5f8d)
- [Error Messages UX - Smashing Magazine](https://www.smashingmagazine.com/2022/08/error-messages-ux-design/)
- [Error Message Guidelines - Nielsen Norman Group](https://www.nngroup.com/articles/error-message-guidelines/)
- [Error Feedback Patterns - Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-error-feedback)
- [Hostile Error Messages - Nielsen Norman Group](https://www.nngroup.com/articles/hostile-error-messages/)

---

## First-Run and Onboarding

### Raycast vs Alfred Approaches

**Raycast:**
- Immediately offers to replace Spotlight shortcut
- Requires granting Accessibility permissions
- Excellent hands-on walkthrough of major features
- "Show Onboarding" command to revisit later
- Unified, cohesive interface across extensions
- Recommended for new users due to ease of mastery

**Alfred:**
- Option to skip setup completely
- Several settings adjustable at first launch
- Uses sentence-based interface (vs Raycast's menus)
- Parts feel more distinct/separate

### Progressive Disclosure

**Definition:** UX technique that reduces cognitive load by gradually revealing information as needed.

**Benefits:**
- Improves experience for new users
- Simplifies the interface
- Creates better understanding of features
- Reduces learning curve

**Implementation Patterns:**

1. **Checklists** - Break features into steps, each opening new guidance
2. **Tooltips** - Triggered for new users to show features
3. **Step-by-step flows** - One question/action per screen

### Onboarding Best Practices

1. **Layer features:**
   - Core tools prominently displayed
   - Advanced functions in sub-menus or collapsible sections

2. **Track new-user behavior:**
   - Understand which features retained users focus on
   - Highlight those in onboarding
   - Minimize distractions

3. **Start minimal:**
   - Begin with simplest action (profile, permissions)
   - Reveal optional features over time

4. **Provide escape hatches:**
   - Allow skipping for experienced users
   - Make it easy to revisit onboarding later

### Mobile Considerations (Applicable to Launcher UX)

- Limited screen space = simpler main interface
- Focus on main actions
- Use tabs and modals for additional content
- One task at a time to minimize cognitive load

### Sources
- [Alfred vs Raycast - Medium](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89)
- [Raycast vs Alfred - Raycast](https://www.raycast.com/raycast-vs-alfred)
- [Progressive Disclosure - Interaction Design Foundation](https://www.interaction-design.org/literature/topics/progressive-disclosure)
- [Progressive Disclosure Examples - Userpilot](https://userpilot.com/blog/progressive-disclosure-examples/)
- [Mobile Onboarding Best Practices - Design Studio](https://www.designstudiouiux.com/blog/mobile-app-onboarding-best-practices/)

---

## Command Palette Accessibility

### Benefits of Command Palettes

- Single entry point for all functionality
- Reduces need for complex menu navigation
- Helps users discover features
- Speeds up workflows for power users
- Search is easier than remembering shortcuts

### Accessibility Advantages

- Direct access to all commands for assistive technology users
- Reduces need to navigate hidden modals/submenus
- Can improve experience when focus management is done correctly

### Keyboard Navigation Requirements

- Clear keyboard shortcut to open (e.g., Cmd+K, Cmd+Space)
- Arrow keys for list navigation
- Enter to select
- Escape to dismiss
- Focus trapping when open
- Tab order management

### Implementation Considerations

- Proper ARIA roles for search and list
- Announce results count to screen readers
- Clear loading/no-results states
- Accessible modal overlay
- Support for screen readers

### Sources
- [Command Palette UX Patterns - Medium](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)
- [Command Palette UI Design - Mobbin](https://mobbin.com/glossary/command-palette)
- [Command Palettes for the Web - Rob Dodson](https://robdodson.me/posts/command-palettes/)
- [GitHub Command Palette - GitHub Docs](https://docs.github.com/en/get-started/accessibility/github-command-palette)

---

## Recommendations for Script Kit

Based on this research, here are specific recommendations for Script Kit's state handling:

### Empty State: No Scripts

```
+------------------------------------------+
|                                          |
|         [Script icon illustration]       |
|                                          |
|        No scripts yet                    |
|                                          |
|   Scripts let you automate anything.     |
|   Get started in seconds.                |
|                                          |
|   [Create Script]   [Browse Store]       |
|                                          |
+------------------------------------------+
```

**Copy suggestions:**
- "No scripts yet. Create your first automation."
- "Your script library is empty. Let's fix that!"
- Avoid: "No data" or technical language

### Empty State: No Search Results

```
+------------------------------------------+
|  Search: "foobar"                        |
+------------------------------------------+
|                                          |
|   No matches for "foobar"                |
|                                          |
|   Try:                                   |
|   - Checking for typos                   |
|   - Using fewer keywords                 |
|   - Browsing all scripts                 |
|                                          |
|   [Create "foobar" script]               |
|                                          |
+------------------------------------------+
```

**Features to implement:**
- Fuzzy matching / typo tolerance
- "Did you mean...?" suggestions
- Quick-create option with search term as name
- Show recent/popular commands as alternatives

### Loading States

**For script list loading:**
- Use skeleton screen with 4-5 list item placeholders
- Subtle shimmer animation (1.5-2s cycle)
- Respect `prefers-reduced-motion`

**For script execution:**
- Show subtle loading indicator at top (like Raycast)
- Display script name being executed
- Provide cancel option for long-running scripts

**For extension/store loading:**
- Skeleton cards for extension grid
- Progressive loading (show items as they arrive)

### Error States

**Network errors:**
```
+------------------------------------------+
|                                          |
|   [Offline icon]                         |
|                                          |
|   Can't connect to Script Kit Cloud      |
|                                          |
|   Check your internet connection and     |
|   try again. Local scripts still work.   |
|                                          |
|   [Retry]   [Work Offline]               |
|                                          |
+------------------------------------------+
```

**Script execution errors:**
- Show error message with script name
- Display relevant error details (not full stack trace)
- Offer: "Edit Script" / "View Logs" / "Dismiss"
- Keep error visible until dismissed (no auto-hide for errors)

**Permission errors:**
- Clear explanation of what permission is needed
- Link to system preferences
- Option to skip/cancel

### First-Run Experience

**Recommended flow:**

1. **Welcome** - Brief value proposition
2. **Permissions** - Request necessary access with explanations
3. **Keyboard Shortcut** - Set/confirm launcher shortcut
4. **Quick Tour** - Interactive walkthrough of 3-4 key features
5. **First Script** - Guided creation of simple script

**Key principles:**
- Allow skipping at every step
- Make it easy to revisit ("Show Onboarding" command)
- Don't overwhelm - focus on essential features
- Use progressive disclosure for advanced features

### Accessibility Checklist

- [ ] All empty states have clear, non-blaming copy
- [ ] Loading states announce to screen readers
- [ ] Error messages include problem + cause + solution
- [ ] No color-only indicators (use icons + text)
- [ ] Reduced motion preference respected
- [ ] Focus properly managed during state transitions
- [ ] Skeleton screens don't read out column headers

### Implementation Priority

1. **High:** No-results empty state with suggestions
2. **High:** Script execution loading indicator
3. **High:** Error states with recovery actions
4. **Medium:** Skeleton screens for list loading
5. **Medium:** First-run onboarding flow
6. **Low:** Celebratory empty states (all tasks done)
7. **Low:** Starter content / sample scripts

---

## Summary

The best launcher experiences share common patterns:

1. **Never leave users at dead ends** - Always provide next steps
2. **Be specific and helpful** - Generic messages frustrate users
3. **Match loading indicators to duration** - Spinners for short, skeletons for medium, progress for long
4. **Handle errors gracefully** - Focus on recovery, not blame
5. **Onboard progressively** - Don't overwhelm new users
6. **Design for keyboard-first** - Launchers are productivity tools

Script Kit can differentiate by:
- Offering to create scripts from failed searches
- Showing cached/local content when offline
- Providing contextual suggestions based on usage patterns
- Making onboarding revisitable and skippable
- Celebrating automation wins (scripts run, time saved)
