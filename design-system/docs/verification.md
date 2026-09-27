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

Final results recorded on 2026-09-27 on the application that adopts Rose Atelier (D38), branch
`feat/rose-atelier-redesign` at `f71e5ac` (the six fixes from the first final battery: the app
shell at 400 % zoom and large text, the welcome staying clear of the view bar, primary buttons in
forced colours, the hidden import file inputs, Export disabled with no animation, and the canvas
redrawing after an operation made outside a gesture). Windows 11 Pro, Node 22.23.2, Playwright
1.63.0 with its Chromium; the editor served by `npm start` on port 4260 with the real engine
(`LP_ENGINE_PREBUILT=1`); the dev server had stopped between runs and was restarted identically.
This is evidence for the redesign, not a WCAG conformance certificate.

### Automated battery (final re-run)

| Command | Result |
| --- | --- |
| `npm run lint --prefix frontend` | PASS: ESLint on app, shared and admin; Prettier clean |
| `npm run test:ci --prefix frontend` | PASS: app 582, shared 53, admin 145 tests |
| `npm run test:tools --prefix frontend` | PASS: 43 tests, colour parity with `tokens/tokens.css` included |
| `npm run build --prefix frontend` | PASS: app and admin, no budget warning |
| `npm run i18n:check --prefix frontend` | PASS: both catalogues complete |
| `npm run i18n:check-bundle --prefix frontend` | PASS: no catalogue in 52 scripts |
| `npm run test:engine --prefix frontend` | PASS: 16 tests in Chromium |
| `npm run e2e --prefix frontend` | PASS: 10 tests; the accessibility file passed within the run |
| `npm run e2e:visual --prefix frontend` | PASS: 5 tests, screenshots at 1440, 1024, 768 and 390 px, both themes, both languages |
| `npm run e2e:hosted --prefix frontend` | NOT RUN: ports 8460, 8461, 8463 and 8464 are held by the local stack's `server` and `admin` containers, which the method forbids stopping |
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

### Review matrix for the editor (final re-check)

Chromium through Playwright, Windows 11, French interface, `http://localhost:4260/editor`.
Zoom and forced colours are emulated as noted: they approximate, and do not replace, the browser
setting and Windows contrast themes. The six defects recorded in the first final battery
(`docs/plans/reports/final-failures.md`, zones A–F) were re-checked live in this session, after
their fix commits (`bd468e1`, `f943f8e`, `0ec6729`, `f71e5ac`); each now passes.

| Review | Viewport and method | Result | Remaining defect |
| --- | --- | --- | --- |
| Keyboard | 1280 × 800; Tab, Enter, Escape and the shortcuts only | PASS: the skip link reaches the document bar; create, draw, layers, frames, tags, preview and export by keyboard (e2e); Ctrl+S opens "Sign in to save", focus inside, Escape returns the focus to its origin; "Import image" opens the file picker from Enter; the canvas now redraws on the next paint after an import or an undo/redo made outside a drawing gesture (`canvas.ts` reads the engine state on every publish) | — |
| Zoom 200 % text | 1280 × 800, root font size 200 %, and the viewport-shrink equivalent (640 × 400) | PASS: the editor's own grid (`.editor`, `overflow-y: auto`) scrolls to its full content in both simulations (checked: `scrollHeight` 2372 px in 537 px at root font size 200 %, `scrollTop` reaches its maximum at 640 × 400) | Root font size is a simulation of the browser's text-size setting, not the setting itself; recheck on a real machine |
| Zoom 400 % | 320 × 200 CSS px (1280 × 800 at 400 %) | PASS: the editor page is now 200 px tall (was 0 px); `ion-app` scrolls the shell (`max-height: 30rem` rule) and `.editor` scrolls its grid to the bottom (`scrollTop` reaches 745 of 745 px max) | — |
| 320 px | 320 × 640 | PASS with a reserve: no sideways page scroll; one column scrolls in 391 px under the fixed shell (249 px) | Same shell height |
| Forced colours | 1280 × 800, `forcedColors: 'active'` | PASS for "Export": ButtonText (`rgb(0,0,0)`) on ButtonFace (`rgb(255,255,255)`) with a black border, checked live; tools, swatch ring and check, active layer "Active", active frame, focus rings and dialog borders stay visible; the artwork keeps its colours | Outside the app's files: the admin's `.primary` button and both apps' skip link still use HighlightText on Highlight (not re-checked; owned by the admin/app-shell maintainer) |
| Appearance | 1280 × 800; settings and `prefers-color-scheme` / `prefers-reduced-motion` emulation | PASS: light, dark and system themes apply without reload, inside open Ionic dialogs too; "Reduce" stops the dialogs' 250 ms fade and slide; canvas surround and checkerboard neutral in both themes | — |
| Pointer and touch | 390 × 844, fine pointer | PASS for 24 px (WCAG 2.5.8); 37 of 53 editor controls are 36 px; the shared 44 px rule under `(pointer: coarse)` was not observed here | Touch device review |
| Semantics | Accessibility tree snapshots | PASS: named regions, one `main`, tabs named, `aria-current` on the active layer; each import control now exposes a single named button ("Importer une image", "Importer une planche de sprites"), its file input `aria-hidden`, checked live | — |
| Recovery | 1280 × 800 | PASS: "Exporter" is disabled on the welcome (no document), matching "Enregistrer"; failed preview and export states exist and are unit tested | — |
| Screen reader | — | NOT RUN: NVDA, VoiceOver and TalkBack cannot run in this environment | Human review |
| Contrast | Script plus axe on rendered screens | PASS: 64 token pairs; no axe contrast violation once animations end | — |

