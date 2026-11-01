# Switch the subject: tab variations

This folder iterates on design D from [`../attention`](../attention). The things you might mean when you open Script Kit become tabs:

- what you selected
- what you copied
- the window you came from
- what you just ran

⇥ moves between them, and the rows below show what you can do with the selected one. The window opens on the newest thing you did on purpose (the attention rule), or on the window when nothing is newer. The top uses the one-row layout with the status in the bottom-left capsule.

## Variations

| | Where the tabs live |
|---|---|
| D · Last round | Small pills in the first header, the selected one filled, ⇥ at the far right. Kept for comparison. |
| T1 · Underlined tabs | The first header is the tab strip. The selected tab is underlined in gold and names its error, link or window. ⇥ sits right after it, pointing at the next tab. |
| T2 · Tab bar | Browser-style tabs with ages in a 38 pt bar under the input. The selected tab opens into a panel holding its actions. |
| T3 · Queue | The selected subject is the input's icon and names the first header. The others wait as icons at the end of the input row, in ⇥ order. |
| T4 · Sidebar | Subjects run down the left with their detail and age; actions are on the right. The last tab, Everything else, holds the inbox, flows and suggestions. |
| T5 · Accordion | Each subject is a row. The selected one is open, with its actions indented under it. |

## Shared behavior

These keys work the same way in every variation. The native launcher adopts T2 with newest-first ordering; the other variants and ordering switch are design comparisons.

- **⇥ / ⇧⇥** move between subjects on an empty input. With words typed, ⇥ keeps its current job of handing the sentence to the agent.
- **⌘V** jumps to the Copied tab.
- **↑** jumps to Last run; ↑ again steps back through earlier runs.
- **↵** runs the highlighted row on the selected subject.
- **Space** puts the subject in the input so you can say what to do with it.
- **Esc** lets go, clears the words, then returns to the subject the window opened on.
- **A gold dot** marks a subject that's still new and isn't the one the window opened on.
- **Holding ⌘** shows each tab's own key: ␣ on the selected tab, ⌘V on Copied, ↑ on Last run.

The **Tab order** control compares two orders:

- **Newest first** puts the opening subject first and drops a copy after 2 minutes or once you've passed over it; ⌘V still reaches it.
- **Same place every time** keeps the window, the selection, the copy and the last run in fixed positions, and dims an old copy instead of removing it.

## Using it

Open `index.html` directly; there is no server or build step. The page has stories, buttons that make things happen, a running clock (with Pause) and light and dark appearance. The right panel lists every tab, its state, and how many ⇥ presses reach it.

Agents can drive the same model:

- `window.mockup.state()` returns the tabs, the current tab, the leader, the rows and the status.
- `mockup.set({variant: 'T2', order: 'fixed', theme: 'light'})`
- `mockup.story('all')`
- `mockup.happen('copyLink')`
- `mockup.advance(120)`
- `mockup.pause()`

The URL hash is `#<variant>-<story>-<theme>-<order>`, for example `#T1-all-dark-leader`.

## Baseline before subject tabs

The pre-tab launcher behaved differently:

- Tab on an empty input does nothing.
- Tab with words typed switches who handles the sentence (`crates/sk-one-window/src/window/keys.rs`).
- ↑ stages the last run as a chip in the input.
- ⌘V on an empty input stages the clipboard (`window/clipboard.rs`).

The tab model moves ↑ and ⌘V from staging a subject in the input to selecting a tab, and leaves staging to Space. It also adds the ↑-again history walk.

The rule needs times for selection, copy and window switch, plus a record of what each opening showed. The tab list needs an order and a current tab. All of that is reducer state, so it can be proven in `crates/sk-one-window` tests before any drawing changes.

Icons are Lucide (ISC), copied from `vendor/script-kit-lucide-assets`.
