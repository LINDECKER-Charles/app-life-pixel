# Life Pixel

Pixel-art animations that ship as a few kilobytes of dependency-free WebAssembly.

> **Status: design phase.** There is nothing to install yet. The specification lives in
> [`docs/`](docs/), the roadmap in [docs/product.md](docs/product.md). Feedback on the design is
> welcome in the issues.

## What it is

- **An editor made for pixel art** — palette, layers, frames, onion skin — in the browser, on the
  desktop and on Android.
- **Exports that run wherever a browser engine runs**: one `.wasm` file — a tiny player plus your
  animation — and a loader budgeted at 2 KB. No framework, no build step, no dependency. The
  classic formats (GIF, APNG, sprite sheets) are there too.
- **Built for AI agents as much as for people**: an MCP server lets Claude, or any MCP client,
  draw, preview, export and integrate an animation from your editor or your terminal.

## In your app

```html
<script type="module" src="/assets/life-pixel.js"></script>
<life-pixel src="/assets/mascot.wasm" tag="idle" alt="The mascot waving"></life-pixel>
```

```js
document.querySelector('life-pixel').tag = 'jump';
```

## Ways to use it (planned)

| | Runs | Your data | MCP | Price |
|---|---|---|---|---|
| Web app | hosted by us | on our servers | over HTTP | free; paid plans add hosted embeds, sync and teams |
| Android app | on your phone | on our servers | — | your web account |
| Desktop app | on your computer, offline | on your disk | local (stdio) | free |
| Self-hosted server | on your server | on your servers | over HTTP | free |

## Documentation

| Document | Content |
|---|---|
| [Product](docs/product.md) | vision, principles, scope, milestones |
| [Architecture](docs/architecture.md) | components, crates, distributions |
| [Export and integration](docs/export.md) | the WASM bundle, the player, the `<life-pixel>` element |
| [MCP server](docs/mcp.md) | tools, transports, authentication |
| [Admin console](docs/admin-console.md) | metrics, logs, support, administration |
| [Pricing](docs/pricing.md) | plans and storage quota |
| [DevOps](docs/devops.md) | environments, branches, CI/CD, deployment |
| [Security model](docs/security-model.md) | threat model, supply chain, privacy |
| [Internationalisation](docs/i18n.md) | languages and catalogues |
| [Decisions](docs/decisions.md) | what is settled, proposed, open |

## Contributing

Read the [contributing guide](.github/CONTRIBUTING.md) and the
[code of conduct](.github/CODE_OF_CONDUCT.md). Questions: [support](.github/SUPPORT.md).
Vulnerabilities are reported privately: [security policy](.github/SECURITY.md).

## Licence

Life Pixel uses two licences, one per kind of component:

| Component | Licence |
|---|---|
| What ships inside your app — the export format (`crates/format`), the player (`crates/player`), the loader and the integration snippets (`player-js`) | [MIT](LICENSE-MIT) |
| Everything else — editor, server, MCP server, CLI, desktop and mobile apps, admin console | [AGPL-3.0-only](LICENSE) |

Embedding an export in your app, open source or not, only involves the MIT licence. The AGPL
applies if you modify Life Pixel itself and offer it to others, including over a network.
