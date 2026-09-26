# Foundations: Rose Atelier

Status: a complete design proposal for Life Pixel, ready for incremental implementation. The
reference tokens live in [tokens.css](../tokens/tokens.css); they do not replace production styles.
Product behaviour continues to follow [the product specification](../../docs/product.md).

## Character and purpose

Life Pixel feels like a small, welcoming animation atelier: warm paper, rose stationery, rounded
tools and a chibi companion. The artwork remains the centre of attention. A beginner should feel
invited to make a first sprite; an experienced artist should find a calm, precise workspace.

The signature is a blush frame around a neutral working canvas, with a small pixel companion at
meaningful moments. Rounded controls echo the companion's silhouette. Straight timeline tracks
and disciplined alignment make editing feel reliable. Softness belongs to the application shell;
pixel edges remain exact inside the artwork.

| Principle | Design consequence |
| --- | --- |
| Warm welcome | Cream background, rose accents, clear verbs, encouraging empty states |
| Confident making | Predictable tools, visible save state, one prominent next action |
| Small delights | A companion on the welcome screen or completion state; quiet during editing |
| Honest outcomes | Guest work, local saving, cloud saving and export have distinct labels |
| Art comes first | Neutral canvas surround; no tinted filter, blur or shadow on document pixels |
| Useful everywhere | Consistent patterns on web, desktop and Android; reduced admin decoration |

## Colour roles

Use semantic tokens in components, never a colour name to imply behaviour. Values in this table
are approved pairings within this proposal. They are not a claim that the current application
already uses or passes them. Light mode is the primary visual direction; dark mode is warm plum.

| Token suffix (`--lp-`) | Light | Dark | Role |
| --- | --- | --- | --- |
| `bg` | `#fff8f5` | `#241b23` | Page and application shell |
| `surface` | `#ffffff` | `#30242e` | Fields, cards and primary panels |
| `surface-soft` | `#ffedf1` | `#3c2a38` | Welcome inset, quiet section or hover context |
| `surface-raised` | `#ffffff` | `#392b36` | Floating menus and dialogs |
| `text` | `#382631` | `#fff4f7` | Headings, body, labels |
| `text-muted` | `#735b66` | `#d3b8c7` | Supporting information; still readable |
| `accent` | `#a93663` | `#f3a6c5` | Primary action, selected marker, underlined links |
| `accent-hover` | `#8b294f` | `#ffc4da` | Primary action hover and press |
| `on-accent` | `#ffffff` | `#35212c` | Text and icons inside primary actions |
| `accent-soft` | `#ffe0ec` | `#4f2c40` | Selected row or soft action background |
| `accent-text` | `#8b294f` | `#ffbed8` | Text on soft accent backgrounds |
| `border` | `#ead5dd` | `#624758` | Decorative divisions only |
| `control-border` | `#a17c8c` | `#a8879a` | Necessary field or control boundary |
| `focus` | `#7950b1` | `#c8adff` | Keyboard focus outline |

Rose, peach, mint and lavender tokens provide decorative swatches and illustration backgrounds.
They never replace semantic feedback: a mint illustration is not a success message. Use `text`
for labels on those fills. Do not apply container opacity to text, icons or interactive controls.

| Feedback | Foreground / light background | Foreground / dark background |
| --- | --- | --- |
| Success | `#286447` / `#e8f5ed` | `#abddc7` / `#23463b` |
| Warning | `#775113` / `#fff2d8` | `#f2d59b` / `#4c3b20` |
| Danger | `#a12f46` / `#ffedf0` | `#ffb9bb` / `#512d37` |
| Information | `#40548e` / `#edf1ff` | `#bfcbff` / `#303c5c` |

Every feedback style includes an icon and explicit text. Destructive actions use the danger
foreground on the danger background or an outlined surface button. A solid danger button needs
its own verified foreground pair; `on-accent` must not be reused blindly in dark mode.

### Verified contrast pairs

Ratios below use relative sRGB luminance for the opaque token values, rounded to two decimals.
Normal text requires 4.5:1; essential control boundaries require 3:1 against their adjacent fill.
The [accessibility specification](accessibility.md) records the wider acceptance criteria.

