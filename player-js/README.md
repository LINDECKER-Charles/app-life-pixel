# @life-pixel/player

The `<life-pixel>` element: it plays the animations Life Pixel exports as `.wasm` files, on a
canvas at the animation's native size, scaled pixel-exact by CSS. No dependency, under 2 KiB
gzipped. MIT.

```html
<script type="module" src="/assets/life-pixel.js"></script>
<life-pixel src="/assets/mascot.wasm" tag="idle" alt="The mascot waving"></life-pixel>
```

With a bundler, `import '@life-pixel/player';` defines the element once; its types add
`<life-pixel>` to `HTMLElementTagNameMap`.

## The element

| Attribute | Default | Effect |
|---|---|---|
| `src` | — | the `.wasm` export; changing it loads the new one |
| `tag` | the first tag | the tag played, by name; empty or unknown: the default, with a console warning when unknown |
| `autoplay` | on | `autoplay="false"` waits for `play()` |
| `loop` | the tag's mode | `loop` or `loop="true"`: loop; `loop="false"`: once |
| `alt` | the animation's title | the accessible name; `alt=""` marks the element decorative |
| `motion` | `auto` | `always` plays even when reduced motion is preferred |

- Properties: `src` and `tag` reflect their attributes; `playing` is read-only.
- Methods: `play()`, `pause()`, `seek(frame)` — a frame of the current range, counted from its
  first frame. They act on a loaded animation: after `load`.
- Events, which do not bubble: `load` (the first frame is drawn), `error` (nothing is drawn, and
  one console warning says why — a player status is listed in the [ABI][statuses]), and `tagend`
  (the range reached its end: once for a range played once, on every loop otherwise).

```js
const mascot = document.querySelector('life-pixel');
mascot.tag = 'jump';
mascot.addEventListener('tagend', () => { mascot.tag = 'idle'; });
```

The element pauses while it is off-screen and while the tab is hidden. When the visitor prefers
reduced motion, it shows the first frame of its range and waits for `play()`, unless
`motion="always"`. It is an image (`role="img"`) named by `alt`, or by the animation's title.

The loader compiles an export once per URL, however many elements play it. It streams the
compilation when the server sends `application/wasm`, and falls back to downloading the file
otherwise. A page with a Content Security Policy needs `'wasm-unsafe-eval'` in `script-src`.

## Snippets

`snippets/` holds the integration snippets for HTML, Angular, React and Vue. Life Pixel's
compiler renders them for the export dialog and for its MCP tools, replacing each placeholder:

| Placeholder | Value | Escaping |
|---|---|---|
| `{{loader}}` | the URL of `life-pixel.js` | HTML attribute |
| `{{src}}` | the URL of the export | HTML attribute |
| `{{tagAttribute}}` | ` tag="…"`, or nothing for an animation without tags | HTML attribute, for the name |
| `{{alt}}` | the accessible name | HTML attribute |
| `{{altExpression}}` | the accessible name as a JavaScript string literal | JavaScript string |
| `{{className}}` | the file stem in PascalCase, followed by `Animation` | none: an identifier |

In `angular.ts`, the element sits in a TypeScript template literal: a renderer also escapes what
ends or interpolates it there — `` ` ``, `$`, `\` — and Angular's `{{`.

## Development

Run with the Node.js version of the repository's `.nvmrc`, in `player-js/`:

```shell
npm ci
npm run build    # type-checks, then bundles src/ into life-pixel.js, which is committed
npm test         # Playwright, on Chromium: npx playwright install chromium once
npm run size     # fails beyond 2,048 bytes gzipped
```

`life-pixel.js` is committed so that the compiler embeds it and the app ships it without running
npm: a change to `src/` commits its rebuilt `life-pixel.js`, and CI fails when they differ.
`life-pixel.d.ts` is written by hand, and follows every change of the element's interface.

The tests play `tests/fixtures/fake-player.wat`, a hand-written player implementing
[ABI v1][abi], compiled at test time with `wabt`; every file is served through `page.route`,
without a server.

[abi]: https://github.com/LINDECKER-Charles/app-life-pixel/blob/dev/crates/format/README.md#player-abi-v1
[statuses]: https://github.com/LINDECKER-Charles/app-life-pixel/blob/dev/crates/format/README.md#statuses
