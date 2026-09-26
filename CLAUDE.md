# Life Pixel

> `CLAUDE.md` and `AGENTS.md` hold the same text, byte for byte: Claude Code reads the first,
> the other coding agents read the second. Change both in the same commit —
> `cmp CLAUDE.md AGENTS.md` must stay silent, and CI will check it.

Life Pixel is an open-source editor for pixel-art animations. An animation compiles into a
self-contained WebAssembly bundle of a few kilobytes that any web page or webview plays without
a dependency. The product ships as a hosted service (web and Android), a desktop app and a
self-hostable server, and AI agents drive it through an MCP server.

**Status: design phase.** There is no application code yet. `docs/` is the specification, and
the layout, commands and rules below are the target the first code must follow. When code and
documentation disagree, fix one of them in the same pull request.

## Language

English everywhere in the repository: code, comments, documentation, commit messages, pull
requests, issues. The interface is translated (English and French today) and never hard-codes
a user-facing string — see [docs/i18n.md](docs/i18n.md).

## Documentation map

| Document | Answers |
|---|---|
| [docs/product.md](docs/product.md) | what we build, for whom, in which order |
| [docs/implementation-plan.md](docs/implementation-plan.md) | how the milestones split into tasks for parallel agents, and in which order |
| [docs/v1/](docs/v1/README.md) | V1's technical design: scope, method, and what each task builds and how it is tested |
| [docs/architecture.md](docs/architecture.md) | components, crates, data flow, distributions |
| [docs/export.md](docs/export.md) | how an animation becomes a WASM bundle, and how an app plays it |
| [crates/format/README.md](crates/format/README.md) | the payload and the player ABI, byte by byte |
| [docs/mcp.md](docs/mcp.md) | MCP tools, transports, authentication, limits |
| [docs/admin-console.md](docs/admin-console.md) | the admin, support and metrics console |
| [docs/pricing.md](docs/pricing.md) | plans, storage quota, billing rules |
| [docs/devops.md](docs/devops.md) | environments, branches, CI/CD, deployment on the shared VPS |
| [docs/security-model.md](docs/security-model.md) | threat model, supply chain, privacy |
| [docs/i18n.md](docs/i18n.md) | languages, catalogues, adding a language |
| [docs/decisions.md](docs/decisions.md) | the decision log: accepted, proposed, open |

A **proposed** decision is not settled: do not build on it without the maintainer's go-ahead.

## Repository layout (target)

| Path | Role |
|---|---|
| `crates/core` | document model, validation, domain limits, editing operations, compositing — pure |
| `crates/format` | versioned binary format of a compiled animation — `no_std` decoder |
| `crates/player` | the runtime compiled to WebAssembly — tiny, `no_std`, no wasm-bindgen |
| `crates/compiler` | document → exports: WASM bundle, GIF, APNG, sprite sheet, PNG frames |
| `crates/editor-wasm` | wasm-bindgen facade over `core` and `compiler` for the editor |
| `crates/service` | use cases (edit, save, export, quotas), their storage ports, the local adapter |
| `crates/mcp` | MCP tools over `service`, independent of the transport |
| `crates/server` | HTTP API and MCP endpoint, Postgres and object-storage adapters |
| `crates/admin-server` | admin, support and metrics API |
| `crates/cli` | `life-pixel` binary: local export, MCP over stdio |
| `player-js` | npm package `@life-pixel/player`: the loader and the `<life-pixel>` element |
| `frontend` | Angular workspace: `projects/app` (editor, Ionic), `projects/admin`, `projects/shared` |
| `i18n` | translation catalogues, one JSON file per language: served at runtime, shipped by the desktop app, used for emails |
| `tauri` | Tauri 2 shell around `projects/app`: desktop with the local backend, Android on the hosted API |
| `xtask` | build commands Cargo cannot express: the player, the engine, the size checkpoint, the desktop bundle |
| `samples` | sample animations drawn for the project, dedicated to CC0 |
| `docker`, `compose*.yaml` | images and deployment overlays |

Dependency direction between crates:

