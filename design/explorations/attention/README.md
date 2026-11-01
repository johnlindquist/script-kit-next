# What comes first: ranking what's competing for the top

An interactive exploration of how the main menu could choose between the things you might mean when you open Script Kit. These include text you selected, something you just copied, the window you came from, what you just ran (which ↑ brings back), and runs or failures that need you. It builds on the one-row layout in [`../one-row`](../one-row): the input row has nothing above it, and status sits in the bottom-left capsule.

## The rule

1. **Anything that needs you comes first, and isn't ranked.** This covers a run about to go, a question, or a failure you haven't seen. It takes the status place, and Stop is always there.
2. **Past and present don't compete.** What you're pointing at now is "this": the input's icon and the first rows. What you already did reports in the status place.
3. **Among the present, the newest thing you did on purpose leads.** Selecting, copying and switching windows each count as a moment. The window is the floor.
4. **News fades, and ignored news steps back.** A copy leads for 2 minutes; a selection leads while it's selected. Once you've seen something and passed over it, it stops leading, but its key still works.
5. **What you did steps behind you.** The last run stays in the status place for 2 minutes, while ⌘Z can take it back, then waits on ↑. Pressing ↑ again walks back through earlier runs.

There are two more rules. Only the leader gets rows; anything else still fresh shows just its name and key. Text that is both selected and copied counts once.

Each candidate keeps one key: Space acts on "this", ⇥ (on an empty input) picks the next, ⌘V picks what you copied, ↑ picks the last run, and ⌘Z undoes or stops. Holding ⌘ shows all of them.

## Designs

| | How the choice is shown |
|---|---|
| A · Today | What the app does now. The line holds the last run at any age; a dashed chip offers the window or selection; the clipboard stays hidden until ⌘V. |
| B · One leader | The leader is the input's icon and supplies the first three rows, under a header naming it. Fresh runners-up appear at that header's right end with their key. |
| C · Just now | No leader. A "Just now" section lists everything from the last ten minutes, newest first, past and present together. |
| D · Switch the subject | Every candidate is a small tab in the first header, with the leader selected. ⇥ moves to the next tab. |
| E · Quiet until asked | Only the input's icon shows the leader. Holding ⌘ adds a Keys section with one row per candidate. |

The rule control switches between "newest wins" and a fixed order (selection, then copy, then window, at any age), so you can see why fading matters.

## Using it

Open `index.html` directly; there is no server or build step. Stories set up a situation. The "Make something happen" buttons change it. The clock runs in real time, and Pause freezes it. The right panel lists each candidate, when it happened, and where it ended up.

Agents can drive the same model:

- `window.mockup.story('all')`
- `mockup.happen('copyLink' | 'select' | 'wait2m' | 'glance' | …)`
- `mockup.set({design: 'D', rule: 'fixed', theme: 'light'})`
- `mockup.advance(seconds)`
- `mockup.pause()`
- `mockup.state()`, which returns the leader, "this", whispers, status, rows and a world summary.

The URL hash is `#<design>-<story>-<theme>-<rule>`, for example `#B-all-dark-newest`.

## Baseline before subject tabs

The baseline comes from `crates/sk-one-window/src/window/show.rs`. The line shows the first of these that applies:

1. a waiting run
2. a question
3. a view's or notice's words
4. running work
5. the last run, at any age
6. the window behind, but only before anything has ever run

The window is offered as a dashed chip at the input's end, and it names selected text when there is some. ↑ stages the last run (`window/keys.rs`). ⌘V on an empty input stages the clipboard (`window/clipboard.rs`), but nothing shows the clipboard before you press it. Tab on an empty input was unused in that baseline; with typed words it switches who handles the sentence.

To adopt the rule, the reducer needs timestamps for selection, copy and window switch. It also needs a record of what each opening showed, so ignored news can step back, plus the ↑↑ history walk. All of that is reducer state with no GPUI involved, so it can be proven in `crates/sk-one-window` tests before any drawing changes.

The mockup's values come from the Rust theme and contract files. Icons are Lucide (ISC), copied from `vendor/script-kit-lucide-assets`.
