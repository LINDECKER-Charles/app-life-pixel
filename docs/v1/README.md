# V1 — technical design

V1 is the first release of Life Pixel: the MVP of [product.md](../product.md), built through
milestones M1 to M5. This folder is its technical design. For every task of the
[implementation plan](../implementation-plan.md) it says what to build — crates, modules, types,
formats, routes, schemas, screens, commands — and how to test it, so that an orchestrator can split
V1 into briefs directly, and no agent has to write a design before it codes.

A task follows its section where the section is precise, and decides where it is silent; it reports
every departure with its reason. When a part of this design becomes a lasting public contract — the
export format, the player ABI, the HTTP API —, the task that builds it also writes it where it will
live: `crates/format/README.md`, [export.md](../export.md), the OpenAPI description. After V1,
those are the reference; this folder describes how V1 was built.

## Map

| File | Tasks | Covers |
|---|---|---|
| [foundations.md](foundations.md) | F1, F2, F3, H0 | CI workflows, Cargo workspace, `xtask`, front-end workspace, i18n tooling, local stack |
| [format-player.md](format-player.md) | P1, P2, L1, C1, S1, and the format types of F2 | payload v1, ABI v1, codec, player, loader and element, snippets, WASM export, size checkpoint |
| [core.md](core.md) | K1, K2, C2 | document model, limits, serialization, text grid, compositing, operations, history, import, classic exports |
| [editor.md](editor.md) | W0, U1 to U6, W1 | engine interface, worker, `editor-wasm`, shell, canvas, timeline, palette, tools, export dialog, editor journey |
| [service.md](service.md) | H1, H2, A1 | ports, errors and codes, library use cases, contract suites, local library, use cases for agents |
| [server.md](server.md) | H3, H4, H13 | configuration, middleware, problems, OpenAPI and client, database, object storage, test databases, product events |
| [accounts.md](accounts.md) | H5, H6, H7, H8, H16 | accounts, sessions, emails, library and account API, their screens, legal pages |
| [support-admin.md](support-admin.md) | H9, H10, H11, H12, H17 | support requests, internal admin API, admin server, admin console, hosted journey |
| [operations.md](operations.md) | H14, H15 | images, deployment overlay, delivery workflows, self-hosting, backups |
| [mcp-cli.md](mcp-cli.md) | A2, A3, A4, A5 | MCP tools, hosted endpoint, tokens, CLI, agent journey |
| [desktop.md](desktop.md) | T1 to T5, R1 | Tauri shell, desktop adapters, bundled CLI, updater and release, desktop journey, release readiness |

## Scope

### In V1

| Area | V1 delivers | Milestone |
|---|---|---|
| Export | the WASM bundle and its loader; GIF, APNG, sprite sheet with JSON, PNG frames; snippets for HTML, Angular, React and Vue | M1 |
| Editor | the MVP editor of product.md in the browser, usable without an account (D37) | M2 |
| Accounts and library | sign-up, email verification, sign-in, password reset; projects and animations saved, searched, duplicated, deleted; the 100 MB free quota; data export; account deletion | M3 |
| Support | requests sent from the app, handled in the admin console, answered by email and in the app | M3 |
| Admin console | health, metrics, logs and alerts of staging and production; users; support requests; audit log | M3 |
| Operations | local stack, `app` and `admin` images, build, promotion and deployment workflows, backups, self-hosting example | M3 |
| Legal pages | terms, privacy policy, legal notice | M3 |
| MCP | the tools of [mcp.md](../mcp.md), hosted with personal access tokens and local over stdio | M4 |
| Desktop | Windows, macOS and Linux on a local library, every export, the bundled CLI and its MCP server, the signed updater | M5 |
| Languages | English and French, complete | all |

### Not in V1

- **Android** (M6), and the OAuth 2.1 authorization server it shares with MCP connectors.
- **Paid plans and billing** (M7), and what they sell: live embeds, sync, version history, team
  workspaces. V1 has one plan, `free`, whose values come from configuration (D31).
- The rest of the "Next" list of product.md: social sign-in, the state machine, palette variants,
  Aseprite import, iOS, native players. Animated WebP export, and the single-file `<name>.js`
  export of export.md, come later too.
- In the admin console: plan and quota overrides, feature flags, maintenance banner, moderation,
  and the product metrics that need billing.
- Telemetry from the desktop app: V1's desktop app sends nothing at all (D24).

## Definition of done

V1 is **complete** when every task of M1 to M5 is merged into the integration branch and, there:

1. every check of AGENTS.md passes, with those each task added for its area;
2. four journeys pass end to end — **visitor** (U6): draw → animate → export → play, without an
   account; **account** (H17): sign up → verify → save → library → quota → data export → delete
   the account, and a support request answered from the admin console, on the local stack;
   **agent** (A5): create → draw → preview → tag → export → snippet over stdio and over HTTP, and
   the export plays; **desktop** (T5): the app opens its library, edits, exports, and the bundled
   CLI serves the same library;
