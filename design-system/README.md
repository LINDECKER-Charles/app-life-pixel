# Life Pixel design system

**Rose Atelier · 0.1.0 · Design proposal**

A warm, light-pink creative workspace with a small chibi companion. Soft cream surfaces,
raspberry actions, rounded typography and crisp pixel details welcome the artist; neutral canvas
surroundings keep the artwork accurate. This folder is the complete design reference for the
next UX pass on Life Pixel V1.

## Open the interactive reference

From the repository root, with Node.js available:

```shell
node design-system/scripts/serve.mjs
```

Open [the local reference](http://127.0.0.1:4265/design-system/preview/). It needs this local
server because pages and the active translation catalogue load at runtime. No application build,
Rust engine, account, external font request or new npm dependency is required.

The reference includes five views: universe, foundations, components, studio and journeys.
Switch French/English and light/dark, open and validate a creation dialog, navigate with the
keyboard, select tools/colours/frames, change zoom and open the export-format panel.
The studio is an interaction and layout specimen: it does not draw, compile, persist projects
or download animation exports. These operations remain the real application's responsibility.

## Read in this order

| Reference | What it defines |
|---|---|
| [Vision](docs/vision.md) | Personality, hierarchy, identity, assets and boundaries |
| [Foundations](docs/foundations.md) | Colours, type, spacing, themes, density, layout and motion |
| [Components](docs/components.md) | Anatomy, variants, states and keyboard contracts |
| [Journeys](docs/journeys.md) | Creation, integration, library, recovery and distribution |
| [Patterns](docs/patterns.md) | Compositions, errors, loading, empty and destructive states |
| [Content](docs/content.md) | Voice, bilingual microcopy, terminology and runtime translation |
| [Accessibility](docs/accessibility.md) | Contrast, focus, input alternatives and acceptance |
| [Adoption](docs/adoption.md) | Code mapping, rollout, ownership and definition of done |
| [Verification](docs/verification.md) | Checks, recorded results and remaining manual review |

## Source of truth

| Path | Role |
|---|---|
| `tokens/tokens.css` | Canonical semantic CSS tokens: light, dark, system and forced colours |
| `assets/` | Original pixel mascot, mark and sprout; locally hosted Nunito and its licence |
| `preview/` | Responsive browser reference using those tokens |
| `../i18n/en.json`, `../i18n/fr.json` | All preview UI copy under `design_system.*` |
| `scripts/` | Local allowlisted static server and reference verification |

Use semantic tokens, not copied colour literals. Artwork has its own fixed palette: theme changes
affect interface chrome, never recolour a user's animation. Reference colour swatches intentionally
show the light-theme palette in both themes. All other UI surfaces follow the selected theme.

## Scope and current status

- **Designed:** visual foundations, component contracts, UX flows and distribution adaptations.
- **Implemented here:** tokens, original SVG assets, local font and an interactive reference.
- **Existing V1 audited:** editor, exports, authentication, library, MCP, desktop and admin.
- **Next adoption work:** apply the system incrementally to Angular/Ionic using
  [the implementation map](docs/adoption.md). Existing production SCSS and application components
  are unchanged by this proposal.

Welcome guidance, illustrated library thumbnails and other UX improvements are explicitly marked
as proposals. Android is a future adaptation. There is no invented V1 billing, gallery, cloud sync,
team workspace or generative AI service. No architecture decision is silently accepted here.

## Check the reference

```shell
node design-system/scripts/check.mjs
```

For browser verification, start the server in another terminal and use the Playwright dependency
provided by the frontend workspace (after its normal `npm ci`):

```shell
node design-system/scripts/browser-check.mjs
```

Browser captures are written under `design-system/review/` and ignored by Git. The design reference
has no runtime JavaScript dependency. Browser verification is optional tooling, not a requirement
to view the reference. See [verification](docs/verification.md) for the actual checks run.

## Versioning

This is a 0.1.0 proposal, ready for review and incremental implementation. The maintainer owns
acceptance. Update the affected tokens, contracts, reference, both catalogues and verification
evidence together. Record breaking token or interaction changes before migration; do not replace
an existing production token silently. See [adoption](docs/adoption.md) for release gates.

Project source and original design assets follow the repository's AGPL-3.0-only licence. The bundled
Nunito font retains its [SIL Open Font License](assets/fonts/OFL.txt). These assets are not part of
the MIT player runtime or automatically included in users' compiled animation exports.
