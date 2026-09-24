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
| Docker | recent | local Postgres and object storage, image checks |

The Tauri apps need the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) of
your operating system. For Android, add Android Studio with its SDK and NDK, a JDK, and the Rust
Android targets:

```shell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

The development loop — which commands start what — is written down with the first code.

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
