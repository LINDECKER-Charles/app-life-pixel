# Contributing to Life Pixel

Thank you for your interest in the project. This guide explains how to report a problem, prepare
your environment, write code that follows the repository's conventions, and open a pull request.

Taking part in the project means respecting the [code of conduct](CODE_OF_CONDUCT.md).

**The project is in its design phase.** Right now, the most useful contributions are feedback on
the specification in [`docs/`](../docs/) — especially [decisions.md](../docs/decisions.md),
which lists what is still proposed or open. Code contributions start with the milestones of
[product.md](../docs/product.md).

## Licence of contributions

The repository has two licences. `crates/format`, `crates/player` and `player-js` — what ships
inside users' apps — are under the [MIT licence](../LICENSE-MIT); everything else is under the
[AGPL-3.0-only](../LICENSE). By offering a contribution, you agree that it is published under
the licence of the part it touches, and you confirm that you have the right to submit it. There
is no CLA to sign.

Code never moves from the AGPL side to the MIT side, and an MIT part never depends on an AGPL
one.

Do not bring in code, fonts, icons, palettes or artwork whose licence is unknown or incompatible.
Every new Rust dependency passes the licence allow-list of `cargo deny`.

## Before opening an issue

1. Check the documentation, starting with [product.md](../docs/product.md) — its non-goals
   included.
2. Search the existing issues, open and closed.
3. Pick the right channel:

| Situation | Channel |
|---|---|
| Reproducible bug | **Bug report** issue |
| New idea or need | **Feature request** issue |
| Question | see [SUPPORT.md](SUPPORT.md) |
| Problem with your hosted account, plan or data | the in-app support form — never a public issue |
| **Security vulnerability** | **never a public issue** — see [SECURITY.md](SECURITY.md) |

Never paste a token, an email address, a `.env` file or a creation you do not want public into
an issue.

## Preparing the environment

| Tool | Version | Pinned by |
|---|---|---|
| Rust | 1.98, with `rustfmt`, `clippy` and the `wasm32-unknown-unknown` target | `rust-toolchain.toml` — rustup installs it on the first `cargo` call |
| Node.js | 24.21.0 LTS | `.nvmrc` |
| cargo-deny | latest | `cargo install cargo-deny --locked` |
| Rust nightly, cargo-fuzz | latest nightly, cargo-fuzz 0.13 | fuzzing only: `rustup toolchain install nightly --profile minimal`, `cargo install cargo-fuzz --locked` |
| Docker | recent | local Postgres and object storage, image checks |
| `img2webp` | libwebp's command-line tools | the size checkpoint's WebP measurement only |
| Tauri CLI | 2 | the desktop app: `cargo install tauri-cli --version ^2 --locked` |

The Tauri apps need the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) of
your operating system: on macOS the Xcode command-line tools, on Windows the Microsoft C++ build
tools and WebView2, on Debian or Ubuntu WebKitGTK and its companions — without them, `cargo build`
and `cargo clippy --workspace` stop at `tauri/`:

```shell
sudo apt-get install libwebkit2gtk-4.1-dev libxdo-dev libssl-dev librsvg2-dev
```

For Android, add Android Studio with its SDK and NDK, a JDK, and the Rust
Android targets:

```shell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

The development loop — which commands start what. Each task that adds a command lists it here as
it lands; this is the loop so far.

### Repository checks

```shell
cmp CLAUDE.md AGENTS.md
docker run --rm -v "$PWD":/repo -w /repo rhysd/actionlint:latest@sha256:b1934ee5f1c509618f2508e6eb47ee0d3520686341fec936f3b79331f9315667 -color
```

### Rust

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask check-boundaries
```

`cargo xtask --help` lists the repository's build commands.

### Player

`crates/player` builds to the WebAssembly player, reproducibly, into `target/player/`. Its hash
is committed in `crates/player/player.sha256`: a change to the player or the format commits the
new hash, and CI fails when they differ.

```shell
cargo xtask build-player                  # the module, its size and its sha256
cargo xtask build-player --check          # the hash, the 16 KiB budget, the v1 fixture in wasmi
cargo xtask build-player --write-hash     # after a change: update player.sha256, then commit it
cargo clippy -p life-pixel-player --all-targets --target wasm32-unknown-unknown -- -D warnings
```

### Editor engine

