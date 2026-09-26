# Life Pixel

Pixel-art animations that ship as a few kilobytes of dependency-free WebAssembly.

> **Status: V1 built.** M1 to M5 of [docs/product.md](docs/product.md) are merged and verified —
> the editor, the hosted accounts and library, the admin console, the MCP server and the desktop
> app all run from this repository today. What is not yet true: the maintainer has not deployed
> the hosted service, signed the desktop installers or published `@life-pixel/player` — see
> [docs/v1/README.md](docs/v1/README.md#release) for the remaining release steps. Until then, run
> V1 yourself with the instructions below.

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

## Ways to use it

| | Runs | Your data | MCP | Price |
|---|---|---|---|---|
| Web app | hosted (not deployed yet, see below) or self-built | on the server you run | over HTTP | free; paid plans (embeds, sync, teams) are M7, not in V1 |
| Android app | on your phone | on our servers | — | M6, not in V1 |
| Desktop app | on your computer, offline | on your disk | local (stdio) | free |
| Self-hosted server | on your server | on your servers | over HTTP | free |

## Install and run V1

The maintainer has not deployed a public instance yet (see
[what only the maintainer provides](docs/v1/README.md#what-only-the-maintainer-provides)), so
every path below is self-built, from a checkout of this repository, following
[.github/CONTRIBUTING.md](.github/CONTRIBUTING.md) for the tools and versions.

- **Web (local or self-hosted)**: `cp .env.example .env && docker compose --profile app up -d --wait`
  builds and serves the app on `http://localhost:8460` and the admin console on `:8463`, backed by
  the local stack (Postgres, S3Mock, Mailpit). To run it on your own server and domain, follow
  [`docker/selfhost/README.md`](docker/selfhost/README.md), which uses your own Postgres, SMTP and
  reverse proxy instead of the local stack's stand-ins.
- **Desktop**: `cargo xtask build-desktop --debug` (or `cargo tauri dev` from `tauri/` while
  developing) bundles Windows, macOS or Linux locally; today's build is unsigned and has no
  updater, since the signing keys are the maintainer's to provide — see
  [docs/v1/desktop.md](docs/v1/desktop.md#t4--updater-and-release). Signed installers and the
  auto-updater will ship from a tagged GitHub Release once the maintainer completes it.
- **CLI**: `cargo build -p life-pixel-cli --release` produces the `life-pixel` binary (`mcp`,
  `list`, `export` on a local library); it also ships as a sidecar inside the desktop bundle.
- **MCP**: for an agent working locally, `claude mcp add life-pixel -- "<path to life-pixel>" mcp
  --library "<library path>"` (the desktop app's Settings → Agents page prints this command with
  the right paths); for the hosted server, once deployed, a personal access token from
  `settings/tokens` gives the equivalent `claude mcp add --transport http` command — see
  [docs/mcp.md](docs/mcp.md).

## Documentation

| Document | Content |
|---|---|
| [Product](docs/product.md) | vision, principles, scope, milestones |
| [Design system](design-system/README.md) | Rose Atelier: visual language, UX journeys and preview |
| [Implementation plan](docs/implementation-plan.md) | the tasks of the next milestones, and their order |
| [V1 technical design](docs/v1/README.md) | the first release: what each task builds and how it is tested, the method, the release |
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