### Remaining human review

- NVDA on Windows for the editor, the welcome, the tabs, the view bar status and the dialogs;
  VoiceOver or TalkBack when a mobile surface is targeted.
- Browser text size at 200 % and Windows contrast themes on a real machine (this session used
  root-font-size and viewport-shrink simulations, both now passing).
- Touch targets on a coarse pointer, and the hosted journeys (`e2e:hosted`) once ports
  8460–8464 are free (the local stack's own `server`/`admin` containers hold them for other
  purposes; the method forbids stopping them).
- Forced colours outside the app's files: the admin's primary button and the skip links of both
  the app and admin shells (same HighlightText-on-Highlight pattern as the fixed "Export" button).

## Artwork completion (D39, 2026-09-27)

The [asset inventory](assets.md) records the added images, canonical sources and generation prompt.
This pass adds browser/touch/desktop icons, welcome artwork, library and export Pip variants,
theme previews and a not-found illustration. It also improves welcome and library reflow.

| Check | Observed result |
| --- | --- |
| `npm run lint --prefix frontend` | PASS: app/shared/admin ESLint and Prettier; generated visual output is excluded from formatting |
| `npm run test:ci --prefix frontend` | PASS: 583 app, 53 shared, 145 admin tests; 13 supporting-page tests also re-run after the final layout edits |
| `npm run test:tools --prefix frontend` | PASS: 43 tests, including complete asset copying and token parity |
| `npm run build --prefix frontend` | PASS: production app and admin, no budget warning |
| `npm run i18n:check --prefix frontend` | PASS: both catalogues complete; no UI copy added |
| `npm run i18n:check-bundle --prefix frontend` | PASS: catalogues remain outside the application scripts |
| `npm run e2e --prefix frontend` | PASS: 10 tests, including pointer/keyboard drawing, export/player and axe checks; settings axe re-run after the mobile refinement |
| `npm run e2e:visual --prefix frontend` | PASS: 14 tests, four widths, both languages and both themes; 80 additional library/export captures |
| `node design-system/scripts/check.mjs` | PASS: 64 contrast pairs, 146 preview keys, 25 reference asset links |
| Icon decoding and dimensions | PASS: SVG, PNG, ICO and ICNS; web 16/32/48 and touch 180; desktop through 1024 px |
| `cargo xtask build-desktop --debug` | BLOCKED at executable replacement: Windows denies removal of `target/debug/life-pixel-desktop.exe`, held by the running Life Pixel process. Sidecar, frontend and desktop library compiled; full desktop binary build is not verified |
| Instruction parity and whitespace | PASS: `AGENTS.md` and `CLAUDE.md` have identical SHA-256 hashes; `git diff --check` clean |

Reviewed real Chromium captures for welcome, library/project states, settings, not-found and export
completion at 1440, 1024, 768 and 390 px, across English/French and light/dark. At 390 × 844 the
French welcome fits its scroll region with the full guest notice visible. All 64 library captures
and 16 export-success captures report zero broken images, no horizontal page overflow and no
uncaught page errors. Library routes use synthetic API fixtures; this is frontend evidence, not a
hosted backend test. Screenshots wait for image decoding, fonts and finite Ionic animations.

The native desktop application was left running. Icon files were decoded and inspected, but the
installed application, macOS bundle and operating-system icon caches were not reviewed in this pass.
Existing screen-reader, touch-device and hosted-backend review limitations above remain applicable.
