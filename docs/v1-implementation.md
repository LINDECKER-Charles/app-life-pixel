# V1 implementation

V1 is the first release of Life Pixel: the MVP of [product.md](product.md), built through
milestones M1 to M5. This document is what an orchestrator reads before it turns V1 into agents'
briefs, and what those briefs point back to: what V1 contains and leaves out, when it is done,
the contracts its tasks meet on, the environment they run in, how their work comes together, and
what only the maintainer can do. The tasks themselves — what each delivers, owns and waits for,
and in which order — are in the [implementation plan](implementation-plan.md).

## Scope

### In V1

| Area | V1 delivers | Milestone | Tasks |
|---|---|---|---|
| Foundations | continuous integration, the Cargo workspace and the export contract, the front-end workspace | M1, M2 | F1, F2, F3 |
| Export | the WASM bundle and its loader; GIF, APNG, sprite sheet with JSON, PNG frames; the snippets for HTML, Angular, React and Vue | M1 | P1, P2, L1, C1, C2, S1 |
| Editor | the MVP editor of product.md, in the browser, usable without an account (D37) | M2 | K1, K2, W0, W1, U1 to U6 |
| Accounts and library | sign-up, email verification, sign-in, password reset; projects and animations saved, searched, duplicated and deleted; the 100 MB free quota; data export and account deletion | M3 | H1, H3 to H8, H17 |
| Support | requests sent from the app, handled in the admin console, answered by email and in the app | M3 | H9, H10, H12 |
| Admin console | health, metrics, logs and alerts of staging and production; users; support requests; the audit log | M3 | H10 to H13 |
| Operations | the local stack; the `app` and `admin` images; the build, promotion and deployment workflows; backups; a self-hosting example | M3 | H0, H14, H15 |
| Legal pages | terms, privacy policy, legal notice | M3 | H16 |
| MCP | the tools of [mcp.md](mcp.md), on the hosted endpoint with personal access tokens and over stdio through the CLI | M4 | A1 to A5 |
| Desktop | Windows, macOS and Linux, on a local library: every export, the bundled CLI and its MCP server, the signed updater | M5 | H2, T1 to T5 |
| Languages | English and French, complete | all | every task |

### Not in V1

- **Android** (M6), and the OAuth 2.1 authorization server it shares with MCP connectors.
- **Paid plans and billing** (M7), and what they sell: live embeds, sync, version history, team
  workspaces. V1 has a single plan, free, whose values — the 100 MB quota, the fair-use
  ceilings — come from configuration (D31).
- The rest of the "Next" list of product.md: social sign-in, the state machine, palette variants,
  Aseprite import, animated WebP, iOS, native players.
- In the admin console: plan and quota overrides, feature flags, the maintenance banner,
  moderation, and the product metrics that need billing — conversion, churn, revenue.
- Telemetry from the desktop app: V1's desktop app sends nothing at all. The opt-in of D24 comes
  later.

## Definition of done

