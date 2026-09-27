# Application identity assets

The Rose Atelier pixel heart is shared by the web app, admin console and desktop packages.
These are original project assets under AGPL-3.0-only, derived from `../mark.svg`.

- `favicon.svg` uses a compact cream tile to stay legible in light and dark browser tabs.
- `app-icon.svg` adds clear space for home-screen and desktop launchers.
- Both web apps serve the SVG, a 16/32/48 px ICO fallback and a 180 px Apple touch icon.
- `tauri/icons/` contains PNGs, a multi-resolution Windows ICO and a macOS ICNS container.

Regenerate all derivatives from the repository root with:

```shell
node design-system/scripts/generate-icons.mjs
```

The generator uses the frontend's existing Playwright installation and Chromium. It rasterizes
the SVG directly with crisp pixel edges, preserving the source palette across themes. Edit the
canonical mark, then regenerate; do not retouch the generated bitmaps independently.

The plain source mark remains transparent for in-app branding. Only operating-system and browser
identity icons have the cream tile. No application manifest or installation capability is implied.
