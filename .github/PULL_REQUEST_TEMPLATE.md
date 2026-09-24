# Pull request

## Why

<!-- The problem being solved, not the list of files touched. Link the issue if there is one:
     `Closes #123`. -->

## What changes for the person using Life Pixel

<!-- What becomes visible, different or impossible — in the editor, an export, the MCP tools or
     the API. "No observable change" is a valid answer for a refactoring. -->

## How to verify it

<!-- The exact path to observe the result: command, screen, test that fails without the
     change. -->

## Type of change

- [ ] `feat` — new feature
- [ ] `fix` — bug fix
- [ ] `docs` — documentation only
- [ ] `refactor` — rework with no behaviour change
- [ ] `perf` — performance
- [ ] `test` — tests only
- [ ] `build` / `ci` / `chore` — tooling and maintenance
- [ ] Breaking change (`BREAKING CHANGE:` present in a commit)

## Checks

- [ ] The pull request targets `dev`; the branch follows `type/short-description` and covers a
      single topic.
- [ ] Commits follow Conventional Commits in English, with the repository's scopes.
- [ ] The change is covered by tests shipped here, which fail without it.
- [ ] Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` and `cargo deny check`
      pass.
- [ ] Front-end: lint, format check, unit tests and build pass.
- [ ] Every new user-facing string exists in every i18n catalogue.
- [ ] The size, naming and slicing limits of `AGENTS.md` are honoured.
- [ ] The documentation follows the code (`docs/`, and `AGENTS.md` with `CLAUDE.md` together).

## Product boundaries

- [ ] Exports stay data: nothing is compiled or generated at export time.
- [ ] The player stays within its size budget, or the change is discussed here.
- [ ] A change to the export format bumps its version and keeps older exports playable.
- [ ] `crates/format`, `crates/player` and `player-js` (MIT) depend on no AGPL code.
- [ ] No third-party tracker, and nothing sent from the desktop app or a self-hosted server
      without the user's opt-in.
- [ ] No secret, token or personal data in code, tests, fixtures or logs.

<!-- If a box cannot be ticked, explain why below. -->

## Additional notes

<!-- Screenshots for a visual change, measurements for a performance or size change, points to
     watch during review. -->