V1 is **complete** when every task of M1 to M5 is merged into the integration branch (see
[Building V1 in parallel](#building-v1-in-parallel)) and, on that branch:

1. every check of AGENTS.md passes, those each task added for its area included;
2. four journeys pass end to end:
   - **visitor** (U6): draw → animate → export → play in a page, without an account;
   - **account** (H17): sign up → verify the address → save → find the animation in the
     library → reach the quota → export the data → delete the account, on the local stack; and
     a support request answered from the admin console;
   - **agent** (A5): an MCP client creates, draws, previews, tags and exports an animation and
     gets its snippet, over stdio and over HTTP, and the export plays;
   - **desktop** (T5): the app built for the host system opens its library, edits and exports,
     and the bundled CLI serves the same library over stdio;
3. the size checkpoint (S1) is recorded in [export.md](export.md), and every budget holds.

V1 is **released** once the maintainer has completed the [release](#release): the hosted service
in production on `lifepixel.tech`, the signed installers and `@life-pixel/player` published, the
tag `v1.0.0` on `main`.

## What only the maintainer provides

Agents build, test and commit. They never push, and never touch an account, a secret, a DNS
record or the VPS. Nothing in the build waits for the inputs below — each is needed by a step of
the release —, but the slow ones should start early.

| Input | Needed by | Meanwhile |
|---|---|---|
| Pushing the integration branch; opening the pull requests | CI on GitHub, staging | integration steps run the same checks locally |
| DNS records for `lifepixel.tech` (D28) | staging | the local stack |
| Scaleway: a bucket per environment in `fr-par`, production's versioned; the backup bucket in `nl-ams`, with credentials that cannot delete; Transactional Email with SPF, DKIM and DMARC (D34, D35) | staging | the object storage and mail catcher of the local stack |
| The GitHub secrets of [devops.md](devops.md) | the deployment workflows | the workflows checked with `actionlint` |
| The `infra-vps` changes of [admin-console.md](admin-console.md): read access to VictoriaLogs, a query-only proxy to VictoriaMetrics, Life Pixel's alert rules | the admin console on staging | fakes of those APIs in tests |
| The first admin account of each environment | the admin console | created locally by the admin server's command |
| The identity printed in the legal notice; the validation of the legal texts | production | placeholders from configuration |
| The images made public on GHCR, once first pushed | self-hosters | — |
| The Apple Developer Program, a Windows signing service, the updater key and its offline copy (D32) — about a month ahead | signed installers | unsigned local builds, updater off |
| The `@life-pixel` npm scope, with trusted publishing for this repository | publishing the loader | `npm pack` |

## Choices this plan makes

The accepted decisions leave the points below open, and parallel tasks must not settle them
differently. They are implementation choices, not entries of [decisions.md](decisions.md):
changing one before the build starts only takes an edit of this document.

| Choice | Concerns |
|---|---|
| V1 is the MVP of product.md, through M1 to M5; Android comes after it | the scope |
| Index 0 of every palette is transparent; a palette holds up to 256 entries, index 0 included | K1, P1, C1, C2, U4 |
| Layers flatten by palette index: the topmost visible non-transparent index wins. V1 layers have neither opacity nor blend mode | K1, C1, C2, W1 |
| A tag loops or plays once; ping-pong comes later, with a new format version | K1, F2, P2, L1 |
| The initial domain limits of the [document model](#document-model) | K1, and everything that validates |
| Documents are versioned JSON, with run-length encoded cels | K1, H2, H4 |
| The compiler's build script builds the player it embeds, in a target directory of its own, with the settings of the `player` profile that `cargo xtask build-player` uses too | P2, C1 |
| The OpenAPI descriptions and the clients generated from them are committed, and CI checks that they are current | H3, H11, every API consumer |
| Sign-up sends a verification link; an unverified address blocks nothing in V1 | H5 |
| Legal texts are Markdown files per language under `i18n/legal/`, served by the i18n endpoint; the identity comes from configuration, so that a self-hosted server shows its own | H16 |
| Database tests each get a database of their own, with a random name, on the shared Postgres | H4, and every task with database tests |
| Admin accounts live in a database of the admin server's own | H11 |
| Product events carry a keyed hash of the account, never its id; the app reports one event, `export_completed` | H13 |
| The default library is a `Life Pixel` folder in the user's documents, shared by the desktop app and the CLI; a local MCP server writes into its working directory unless told otherwise | H2, A4, T1 |

## Contracts

Where parallel tasks meet. Each contract has one owner; the others rely on it, and ask the
owner, through the orchestrator, for any change.

| Contract | Owner | Relied on by |
|---|---|---|
| [Workspace](#workspace) | F2, F3 | every task |
| [Export format and player ABI](#export-format-and-player-abi) | F2 | P1, P2, L1, C1 |
| [Document model](#document-model) | K1 | every Rust task |
| [Editing operations](#editing-operations) | this document | W0, K2, W1, A1 |
| [Editor engine](#editor-engine) | W0 | U2 to U5, W1, H8 |
| [Snippets](#snippets) | L1 | U5, A1 |
| [Service](#service) | H1 | H2, H4, every use case |
| [HTTP API](#http-api) | H3 | every route, every API consumer |
| [Local library](#local-library) | H2 | A4, T1 |
| [Product events](#product-events) | H13 | the features, through H1's port |
| [Admin](#admin) | H10, H11 | H12 |

### Workspace

- The root `Cargo.toml` globs its members — `crates/*` and `xtask`; T1 adds `tauri` —, so that a
  new crate needs no edit there. It sets `resolver = "3"`, `[workspace.package]` with
  `edition = "2024"` and `rust-version = "1.98"`, the `[workspace.lints]` of AGENTS.md, and a
  sorted `[workspace.dependencies]`, one dependency per line. Each crate declares its `license`.
- A `player` profile builds the WebAssembly player: it inherits `release` and sets
  `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"` and `strip = true`.
  Every other build keeps the standard profiles.
- `xtask/` holds the build commands that Cargo alone cannot express — `cargo xtask build-player`
  (P2), `cargo xtask build-editor` (W1) —, through an alias in `.cargo/config.toml`. It is build
  tooling: its commits take the `build` type, without a scope.
- `frontend/` is one Angular workspace with `projects/app` and `projects/shared` — H12 adds
  `projects/admin` —, whose `package.json` provides the `lint`, `test:ci`, `build`, `e2e` and
  `i18n:check` scripts.
- The catalogues hold flat keys, one per line, sorted. The catalogue check enforces the order,
  so that additions under different prefixes merge without conflict.

### Export format and player ABI

F2 specifies payload v1 and ABI v1 byte by byte in `crates/format/README.md`, from the draft of
[export.md](export.md) and this outline:

- Little-endian, every length before its content, no padding. The decoder checks the magic bytes
  and both versions first, then every length and index against the bytes it holds and the bounds
  of the format; it refuses trailing bytes, and answers anything wrong with an error status —
  never a panic.
- After the header — magic bytes, format version, ABI version, width, height — come the palette
  (up to 256 RGBA entries, entry 0 transparent), the title, the tags (name, first and last frame,
  loop mode), and the frames (duration in milliseconds, pixels).
- A frame is either a **key frame** — runs and literals of palette indices over the whole image —
  or a **delta frame** — skips, runs and literals against the previous frame. The encoder writes a
  key frame at frame 0, at the first frame of every tag, and wherever a delta would not be
  smaller; seeking decodes from the nearest key frame before the target.
- The format's bounds — what the decoder accepts — cover at least the domain limits of `core`. A
  test of `compiler`, the first crate to see both, checks it.

The draft ABI lacks what the `<life-pixel>` element of export.md needs. F2 adds the following, or
an equivalent, and updates export.md:

| The element needs | ABI addition |
|---|---|
| to check the ABI before calling anything else | `abi_version() -> u32` |
| `tag="idle"`: a tag found by its name | `tag_count()`, `tag_name_ptr(index)`, `tag_name_len(index)` |
| the `tagend` event | `tick` returns flags: a new image is ready; the tag reached its end |
| the `loop` attribute | `set_loop(mode)`: the tag's own mode, loop, or once |
| an accessible name from the title | `title_ptr()`, `title_len()` |

The player exports its memory and imports nothing. `load` answers with a status whose values F2
lists: bad magic, unknown format version, unknown ABI, malformed payload, limit exceeded.

### Document model

- A **project** has a name and groups **animations**. An animation has a title, a canvas size, a
  palette, ordered layers — a name, shown or hidden —, ordered frames — a duration —, cels — the
  pixels of one layer on one frame, as palette indices, possibly empty —, and tags — a name, a
  first and a last frame, `loop` or `once`.
- Compositing follows the [choices above](#choices-this-plan-makes). `core` is the only code that
  composites: the editor, the compiler and the previews call it.
- Project and animation ids come from `service`, since `core` has no randomness.
- The document serializes to JSON with a `version` field, and is migrated on read. K1 documents
  the model, the serialization and the text grid of `write_frame` in `crates/core/README.md`.

The initial domain limits, constants of `core`. K1 may revise one, giving its reason in its
report; S1 checks them against the sample animations.

| Limit | Initial value |
|---|---|
| Canvas | 1 to 512 pixels a side |
| Frames | 1 to 1,024 |
| Layers | 1 to 64 |
| Pixels of the non-empty cels, all together | 16,777,216 |
| Palette | up to 256 entries, entry 0 transparent |
| Frame duration | 10 to 65,535 ms, 100 ms by default |
| Tags | up to 64; a name of 1 to 32 characters among `a`–`z`, `0`–`9`, `-` and `_`, starting with a letter, unique in its animation |
| Names and titles | 1 to 100 characters, no control character |
| Imported image | 4,096 pixels a side and 16 MiB, checked before decoding |

### Editing operations

W0 types them in TypeScript before K2 writes them in Rust: this list is their common source, and
W1 maps one onto the other.

| Group | Operations |
|---|---|
| Pixels | paint a stroke — the pencil with an index, the eraser with index 0 —; fill a contiguous area, 4-connected; draw a line; draw a rectangle, outlined or filled; move the pixels of a rectangular selection |
| Palette | add, edit, remove and reorder entries |
| Layers | add, delete, reorder, rename, show or hide |
| Frames | add, duplicate, delete, reorder, set the duration |
| Tags | add, edit, delete |
| Import | a PNG image into a cel; a sprite sheet into frames, cut along a grid |

Every operation has its inverse, for undo and redo. Import maps each pixel to the nearest palette
entry, and a pixel more than half transparent to index 0; K2 documents the distance it uses.

### Editor engine

W0 writes the interface in TypeScript, with an in-memory mock; W1 implements it over
`editor-wasm`.

- **Asynchronous.** The engine runs in a Web Worker; pixel buffers cross as transferable objects.
- **Commands apart from queries.** Commands — create, open, apply an operation, undo, redo —
  resolve without a value. The state — a summary of the document, the selection, whether undo and
  redo are available, whether unsaved work exists — is published to subscribers, which the app
  turns into signals.
- **Queries** render a frame with its visible layers and the onion-skin settings, serialize the
  document, and export it in a format with the size of the result, so that the export dialog
  shows the sizes side by side.
- **One contract suite**, written by W0, runs against the mock and, from W1 on, against the real
  engine in a browser.
- From W1 on, `npm run build --prefix frontend` runs `cargo xtask build-editor` first: the checks
  of AGENTS.md keep working as they are.

### Snippets

The integration snippets are template files with placeholders, kept in `player-js` under the MIT
licence like the loader. The export dialog and the MCP tool `get_embed_snippet` both read them.

### Service

- `service` defines its ports as focused traits — library index, document bodies, clock, ids,
  product events; each feature adds its own, such as accounts or support —, with an in-memory
  adapter for each, used by its tests.
- H1 writes a **contract suite** per storage port. Every adapter — in memory, local files (H2),
  Postgres and object storage (H4) — passes the same suite.
- Use cases take the caller's identity and fail with typed errors carrying a stable code and its
  parameters: `quota.storage_exceeded` with `used`, `limit` and `requested`. One list gathers
  every code, and a test checks that each has its `errors.<code>` key in every catalogue.
- Compiling and rendering are synchronous; async callers run them on `spawn_blocking`.

### HTTP API

On top of the rules of AGENTS.md:

- **Problems**: `application/problem+json` with `type` (`urn:life-pixel:problem:<code>`),
  `status`, `code` and `params`, and no sentence.
- **Pagination**: `?cursor=…&limit=…`, answered with `items` and `nextCursor`; 50 items by
  default, 100 at most.
- **Client version**: every client sends its platform and version in a header. Below the minimum
  configured for its platform, the server answers `426` with `client.update_required`.
- **Sessions**: the `__Host-` session cookie of [security-model.md](security-model.md). A
  state-changing request carries its CSRF token in `X-CSRF-Token` and passes the origin check;
  bearer tokens carry no cookie and skip it.
- **Concurrency**: a stored document has a version. A save sends it in `If-Match`, and gets `412`
  with `document.version_conflict` when the document has moved on.
- **OpenAPI**: the server binary writes its description (`life-pixel-server openapi`); a script
  of `frontend/package.json` generates the client into `projects/shared`. Both are committed, and
  CI regenerates them to prove they are current. H11 does the same for the admin server.

| Routes | Added by |
|---|---|
| `GET /healthz`; `/metrics` on the internal port; `GET /i18n/languages.json`, `GET /i18n/{language}.json` | H3 |
| `GET /i18n/legal/{language}/{page}.md` | H16 |
| under `/api/v1/auth/`: sign-up, email verification, sign-in, sign-out, the current session, password reset and change | H5 |
| `/api/v1/account`: usage, language, data export, deletion | H6 |
| `/api/v1/projects`, `/api/v1/animations` and their duplication; `/api/v1/animations/{id}/document` | H6 |
| `/api/v1/support-requests`, their messages and screenshot | H9 |
| `POST /api/v1/events` | H13 |
| `/api/v1/tokens`; `/mcp`; `/api/v1/exports/{link}` | A3 |
| the internal admin API, on its own listener | H10 |

### Local library

- The adapter writes atomically — a temporary file, then a rename —, so that the desktop app and
  the CLI can share a library at the same time. H2 documents the layout of the folder in
  `crates/service/README.md`.
- The default library is a `Life Pixel` folder in the user's documents, for the desktop app and
  the CLI alike; each accepts another one — a setting, a flag.
- An MCP server over stdio writes exports only into its allowed directories: its working
  directory — the project an MCP client started it in —, plus those its flags add.

### Product events

- Pseudonymous: a keyed hash of the account, never its id; no pixel, no free text; kept 13
  months.
- The server records the events of its use cases. The app may only send V1's allow-list —
  `export_completed`, with the format and a size class —, and a visitor's events carry no
  identifier.

### Admin

- `server` exposes the internal admin API on a second port, on the environment's private network
  only — never on `edge`. The admin server authenticates to it with a secret from configuration
  and passes the admin's identity, which `server` writes to the audit log with every action.
- Admin accounts live in a database of the admin server's own, on the environment's Postgres,
  under a role that cannot reach the application's database.

## Environment

### Tools

| Tool | Version | For |
|---|---|---|
| Rust | `rust-toolchain.toml` | every crate, and the `wasm32-unknown-unknown` target |
| Node.js | `.nvmrc` | the front-end and `player-js` |
| Docker, with Compose | recent | the local stack, the images |
| cargo-deny | latest | the dependency policy |
| cargo-fuzz and a nightly toolchain | latest | P1's fuzz target |
| wasm-bindgen-cli | the version of the `wasm-bindgen` crate in `Cargo.lock` | `cargo xtask build-editor` |
| Playwright's browsers | those of the pinned Playwright | the end-to-end suites |
| libwebp's command-line tools | recent | S1's measurements only |
| actionlint | latest | every workflow change |
| Tauri's prerequisites for the host system | Tauri 2 | M5 |

Tools are installed once; no build step downloads a binary, and the player and the editor are
built from the sources of the same commit.

### Local stack

`docker compose up -d` (H0) starts Postgres, an S3-compatible object storage, and a mail catcher
with an HTTP API, from which the end-to-end suites read verification emails. One instance serves
every agent and every test run:

- its ports are bound to `127.0.0.1`, fixed, and listed in `.env.example`;
- **no two test runs share a database.** A helper of H4 gives each test a database with a random
  name on the shared server, migrates it, then drops it. `#[sqlx::test]` does not fit: as of
  sqlx 0.8 it names a test's database after the test's path, so two worktrees running the same
  test on one server drop each other's database;
- object-storage tests work under a random prefix, mail tests with a random recipient;
- the server and the admin server run on the host against the stack; H14 builds and checks their
  images.

A task that creates an area writes its commands into the development loop of
[CONTRIBUTING.md](../.github/CONTRIBUTING.md) and into the checks of AGENTS.md.

## Building V1 in parallel

The [rules of the split](implementation-plan.md#rules-of-the-split) apply. This is how the work
comes together.

### Branches

- V1 is built on an **integration branch** cut from `dev`, never on `dev` itself. The maintainer
  brings it into `dev` through pull requests that stop at the milestone checkpoints below, so that
  each review ends on a verified commit.
- Each task works on a branch of its own, named after it (`feat/format-codec`), cut from the
  integration branch once everything it waits for is there, and kept after its merge.
- Only integration steps merge into the integration branch, and no task rewrites another's
  commits.
- Commits follow AGENTS.md: Conventional Commits in English, with the repository's scopes.
- Nobody but the maintainer pushes.

### Waves and integration steps

The waves of the [order](implementation-plan.md#order) run one after the other. After each:

1. an **integration step** merges the wave's branches into the integration branch — conflicts on
   shared files are additions on both sides, and both stay —, runs every check of AGENTS.md and
   every end-to-end suite that exists against the local stack, and writes the failures down;
2. **fixes** follow that report, each on a branch of its own, on disjoint files;
3. a **verification step** merges the fixes and runs the checks again. The next wave starts from
   the commit it verified.

The milestone checkpoints — M1 once S1 has measured, M2 with U6, M3 with H17, M4 with A5, M5 with
T5 — are verification steps that also run their milestone's journey.

### Who runs what

| Who | Runs |
|---|---|
| A task | formatting, lints and tests of the crates and projects it touched — `cargo clippy -p …`, `cargo test -p …`, the project's unit tests —, its database tests included, through H4's helper |
| An integration or verification step | everything AGENTS.md lists, the size budgets, and every end-to-end suite that exists |

End-to-end suites and full builds are heavy: steps run them, one at a time on a machine, and a
task only runs the suite it owns.

### Shared files

Tasks that run at the same time own disjoint paths. The files below are shared: they only receive
additions, placed so that parallel additions merge.

| File | Rule |
|---|---|
| the root `Cargo.toml` | members globbed; `[workspace.dependencies]` sorted |
| `.github/workflows/_verify.yml`, `.github/dependabot.yml` | a job per area; an entry per ecosystem and directory |
| `i18n/*.json` | keys under the task's own prefix, sorted |
| `frontend/projects/app/src/app/app.routes.ts` | a lazy route per feature |
| `compose.yaml` | a block per service |
| the module lists of `crates/service` and `crates/server` | a line per module |
| AGENTS.md's checks, with CLAUDE.md; the documentation maps; CONTRIBUTING.md's development loop | a line or a row per addition |

### When a contract must change

Its owner changes it, and the orchestrator warns every task that relies on it. A task that needs
a change it does not own stops and reports it, rather than working around the contract.

## Release

The maintainer's steps, in order:

1. Provide the [inputs](#what-only-the-maintainer-provides) that staging needs.
2. Push the integration branch, and open the pull requests into `dev`, checkpoint by checkpoint.
   A green CI promotes `dev` to `test` and deploys staging.
3. On staging: `/healthz` answers over TLS, the first admin account exists, the account journey
   passes by hand, and the admin console shows staging's health, metrics and logs.
4. Rehearse a restore from the backup bucket, as [devops.md](devops.md) requires before
   production.
5. Configure the legal identity, and validate the legal texts.
6. Fast-forward `main` to the validated `test` commit: production deploys.
7. With the signing material in the `release` environment, tag `v1.0.0` on `main`: `release.yml`
   publishes the installers, `@life-pixel/player`, the Docker version tags and the GitHub
   Release.
