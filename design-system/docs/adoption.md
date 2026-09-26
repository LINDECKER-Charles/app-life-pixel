# Adoption and maintenance

Status: isolated design proposal. The `design-system/` directory defines the visual direction,
component contracts and proposed journeys. Its reference screens are review artifacts, not live
routes, and do not implement persistence, the Rust engine, authentication or exports.

The warm cream and pale pink surfaces, raspberry action color and occasional chibi bunny Pip belong
to the product shell. Artist pixels and the drawing workspace remain neutral. Cute expression is
concentrated in welcome, empty and success moments; operational detail remains precise.

## Verified V1 integration points

Paths below are relative to the repository root. The audit read these sources; it did not run the
application. Production behavior must be verified when adopting each component.

| Current source | Responsibility and adoption seam |
| --- | --- |
| `frontend/projects/shared/src/styles/_tokens.scss` | Shared visual tokens |
| `frontend/projects/shared/src/styles/_ionic.scss` | Ionic variables mapped to shared tokens |
| `frontend/projects/shared/src/styles/_accessibility.scss` | Focus and reduced-motion rules |
| `frontend/projects/shared/src/styles/_index.scss` | App shared-style entry point |
| `frontend/projects/shared/src/lib/appearance/apply-appearance.ts` | Root appearance attributes |
| `frontend/projects/shared/src/public-api.ts` | API/i18n/appearance exports; no UI kit today |
| `frontend/projects/app/src/styles.scss` | Ionic CSS loaded before shared tokens |
| `frontend/projects/admin/src/styles.scss` | Native controls, tables and uPlot colors |
| `frontend/projects/app/src/app/app.routes.ts` | Lazy app routes and capability guards |
| `frontend/projects/admin/src/app/app.routes.ts` | Guarded admin routes |
| `frontend/projects/app/src/app/shell/app-header.html` | App navigation and account slot |
| `frontend/projects/app/src/app/editor/editor-page.html` | Editor region composition |
| `frontend/projects/app/src/app/editor/editor-page.scss` | Desktop grid; stacked below 768 px |
| `frontend/projects/app/src/app/tools/tool-bar.html` | Native controls with shortcuts and toggles |
| `frontend/projects/app/src/app/canvas/render/canvas-renderer.ts` | Neutral drawing checkerboard |
| `frontend/projects/app/src/app/palette/palette-panel.html` | Palette state and user colors |
| `frontend/projects/app/src/app/timeline/timeline.html` | Layers, frames, tags and playback |
| `frontend/projects/app/src/app/library/lists/animation-list.html` | Search, rows and pagination |
| `frontend/projects/app/src/app/library/lists/project-list.html` | Project creation and actions |
| `frontend/projects/app/src/app/library/save/save-dialog.html` | Save/quota/conflict decisions |
| `frontend/projects/app/src/app/export/export-dialog.ts` | Ionic export overlay composition |
| `frontend/projects/app/src/app/account/password-field.ts` | Existing reusable password control |
| `frontend/projects/app/src/app/settings/settings-page.html` | Appearance/platform preferences |
| `frontend/projects/admin/src/app/ui/confirm-action.html` | Native confirmation dialog |
| `frontend/projects/admin/src/app/ui/empty-state.ts` | Different reasons for missing admin data |
| `frontend/projects/admin/src/app/monitoring/chart-renderer.ts` | uPlot and chart tokens |

## Preserve the token contract

V1 already has CSS custom properties. Adopt through that surface before replacing feature markup.
Retain these aliases until every consumer has moved, including Ionic RGB properties and admin.

| Existing token family | Design-system meaning |
| --- | --- |
| `--lp-color-background` and `-rgb` | Main application background |
| `--lp-color-surface` | Grouped or raised supporting region |
| `--lp-color-text` and `-rgb` | Primary readable foreground |
| `--lp-color-text-muted` | Supporting text with normal-text contrast |
| `--lp-color-border` | Strong boundary for perceivable controls |
| `--lp-color-accent` and `-rgb` | Primary action and selection emphasis |
| `--lp-color-on-accent` and `-rgb` | Foreground on primary action |
| `--lp-color-danger` and `-rgb` | Destructive action and error emphasis |
| `--lp-color-on-danger` and `-rgb` | Foreground on filled danger |
| `--lp-color-focus` | Keyboard focus indicator |
| `--lp-space-1` through `--lp-space-6` | Existing spacing scale |
| `--lp-radius-small/medium/large/round` | Shape scale by component purpose |
| `--lp-font-family` and `--lp-font-family-mono` | UI and technical text families |
| `--lp-font-size-small/body/large/title` | Existing type hierarchy |
| `--lp-font-weight-regular/bold` | Existing text weights |
| `--lp-font-line-height` | Body line-height |
| `--lpa-chart-1` through `--lpa-chart-4` | Distinguishable operational series |
| `--lpa-chart-grid` | Decorative chart grid |

