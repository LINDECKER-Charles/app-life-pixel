# Samples

Four animations drawn for this project, used by `cargo xtask measure-sizes` (S1's size checkpoint,
[docs/v1/format-player.md](../docs/v1/format-player.md#s1--size-checkpoint)) and by
[`demo.html`](demo.html). Each is a `life-pixel/animation` document whose cels are text grids
([docs/v1/core.md](../docs/v1/core.md#text-grid)) — readable and diffable as plain JSON.

| Sample | Size | Frames | Tags | Colours |
|---|---|---|---|---|
| `mascot-wave` | 32 × 32 | 8 | `idle`, `wave` | 12 |
| `loader-dots` | 16 × 16 | 6 | — | 4 |
| `hero-run` | 48 × 48 | 16 | `idle`, `run`, `jump` (once) | 24 |
| `empty-state` | 128 × 96 | 12 | — | 16 |

## Licence

To the extent possible under law, the authors have dedicated all copyright and related and
neighbouring rights to these four animations (`mascot-wave.json`, `loader-dots.json`,
`hero-run.json`, `empty-state.json`) to the public domain worldwide, under the
[CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) dedication. This work is
published from the United States. You can copy, modify, distribute and use them, even
commercially, without asking permission — no attribution required.

`demo.html` (the page that plays them) follows the repository's own licence, not CC0.
