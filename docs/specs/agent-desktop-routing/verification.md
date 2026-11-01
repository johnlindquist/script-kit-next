# Jev, mouse input, and two agent cursors

**Kit's filename action does not trigger a mouse.** Its adapter can inspect a window, read a field and issue `set-value`; it does not expose `--headed`, raw pointer input, or an autonomous Jev executor. The mouse tests below exercise a separate, explicitly authorized harness. [Screenshots and measurements](https://clever-yarrow-4jyg.here.now/#two-cursors).

## Experiment

A disposable, mount-free Debian ARM64 VM ran GTK, AT-SPI, Openbox and Xvfb. A fixture offered only “Amber bell” and “Indigo bell” and independently logged each button’s `down`, `up`, and `clicked` events. No host display, clipboard, pointer, credential or personal data was given to the guest. Native binary: official Linux `agent-desktop` 1.0.2; upstream `scripts/jev/act.mjs` question builder and decision policy: `5261b528`. The host asked TypeSafe `jev-1.14.0` about synthetic button descriptions, never an image.

The harness pre-authorized a particular button and `click` for each positive case; no verb substitution, new target or delivery retry. It required the upstream `act` decision, the matching real ref and an additional presence threshold of 0.70. **This is harness policy, not a production Kit mouse grant.** The harness, not Jev, chose headed execution. An ordinary ref click need not physically move a pointer.

| Check | Observed |
|---|---|
| Upstream request shape | Both labeled clicks picked the right target; presence was 0.58/0.57 because the labels were in another question’s choices. The extra gate withheld dispatch. |
| Same settings, add visible button names to shared state | Presence was 0.98 for both labeled targets. The absent green bell returned no match and presence 0.01. |
| One agent | A headed qualified-ref click produced one native GTK press/release/activation on the expected button. |
| Two agents | Two native clients in the **same** `session start --cursor --multi-agent` session used distinct agent IDs. Each clicked its own Jev-selected button once. Two independently positioned cursor windows were visible. The input sequences did not interleave; one client recorded seven interaction-lease contention waits. |
| Negative controls | Without an agent ID the multi-agent action returned `INVALID_ARGS`; raw `mouse-click` without `--headed` returned `POLICY_DENIED`. Neither produced a GTK event. |
| Cleanup | Ending the session removed both cursor windows. The VM and its image were removed. |

The CLI itself returned `delivered_unverified`; the fixture callbacks establish the effects. The result does **not** establish two independent OS mice, parallel drags, macOS parity, or a shipped general mouse tool in Kit. In upstream `run.mjs`, `--cursor` starts a new session and defaults the agent identity to `jev-desktop`. Two independent `run --cursor` invocations are not the shared-session experiment above.

## Launcher trust checks

The opt-in launcher classifier matched ten predeclared first-round outcomes in a small synthetic catalog (116 ms p50, 172 ms p95; 44,103 input tokens). In the separate filename cases the raw choice was correct 6/7 times; the unchanged inspection gate made the intended offer/withhold decision 7/7 times. One advice question received `fill_only` at 0.58; it was not actionable. Neither sample estimates field accuracy on personal commands. [Reproduce the launcher holdout and typing trace](../../guides/jev-intent.md#measuring-readings).

The hidden Script Kit renderer, with fixture answers and no host input, showed `Open Slack → Tile Slack Right Half` on Enter, opened `brain:` search for fact recall, staged then cancelled `Quit Slack`, and explained that a mixed offline request could not run without claiming it was sent. It did not launch Slack, contact any messaging service, or test a real macOS Save sheet.
