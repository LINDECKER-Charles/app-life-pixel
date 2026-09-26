# Reference verification

Recorded on 2026-09-26 for Rose Atelier 0.1.0. These results cover the isolated design reference,
not a redesigned production application or a full WCAG conformance audit.

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