| Foreground / background | Light | Dark | Intended use |
| --- | --- | --- | --- |
| `text` / `bg` | 13.42:1 | 15.55:1 | Body copy |
| `text` / `surface` | 14.10:1 | 13.78:1 | Fields, labels, headings |
| `text` / `surface-soft` | 12.51:1 | 12.36:1 | Soft panels |
| `text-muted` / `surface` | 6.15:1 | 8.08:1 | Help and metadata |
| `text-muted` / `surface-soft` | 5.45:1 | 7.25:1 | Soft-panel metadata |
| `on-accent` / `accent` | 6.19:1 | 7.88:1 | Primary action |
| `on-accent` / `accent-hover` | 8.31:1 | 10.08:1 | Primary action hover |
| `accent` / `surface` | 6.19:1 | 7.81:1 | Link or selected icon |
| `accent` / `surface-soft` | 5.49:1 | 7.01:1 | Link on soft panel |
| `control-border` / `surface` | 3.64:1 | 4.65:1 | Input edge |
| `control-border` / `surface-soft` | 3.23:1 | 4.17:1 | Input edge on soft panel |
| `focus` / `surface` | 5.84:1 | 7.67:1 | Focus with offset |
| `focus` / `surface-soft` | 5.18:1 | 6.88:1 | Focus with offset |
| `success` / `success-bg` | 6.23:1 | 6.91:1 | Success message |
| `warning` / `warning-bg` | 6.38:1 | 7.56:1 | Warning message |
| `danger` / `danger-bg` | 6.19:1 | 7.25:1 | Error message |
| `info` / `info-bg` | 6.48:1 | 6.85:1 | Informational message |

Recalculate whenever either value changes. These ratios do not validate text on arbitrary artwork,
transparent overlays, disabled controls or every possible token combination. `border` is purposely
subtle and must never be the only indicator of an input, selection or keyboard focus.

## Typography

The rounded UI face is Nunito. The preview uses the bundled variable font in `assets/fonts/`,
with its OFL licence and no external runtime request. The token stack falls back to Trebuchet MS,
Segoe UI and the platform sans serif. Production can adopt the same local files with
`font-display: swap`; validate fallbacks. Use the system monospace stack for dimensions, frame
timing, code and hexadecimal colour values. The display weight of 800 is reserved for the brand
and welcome headline; standard headings use 700.

| Role | Token | Default size | Weight / line height |
| --- | --- | --- | --- |
| Welcome display | `text-display` | 36-64 px, fluid | 800 / 1.15 |
| Page heading | `text-3xl` | 40 px | 700 / 1.25 |
| Section heading | `text-2xl` | 32 px | 700 / 1.25 |
| Panel heading | `text-xl` | 24 px | 700 / 1.25 |
| Introductory copy | `text-lg` | 18 px | 400 / 1.6 |
| Body and input | `text-md` | 16 px | 400 / 1.6 |
| Control and metadata | `text-sm` | 14 px | 600 / 1.35 |
| Tertiary technical annotation | `text-xs` | 12 px | 400 / 1.6 |

Sizes assume a 16 px browser default and use rem units. Never set the root font size to a fixed
pixel value. Use 16 px for text inputs on touch screens. Do not use 12 px for instructions,
errors or navigation. Keep copy near 65 characters per line, sentence case, left aligned, with
ordinary heading hierarchy. Pixel lettering is reserved for optional decorative artwork.

## Space, geometry and density

The base rhythm is 4 px at the default root size. Token steps are 0, 4, 8, 12, 16, 20, 24, 32,
40, 48, 64, 80 and 96 px. Use 8 px within controls, 12 px between related controls, 16 px between
fields, 24 px inside panels and 32-48 px between page sections. Editing panels can use 16 px
padding. Content groups are separated by space before another enclosing card is introduced.

Radii are 4 px for swatches and keycaps, 8 px for compact tools, 14 px for fields and buttons,
20 px for panels and 32 px for a welcome illustration frame. Pill geometry belongs to badges
and segmented selectors. The canvas and pixel thumbnails preserve square artwork edges.

Default controls have a minimum height of 44 px. Desktop compact controls may be 36 px with a
fine pointer; coarse-pointer controls retain 44 px targets. Height is a minimum, so text zoom
and French labels can expand it. The editor is dense through alignment and spacing, not tiny text.
Reserve space for focus outlines; clipping overflow must not crop them.

