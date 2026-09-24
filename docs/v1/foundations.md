# Foundations — F1, F2, F3, H0

The ground every other task stands on: CI, the two workspaces, the i18n tooling and the local
stack. Nothing here holds product logic.

## F1 — Continuous integration

Files: `.github/workflows/ci.yml`, `_verify.yml`, `security.yml`, and `.github/dependabot.yml`.

Rules for every workflow, now and later:

- actions are pinned to a full commit SHA, the version in a comment: `actions/checkout@<sha> # v5`;
- `permissions: {}` at the top; each job asks only for what it uses, `contents: read` by default;
- `concurrency: { group: "${{ github.workflow }}-${{ github.ref }}", cancel-in-progress: true }`
  on pull requests; pushes are never cancelled;
- no secret reaches a pull request job, and `pull_request_target` is never used;
- a job that needs the Rust toolchain runs `rustup show`, which installs what `rust-toolchain.toml`
  pins; a job that needs Node uses `actions/setup-node` with `node-version-file: .nvmrc`.

### ci.yml

```yaml
name: CI
on:
  pull_request:
    branches: [dev]
  push:
    branches: [dev, main]
permissions: {}
jobs:
  verify:
    uses: ./.github/workflows/_verify.yml
    permissions:
      contents: read
```

H14 adds the delivery jobs that run on pushes after `verify` ([operations.md](operations.md)).

### _verify.yml

`on: workflow_call`. One job per area; F1 writes `repository`, and each later job is added by the
task named, with the commands of its section.

| Job | Added by | Runs |
|---|---|---|
| `repository` | F1 | `cmp CLAUDE.md AGENTS.md`; actionlint on `.github/workflows/`, through the `rhysd/actionlint` image pinned by digest |
| `rust` | F2 | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check-boundaries`; cache through `Swatinem/rust-cache`; T1 adds Tauri's Linux system packages |
| `rust-stack` | H4 | copies `.env.example` to `.env`, starts the local stack with `docker compose up -d --wait` — not as job services, which start before the checkout and without `init.sh` —, then `cargo test -p life-pixel-server --features stack-tests` — H11 adds `-p life-pixel-admin-server`; the packages are named, so that `tauri/` is not compiled here |
| `fuzz-smoke` | P1 | nightly toolchain, `cargo fuzz run decode -- -max_total_time=60` in `crates/format` |
| `player` | P2 | `cargo xtask build-player --check` |
| `player-js` | L1 | `npm ci`, `npm run build` then `git diff --exit-code` on the committed build, `npm test`, `npm run size` |
| `frontend` | F3 | `npm ci`, `lint`, `test:ci`, `build`, `i18n:check`; from W1 on, it installs the Rust toolchain and wasm-bindgen-cli, builds the engine once, runs `test:engine`, and uploads the engine as the `engine` artifact |
| `api` | H3 | `npm run api:generate --prefix frontend`, then `git diff --exit-code` |
| `e2e-editor` | U6 | the `engine` artifact with `LP_ENGINE_PREBUILT=1`, Playwright browsers, `npm run e2e --prefix frontend` |
| `e2e-hosted` | H17 | the local stack as in `rust-stack`; `npm ci`, the `engine` artifact with `LP_ENGINE_PREBUILT=1`, Playwright browsers; `npm run e2e:hosted --prefix frontend` |
| `docker` | H14 | hadolint on `docker/`, `docker compose config` for each overlay, image builds without push |
| `mcp` | A5 | the local stack as in `rust-stack`; `cargo test -p life-pixel-cli --test mcp_stdio` and `cargo test -p life-pixel-server --features stack-tests --test mcp_http` |
| `desktop` | T1, then T5 | Tauri's system packages, `cargo install tauri-cli --version ^2 --locked`, `npm ci`, the `engine` artifact with `LP_ENGINE_PREBUILT=1`, `cargo xtask build-desktop --debug`; T5 adds `tauri-driver` and WebKitGTK's WebDriver, and runs its smoke test under Xvfb |

### security.yml

Triggers: `pull_request` and `push` on `dev` and `main`, weekly `schedule`, `workflow_dispatch`.

| Job | Added by | Runs |
|---|---|---|
| `codeql` | F1 (`actions`), F2 (`rust`), F3 (`javascript-typescript`) | a matrix over the languages, `build-mode: none`; `security-events: write` |
| `dependency-review` | F1 | pull requests only; `fail-on-severity: moderate` |
| `cargo-deny` | F2 | `cargo deny check` |
| `npm-audit` | F3, L1 | `npm audit --audit-level=moderate` in `frontend/` and `player-js/` |

### dependabot.yml

```yaml
version: 2
updates:
  - package-ecosystem: github-actions
    directory: /
    schedule: { interval: weekly }
    groups:
      actions: { patterns: ["*"] }
