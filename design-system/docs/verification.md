# Reference verification

Recorded on 2026-09-26 for Rose Atelier 0.1.0. These results cover the isolated design reference,
not a redesigned production application or a full WCAG conformance audit.
The results observed on the redesigned application are recorded separately below, in
[Application verification](#application-verification).

## Automated results

| Check | Result |
|---|---|
| `node design-system/scripts/check.mjs` | PASS: 64 contrast pairs, 146 translation references |
| Catalogue integrity | PASS: EN/FR parity, active languages and instruction equality |
| Asset integrity in the same check | PASS: 25 unique local HTML/CSS/module references |
| `node design-system/scripts/browser-check.mjs` | PASS: 102 assertions in Chromium |
| Browser runtime | No unexpected HTTP/page errors or external resource requests |
| Browser widths | 1440, 1024, 768, 390 and 320 CSS px; all five sections have no page overflow |
| Prettier 3.9.8 | PASS: new HTML/CSS/MJS and the two modified translation catalogues |
| `git diff --check` | PASS: no whitespace errors in tracked changes |
| Original assets | SVGs render locally; Nunito loads from the bundled font file |

The static checker reads the canonical light/dark tokens. It verifies normal text at 4.5:1 and
control/focus boundaries at 3:1 against specified surfaces, plus status foreground/background
pairs. Decorative borders are deliberately excluded from the control-boundary claim. It does not
measure every possible rendered combination; production composition must be checked again.

Browser assertions cover single-section navigation, local catalogue loading, labels, language
switching, retained user input, dark mode, form validation, escaped user text, Escape and focus
restoration, tool/colour/frame selection, zoom, export-format disclosure, reduced motion and the
skip link. A simulated catalogue request failure verifies a visible error and a selector reset
that lets the user retry. No drawing, persistence or export compilation is claimed by this demo.

Five screenshots were produced under ignored `design-system/review/`:

- `overview-desktop.png`: French welcome, Pip, brand palette and desktop navigation.
- `studio-desktop.png`: French editor layout and selected control states.
- `studio-dark.png`: English dark-theme editor with the neutral artwork checkerboard.
- `overview-mobile.png`: stacked welcome at 390 px.
- `studio-mobile.png`: horizontal toolbar, canvas, inspector and timeline at 390 px.

Desktop and mobile compositions, theme contrast, typography, mascot proportions and canvas fidelity
were visually inspected. Captures scroll to the document top and finish finite CSS animations to
avoid photographing a partially transitioned theme or offset fixed navigation.

## Local execution details

The preview server binds only to `127.0.0.1:4265`. Server smoke checks verified HTML MIME types,
cache policy, catalogues, HEAD requests, the root redirect, traversal rejection and denial of
repository sources, scripts and `.env`. Only design assets/reference documents and the three
approved i18n JSON files are exposed.

This checkout has no `frontend/node_modules`. Browser verification used the available bundled
Playwright runtime with an explicitly selected installed Chromium executable. To run with the
frontend's normal dependencies, install its declared packages and Playwright browser as usual,
then run the documented script. `PLAYWRIGHT_CHROMIUM_EXECUTABLE` can select an already installed
compatible Chromium; `PREVIEW_URL` can point to a different local preview port.

The standalone reference itself requires only Node's standard library and a current browser.
Prettier was run from the local package cache; no package manifest or lockfile was modified.

## Checks that could not run

`npm run i18n:check --prefix frontend` was attempted and stopped with `ERR_MODULE_NOT_FOUND` for
`@messageformat/parser`, because frontend dependencies are absent. The standalone checker passed
catalogue parity and reference validation; it does not substitute for the repository's ICU parser
and legal-page checks. New preview messages contain no ICU parameters or plurals.

Production Angular lint/unit/build/engine checks and Rust/server/desktop checks were not run:
production code and build configuration were not changed. Run the relevant application checks
during adoption, following [adoption.md](adoption.md).

## Remaining human review before production adoption

- Review the direction and information hierarchy with the maintainer and representative creators.
- Test the real editor at 200% and 400% zoom, with long translations and real project names.
- Verify focus and announcements with screen readers, Ionic overlays and system high contrast.
- Test real drawing, keyboard pixel editing, imports, animation playback and exported files.
- Exercise live account, quota, conflict, local storage and offline failures.
- Validate Android touch/gesture behaviour when M6 is implemented; no mobile release is implied.

The [accessibility checklist](accessibility.md) and [journey criteria](journeys.md)
define the remaining acceptance work. Automated checks provide evidence, not certification.

## Application verification

Recorded on 2026-09-27 on the application that adopts Rose Atelier (D38), branch
`feat/rose-atelier-redesign` at `e1f36e7` plus the final end-to-end and documentation changes.
Windows 11 Pro, Node 22.23.2, Playwright 1.63.0 with its Chromium; the editor served by
`npm start` on port 4260 with the real engine (`LP_ENGINE_PREBUILT=1`). This is evidence for the
redesign, not a WCAG conformance certificate.

### Automated battery

| Command | Result |
| --- | --- |
| `npm run lint --prefix frontend` | PASS: ESLint on app, shared and admin; Prettier clean |
| `npm run test:ci --prefix frontend` | PASS: app 576, shared 53, admin 145 tests |
| `npm run test:tools --prefix frontend` | PASS: 43 tests, colour parity with `tokens/tokens.css` included |
| `npm run build --prefix frontend` | PASS: app and admin, no budget warning |
| `npm run i18n:check --prefix frontend` | PASS: both catalogues complete |
| `npm run i18n:check-bundle --prefix frontend` | PASS: no catalogue in 52 scripts |
| `npm run test:engine --prefix frontend` | PASS: 16 tests in Chromium |
| `npm run e2e --prefix frontend` | PASS: 10 tests; the accessibility file also passed four times in a row (32) |
| `npm run e2e:visual --prefix frontend` | PASS: 5 tests, screenshots at 1440, 1024, 768 and 390 px, both themes, both languages |
| `npm run e2e:hosted --prefix frontend` | NOT RUN: ports 8460, 8461, 8463 and 8464 are held by the local stack's `server` and `admin` containers, which the suite needs free |
| `node design-system/scripts/check.mjs` | PASS: 64 contrast pairs, 146 preview keys, 25 assets |
| `cmp CLAUDE.md AGENTS.md` | PASS: identical |

The editor suite runs the whole first-creation journey twice — with the pointer, then with the
keyboard alone —: welcome, create, draw two frames, the second on a layer chosen in the inspector,
durations, tags, undo and redo, play the preview, export, then play the downloaded WASM export.
axe (WCAG 2.2 AA tags) finds no serious or critical violation on the welcome, the editor, the
export, shortcuts and settings screens, and at 390 px on the welcome, each inspector tab and the
export dialog. axe now waits for the animations of Ionic's shadow roots, which
`document.getAnimations()` does not list: measuring a dialog while it fades in had produced an
intermittent `color-contrast` failure on the shortcut key caps.

Measured by the `visual` project: at 1366 × 768 and 1024 × 768 the canvas stage is 405 px high
(0.53 of the window) and a 32 × 32 animation at zoom 8 is seen whole; at 320 × 640 the page does
not scroll sideways; every routed page reachable without the hosted stack scrolls to its end at
390 × 640.

### Review matrix for the editor

Chromium through Playwright, Windows 11, French interface, `http://localhost:4260/editor`.
Zoom and forced colours are emulated as noted: they approximate, and do not replace, the browser
setting and Windows contrast themes.

| Review | Viewport and method | Result | Remaining defect |
| --- | --- | --- | --- |
| Keyboard | 1280 × 800; Tab, Enter, Escape and the shortcuts only | PASS: the skip link reaches the document bar; create, draw, layers, frames, tags, preview and export by keyboard (e2e); Ctrl+S opens "Sign in to save", focus inside, Escape returns the focus to its origin; "Import image" opens the file picker from Enter | After an image import the canvas shows the new pixels only after the next operation (the frame thumbnail shows them at once) |
| Zoom 200 % text | 1280 × 800, root font size 200 % | FAIL: the desktop grid keeps its layout (rem media queries do not follow a root style), the rail clips its tools, the stage shrinks to about 110 px and the lower inspector and the timeline are cut off with no page scroll | Recheck with the browser's text-size setting, which moves the rem breakpoints |
| Zoom 400 % | 320 × 200 CSS px (1280 × 800 at 400 %) | FAIL: the header (133 px) and the footer (116 px) take the whole height; the editor gets 0 px | The shell's header and footer never scroll away |
| 320 px | 320 × 640 | PASS with a reserve: no sideways page scroll; one column scrolls in 391 px under the fixed shell (249 px) | Same shell height |
| Forced colours | 1280 × 800, `forcedColors: 'active'` | Partial: tools, swatch ring and check, active layer "Active", active frame, focus rings and dialog borders stay visible; the artwork keeps its colours | Primary buttons ("Export", "Back to editing" observed) show a blank label: text in `HighlightText` over the browser's `Canvas` backplate |
| Appearance | 1280 × 800; settings and `prefers-color-scheme` / `prefers-reduced-motion` emulation | PASS: light, dark and system themes apply without reload, inside open Ionic dialogs too; "Reduce" stops the dialogs' 250 ms fade and slide; canvas surround and checkerboard neutral in both themes | — |
| Pointer and touch | 390 × 844, fine pointer | PASS for 24 px (WCAG 2.5.8); 37 of 53 editor controls are 36 px; the shared 44 px rule under `(pointer: coarse)` was not observed here | Touch device review |
| Semantics | Accessibility tree snapshots | PASS with a reserve: named regions, one `main`, tabs named, `aria-current` on the active layer | The hidden file inputs of the two import buttons are exposed as extra "Import image" and "Import sprite sheet" buttons, enabled with no animation |
| Recovery | 1280 × 800 | Partial: failed preview and export states exist and are unit tested | "Export" is enabled with no animation, and its dialog then lists every format as "Could not export" |
| Screen reader | — | NOT RUN: NVDA, VoiceOver and TalkBack cannot run in this environment | Human review |
| Contrast | Script plus axe on rendered screens | PASS: 64 token pairs; no axe contrast violation once animations end | — |

### Remaining human review

- NVDA on Windows for the editor, the welcome, the tabs, the view bar status and the dialogs;
  VoiceOver or TalkBack when a mobile surface is targeted.
- Browser text size at 200 % and Windows contrast themes on a real machine.
- Touch targets on a coarse pointer, and the hosted journeys once ports 8460–8464 are free.