Use the small shadow for floating toolbar groups, medium for popovers and large for dialogs.
Ordinary panels rely on a fill or divider. Avoid stacking shadows at every nesting level.
Layer order: base 0, sticky 10, dropdown 20, scrim 30, dialog 40, popover 50, toast 60, tooltip 70.
Native dialog top-layer behaviour takes precedence; popovers must stay in their owning dialog.

## Responsive composition

| Available width | Library and settings | Editor |
| --- | --- | --- |
| Below 48 rem | One column, compact header | Canvas first; tool tabs and sheets |
| 48-74.99 rem | Collapsible navigation, flexible columns | One inspector open at a time |
| From 75 rem | 15 rem sidebar, content up to 80 rem | Flexible canvas; 17 rem inspector |

The timeline defaults to 11 rem high and can grow when more tracks are visible. On small screens
it becomes an explicit frame strip below the canvas; frame details open in a sheet. Preserve the
selected frame, layer, zoom and undo state when layouts change. Do not emulate mobile by shrinking
the whole desktop editor. Use device safe-area insets and dynamic viewport units for sheets.

Breakpoints in tokens document the contract; CSS media queries must use literal rem values.
At 320 CSS px, all surrounding controls reflow without page-wide horizontal scrolling. The spatial
canvas and timeline may pan within clearly bounded regions. Dialogs scroll internally when needed,
with a visible close action and the focused field unobscured by the software keyboard.

## Icons, imagery and the chibi companion

Use a coherent 24-unit outline icon family with round caps and a 1.75-unit stroke. Normal icons
render at 20 px; small metadata at 16 px, prominent actions at 24 px. Optical corrections are
allowed for pixel tools. Filled shapes identify selected states alongside a label or marker.
Emoji are not functional icons: their platform-dependent drawing changes size, meaning and style.

The companion is an original rounded pixel creature: a large head occupying about two thirds of
its height, a tiny body, short limbs, two simple eyes and restrained blush. Rose and cream lead;
mint or peach can support props. Keep the silhouette readable at 32 px and design on a consistent
pixel grid. Pip uses a 48 px source, displayed at integer multiples with pixelated rendering.
Do not derive a mascot from an existing commercial character.

| Context | Expression and placement |
| --- | --- |
| First welcome | Curious pose beside the invitation to create |
| Empty library | Small encouraging pose above one clear next action |
| Export complete | Brief pleased pose next to the available file |
| Loading or saving | Quiet indicator; no continuous bouncing companion |
| Recoverable failure | Omit the companion; practical recovery text comes first |
| Destructive confirmation, billing, admin incident | Omit the companion |

Use one companion per view, outside the canvas, at 48-96 px in normal empty states. Larger welcome
art may use 160-240 px. Illustration never carries an instruction or status by itself. Decorative
images have empty alt text; meaningful sample thumbnails have descriptive labels. No stickers
cover controls, no speech bubbles replace field help, and no celebratory motion interrupts drawing.

## Motion

Motion communicates cause and effect. Use 120 ms for hover and press, 180 ms for disclosure and
240 ms for a dialog or sheet. Prefer opacity and a short 4-8 px translation. Use standard easing
for state changes, enter easing for appearance and exit easing for departure. Avoid springy
overshoot, perpetual bobbing, parallax and animated backgrounds around the editor.

`prefers-reduced-motion` reduces the token durations to zero. Components must also disable any
independent keyframe decoration and smooth scrolling. User-requested animation playback remains
under explicit play/pause control; this preference does not change document timing or exports.

## Appearance preference

Explicit `data-theme="light"` or `data-theme="dark"` on the root overrides the operating system.
An omitted attribute or `data-theme="system"` follows `prefers-color-scheme`. The production
preference UI should offer all three choices and persist through the existing appearance service.
Do not infer a person's identity, age or gender from the selected theme.

Transparency checker colours remain neutral `#ffffff` and `#ececee` in both themes. The darker
surround is UI, never part of the exported image. Selected pixels, onion skin and palette data
continue to come from the engine; this design system defines only the surrounding controls.