Add a separate subtle border token before replacing pale panel dividers. Do not globally soften
`--lp-color-border`: V1 uses it on real inputs and controls. Add semantic success, warning and info
roles with measured foreground/background pairs. Do not repurpose brand pink as every status color.

The present spacing suffixes are ordinal: `--lp-space-5` is 1.5 rem and `--lp-space-6` is 2 rem.
Keep existing meanings or migrate explicitly; never assume the suffix is a count of four pixels.
When design tokens become executable production assets, establish one canonical source and generate
other formats from it. Do not maintain independent hand-edited palettes in the preview and SCSS.

## Appearance, Ionic and rendering constraints

Theme state uses root `data-theme="system|light|dark"`. System follows `prefers-color-scheme`;
an explicit light preference overrides dark media queries. Motion uses root `data-motion` and
`prefers-reduced-motion`. Preserve preference persistence and immediate application without reload.

Ionic must continue to receive background/text RGB, primary/danger contrast, tint and shade values.
Ordinary CSS selectors do not reach Ionic shadow content. Prefer documented Ionic custom properties
and parts, and retain host focus handling alongside native `:focus-visible`. Avoid `::ng-deep`.
App dialogs use Ionic; admin dialogs use native HTML. Sharing visual tokens does not require adding
Ionic to the admin or replacing working focus management.

Overlays may render outside a component's local subtree. Place theme variables on the root, and
scope overlay classes deliberately. Review stacking: current engine notifications use a high
z-index; a token scale must account for Ionic overlays and browser top-layer dialogs.

The app uses Angular standalone components, OnPush, signals, `input()`/`output()` and `inject()`.
Use focused shared primitives or directives only when actual repeated behavior justifies them.
Keep feature orchestration in its existing services. Never place HTTP or drawing logic in a button,
modal wrapper or design token module.

The Rust engine owns document limits, rendering and validation. Preserve worker execution and
`editor-wasm` boundaries. UI preview illustrations are not alternate compositors. Do not ship a
JavaScript pixel engine, fabricated quota values or duplicated frame/palette limits.

V1 has a 30-second worker recovery snapshot after edits; it is not persistent autosave. Save status
must follow library operations. A design that says “Autosaved” requires separately implemented and
tested persistence. Current library items lack the thumbnail/recent/sort features suggested
by richer gallery concepts; classify those as feature work before production adoption.

## Runtime language and assets

All production user-facing strings are runtime Transloco keys in the English and French catalogues.
Do not import catalogue JSON into a component bundle. New labels, tooltips, accessible names, error
recovery copy and status messages land in both languages together.

Keep API error codes stable and translate their parameters in the interface. Static design documents
remain English. Reference-preview fixture copy is documentation, never a replacement production
catalogue. Do not reuse example names, successful save messages or mocked values as live state.

Use local assets and local fonts with recorded provenance and compatible licenses. The system font
fallback must remain complete while custom fonts load. Decorations have empty alt text; meaningful
illustrations receive concise translated descriptions only when they convey additional information.
Pip does not speak for a real agent, promise support availability or block access to a next step.

## Phased rollout

### 1. Review the isolated system

Review foundations, component contracts and journeys together with the reference preview. Record
which screens show existing behavior and which propose navigation/features. Evaluate desktop,
small laptop, touch and mobile editor density before selecting production layouts.
Resolve any proposed architectural decision in `docs/decisions.md` with the maintainer first.

### 2. Adopt foundations

Map approved semantic tokens onto `_tokens.scss`; preserve compatibility aliases and theme handling.
Update the Ionic mapping and admin chart pairs in the same change. Check focus, inputs, selected
swatches, modal labels and notification contrast before expanding to page layout.
Keep neutral rendering colors independent from the shell palette.

### 3. Consolidate proven repeated controls