- `core` and `format` depend on no other workspace crate and perform no I/O.
- `player` depends on `format` only.
- `service` never knows about HTTP, Tauri or MCP; `mcp` never knows its transport.
- `server`, `admin-server`, `cli` and `tauri` are leaves: nothing depends on them.

## Architecture invariants

Each rule protects something. Do not break one without an accepted decision in
`docs/decisions.md`.

1. **One Rust core.** Domain rules — model, validation, limits, rendering — live in `core`, and
   the export format in `format`, nowhere else. The editor (through `editor-wasm`), the server,
   the MCP tools, the CLI and the compiler consume them; TypeScript never re-implements one.
   This is what guarantees that the editor shows exactly what the export plays.
2. **An export is data, never code.** Compiling appends a validated payload to a prebuilt,
   versioned player. Nothing runs `rustc`, a shell or generated code at export time, and no user
   input ever becomes executable.
3. **The player stays small.** `no_std`, no wasm-bindgen, no import beyond what the loader
   provides. The size budgets of `docs/export.md` are enforced by CI; exceeding one is a design
   discussion, not a threshold to raise.
4. **The format is a public contract.** Exports live for years inside other people's apps. Any
   change to the format bumps its version; a player refuses — never misreads — a payload it does
   not know, and fixture tests keep every released version playable.
5. **One code base for every distribution.** Hosted-only concerns (billing, plans) sit behind
   configuration. A self-hosted server or the desktop app runs without them and never calls our
   infrastructure.
6. **Persistence behind ports.** `service` defines the storage traits. `server` implements them
   with Postgres and S3-compatible object storage; the local adapter of `service` (desktop, CLI)
   stores projects as files in a library folder. No SQL or filesystem access outside adapters.
7. **The API speaks codes, the interface speaks languages.** The API returns stable error codes
   with parameters, never sentences. Every user-facing string lives in the i18n catalogues, and
   English and French stay complete.
8. **Telemetry is first-party, and opt-in outside the hosted service.** No third-party tracker
   anywhere; the desktop app and self-hosted servers send nothing unless the user opts in.
9. **Limits are enforced on the server.** The client validates for comfort; `service` validates
   for real, with the constants of `core`.
10. **What ships in users' apps stays MIT.** `crates/format`, `crates/player` and `player-js` —
    the integration snippets included — are MIT (`LICENSE-MIT`); everything else is
    AGPL-3.0-only (`LICENSE`). An MIT part never depends on an AGPL one, and code never moves
    from the AGPL side to the MIT side without the maintainer's decision. Each crate and package
    declares its licence in its manifest (`license = "MIT"` or `"AGPL-3.0-only"`).

## The shared VPS

Staging and production run on a VPS shared with other projects. Its edge (Caddy), its shared
Docker networks and its observability stack belong to the private `infra-vps` repository. A
mistake here can take other projects down.

- Never deploy, restart or reconfigure the edge from this repository. The deploy job only checks
  that the `edge` network exists, and fails otherwise.
- No `ports:` in deployment files. A public service joins the external `edge` network and
  declares its domain with `caddy` labels.
- Never add a `log` directive or a `caddy.import: journal-acces` label to a site: it silently
  drops every other site of the host from the access log.
- Every service has a memory limit (`deploy.resources.limits`) and a healthcheck.
- Metrics: `/metrics` on an internal port; the service joins the external `observability`
  network and carries the `prometheus.scrape`, `prometheus.port` and `prometheus.path` labels.
  Traces: OTLP to `http://otel-collector:4318`. Logs: JSON on stdout, one event per line.
- Metric labels stay low-cardinality: a route template, never a raw path, an id or an email.
- Secrets live in GitHub secrets and reach the host as a `.env` file, never in the repository.

## Workflow

- Branches start from `dev` — the default branch — and come back to it through a pull request,
  contributors and Dependabot alike. `test` and `main` only move by fast-forward promotion, never
  by a direct commit.