```

Later entries, same shape: `cargo` on `/` (F2), `npm` on `/frontend` (F3) and `/player-js` (L1),
`docker-compose` on `/` (H0), `docker` on `/docker` (H14).

**Tests and checks**: actionlint passes on every workflow; the `repository` job's commands pass
locally. The workflows first run on GitHub when the maintainer pushes.

## F2 — Cargo workspace and export contract

### Root files

`Cargo.toml`:

```toml
[workspace]
members = ["crates/*", "xtask"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.98"
publish = false
repository = "https://github.com/LINDECKER-Charles/app-life-pixel"

[workspace.dependencies]
# One line per dependency, sorted. Each task adds those it uses.
life-pixel-format = { path = "crates/format" }

[workspace.lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[workspace.lints.clippy]
cognitive_complexity = "warn"
expect_used = "warn"
too_many_arguments = "warn"
too_many_lines = "warn"
unwrap_used = "warn"

[profile.player]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Every member's manifest takes the workspace's `version`, `edition`, `rust-version` and `publish`,
declares `license = "AGPL-3.0-only"` — `"MIT"` for `format` and `player` — and `[lints]
workspace = true`, except the player, whose own table is in
[format-player.md](format-player.md#p2--player). Warnings fail CI (`-D warnings`), so the lints
above are enforced.

- `.cargo/config.toml`: `[alias] xtask = "run --quiet --package xtask --"`.
- `clippy.toml`: add `allow-unwrap-in-tests = true` and `allow-expect-in-tests = true`.
- `deny.toml`:

```toml
[graph]
all-features = true

[advisories]
yanked = "deny"

[licenses]
allow = ["Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause",
         "CDLA-Permissive-2.0", "ISC", "MIT", "MPL-2.0", "Unicode-3.0", "Zlib"]
confidence-threshold = 0.93

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

Our own crates are unpublished (`publish = false`), so `licenses.private` skips them; the licence
boundary between them is checked by `xtask` instead.

### xtask

`xtask/` is a binary crate (`AGPL-3.0-only`) with a `clap` command per subcommand, one file each
under `xtask/src/commands/`; its commits use the scope `xtask`. F2 writes the dispatcher and one
command:

- `cargo xtask check-boundaries` reads `cargo metadata` and fails unless `life-pixel-format` has
  no dependency, `life-pixel-player` depends on `life-pixel-format` only, both declare `MIT`,
  every other member declares `AGPL-3.0-only`, and `player-js/package.json` declares `MIT` once
  it exists.

Later commands: `build-player` (P2), `build-editor` (W1), `measure-sizes` (S1), `build-desktop`
(T1), `build-sidecar` (T3).

### crates/format

F2 creates the crate with the public types and function signatures of
[format-player.md](format-player.md#rust-api), bodies left to P1 (`todo!()`), and writes
`crates/format/README.md` from the [payload](format-player.md#payload-v1) and
[ABI](format-player.md#abi-v1) sections: it becomes their reference. [export.md](../export.md)
links to it, and its draft ABI table gives way to the final one.

**Tests and checks**: the workspace builds; fmt, clippy, tests, `cargo deny check` and
`cargo xtask check-boundaries` pass; the `rust` and `cargo-deny` jobs, the `rust` CodeQL language
and the `cargo` Dependabot entry exist; CONTRIBUTING.md's development loop names the Rust commands.

## F3 — Front-end workspace

### Scaffolding

```shell
npx -y @angular/cli@22 new frontend --create-application=false --package-manager=npm --skip-git
cd frontend
npx ng generate application app --prefix=lp --style=scss --routing --ssr=false
npx ng generate library shared --prefix=lp
npm install @ionic/angular@9 @jsverse/transloco @jsverse/transloco-messageformat
npm install --save-dev @playwright/test @axe-core/playwright @messageformat/parser prettier
npx ng add angular-eslint
```

The CLI's defaults of the pinned version stay: standalone components, zoneless change detection,
Vitest through Angular's unit-test builder, the application builder. `projects/admin` comes with
H12.

### Settings

- TypeScript: `strict`, `noImplicitOverride`, `noImplicitReturns`,
  `noPropertyAccessFromIndexSignature`, `noFallthroughCasesInSwitch`; Angular's `strictTemplates`.
- Prettier: `.prettierrc.json` `{ "printWidth": 100, "singleQuote": true }`; generated files in
  `.prettierignore`.
- ESLint (`eslint.config.js`): angular-eslint's TypeScript, template and template-accessibility
  recommended sets, plus these rules — test files (`*.spec.ts`, `e2e/`) relax the size ones:

| Rule | Setting |
|---|---|
| `@typescript-eslint/no-explicit-any`, `@typescript-eslint/no-non-null-assertion` | error |
| `max-lines` | error at 400 |
| `max-lines-per-function` | error at 30, blank lines and comments skipped |
| `max-params`, `max-depth`, `complexity` | error at 3, 3 and 10 |
| `@angular-eslint/component-selector` | element, prefix `lp`, kebab-case |
| `@angular-eslint/prefer-on-push-component-change-detection`, `@angular-eslint/template/prefer-control-flow` | error |

- `angular.json`: the app's assets copy the catalogues, so that a static build and the desktop app
  serve them at `/i18n/`:
  `{ "glob": "**/*", "input": "../i18n", "output": "i18n" }`.
- Playwright: `frontend/playwright.config.ts`, `testDir: "e2e"`; U6 and H17 add their projects.

### Scripts of frontend/package.json

| Script | Command |
|---|---|
| `start` | `ng serve app --port 4260`; W1 puts `node tools/prebuild.mjs &&` in front |
| `build:app` | `ng build app`; W1 puts `node tools/prebuild.mjs &&` in front |
| `build` | `npm run build:app`; H12 adds `build:admin` and runs both |
| `test:ci` | `ng test app --watch=false && ng test shared --watch=false`; H12 adds `ng test admin --watch=false` |
| `lint` | `ng lint && prettier --check .` |
| `format` | `prettier --write .` |
| `i18n:check` | `node tools/check-i18n.mjs` |

Later scripts: `test:engine` (W1), `e2e` (U6), `e2e:hosted` (H17), `api:generate` (H3),
`start:admin` and `build:admin` (H12).

### Internationalisation

- `i18n/languages.json`:
  `[{ "code": "en", "name": "English" }, { "code": "fr", "name": "Français" }]`.
- `i18n/en.json`, `i18n/fr.json`: flat keys, one per line, sorted, matching
  `^[a-z0-9_]+(\.[a-z0-9_]+)+$`. F3 adds `app.name` and the `common.` keys it uses (`common.ok`,
  `common.cancel`, `common.close`, `common.retry`, `common.loading`).
- `projects/shared/src/lib/i18n/provide-i18n.ts` provides Transloco: HTTP loader on
  `/i18n/<code>.json`, `fallbackLang: "en"`, `reRenderOnLangChange: true`, messageformat for ICU,
  missing keys logged in development; `/i18n/languages.json` is read at start-up
  (`provideAppInitializer`). The initial language is the account's, else the desktop settings',
  else the first of `navigator.languages` that exists, else English.
- `frontend/tools/check-i18n.mjs` fails when: a catalogue lacks a key of `en.json` or has one it
  lacks; keys are not sorted or not flat; an ICU message does not parse (`@messageformat/parser`);
  `languages.json` does not list exactly the catalogues; an `email.` message uses more than simple
  `{name}` arguments. H16 extends it to the legal files.

**Tests and checks**: `lint`, `test:ci`, `build` and `i18n:check` pass; a unit test switches the
language and sees a translated text change without reloading; a check shows that no catalogue is
in the JavaScript bundle. The `frontend` job, the `javascript-typescript` CodeQL language, the
`npm-audit` job and the `npm` Dependabot entry exist.

## H0 — Local stack

`compose.yaml` holds what every environment shares — H14 adds the application's services;
`compose.override.yaml` what only a developer machine runs. Every image is pinned by digest, and
every service has a memory limit and a healthcheck when its image carries a tool for one.

```yaml
# compose.yaml
name: life-pixel
services:
  postgres:
    image: postgres:18-alpine@sha256:<digest>
    environment:
      POSTGRES_PASSWORD: ${LP_POSTGRES_PASSWORD:?}
      LP_DB_PASSWORD: ${LP_DB_PASSWORD:?}
      LPA_DB_PASSWORD: ${LPA_DB_PASSWORD:?}
    volumes:
      - pgdata:/var/lib/postgresql
      - ./docker/postgres/init.sh:/docker-entrypoint-initdb.d/10-life-pixel.sh:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 20
    deploy:
      resources:
        limits: { memory: 512M }
    restart: unless-stopped
volumes:
  pgdata:
```

Postgres 18 images keep their data under `/var/lib/postgresql`, hence the mount point.
`COMPOSE_PROJECT_NAME` overrides `name` on the hosts (devops.md).

`docker/postgres/init.sh` runs once, on an empty volume. It creates the role `life_pixel`, owner of
the database `life_pixel`, and the role `life_pixel_admin`, owner of `life_pixel_admin`, with the
passwords of the environment, and revokes `CONNECT` on both from `PUBLIC`. The `pg_trgm` and
`citext` extensions are trusted, so the migrations create them as owners.

```yaml
# compose.override.yaml
services:
  postgres:
    ports: ["127.0.0.1:5460:5432"]
  objectstore:
    image: adobe/s3mock@sha256:<digest>
    environment:
      COM_ADOBE_TESTING_S3MOCK_STORE_INITIAL_BUCKETS: life-pixel-local
    ports: ["127.0.0.1:5461:9090"]
    deploy:
      resources:
        limits: { memory: 512M }
  mail:
    image: axllent/mailpit@sha256:<digest>
    environment:
      MP_SMTP_AUTH_ACCEPT_ANY: "1"
      MP_SMTP_AUTH_ALLOW_INSECURE: "1"
    ports: ["127.0.0.1:5462:1025", "127.0.0.1:5463:8025"]
    deploy:
      resources:
        limits: { memory: 128M }
```

H0 checks the variable S3Mock reads its initial buckets from against the pinned version's
documentation. `.env.example` starts with the stack's variables — `LP_POSTGRES_PASSWORD`,
`LP_DB_PASSWORD`, `LPA_DB_PASSWORD`, all `local` — and
`LP_TEST_DATABASE_URL=postgres://postgres:local@127.0.0.1:5460/postgres`, which the
[test databases](server.md#test-databases) use.

**Tests and checks**: `docker compose up -d --wait` brings the three services up on their ports;
`psql` connects as `life_pixel` and as `life_pixel_admin`; the bucket accepts a `PUT` and a `GET`;
Mailpit's API lists a message sent to its SMTP port; `docker compose config` passes. The
`docker-compose` Dependabot entry exists, and CONTRIBUTING.md says how to start and reset the stack.