3. the size checkpoint (S1) is recorded in [export.md](../export.md), and every budget holds.

V1 is **released** once the maintainer has done the [release](#release): production on
`lifepixel.tech`, signed installers and `@life-pixel/player` published, tag `v1.0.0` on `main`.

## What only the maintainer provides

Agents build, test and commit. They never push, and never touch an account, a secret, a DNS record
or the VPS. Nothing in the build waits for these inputs; each is needed by a release step.

| Input | Needed by | Meanwhile |
|---|---|---|
| Pushing the integration branch; opening the pull requests | CI on GitHub, staging | integration steps run the same checks locally |
| DNS records for `lifepixel.tech` (D28) | staging | the local stack |
| Scaleway: a bucket per environment in `fr-par`, production's versioned, with a 30-day expiry of old versions; the backup bucket in `nl-ams` with a six-month expiry, and credentials that can write but not delete; Transactional Email with SPF, DKIM and DMARC (D34, D35) | staging | S3Mock and Mailpit on the local stack |
| The GitHub secrets of [devops.md](../devops.md); the `staging`, `production` and `release` environments; the deploy key `PROMOTION_DEPLOY_KEY` and the `test` ruleset that lets it fast-forward | the delivery workflows | `actionlint` |
| The `infra-vps` changes of [admin-console.md](../admin-console.md): read access to VictoriaLogs, a query-only proxy to VictoriaMetrics, Life Pixel's alert rules, and the label selectors of Life Pixel's metrics and logs | the admin console on staging | fakes of those APIs in tests |
| The first admin account of each environment | the admin console | the admin server's `create-admin` command, locally |
| The legal identity (`LP_LEGAL_*`); the validation of the legal texts | production | placeholders |
| The backups' two encryption passwords (rclone `crypt`), with an offline copy | backups | throwaway passwords, locally |
| Apple Developer Program, Windows signing, the updater key and its offline copy (D32) — a month ahead | signed installers | unsigned local builds, updater off |
| The `@life-pixel` npm scope, with trusted publishing for this repository | publishing the loader | `npm pack` |

## Choices this design makes

The accepted decisions leave these open; the design settles them so that parallel tasks agree.
They are implementation choices, not entries of [decisions.md](../decisions.md): changing one
before the build starts only takes an edit here.

| Choice | Where |
|---|---|
| V1 is the MVP of product.md, through M1 to M5 | scope |
| Index 0 of every palette is transparent; layers flatten by index, topmost visible non-zero index wins; no layer opacity or blend mode | [core.md](core.md) |
| A tag loops or plays once; ping-pong would take a new format version | [core.md](core.md), [format-player.md](format-player.md) |
| The domain limits of core.md, account limits included, live in `core` and reach the interface through the engine | [core.md](core.md#limits) |
| Documents are versioned JSON; cels are run-length encoded, or text grids when a person writes them | [core.md](core.md#serialization) |
| The compiler's build script builds the player it embeds, with code shared with `cargo xtask build-player` | [format-player.md](format-player.md#c1--wasm-export) |
| The loader's build output is committed, so that Rust and Angular include it without running npm | [format-player.md](format-player.md#l1--loader-and-element) |
| Snippets are templates of `player-js`, rendered by `compiler` for the app and for MCP | [format-player.md](format-player.md#snippets) |
| Problems use `urn:life-pixel:problem:<code>`; a full quota answers `409` | [server.md](server.md#problems) |
| The typed client is generated by `openapi-typescript` and wrapped per API area in `projects/shared` | [server.md](server.md#openapi-and-the-typed-client) |
| Tests that need the local stack sit behind a `stack-tests` feature, and each gets a database with a random name | [server.md](server.md#test-databases) |
| Local object storage is Adobe S3Mock, local email Mailpit | [foundations.md](foundations.md#h0--local-stack) |
| Sign-up says when an address is taken (rate-limited); a verification link is sent; an unverified address blocks nothing | [accounts.md](accounts.md#h5--accounts-and-sessions) |
| Sessions last 30 days, sliding; CSRF tokens are derived from the session; passwords are 12 to 128 characters, hashed with Argon2id | [accounts.md](accounts.md#h5--accounts-and-sessions) |
| Visitors' preferences are not stored; a signed-in person's language is stored in the account, the desktop's in its settings file | [editor.md](editor.md#u1--application-shell) |
| Account data export is a zip laid out like a desktop library | [accounts.md](accounts.md#h6--library-and-account-api) |
| Legal texts are Markdown files per language under `i18n/legal/`, the identity comes from configuration | [accounts.md](accounts.md#h16--legal-pages) |
| Admin accounts live in a database of the admin server's own; the internal admin API listens on its own port behind a shared secret | [support-admin.md](support-admin.md) |
| Product events carry a keyed hash of the account; the app sends one event, `export_completed` | [server.md](server.md#h13--product-events) |
| Backups run in upstream images — Postgres's for the dumps, rclone's for the encrypted upload —, so D22's two images stay the only ones built | [operations.md](operations.md#h15--backups) |
| Export formats are named `wasm`, `gif`, `apng`, `sprite_sheet`, `png_frames` everywhere: Rust, TypeScript, MCP, events | [editor.md](editor.md#w0--engine-interface) |
| The timeline's preview plays a fresh export in `<life-pixel>`: the preview is the shipping player | [editor.md](editor.md#u3--timeline) |
| Versions are integers below 2^53, so that JavaScript numbers carry them exactly | [service.md](service.md#h2--local-library) |
| The default library is a `Life Pixel` folder in the user's documents; a local MCP server writes into its working directory unless told otherwise | [service.md](service.md#h2--local-library) |

## Repository layout

```text
Cargo.toml  deny.toml  .cargo/config.toml   F2
xtask/                        F2, then P2, W1, S1, T1, T3 add their commands
crates/
  format/                     F2 types, P1 codec                         MIT
  player/                     P2                                         MIT
  core/                       K1, K2
  compiler/                   C2, C1
  editor-wasm/                W1
  service/                    H1, H2, A1; H5, H9, H13, A3 add modules
  server/                     H3, H4; H5, H6, H9, H10, H13, H16, A3 add modules
  admin-server/               H11
  mcp/                        A2
  cli/                        A4
player-js/                    L1; C1 adds a test of real exports         MIT
frontend/
  projects/app/               W0, U1 to U5, W1; H7, H8, H9, H16, A3, T2, T3 add features
  projects/shared/            U1; H3 and H11 add the API clients
  projects/admin/             H12
  tools/                      F3 (i18n check), H3 (API generation), W1 (engine prebuild)
  e2e/                        U6 (editor/), H17 (hosted/)
i18n/                         F3, then every task under its prefix; legal/ by H16
samples/                      S1
scripts/deploy/               H14
scripts/backup/               H15
tauri/                        T1, T3, T4, T5
docker/                       H0 (postgres/), H14 (the rest: images, selfhost/)
compose.yaml                  H0, H14
compose.override.yaml         H0; H14 adds the application's local services
compose.deploy.yaml           H14, H15
.github/                      F1, then a job per area
```

## Conventions every task follows

- **Code**: the conventions and limits of AGENTS.md. Library crates document their public items;
  errors are typed and carry a code from the [list of codes](service.md#codes).
- **Strings**: every user-facing string is a key of `i18n/en.json` and `i18n/fr.json`, under the
  prefix of the task's feature, added in the same commit as its use; a task that creates an error
  code adds its `errors.<code>` key in both catalogues, in the same commit.
- **Tests**: shipped with the code, as its section lists them; each fails without the behaviour.
- **Checks**: a task that creates an area adds its CI job to `_verify.yml`, its ecosystem to
  Dependabot, and its commands to the checks of AGENTS.md and to CONTRIBUTING.md's development
  loop; a task that creates a top-level directory adds its line to the commit scope map.
- **Report**: the commands run and their results, the departures from this design, and anything
  left for the maintainer.

## Local environment

### Tools

| Tool | Version | For |
|---|---|---|
| Rust | `rust-toolchain.toml` | every crate, the `wasm32-unknown-unknown` target included |
| Node.js | `.nvmrc` | the front-end and `player-js` |
| Docker with Compose | recent | the local stack, the images |
| cargo-deny | latest | the dependency policy |
| cargo-fuzz, a nightly toolchain | latest | P1's fuzz target |
| wasm-bindgen-cli | the `wasm-bindgen` version of `Cargo.lock` | `cargo xtask build-editor` |
| Playwright browsers | those of the pinned Playwright | end-to-end suites |
| libwebp's `img2webp` | recent | S1's measurements only |
| actionlint | latest | workflow changes |
| Tauri's system prerequisites | Tauri 2 | M5 |

Tools are installed once. No build step downloads a binary; the player and the engine are built
from the sources of the same commit.

### Ports

Everything local binds `127.0.0.1`, on ports chosen away from common defaults.

| Service | Local | In its container |
|---|---|---|
| Postgres | 5460 | 5432 |
| S3Mock (object storage) | 5461 | 9090 |
| Mailpit SMTP / web and API | 5462 / 5463 | 1025 / 8025 |
| server: app and API / metrics / internal admin API | 8460 / 8461 / 8462 | 8080 / 9090 / 9091 |
| admin server: console and API / metrics | 8463 / 8464 | 8080 / 9090 |
| `ng serve`: app / admin console | 4260 / 4263 | — |

### Configuration

`.env.example` (H0, then every task that adds a variable) lists every variable with its local
value; `cp .env.example .env` prepares a machine. The server and the admin server read `.env` from
their working directory when it exists — in development only: containers receive their
environment from Compose. Variables of the server start with `LP_`, of the admin server with
`LPA_`; [server.md](server.md#configuration) and [support-admin.md](support-admin.md) list them.

### Local stack

`docker compose up -d --wait` starts Postgres, S3Mock and Mailpit; `docker compose down -v` resets
them. One instance serves every agent and every test run: no task starts a stack of its own.
Tests that need it are compiled only with the `stack-tests` feature, and each database test gets
its own database, with a random name ([server.md](server.md#test-databases)): parallel runs, from
any worktree, never share one.

## Building V1 in parallel

The [rules of the split](../implementation-plan.md#rules-of-the-split) apply.

### Branches

- V1 is built on an **integration branch** cut from `dev`, never on `dev` itself. The maintainer
  brings it into `dev` through pull requests that stop at the milestone checkpoints below, so that
  each review ends on a verified commit.
- Each task works on a branch of its own, named after it (`feat/format-codec`), cut from the
  integration branch once everything it waits for is there, and kept after its merge.
- Only integration steps merge into the integration branch; no task rewrites another's commits.
- Commits follow AGENTS.md: Conventional Commits in English, with the repository's scopes.
- Nobody but the maintainer pushes.

### Waves and integration steps

The waves of the [order](../implementation-plan.md#order) run one after the other. After each:

1. an **integration step** merges the wave's branches — conflicts on shared files are additions on
   both sides, and both stay —, runs every check of AGENTS.md and every end-to-end suite that
   exists against the local stack, and writes the failures down;
2. **fixes** follow that report, each on a branch of its own, on disjoint files;
3. a **verification step** merges them and runs the checks again; the next wave starts from the
   commit it verified.

The milestone checkpoints — M1 once S1 has measured, M2 with U6, M3 with H17, M4 with A5, M5 with
T5 — are verification steps that also run their milestone's journey.

### Who runs what

| Who | Runs |
|---|---|
| A task | formatting, lints and tests of the crates and projects it touched — `cargo clippy -p …`, `cargo test -p …`, with `--features stack-tests` when it has database tests —, and the unit tests of its front-end project |
| An integration or verification step | everything AGENTS.md lists, `stack-tests` included, the size budgets, every end-to-end suite that exists |

End-to-end suites and full builds are heavy: steps run them, one at a time on a machine, and a task
only runs the suite it owns.

### Shared files

Tasks running at the same time own disjoint paths. These files are shared: they only receive
additions, placed so that parallel additions merge.

| File | Rule |
|---|---|
| root `Cargo.toml` | members globbed; `[workspace.dependencies]` sorted, one per line |
| `.github/workflows/_verify.yml`, `.github/dependabot.yml` | a job per area; an entry per ecosystem and directory |
| `i18n/*.json` | flat keys under the task's prefix, sorted |
| `frontend/package.json` scripts, `frontend/projects/app/src/app/app.routes.ts` | a line per script; a lazy route per feature |
| `.env.example`, `compose.yaml` | a block per area; a block per service |
| module lists of `crates/service`, `crates/server`, the server's router | a line per module |
| AGENTS.md checks with CLAUDE.md, the documentation maps, CONTRIBUTING.md's development loop | a line or a row per addition |
| `Cargo.lock`, `frontend/package-lock.json`, `player-js/package-lock.json` | never merged by hand: the integration step regenerates them after the merge (`cargo metadata`, `npm install --package-lock-only`) |
| the OpenAPI descriptions and the clients generated from them | the same: the integration step reruns `npm run api:generate` after the merge |

### When a contract must change

Its owner changes it, and the orchestrator warns every task that relies on it. A task that needs a
change it does not own stops and reports it, rather than working around the contract.

## Release

The maintainer's steps, in order:

1. Provide the [inputs](#what-only-the-maintainer-provides) that staging needs.
2. Push the integration branch, and open the pull requests into `dev`, checkpoint by checkpoint. A
   green CI promotes `dev` to `test` and deploys staging.
3. On staging: `/healthz` answers over TLS, the first admin account exists, the account journey
   passes by hand, the admin console shows staging's health, metrics and logs.
4. Rehearse a restore from the backup bucket ([operations.md](operations.md#restore)).
5. Configure the legal identity, and validate the legal texts.
6. Fast-forward `main` to the validated `test` commit: production deploys.
7. With the signing material in the `release` environment, tag `v1.0.0` on `main`: `release.yml`
   publishes the installers, `@life-pixel/player`, the Docker version tags and the GitHub Release.
