# Application artwork

The 2026-09-27 visual pass completes the Rose Atelier identity across the application and its
distributed icons (D39). Artwork stays local, contains no UI text, and is decorative where the
adjacent heading or control already explains the context. No asset enters a compiled animation.

## Inventory

| Asset | Source under `design-system/assets/` | Used by |
| --- | --- | --- |
| Pixel heart | `mark.svg` | App header; source of all identity derivatives |
| Browser icon | `identity/favicon.svg` | App and admin tabs, with 16/32/48 px ICO fallback |
| Application icon | `identity/app-icon.svg` | 180 px touch icons and Tauri PNG/ICO/ICNS |
| Atelier welcome | `illustrations/pip-atelier.webp` | Empty editor; 768 × 576, about 30 kB |
| Library Pip | `illustrations/pip-library.svg` | Empty library and project, 48-unit grid |
| Export Pip | `illustrations/pip-export.svg` | Confirmed download, 48-unit grid |
| Missing frame | `illustrations/missing-frame.svg` | Not-found page; no mascot |
| Theme previews | `illustrations/appearance/{light,dark,system}.svg` | Existing settings radio cards |

Library rows use the existing `lp-icon` folder and frame symbols on token-coloured tiles. These
identify the item type and never impersonate a preview of the user's artwork. Search, loading,
errors, authentication, account actions and admin screens retain their restrained presentation.
The canvas, checkerboard and animation pixels are unchanged.

## Sources and reproduction

The favicon and desktop icons derive from the existing vector heart. After installing the
frontend dependencies and its Playwright Chromium browser, run from the repository root:

```shell
node design-system/scripts/generate-icons.mjs
```

This writes the canonical icon derivatives, both apps' public icon files, and all existing
`tauri/icons/` formats. Keep the derivatives and their source in the same change. It does not
change bundle configuration or add installation capabilities.

The welcome was generated with the built-in ImageGen tool, using a rendered copy of `pip.svg`
as its identity reference. The unmodified result is `sources/pip-atelier.png` (1448 × 1086).
The production WebP is a 768 × 576 encoding at quality 0.9 with alpha preserved, generated via
Chromium canvas. Keep the PNG as the editable source; only the WebP ships in the application.
The other new illustrations are editable native SVGs extending the original Rose Atelier assets.
All project artwork follows the repository's AGPL-3.0-only licence; Nunito retains its OFL licence.

`frontend/tools/copy-design-assets.mjs` explicitly lists the production illustrations and fonts.
Sources, asset documentation and design references do not ship in the app. Each illustration has
fixed dimensions to avoid layout shift; there is no automatic decorative motion or remote request.

## Welcome generation prompt

```text
Use case: illustration-story
Asset type: production decorative welcome illustration for Life Pixel, a pixel-animation editor with the accepted Rose Atelier identity.
Primary request: create an original, restrained pixel-art vignette of Pip at a tiny creative atelier, suitable for a 240 by 180 CSS pixel illustration.
Input image 1 is the existing Pip mascot identity reference: preserve cream rabbit fur, upright asymmetric ears, dark muted plum pixel outlines, rose cheeks and tiny raspberry heart. Keep the character recognizably the same, one single Pip only.
Scene: Pip stands beside a small cream-and-rose artist's easel showing a tiny raspberry pixel heart, with a small mint sprout and a few carefully placed loose pixel squares. A small blush ground tile anchors the group. Very simple composition, no room background.
Style: authentic clean hand-crafted pixel art with a consistent coarse square grid, stepped edges, large readable clusters, 2 or 3 flat tones per object. No outlines thinner than one logical pixel. Soft cream, muted plum #734958, raspberry #a93663, rose #f5b5c9 and restrained mint accents.
Composition: compact centered horizontal vignette, 4:3 image, entire ears/easel/feet visible with generous transparent clear space all around. Designed to read at small size. The easel is secondary, Pip is the focus.
Constraints: actual transparent background, no text or letters, no interface mockup, no photo, no glossy 3D, no blur or gradients, no confetti animation, no external brands, no extra characters, no fake browser frame.
```

## Review

Run `npm run e2e:visual --prefix frontend` and review the four widths (1440, 1024, 768 and 390),
both languages and both themes. Image decoding is awaited before capture. Library screenshots
use explicit API fixtures to exercise the actual app components without a hosted account.
See [verification.md](verification.md) for commands, observed results and limitations.