- Branch names follow `type/short-description`: a commit type in its short form, then 2 to 5
  kebab-case words without accents — `feat/onion-skin`, `fix/gif-palette-order`. One branch =
  one topic.
- A push to `dev` runs CI; when it is green, `test` advances to that commit and staging is
  deployed. Production is a fast-forward of `main` to a commit already validated on `test`.
  Details in [docs/devops.md](docs/devops.md).

## commit

Commit convention (maintained by /commit, initialised by /b-hive-init).
- Style: Conventional Commits — language: en
- Scopes (path → scope):
  - `crates/core/**` → `core`
  - `crates/format/**` → `format`
  - `crates/player/**`, `player-js/**` → `player`
  - `crates/compiler/**` → `compiler`
  - `crates/editor-wasm/**` → `editor-wasm`
  - `crates/service/**` → `service`
  - `crates/mcp/**` → `mcp`
  - `crates/server/**` → `server`
  - `crates/admin-server/**` → `admin-server`
  - `crates/cli/**` → `cli`
  - `frontend/projects/app/**` → `app`
  - `frontend/projects/admin/**` → `admin`
  - `frontend/projects/shared/**` → `shared`
  - the rest of `frontend/**` (workspace configuration) → `frontend`
  - `i18n/**` → `i18n`
  - `tauri/**` → `tauri`
  - `docker/**`, `compose*.yaml` → `infra`
  - `.github/workflows/**`, `.github/dependabot.yml`, `.github/CODEOWNERS` → `ci`
  - `docs/**`, `README.md`, `CLAUDE.md`, `AGENTS.md`, the rest of `.github/**` → `docs`
  - `scripts/**` → `scripts`
  - `xtask/**` → `xtask`
  - `samples/**` → `samples`
  - cross-cutting files at the root (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
    `rustfmt.toml`, `clippy.toml`, `deny.toml`, `.nvmrc`, `.editorconfig`, `.gitignore`,
    `.gitattributes`, `LICENSE`, `LICENSE-MIT`) → no scope: `build` for dependencies and
    toolchains, `chore` otherwise

Subject: `type(scope): description` — imperative mood, lowercase initial, no trailing period, 72
characters at most. The optional body explains the *why*; a breaking change carries a
`BREAKING CHANGE:` footer. One commit = one coherent change: never mix two topics.

Cross-cutting rules: tests (`#[cfg(test)] mod tests`, `crates/*/tests/**`, `*.spec.ts`) travel
with the code they test; changelog entries are attached to the feature/fix commit they document.

Examples: `feat(compiler): append the payload as a custom section`,
`fix(player): stop on the last frame of a non-looping tag`, `build: bump axum to 0.8.9`.

## Code conventions

These conventions apply to all the project's code. The **numeric limits** are ceilings to
respect; the **principles** are defaults to follow, unless there is an explicit, justified reason
to depart from them.

### Guiding principles

