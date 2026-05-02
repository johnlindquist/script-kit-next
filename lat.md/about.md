# About

This page documents the launcher-native About route, including layout, keyboard behavior, update state, and shared branding data sources.

## Route contract

The About route is a full-window `AppView` surface restored back to its previous launcher route on explicit dismissal.

[[src/main_sections/app_view_state.rs#AppView]] declares `About` with prior-route state and an update-state handle. Its contract uses `FeedbackSurface`, `NoEditableInput`, `ContentPane`, `DismissPolicy::explicit_cmd_w_only()`, and semantic surface `about`.

## Layout

The About layout presents product identity, version, creator attribution, community links, update status, and acknowledgements without persistent instructional chrome.

[[src/about/render.rs#render_about_surface]] renders the surface with the shared chrome tokens, a 52px header, centered content card, quick-action row, update card, acknowledgements disclosure, and footer.

## Update states

The update card mirrors the tray update state so users can check and open releases from the launcher route.

[[src/updates.rs#UpdateState]] drives the copy: idle shows the current version, checking disables the button, up-to-date confirms freshness, available opens the release URL, and errors stay retryable.

## Keyboard behavior

The surface owns focus without exposing the launcher filter input, so keyboard navigation stays on About controls.

Escape dismisses back to the previous route, Tab walks the close, link, update, and acknowledgements controls, and Enter or Space activates the focused control through GPUI button semantics.

## Branding constants

Brand copy and URLs are shared so the tray and launcher-native About surface do not drift.

[[src/branding.rs#LOGO_SVG]] stores the shared logo source beside the GitHub, Discord, X, app-name, and tagline constants. Tray rendering keeps its SVG rasterization helper local while importing the shared data.

## Storybook coverage

Storybook covers the About surface as a canonical launcher state with update-state variants.

The `about_surface/default` story renders the 750x500 route for Idle, Checking, UpToDate, Available, and Error states so the route can be inspected without opening the tray menu.