Start with button treatment, form-field spacing, status/empty messages and dialog anatomy. Reuse
`password-field`, `confirm-action` and existing feature services. Add a shared primitive only after
its behavior and consumers are clear; do not replace every native element with a wrapper.
Document the input/output contract and test observable behavior where it changes.

### 4. Implement the first-creation journey

Adopt the welcome/new-animation/editor/export sequence incrementally. Keep the current `/editor`
entry usable while a new landing or onboarding route is developed. New route behavior is feature
work, with translated labels, keyboard navigation and tests, not a token-only change.
Review canvas area at the smallest supported workspace and French string expansion.

### 5. Extend library and supporting journeys

Adopt saving, conflict resolution, library, settings, account and support. Preserve current
capability guards for hosted and desktop features. Add gallery thumbnails or additional sorting
only with real data contracts, loading behavior and tests. Preserve recoverable work on failures.

### 6. Adopt admin and distribution variants

Apply restrained admin styles to tables, filters, confirmations and charts. Check production/staging
context, destructive targets and unavailable data. Validate the web, desktop and Android shells,
including touch, safe areas, virtual keyboard and available viewport height.

Each phase is a reviewable change returning to `dev` through a PR. No design phase requires a direct
commit to `main`/`test`, a production deployment or a change to core architecture.

## Verification matrix

| Change | Required evidence |
| --- | --- |
| Documentation or static preview | Valid links, token consistency and visual review |
| Color/type/spacing tokens | Light/dark contrast, native/Ionic controls, zoom and theme switching |
| Form or dialog behavior | Unit tests, focus/keyboard checks, error and pending behavior |
| Editor layout or input controls | Draw/animate/export/play journey and keyboard equivalent |
| Library/account/support | Relevant hosted E2E journey and failure recovery |
| Admin UI | Role/environment context, tables, charts and hosted accessibility checks |
| Runtime copy | Both catalogues complete; `i18n:check` and bundle checks |

Current facilities are Angular unit tests with Vitest, browser engine tests through Chromium,
Playwright editor/hosted suites and axe WCAG 2.2 checks. Axe suites block serious and critical
issues; that does not replace manual keyboard, contrast, zoom, touch and screen-reader checks.
Add viewport/theme coverage where a design change introduces a new risk.

Commands from the repository root, selected according to what changes:

```sh
npm ci --prefix frontend
npm run lint --prefix frontend
npm run test:ci --prefix frontend
npm run build --prefix frontend
npm run i18n:check --prefix frontend
npm run i18n:check-bundle --prefix frontend
npm run test:engine --prefix frontend
npm run e2e --prefix frontend
npm run e2e:hosted --prefix frontend
```

`test:ci` uses a mock engine for ordinary Angular tests. `start`, `build` and `test:engine` invoke
the Rust editor build; `LP_ENGINE_PREBUILT=1` skips it only when generated JS/types/WASM exist.
The editor E2E server runs on port 4260 and starts the real engine. Hosted E2E requires its local
server/database stack. At audit time frontend dependencies and generated engine assets were absent;
this document does not claim these suites ran. Report unavailable checks explicitly.

## Contribution and version governance

The maintainer approves visual direction and product journeys. Feature owners validate behavior;
reviewers validate accessibility and translations. Design proposals cannot silently approve an
architectural change or alter a domain rule.

Use an explicit design-system revision in its index once the first review is accepted. Keep a short
change record alongside the system: decision, affected tokens/components, migration and validation.
Before acceptance, label revisions as proposals; do not imply a released public package version.

- Patch: clarification or correction without changing an approved visual/behavioral contract.
- Minor: additive token, component variant or pattern with existing consumers still valid.
- Major: removed/renamed token, changed semantic meaning or incompatible component interaction.

Deprecate before removal: name the replacement, list consumers and define the migration release.
Update documentation, preview examples, both language catalogues when applicable, and production
consumers together. Screenshot references support review; behavior tests remain the acceptance gate.

PR review checklist:

- The change solves a named user task and states whether it is design-only or implemented.
- Existing routes, engine contracts, storage capabilities and error codes remain truthful.
- Token usage is consistent; strong control boundaries do not become decorative pink lines.
- Every new state has clear copy, focus behavior and a recovery or next action.
- Light/dark, motion preferences, long translations and small layouts have been inspected.
- Relevant checks report observed results, including any tool or environment limitation.
- User documentation and technical implementation agree; no illustrative interaction is passed off
  as a shipped feature.