- **DRY (Don't Repeat Yourself)** — No duplication of logic or of domain knowledge: a rule lives
  in a single place. Here, a domain rule belongs to `core`: the server, the MCP tools and the
  editor consume it, they never re-implement it. (Nuance: do not abstract before the 3rd
  repetition — a one-off duplication beats a bad abstraction.)
- **KISS (Keep It Simple)** — Choose the simplest solution that actually solves the problem. No
  gratuitous complexity or cleverness.
- **SOLID**:
  - **S — Single responsibility**: a module or a component has only one reason to change. The
    compiler compiles, `service` decides, `server` translates HTTP; none spills into its
    neighbour.
  - **O — Open/closed**: open to extension, closed to modification. A new export format or MCP
    tool is a new implementation, not an edit of the existing ones.
  - **L — Liskov substitution**: any implementation of a trait — a storage adapter, an exporter —
    can replace another without surprising the caller.
  - **I — Interface segregation**: prefer several focused traits or interfaces to one catch-all.
  - **D — Dependency inversion**: depend on abstractions. `service` depends on its ports; the
    adapters implement them.

### Size and complexity limits (verifiable)

| Rule | Limit |
|---|---|
| File size | ≤ 300 lines (warning), 400 maximum |
| Files per folder | ≤ 10 (beyond that, split into subfolders by domain) |
| Function / method size | ≤ 30 lines |
| Number of parameters | ≤ 3 (beyond that, group them into a struct / an object) |
| Nesting depth | ≤ 3 levels |
| Cyclomatic complexity | ≤ 10 per function |
| Line length | ≤ 100 characters (`rustfmt` and Prettier are set to it) |

- **A single public element per file** — one component, service or main type per file, named
  after the file.
- **No magic numbers or strings** — extract them into named constants that explain their intent.
  Domain limits (canvas size, frame count, palette size) are constants of `core`; plan quotas are
  configuration. Neither is repeated as a literal in validation, MCP schemas or the interface.

### Naming

- **Explicit names that reveal intent** — the name says *what* and *why*, not *how*. A long clear
  name beats a short obscure one.
- **Idiomatic casing per language, never mixed**:
  - Rust: `snake_case` for functions, variables, modules and fields; `PascalCase` for types,
    traits and enum variants; `SCREAMING_SNAKE_CASE` for constants and statics; crates are named
    `life-pixel-<name>`.
  - TypeScript: `camelCase` for variables, functions and members; `PascalCase` for classes, types
    and interfaces (no `I` prefix); `UPPER_SNAKE_CASE` for module-level constants; `kebab-case`
    file names; component selectors prefixed with `lp-`.
  - HTTP API: JSON fields in `camelCase`. MCP: tool names and arguments in `snake_case`.
  - Error codes: dot-separated `snake_case` segments, domain first — `quota.storage_exceeded`.
  - i18n keys: lowercase, dot-separated, scoped by feature — `editor.toolbar.pencil`.
- **No cryptic abbreviations** — `frame_count`, not `fc`. Only universal abbreviations are
  tolerated (`id`, `url`, `http`, `rgba`, `fps`).
- **Booleans prefixed** with `is`, `has`, `should`, `can`… (`is_looping`, `hasOnionSkin`).

### Functions

- **One function = one thing** — if you have to write "and" to describe what it does, split it.
- **Favour pure functions** — avoid side effects where possible, and make them explicit when they
  are necessary. `core`, `format` and `compiler` stay pure and testable without I/O.
- **Guard clauses / return early** — handle edge cases and return early instead of nesting
  `if/else`. In Rust, use `?` rather than nested `match` on `Result`.
- **Avoid flag parameters** — a boolean that changes behaviour hides two functions disguised as
  one; split them.
- **CQS (Command Query Separation)** — a function either *changes* state OR *returns* a value,
  never both.

### Rust

- Edition 2024. The toolchain is pinned by `rust-toolchain.toml`; shared dependency versions are
  declared once, in `[workspace.dependencies]`.
- `cargo fmt` is the formatting. `cargo clippy --workspace --all-targets -- -D warnings` passes.
  The root `Cargo.toml` enables `[workspace.lints]`: `unsafe_code = "forbid"` (only the player
  may relax it, each block carrying a `// SAFETY:` comment), and the clippy lints
  `too_many_lines`, `cognitive_complexity` and `too_many_arguments`, whose thresholds are set in
  `clippy.toml` from the limits above.
- Errors: library crates expose typed errors (`thiserror`); `anyhow` only at the top of binaries.
  No `unwrap()` / `expect()` on a fallible path outside tests. A panic never crosses the
  WebAssembly boundary.
- `core`, `format` and `compiler` have no I/O, no async, no clock and no randomness: those are
  injected.
- Async code runs on tokio. CPU-heavy work — compiling, rendering — goes to `spawn_blocking`,
  never onto the executor.
- Values coming from outside are converted with `try_from`, never with a truncating `as`.
- Logs go through `tracing` with structured fields; `println!` is reserved for the CLI's output.
- Public items of library crates are documented (`///`); `missing_docs` warns.
- Every new dependency is justified in the pull request and passes `cargo deny`. The player adds
  none.

### TypeScript and Angular

- TypeScript `strict`. No `any` — take `unknown` and narrow it. No non-null assertion (`!`) to
  silence the compiler.
- Standalone components, `ChangeDetectionStrategy.OnPush`, signals for state (`signal`,
  `computed`, `input()`, `output()`), `inject()` rather than constructor parameters, built-in
  control flow (`@if`, `@for` with `track`), lazy-loaded routes.
- Components display and delegate; logic lives in services and pure functions. HTTP goes through
  the typed client generated from the server's OpenAPI description (`projects/shared`), never
  through `HttpClient` calls scattered across components.
- Pixel work — drawing, compositing, exporting — goes through `editor-wasm`. Anything longer than
  a frame runs in a Web Worker, so the interface never freezes.
- Ionic provides the application shell and the mobile experience; the canvas and the timeline
  are our own components.
- ESLint (angular-eslint) and Prettier (`printWidth: 100`) pass; file layout follows the official
  Angular style guide.
- No user-facing string in a template or in code: translation keys, added to every catalogue of
  `i18n/` in the same commit. Catalogues are fetched at runtime from the i18n endpoint, one
  language at a time — never compiled into the bundle.
- Accessibility targets WCAG 2.2 AA: keyboard-reachable, labelled controls, visible focus,
  sufficient contrast, `prefers-reduced-motion` honoured.

### HTTP API

- JSON over HTTPS under `/api/v1`, described by an OpenAPI document generated from the Rust code.
- Errors are RFC 9457 problem details carrying a stable `code` and its parameters.
- Collections paginate by cursor. A breaking change means a new version prefix; clients send
  their version so that the server can require an update from an outdated mobile app.

## Tests

- **Every feature ships with tests**, delivered alongside it (same branch, same pull request).
- **Only what is needed to test the feature**: the nominal behaviour and the edge cases it
  introduces. No chasing a coverage percentage, no redundant tests; we test neither the framework
  nor third-party libraries.
- A good test fails when the feature's **behaviour** breaks — not when its implementation
  changes.
- A bug fix ships with the test that fails without it.
- Where they live: unit tests in `#[cfg(test)] mod tests` next to the code; integration tests in
  `crates/<crate>/tests/`; rendering and exports checked against golden files; the format decoder
  fuzzed with `cargo fuzz`, because it parses untrusted bytes inside other people's pages;
  Angular unit tests with Vitest (`*.spec.ts`); the critical path — draw, animate, export, play —
  covered end to end with Playwright.

## Before declaring a change done

Run the checks that cover what you touched and report their output. If one cannot run, say so:
never announce a green state you have not seen. These commands become available as the workspace
is scaffolded, and CI runs the same ones.

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p life-pixel-server --features stack-tests  # needs the local stack and a .env
cargo test -p life-pixel-admin-server --features stack-tests  # needs the local stack and a .env
cargo deny check
cargo xtask check-boundaries
cargo xtask build-player --check
cargo xtask build-desktop --debug
npm run lint --prefix frontend && npm run test:ci --prefix frontend && npm run build --prefix frontend && npm run i18n:check --prefix frontend
npm run test:engine --prefix frontend
npm run api:generate --prefix frontend && git diff --exit-code -- crates/server/openapi.json frontend/projects/shared/src/lib/api/schema.d.ts crates/admin-server/openapi.json frontend/projects/shared/src/lib/admin-api/schema.d.ts
npm run build --prefix player-js && git diff --exit-code -- player-js/life-pixel.js && npm test --prefix player-js && npm run size --prefix player-js
cmp CLAUDE.md AGENTS.md
docker run --rm -v "$PWD":/repo -w /repo rhysd/actionlint:latest@sha256:b1934ee5f1c509618f2508e6eb47ee0d3520686341fec936f3b79331f9315667 -color
```

## Security

- No secret, token, personal data or real user content in code, tests, fixtures, logs or issues.
- Validate every input at the boundary, with the limits of `core`; decode uploaded images with
  size limits before anything else touches them.
- Vulnerabilities are reported privately, as described in
  [.github/SECURITY.md](.github/SECURITY.md).