`crates/editor-wasm` is the editor's engine, which the app runs in a Web Worker.
`cargo xtask build-editor` builds it for `wasm32-unknown-unknown`, then writes its JavaScript glue
to `frontend/projects/app/src/app/engine/wasm/generated/` and its module to
`frontend/projects/app/public/engine/`, both ignored by Git. It needs the `wasm-bindgen` command
of the version in `Cargo.lock`, and names the command to install it when it is missing or differs.

```shell
cargo install wasm-bindgen-cli --version "$(cargo pkgid wasm-bindgen | sed 's/.*@//')" --locked
cargo xtask build-editor                  # the module and its glue, into the app
cargo test -p life-pixel-editor-wasm      # EngineCore, on the host
cargo clippy -p life-pixel-editor-wasm --all-targets --target wasm32-unknown-unknown -- -D warnings
```

### Size checkpoint

`samples/` holds four CC0 animations (S1, [docs/v1/format-player.md](../docs/v1/format-player.md#s1--size-checkpoint)).
`cargo xtask measure-sizes` exports each as WASM, GIF, APNG and — through `img2webp`, for the
measurement only — a lossless animated WebP, measures every artefact raw, gzipped and with
brotli, writes the table of [docs/export.md](../docs/export.md#measured-sizes), and builds the
demo of `target/demo/`.

```shell
cargo xtask measure-sizes                 # the table, and target/demo/
git diff --exit-code -- docs/export.md    # the committed table is current
```

### Fuzzing

The format decoder's fuzz target, on the nightly toolchain; CI runs it 60 seconds on each pull
request and 30 minutes every week.

```shell
cd crates/format && cargo +nightly fuzz run decode -- -max_total_time=60
```

### Front-end

Run with the Node.js version of `.nvmrc`, from the root of the repository. `start`, `build` and
`test:engine` first build the editor's engine with `cargo xtask build-editor` — skipped when
`LP_ENGINE_PREBUILT=1` and both its outputs exist —; `start` and `build` then copy the catalogues
of `i18n/` into the app, which serves them at `/i18n/`. `test:ci` runs on the mock engine, and
needs no Rust. `start:admin` serves the admin console and proxies `/api` and `/i18n` to the admin
server on http://localhost:8463, which must run; `build` builds the app, then the console.

```shell
npm ci --prefix frontend
npx --prefix frontend playwright install chromium # once: the browser of test:engine
npm start --prefix frontend                       # the app on http://localhost:4260
npm run start:admin --prefix frontend             # the admin console on http://localhost:4263
npm run format --prefix frontend                  # Prettier, in place
npm run lint --prefix frontend                    # ESLint, then Prettier's check
npm run test:ci --prefix frontend                 # unit tests of app, shared and admin (Vitest)
npm run test:engine --prefix frontend             # W0's contract suite on the engine, in Chromium
npm run test:tools --prefix frontend              # tests of frontend/tools/
npm run build --prefix frontend                   # the app, then the admin console
npm run i18n:check --prefix frontend              # the catalogues of i18n/
npm run i18n:check-bundle --prefix frontend       # after build: no catalogue in the bundle
npm run e2e --prefix frontend                     # the editor's end-to-end path, in Chromium
npm run e2e:hosted --prefix frontend              # after build: the hosted journeys, on the stack
```

`e2e` runs the Playwright project `editor` (`frontend/e2e/editor/`): draw, animate, export, then
play the export with its loader, the same drawing with the keyboard alone, and axe on the
editor's screens. It starts the app with `npm start` on port 4260 — which must be free, or already
serve the app —, so the first run builds the engine; traces of failed tests land in
`frontend/e2e/editor/test-results/`.

### Hosted end to end

`e2e:hosted` runs the Playwright project `hosted` (`frontend/e2e/hosted/`): M3's journeys on the
built app and the admin console — sign-up and saving, the library, two pages saving one
animation, the quota, the data export, a password reset, a support request answered from the
console, a suspension, an account deleted — and axe on every page they open. It needs the local
stack up with its `.env` (see "Local stack"), a `build` of the app and the console, and ports
8460–8464 free.

```shell
npm run build --prefix frontend       # the app, then the admin console
npm run e2e:hosted --prefix frontend  # the hosted journeys, in Chromium
```

Its global setup builds `life-pixel-server` and `life-pixel-admin-server` with their
`stack-tests` feature, creates a test database for each with `test-database create`, serves both
with a free plan of 200,000 bytes on a temporary `file://` storage, and creates an admin with
`create-admin --password-stdin`; its teardown stops them and drops the databases. The journeys
read their emails from Mailpit, at random addresses under `test.life-pixel.invalid`. Traces of
failed tests land in `frontend/e2e/hosted/test-results/`, the servers' logs in
`frontend/e2e/hosted/logs/`.

### Loader

`player-js/` is `@life-pixel/player`, the `<life-pixel>` element. Its build, `life-pixel.js`, is
committed: a change to `player-js/src/` commits the rebuilt file, and CI fails when they differ.
Run with the Node.js version of `.nvmrc`, from the root of the repository.

```shell
npm ci --prefix player-js
npx --prefix player-js playwright install chromium          # once: the browser of the tests
npm run build --prefix player-js                            # type-check, then life-pixel.js
git diff --exit-code -- player-js/life-pixel.js             # the committed build is current
npm test --prefix player-js                                 # Playwright, on the fake player
npm run size --prefix player-js                             # at most 2,048 bytes gzipped
```

### Local stack

Start it:

```shell
cp .env.example .env
docker compose up -d --wait
```

Reset it, dropping all of its data:

```shell
docker compose down -v
```

### Server

`crates/server` is `life-pixel-server`. Run from the root of the repository, with the local stack
started: in development it reads `.env`, copied from `.env.example`, runs the migrations of
`crates/server/migrations/`, then serves on http://localhost:8460. `npm start --prefix frontend`
proxies `/api` and `/mcp` to it, so the app on http://localhost:4260 talks to this server.

```shell
cargo run -p life-pixel-server                        # serve: the public, metrics and admin listeners
cargo run -p life-pixel-server -- migrate             # the migrations, then exit
cargo run -p life-pixel-server -- healthcheck         # exit 0 when /healthz answers 200
cargo run -p life-pixel-server -- openapi             # the API's description, on stdout
cargo run -p life-pixel-server -- admin-openapi       # the internal admin API's, on stdout
```

The stack tests run the server's store, sweeper and migrations against the local stack, each on a
database of its own that it drops when it ends. They need the stack started and `.env` copied
from `.env.example`; `LP_TEST_DATABASE_URL` is the admin connection that creates the databases.

```shell
cargo test -p life-pixel-server --features stack-tests
cargo clippy -p life-pixel-server --all-targets --features stack-tests -- -D warnings
```

To try the server by hand, give it a throwaway database rather than `life_pixel`: `create` prints
its URL, to set as `LP_DATABASE_URL`; `drop` removes it, and refuses any other database.

```shell
cargo run -p life-pixel-server --features stack-tests -- test-database create
cargo run -p life-pixel-server --features stack-tests -- test-database drop <url>
```

A new migration is a file `crates/server/migrations/<UTC timestamp>_<topic>.sql`, compatible
with the version running before it: it adds, it never renames nor drops what that version reads.

A change to a route or to one of its types regenerates the API's description and its TypeScript
types, and the internal admin API's description, and commits them; CI's `api` job fails when
they differ.

```shell
npm run api:generate --prefix frontend
git diff --exit-code -- crates/server/openapi.json crates/server/admin-openapi.json \
  frontend/projects/shared/src/lib/api/schema.d.ts
```

### Admin server

`crates/admin-server` is `life-pixel-admin-server`: the admin console's API, which signs admins in
with a password and a TOTP code, relays the server's internal admin API, and reads the
monitoring sources. Run from the root of the repository, with the local stack started: it reads
the `LPA_` block of `.env`, runs the migrations of `crates/admin-server/migrations/` on its own
database, `life_pixel_admin`, then serves on http://localhost:8463, its metrics on :8464. It
relays to the server on http://localhost:8462; the monitoring sources stay empty locally, and
the console says so.

```shell
cargo run -p life-pixel-admin-server                  # serve
cargo run -p life-pixel-admin-server -- migrate       # the migrations, then exit
cargo run -p life-pixel-admin-server -- healthcheck   # exit 0 when /healthz answers 200
cargo run -p life-pixel-admin-server -- openapi       # the console API's description, on stdout
```

Admins exist only through the command line of the admin server's host. `create-admin` asks for
the password twice without echo — or reads one line of standard input with `--password-stdin` —
and prints the `otpauth://` URI to scan with an authenticator app; nothing shows the secret
again. `disable-admin` ends the admin's sessions and refuses its sign-ins.

```shell
cargo run -p life-pixel-admin-server -- create-admin ops@example.org
printf '%s\n' "$PASSWORD" | cargo run -p life-pixel-admin-server -- create-admin ops@example.org --password-stdin
cargo run -p life-pixel-admin-server -- disable-admin ops@example.org
```

The stack tests run on databases of their own, `lpa_test_…`, owned by `life_pixel_admin` and
created through `LP_TEST_DATABASE_URL`, never on `life_pixel_admin` itself. To try the admin
server by hand, give it a throwaway database as `LPA_DATABASE_URL`, as for the server:

```shell
cargo test -p life-pixel-admin-server --features stack-tests
cargo clippy -p life-pixel-admin-server --all-targets --features stack-tests -- -D warnings
cargo run -p life-pixel-admin-server --features stack-tests -- test-database create
cargo run -p life-pixel-admin-server --features stack-tests -- test-database drop <url>
```

Its description holds its own routes and the relayed ones of `crates/server/admin-openapi.json`,
moved under `/api/admin/v1`; `npm run api:generate --prefix frontend` regenerates it after the
server's, with the console's types, and CI's `api` job fails when they differ.

```shell
npm run api:generate --prefix frontend
git diff --exit-code -- crates/admin-server/openapi.json \
  frontend/projects/shared/src/lib/admin-api/schema.d.ts
```

### Images

`docker/app.Dockerfile` builds the `app` image — the server and the app —, and
`docker/admin.Dockerfile` the `admin` image — the admin server and the console —, both from the
root of the repository, which `.dockerignore` reduces to the paths they copy. Their Rust stage
runs Cargo with 4 jobs (`--build-arg CARGO_BUILD_JOBS=<n>` changes it); the app's installs the
`wasm-bindgen` of `Cargo.lock` unless `--build-arg WASM_BINDGEN_VERSION=<version>` names it.

```shell
docker build -f docker/app.Dockerfile -t ghcr.io/lindecker-charles/life-pixel/app:local .
docker build -f docker/admin.Dockerfile -t ghcr.io/lindecker-charles/life-pixel/admin:local .
```

`docker compose up` starts only the stack; the `app` profile adds both images, on the same
`.env`: `compose.override.yaml` replaces what differs inside a container — the images' listeners
and folders, the stack's addresses. The server answers on http://localhost:8460 and the console
on http://localhost:8463, as when they run from Cargo: stop those first.

```shell
docker compose --profile app up -d --wait            # builds the images when they are missing
docker compose --profile app up -d --wait --build    # rebuilds them
curl -f http://127.0.0.1:8460/healthz
docker compose --profile app rm -sf server admin     # the app's containers, the stack stays
```

The checks of the images, the Compose files and the deployment script, which CI's `docker` job
runs too:

```shell
docker run --rm -v "$PWD":/repo -w /repo hadolint/hadolint@sha256:32dac94127fd60b7b7e3fbfc65e1383b9b5e25c9bfd7b8536de7a539fe68a12d hadolint docker/app.Dockerfile docker/admin.Dockerfile
docker compose --env-file .env.example --profile app config --quiet
docker compose -f compose.yaml -f compose.deploy.yaml --env-file .env.staging.example config --quiet
docker compose -f compose.yaml -f compose.deploy.yaml --env-file .env.prod.example config --quiet
docker compose -f docker/selfhost/compose.yaml --env-file docker/selfhost/.env.example config --quiet
docker run --rm -v "$PWD":/repo -w /repo koalaman/shellcheck@sha256:bb596a0d169b85ddd81d8b6d3a2ff6d5baf5fca10b97f575ebc647c3dff62b3d scripts/deploy/deploy.sh
DEPLOY_DRY_RUN=1 DEPLOY_SHA=HEAD DEPLOY_PATH=/opt/life-pixel-staging \
  DEPLOY_ENV_FILE=.env.staging.example DEPLOY_REGISTRY_USER=ci DEPLOY_REGISTRY_TOKEN_FILE=/dev/null \
  scripts/deploy/deploy.sh
```

`scripts/deploy/deploy.sh` runs only on the VPS, from `_deploy.yml`; its dry run prints the
commands of the six steps. `docker/selfhost/` is the self-hosting example, with its own README.

### Desktop

`tauri/` is `life-pixel-desktop`: the app in a Tauri 2 window, with `service` in-process on a
local library folder. `cargo build` compiles it without the front-end: a debug binary loads the
app from `npm start` on http://localhost:4260. Run with the Node.js version of `.nvmrc`, from the
root of the repository, after `npm ci --prefix frontend`.

```shell
cargo test -p life-pixel-desktop            # every command on a temporary library, the watcher
cargo xtask build-sidecar                   # the life-pixel CLI, into tauri/binaries/ for the host
cargo xtask build-sidecar --target universal-apple-darwin   # both macOS architectures, joined
cargo xtask build-desktop --debug           # sidecar, front-end, then app: macOS .app, Linux .deb
cargo xtask build-desktop                   # the same, in release; unsigned, without an updater
cd tauri && cargo tauri dev                 # the app on the dev server, reloading as you edit
```

`build-desktop` never makes a DMG, whose creation drives the Finder; installers are release.yml's.
It runs `build-sidecar` first and passes `tauri/bundle.conf.json`, which ships the `life-pixel` CLI
beside the app's executable; `cargo build` and `cargo tauri dev` leave it out, and the app's
`settings/agents` page then says so. A target other than the host's needs
`rustup target add <triple>` — both macOS ones for `universal-apple-darwin`. The app watches its
library folder: what an agent writes through the CLI appears in its lists and open animation.
`LIFE_PIXEL_LIBRARY` chooses the library folder of a run, whatever the settings say, and a debug
build saves exports into `LP_EXPORT_DIR` without a dialog: set both to temporary folders when you
try the app, so that it leaves your library alone. The icons come from the pixel-art
`tauri/icons/source.png`, drawn at 32 × 32 and scaled 32 times: after changing it, run
`cargo tauri icon icons/source.png` in `tauri/`, then delete what it makes for mobile and the
Windows Store — `android/`, `ios/`, `Square*Logo.png`, `StoreLogo.png`.

### Desktop end to end

`tauri/tests/e2e/` drives the debug app through `tauri-driver` and WebKitGTK's WebDriver: the app
starts; a new animation drawn with the keyboard is saved into a new project, then changed and saved
again; the library lists it; a GIF export appears in `LP_EXPORT_DIR`; and the bundled
`life-pixel list --library` lists the same animation. Each run starts the app on its own temporary
library, export folder and settings. It finds elements by role and accessible name, as a person
does. It runs on Linux, as CI's `desktop` job does, with Tauri's system packages,
`webkit2gtk-driver` and `xvfb`, the Tauri CLI,
`cargo install tauri-driver --version 2.0.6 --locked` and `npm ci --prefix frontend`:

```shell
cd tauri/tests/e2e
npm ci
npm run lint              # tsc, ESLint and Prettier
npm run build             # the sidecar, then the debug app without a bundle
xvfb-run -a npm test      # the journey; a failure leaves a screenshot in test-results/
```

`tauri-driver` cannot drive macOS. There, run the suite in a throwaway Debian container. It
mounts the repository read-only and copies it, as the front-end build cannot write through the
mount, and keeps `target/` and every `node_modules/` on Docker volumes, so that the Linux build
never mixes with yours — `docker volume rm` them when you are done. `--init` lets `xvfb-run`
see its X server start:

```shell
docker run --rm --init --memory 7g -v "$PWD":/src:ro -w /repo \
  -v life-pixel-e2e-target:/repo/target \
  -v life-pixel-e2e-frontend-modules:/repo/frontend/node_modules \
  -v life-pixel-e2e-modules:/repo/tauri/tests/e2e/node_modules \
  -v life-pixel-e2e-rustup:/usr/local/rustup \
  -v life-pixel-e2e-registry:/usr/local/cargo/registry \
  -v life-pixel-e2e-tools:/opt/cargo-tools \
  -e CARGO_BUILD_JOBS=4 -e LP_ENGINE_PREBUILT=1 -e CI=true \
  rust:1.98-bookworm bash -euxo pipefail -c '
    tar -C /src --exclude=./target --exclude=node_modules --exclude=.angular \
      --exclude=./frontend/dist --exclude=./tauri/binaries --exclude=./tauri/gen \
      --exclude=./tauri/tests/e2e/test-results -cf - . | tar -C /repo -xf -
    apt-get update
    apt-get install -y --no-install-recommends libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
      librsvg2-dev webkit2gtk-driver xvfb xauth xz-utils
    arch=$(dpkg --print-architecture); if [ "$arch" = amd64 ]; then arch=x64; fi
    curl -fsSL "https://nodejs.org/dist/v24.21.0/node-v24.21.0-linux-$arch.tar.xz" | tar -xJ -C /opt
    export PATH="/opt/node-v24.21.0-linux-$arch/bin:/opt/cargo-tools/bin:$PATH"
    cargo install tauri-cli --version ^2 --locked --root /opt/cargo-tools
    cargo install tauri-driver --version 2.0.6 --locked --root /opt/cargo-tools
    npm ci --prefix frontend
    cd tauri/tests/e2e
    npm ci
    npm run build
    xvfb-run -a npm test'
```

A failure's screenshot stays in the container: add
`-v /tmp/life-pixel-e2e-results:/repo/tauri/tests/e2e/test-results` to keep it.

`LP_ENGINE_PREBUILT=1` expects the engine that `cargo xtask build-editor` wrote on your machine;
without it, the container needs `wasm-bindgen-cli` too.

#### The macOS app, by hand

Before a release, and after a change to `tauri/`, check the app on a Mac. Start it from a terminal
with `LIFE_PIXEL_LIBRARY` on a temporary folder, so that it leaves your library alone:

```shell
cargo xtask build-desktop
LIFE_PIXEL_LIBRARY="$(mktemp -d)" \
  "target/release/bundle/macos/Life Pixel.app/Contents/MacOS/life-pixel-desktop"
```

1. **Install**: the commands above start a local, unsigned build. For a release, take the DMG of a
   `release.yml` dry run and drag Life Pixel into Applications — a signed build opens without a
   Gatekeeper warning —, then start its `Contents/MacOS/life-pixel-desktop` the same way. The
   window opens on a new animation, in your system's language.
2. **Draw**: create a 16 × 16 animation; draw with the pointer, then with the keyboard alone —
   Tab to the canvas, `b`, the arrows and Enter —; add a frame and draw on it.
3. **Save**: ⌘S saves it into a new project; change it and save again, without a conflict; the
   library lists the project and the animation; quit, start again, and open it from the library.
4. **Export**: export a GIF: the system's save dialog opens and the file plays in Quick Look;
   export the PNG frames: the folder picker opens and receives every frame.
5. **CLI**: Settings, then Agents, shows the bundled CLI's path; run its
   `"<cliPath>" list --library "<libraryPath>"`: it lists the animation you saved.
6. **No updater**: a local build has none: with the network off, the app starts, saves and exports
   as before, and no update prompt appears, even after the ten seconds a release waits to check.

### CLI

`crates/cli` is `life-pixel-cli`, the `life-pixel` binary: `mcp`, `list` and `export` on a local
library folder, with `service` in-process, no server involved.

```shell
cargo run -p life-pixel-cli -- mcp --allow-dir <dir>   # serves crates/mcp's tools over stdio
cargo run -p life-pixel-cli -- list --json             # the library's animations
cargo run -p life-pixel-cli -- export <id> --format gif # writes its files, prints their paths
```

`--library` chooses the library folder of a command, then `LIFE_PIXEL_LIBRARY`, then the OS
documents folder: set one of the two to a temporary folder when you try the CLI by hand, so that
it leaves your own library alone. `mcp`'s `--allow-dir` widens where an `export` call started by a
model may write, beyond the working directory it already may; a symbolic link target is refused
outright, and an existing file needs the call's own `overwrite`. Messages are French when
`LC_ALL`, `LC_MESSAGES` or `LANG` starts with `fr`, English otherwise.

```shell
cargo test -p life-pixel-cli
cargo clippy -p life-pixel-cli --all-targets -- -D warnings
```

## Code conventions

The full conventions live in [AGENTS.md](../AGENTS.md) (identical to `CLAUDE.md`). The
essentials:

- **DRY, KISS, SOLID** as defaults; departing from them takes an explicit reason;
- **domain rules live in the Rust core**, never re-implemented in TypeScript;
- **an export is data, never code** — nothing is compiled or generated at export time;
- **one public element per file**, named after the file;
- **no magic number or string** — a named constant states the intent;
- **guard clauses** rather than nested `if/else`; **CQS** — a function changes state or returns
  a value, never both;
- idiomatic casing: `snake_case` / `PascalCase` in Rust, `camelCase` / `PascalCase` in
  TypeScript; booleans prefixed with `is`, `has`, `should`, `can`;
- no user-facing string outside the i18n catalogues, and every catalogue updated together.

| Rule | Limit |
|---|---|
| File size | ≤ 300 lines (warning), 400 maximum |
| Files per folder | ≤ 10 |
| Function size | ≤ 30 lines |
| Number of parameters | ≤ 3 |
| Nesting depth | ≤ 3 levels |
| Cyclomatic complexity | ≤ 10 per function |
| Line length | ≤ 100 characters |

Formatting is applied by tools and is not discussed in review: `cargo fmt` for Rust, Prettier
for the front-end. `cargo clippy -- -D warnings` must pass: a warning is a failure, not a detail
for later.

## Tests

**Every feature ships with its tests**, in the same branch and the same pull request: the
nominal behaviour and the edge cases the feature introduces — no more, no less. We test neither
the framework nor third-party libraries, and we do not chase a coverage percentage. A good test
fails when the **behaviour** breaks, not when the implementation changes. A bug fix comes with
the test that fails without it.

## Branches

A branch starts from `dev` and returns to it through a pull request. One branch = one topic.
`test` and `main` only move by promotion, never by hand — see
[devops.md](../docs/devops.md).

```text
type/short-description
```

- **type**: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `style`, `build`, `ci`, `chore` —
  always the short form, `feat/` and never `feature/`;
- **description**: `kebab-case`, without accents, two to five words.

Examples: `feat/onion-skin`, `fix/gif-palette-order`, `docs/mcp-tools`.

## Commits

**Conventional Commits, in English**:

```text
type(scope): description
```

- imperative mood, lowercase initial, no trailing period, **72 characters** at most;
- optional body, reserved for the *why*; a breaking change carries a `BREAKING CHANGE:` footer;
- **one commit = one coherent change** — never two topics in one commit.

The scope is mandatory when the map covers the modified files:

| Path | Scope |
|---|---|
| `crates/<name>/**` | the crate's directory name: `core`, `format`, `compiler`, `editor-wasm`, `service`, `mcp`, `server`, `admin-server`, `cli` |
| `crates/player/**`, `player-js/**` | `player` |
| `frontend/projects/app/**` | `app` |
| `frontend/projects/admin/**` | `admin` |
| `frontend/projects/shared/**` | `shared` |
| the rest of `frontend/**` | `frontend` |
| `i18n/**` | `i18n` |
| `tauri/**` | `tauri` |
| `docker/**`, `compose*.yaml` | `infra` |
| `.github/workflows/**`, `.github/dependabot.yml`, `.github/CODEOWNERS` | `ci` |
| `docs/**`, `README.md`, `AGENTS.md`, `CLAUDE.md`, the rest of `.github/**` | `docs` |
| `scripts/**` | `scripts` |
| `xtask/**` | `xtask` |
| `samples/**` | `samples` |
| configuration at the root | no scope: `build` for dependencies and toolchains, `chore` otherwise |

Tests travel with the code they test and take its scope.

```text
feat(compiler): append the payload as a custom section
fix(player): stop on the last frame of a non-looping tag
docs(docs): describe the export dialog
```

## Opening a pull request

1. Create the branch from `dev`, following the naming above.
2. Write the code, its tests, and update the documentation it affects.
3. Run the checks listed in [AGENTS.md](../AGENTS.md), under "Before declaring a change done".
4. Open the pull request **against `dev`** and fill in the template. Work in progress opens as a
   **draft**.

A good pull request explains **why** the change exists, what it changes for the person using
Life Pixel, and how to verify it. `Closes #123` closes the original issue on merge. Keep pull
requests small and focused: a large rename mixed with a fix cannot be reviewed — split them.

## Review

CI must be green before any review. What is looked at first:

- does the behaviour match what the pull request announces;
- do the tests fail without the change;
- do exports stay data, and does the player stay within its size budget;
- does a change to the export format bump its version and keep older exports playable;
- do the MIT parts stay free of AGPL code;
- are the size, naming and slicing conventions honoured;
- does the documentation follow the code.

Review comments are about the code, never about the person. A question in a review is a
question, not a reproach.
