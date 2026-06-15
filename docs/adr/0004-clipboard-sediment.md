# Clipboard Sediment

Clipboard content becomes memory through earned importance, not per-copy consent dialogs. URLs copied with Cmd+C are kept automatically (inert, low-noise, high-value). Other content is kept when behavior proves value — a re-paste. Kept copies land in a low memory tier and promote through signals (re-paste, recall in chat, `@` reference), reusing the existing attention-signal architecture so resurfacing stays high-signal.

We chose earned tiers over keep-everything-equally (recall drowns in mechanical copy-paste junk and the resurfacing moment dies), over confirm-every-copy (a HUD on 50 copies/hour becomes ignored noise, which destroys its value as a consent signal), and over keep-nothing-without-explicit-action (the product bias is that capture should cost zero decisions).

Constraints: hard rejection rules are non-negotiable and precede every other rule — `org.nspasteboard.ConcealedType` pasteboard items, configured password-manager app sources, and user-defined secret patterns (token/key regexes) are never stored. This is a launch-facing privacy claim: per the AI claim levels in `VISION.md`, do not use "enforced" language until source and receipts prove the rejection path. Clipboard capture must not open per-copy UI.
