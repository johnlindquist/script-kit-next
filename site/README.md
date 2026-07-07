# scriptkit.com — static site

The static marketing site for Script Kit (GPUI). One self-contained
`index.html` (inline CSS, no build step), the numbered screenshot set in
`images/`, and `vercel.json` for static deploys.

- **Owning flow:** `flows/site.md` (page content, download links, deploys).
- **Screenshots:** captured live from the running app; regeneration is owned
  by `flows/screenshots.md`. Raw PNGs live in `.test-screenshots/glamour/`;
  published JPEGs live here in `images/` under the same numbered names.
- **Origin:** adapted from the here.now visual tour published at
  `royal-riddle-za23.here.now` (2026-07).

## Download-link contract

Download buttons use the version-agnostic GitHub form, which always resolves
to the newest release — never hardcode a version:

    https://github.com/johnlindquist/script-kit-next/releases/latest/download/Script-Kit-macos.zip

## Preview locally

    python3 -m http.server --directory site 4173

## Deploy

    cd site && vercel deploy --scope script-kit    # preview
    # production deploys and any scriptkit.com domain changes require
    # explicit user approval — see flows/site.md

The legacy scriptkit.com (Next.js `script-generator` project in the
`script-kit` Vercel team) stays deployed at its .vercel.app URL as the
archive of the old site.
