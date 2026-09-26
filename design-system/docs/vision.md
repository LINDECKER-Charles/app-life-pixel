# Vision: Rose Atelier

## The promise

Life Pixel feels like a small, welcoming animation atelier. It gives beginners a gentle first
step and experienced artists a precise workspace. The character comes from light pink, creamy
paper, soft corners and small pixel illustrations. The product remains a capable creative tool:
clarity, artwork fidelity and recovery always take priority over decoration.

The public product name remains **Life Pixel**. **Rose Atelier** is the name of this design system,
not a new product brand or an account tier. French reference copy calls it **Atelier Rose**.

## Personality and decisions

| Trait | Express it through | Avoid |
|---|---|---|
| Warm | Cream, rose, caring practical copy | White glare, cold corporate copy |
| Cute | One companion, rounded letters, pixel details | Baby talk or childish task names |
| Precise | Neutral canvas, aligned numbers, labelled tools | Rose tint over artwork |
| Calm | Clear next action, grouped controls, motion on request | Confetti loops and moving chrome |
| Capable | Discoverable export, undo and recovery | Hiding useful controls for minimalism |

## Art direction

The interface uses three visual distances:

1. **From the room:** recognise a creamy pink creative space, dark plum text and one raspberry
   action. The art preview is the main visual anchor.
2. **At reading distance:** understand the current task, document state and next useful action.
   Labels and boundaries remain legible in both themes.
3. **Up close:** discover pixel hearts, a chibi companion and thoughtful wording. Delight rewards
   attention without becoming a prerequisite for understanding the interface.

Use broad cream areas, light rose grouping and saturated raspberry only where it earns attention.
Mint, peach and lavender are secondary accents, not separate rainbow navigation categories.
Success, warning, danger and information have their own accessible semantic pairs.

## Identity and original assets

| Asset | Source | Role |
|---|---|---|
| Pixel heart | [`mark.svg`](../assets/mark.svg) | Reference identity and favicon |
| Pip | [`pip.svg`](../assets/pip.svg) | Original cream bunny, rosy cheeks, tiny raspberry heart |
| Sprout | [`sprout.svg`](../assets/sprout.svg) | Growth motif for documentation and first steps |

These are original vector assets authored for this proposal. They are editable SVG, contain no
scripts, remote links or embedded fonts, and use fixed pixel paths with crisp edges. Their colours
are illustration colours rather than application state. Preserve them across themes.

The heart is a proposed design mark, not a replacement of distributed app icons. Before adopting
it, check existing packaging, store assets and the maintainer's brand choice together.

### Pip's construction

- A 48 × 48 coordinate grid; scale at integer multiples when pixel exactness matters.
- Large face and upright ears, compact body and small feet. Dark plum contours and simple eyes.
- Cream fur, rose cheeks, a tiny heart: avoid glossy 3D materials and excessive shading.
- Neutral expression, small greeting and subtle celebration; never distress or blame the user.
- Clear space of at least 6 source pixels on every side; do not crop ears or feet.
- Use 48–96 CSS px beside an empty state, 192–240 px for a welcome illustration. Use a heart mark
  instead of a full mascot at tiny navigation sizes.
- In the interactive studio, the three poses illustrate frame selection with fixed translations;
  they are not three engine-generated animation frames.

Pip can welcome, guide a first creation or quietly acknowledge a completed export. Pip does not
appear inside the drawing tools, account deletion, quota errors, authentication failures or admin
incident details. When artwork itself includes Pip, it is sample content, not interface decoration.

### Marks and icons

Pair the heart with the text name for the first brand encounter. Never use the heart as an
unlabelled save control: users must not confuse saving with liking. Tool icons use a 24 px viewbox,
roughly 1.75 px strokes, rounded ends and simple consistent silhouettes. Label unfamiliar tools;
provide an accessible name for every icon-only control. Pixel artwork and UI line icons serve
different purposes and should not be mixed inside one icon.

## Typography and composition

Nunito carries the friendly voice; locally hosted variable weights avoid third-party runtime calls.
Use extra bold only for the wordmark and expressive display headlines. Product headings use the
regular heading scale, body copy stays comfortable and tabular values use the mono stack.
The reference bundles the original variable TTF with its licence; production may produce a
licensed WOFF2 subset after measuring multilingual coverage and payload cost.

The design reference uses an editorial sidebar and a large introductory specimen. This is its
documentation layout, not a proposal to give the actual editor a permanent documentation sidebar.
The application composition is task-led: shell, document bar, tools, neutral canvas, inspector and
timeline. See [journeys](journeys.md) and [components](components.md) for product layouts.

## Illustration, photography and motion

Pixel illustrations are the image language. Product screenshots show real, neutral-canvas output.
Do not introduce unrelated stock photography, gradients behind editable artwork or fake AI output.
Thumbnails eventually come from the shared renderer, with documented loading/fallback states.

Micro-interactions are fast and purposeful. A short colour transition can acknowledge hover; a
frame selection should update immediately. Nothing in the reference moves automatically. Product
animation playback starts from explicit user intent and remains stoppable. Honour OS and manual
reduced-motion settings for all decorative motion.

## Light, dark and small screens

Light is the presented art direction: warm cream and pale pink. Dark is an equally specified warm
plum environment, not an inverted screenshot. The actual app retains light/dark/system settings.
The reference starts in light to communicate the brief, with a dark toggle available.

Mobile means fewer simultaneous panels, not smaller targets. Preserve the canvas, current tool,
current colour and undo; reveal deeper settings in labelled panels. The phone reference adapts its
specimen below the canvas. Android shipping remains outside V1.

## Quality bar

The next implementation is ready when a person can start without an account, understand temporary
work, draw and animate, save or export, recover from failure and integrate a result with confidence.
The same task must remain possible in French, on a keyboard, with reduced motion and in both themes.
Visual warmth is successful when it supports those actions.

## Sources and rights

- Product scope: [`docs/product.md`](../../docs/product.md), current routes and
  [`docs/decisions.md`](../../docs/decisions.md).
- Typeface: [Nunito source](https://github.com/google/fonts/tree/main/ofl/nunito), bundled with its
  unchanged OFL licence on 2026-09-26. The font is self-hosted by the reference.
- Accessibility target: [W3C WCAG 2.2 reference](https://www.w3.org/WAI/WCAG22/quickref/).
  Measured tokens and automated reference checks do not constitute a full conformance audit.
