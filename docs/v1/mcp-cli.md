# MCP and CLI — A2, A3, A4, A5

The tools of [mcp.md](../mcp.md), independent of their transport (A2); served by the hosted server
with personal access tokens (A3) and by the `life-pixel` CLI over stdio on the local library (A4).
Every tool maps to a use case of [service.md](service.md#a1--use-cases-for-agents): the same
validation, limits and quota as the interface.

## A2 — MCP tools

`crates/mcp`, over `rmcp` and `schemars`:

| Path | Content |
|---|---|
| `src/server.rs` | `LifePixelMcp`: the `rmcp` handler, holding `Library`, `AnimationEditing`, an `ExportDelivery` and a `CallerPolicy` |
| `src/tools/` | `library.rs`, `read.rs`, `editing.rs`, `preview.rs`, `export.rs`, `snippet.rs` |
| `src/resources.rs` | the resource template |
| `src/errors.rs` | errors as tool results |

```rust
/// Where export files go: signed links (A3) or a directory (A4).
#[async_trait]
pub trait ExportDelivery: Send + Sync {
    fn export_schema(&self) -> schemars::Schema; // the `export` tool's arguments differ by transport
    async fn deliver(&self, owner: &Owner, request: ExportCall, files: Vec<ExportFile>)
        -> Result<serde_json::Value, CodedError>;
}
/// Who calls, and whether a tool is allowed: a token's owner and scopes (A3), or `Owner::Local`.
pub trait CallerPolicy: Send + Sync {
    fn owner(&self, context: &rmcp::service::RequestContext<rmcp::RoleServer>) -> Result<Owner, CodedError>;
    fn allow(&self, context: &rmcp::service::RequestContext<rmcp::RoleServer>, scope: Scope)
        -> Result<(), CodedError>;
}
pub enum Scope { Read, Write, Export }
```

`CodedError` is service's ([service.md](service.md#owners-pages-errors)).

### Tools

Names, arguments (snake_case), descriptions and schemas are in English: they address the model.
Bounds in the schemas come from `core::limits`, never from literals. `id` is an animation id;
`frame` a position from 0; `layer_id` an id from `get_animation`, the top layer when omitted;
`index` a palette index, 0 being transparent.

| Tool | Scope | Arguments | Result |
|---|---|---|---|
| `list_animations` | read | `query?`, `project_id?`, `cursor?`, `limit?` (1 to 50, 20) | `{ animations: [{ id, title, project_id, width, height, frame_count, updated_at }], next_cursor }` |
| `create_animation` | write | `title`, `width`, `height`, `project_id` or `project_name` (created when missing), `palette?`, `frame_duration_ms?`, `layer_name?` | the animation's view |
| `get_animation` | read | `id`, `include_pixels?` (false), `frames?` | the view: title, size, `palette` as `#rrggbbaa`, `layers`, `frames` with `duration_ms`, `tags`; with pixels, `grids` of the composited frames |
| `set_palette` | write | `id`, `palette` | the view |
| `write_frame` | write | `id`, `frame`, `layer_id?`, `grid` | the view |
| `draw` | write | `id`, `frame`, `layer_id?`, `operations`: `{op: "pixel", x, y, index}`, `{op: "line", from, to, index}`, `{op: "rectangle", from, to, index, filled?}`, `{op: "fill", x, y, index}` | the view |
| `edit_frames` | write | `id`, `edits`: `{op: "add", position?, duration_ms?}`, `{op: "duplicate", frame}`, `{op: "delete", frame}`, `{op: "move", frame, position}`, `{op: "set_duration", frame, duration_ms}` | the frames and tags |
| `set_tags` | write | `id`, `tags`: `[{ name, first, last, loop: "loop" \| "once" }]` | the tags |
| `render_preview` | read | `id`, `frame?` (omitted: contact sheet), `scale?` | an image content (`image/png`) and a text content: size, scale and layout |
| `export` | export | `id`, `format`: `wasm`, `gif`, `apng`, `sprite_sheet`, `png_frames`; `tag?`, `scale?`; and the transport's own | what `ExportDelivery` returns |
| `get_embed_snippet` | read | `id`, `framework`: `html`, `angular`, `react`, `vue`; `src?`, `loader?`, `tag?`, `alt?` | `{ code, files: [{ name, placement }] }` |

- `from` and `to` are `[x, y]` pairs. Grids follow [core.md](core.md#text-grid), and the
  descriptions of `write_frame` and `get_animation` state the alphabet.
- The resource template `life-pixel://animations/{id}` reads the view without pixels, as JSON.
- A failure is a tool result with `isError: true` and one text content, the JSON
  `{ "code": "…", "params": { … } }`. Names and descriptions coming from users are returned as data
  and never interpreted.
- The server's name is given by its transport (`LP_MCP_SERVER_NAME` hosted, `life-pixel` locally),
  its version is the crate's.

**Tests** through an in-memory transport (`tokio::io::duplex` and an `rmcp` client), on
`InMemoryLibraryStore`: every tool's success and main errors; schema bounds matching `core`; paging;
the preview's image content; a delivery double for `export`; the resource; scopes refused through a
policy double.

## A3 — Hosted MCP endpoint

### Endpoint

- `rmcp`'s Streamable HTTP service mounted on `/mcp` of the public router, behind a layer that
  requires `Authorization: Bearer lp_pat_…`: unknown, revoked or expired tokens answer
  `401 token.invalid` with `WWW-Authenticate: Bearer`; a suspended account `403
  auth.account_suspended`. The rate limit `mcp` applies per token.
- The `CallerPolicy` gives the token's account as owner and checks the tool's scope
  (`token.scope`, with `required`).
- **Daily ceiling**: each tool call adds one to `mcp_usage` for the account and the UTC day; beyond
  the plan's `LP_PLAN_FREE_MCP_CALLS_PER_DAY`, the call fails with `mcp.daily_limit` (`limit`,
  `resetsAt`).
- `export` delivers **signed links**: `GET /api/v1/exports/{link}`, where `link` is the base64url of
  a JSON object — animation `a`, version `v`, format `f`, tag `t`, scale `s`, file name `n`, expiry
  `e` —, a dot, and the base64url of its HMAC-SHA256 with `LP_EXPORT_LINK_SECRET`. A link lives 15
  minutes; the file is compiled on demand from that version and sent with
  `Content-Disposition: attachment`; a link tampered with, expired, or whose animation has changed
  answers `410 export.link_invalid`. Exports are never stored.
- Metrics: `mcp_tool_calls_total{tool, outcome}`, `mcp_tool_duration_seconds{tool}`,
  `exports_total{format, outcome}`, `export_duration_seconds{format}`,
  `export_size_bytes{format}`; each call records `mcp_tool_called`.

### Personal access tokens

```sql
create table access_tokens (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  name text not null,
  token_hash bytea not null unique,     -- SHA-256
  prefix text not null,                 -- "lp_pat_" and 4 characters, for display
  scopes text[] not null,
  created_at timestamptz not null,
  expires_at timestamptz not null,
  last_used_at timestamptz,             -- updated at most once a minute
  revoked_at timestamptz
);
create table mcp_usage (
  account_id uuid not null references accounts (id) on delete cascade,
  day date not null,
  calls integer not null,
  primary key (account_id, day)
);
```

| Route | Body | Answer |
|---|---|---|
| `GET /api/v1/tokens` | — | `200` `[{ id, name, prefix, scopes, createdAt, expiresAt, lastUsedAt }]` |
| `POST /api/v1/tokens` | `{ name, scopes, expiresInDays }` | `201` the record, `token` — shown this once — and `mcp: { serverName, url }` |
| `DELETE /api/v1/tokens/{id}` | — | `204`, revoked |

- A token is `lp_pat_` followed by 32 random bytes in base64url. `scopes` among `read`, `write`,
  `export`; `expiresInDays` one of `TOKEN_EXPIRY_DAYS`; at most `MAX_ACTIVE_TOKENS` active tokens.
- Codes: `token.invalid` (401), `token.scope` (`required`; 403), `token.not_found` (404),
  `token.name` (`max`; 422), `token.expiry` (`allowed`; 422), `token.limit` (`max`; 409),
  `mcp.daily_limit` (`limit`, `resetsAt`; 429), `export.link_invalid` (410).
- The internal admin API's user detail gains the user's tokens, without their hash.

### App

`frontend/projects/app/src/app/tokens/`, route `settings/tokens`, hosted only through T2's
`hostedOnly` guard: the list, with name, prefix,
scopes, dates and last use; a creation dialog — name, scopes (`read` and `write` checked), expiry
(90 days by default) — that shows the token once, with a copy button and the command to register
it, built from the answer's `mcp`: `claude mcp add --transport http <serverName> <url> --header
"Authorization: Bearer <token>"` — so that staging and production get distinct names in one
client; revocation with a confirmation. **Keys**: `tokens.`.

**Tests** (`stack-tests`): authentication failures; scopes; the rate limit; the daily ceiling and
its reset at midnight UTC; token creation, listing, revocation, expiry; links — valid, tampered,
expired, stale —; the metrics; the tokens page with a mocked client, and axe.

## A4 — CLI

`crates/cli`, package `life-pixel-cli`, binary `life-pixel` (`clap`):

| Command | Does |
|---|---|
| `life-pixel mcp [--library <dir>] [--allow-dir <dir>]…` | the MCP server over stdio on the local library |
| `life-pixel list [--library <dir>] [--project <id>] [--query <text>] [--json]` | animations, as a table or JSON |
| `life-pixel export <id> --format <wasm\|gif\|apng\|sprite_sheet\|png_frames> [--out <dir>] [--tag <name>] [--scale <n>] [--overwrite] [--library <dir>]` | writes the files, prints their paths |
| `life-pixel --version` | |

- **Library**: `--library`, else `LIFE_PIXEL_LIBRARY`, else `service::local::default_library_path`
  with what `dirs` finds — the desktop app's default.
- **Local writes**: `export` over MCP takes `directory` (relative to the working directory, which is
  the default) and `overwrite` (false). The directory, canonicalized, must lie inside an allowed
  one — the working directory and each `--allow-dir` —, else `export.directory_not_allowed`; a file
  that is a symbolic link, or a path that leaves through one, is refused (`export.symlink`); an
  existing file needs `overwrite: true` (`export.file_exists`). Files are created with
  `create_new` unless overwriting.
- **Output**: in `mcp` mode, stdout carries the protocol only and logs go to stderr. Elsewhere,
  `println!` prints results; messages come from the catalogues — `cli.` and `errors.` keys, embedded
  at build time with `include_str!` —, in French when `LC_ALL`, `LC_MESSAGES` or `LANG` starts with
  `fr`, in English otherwise. Exit code 0, or 1 with the message on stderr.

**Tests**: `list` and `export` on a temporary library, in both output forms; allowed directories,
symbolic links (Unix) and `overwrite`; `--library` and `LIFE_PIXEL_LIBRARY`; `mcp` answering
`initialize` and `tools/list` over stdio; messages in French and English.

## A5 — MCP end to end

One scripted session, played on each transport by an `rmcp` client:

1. `create_animation` 16 × 16 in a new project; `set_palette` with 6 colours;
2. `write_frame` a grid on frame 0; `draw` a line and a fill;
3. `edit_frames`: add a frame, set both durations; `write_frame` on frame 1;
4. `set_tags`: `blink`, frame 1, played once;
5. `render_preview` of frame 0 and of the contact sheet — the PNGs decode to the announced sizes;
6. `get_animation` with pixels returns the grids written, composited;
7. `export` as WASM; `get_embed_snippet` for HTML contains `<life-pixel`;
8. the exported `.wasm` runs in `wasmi`: `load` succeeds, the tag is `blink`, and frame 0's
   framebuffer matches `core::render::rgba`.

- **stdio**: `crates/cli/tests/mcp_stdio.rs` starts `life-pixel mcp --library <temp> --allow-dir
  <temp>/out` as a child process; the export lands in `out/`.
- **HTTP**: `crates/server/tests/mcp_http.rs` (`stack-tests`) starts the server in-process on a free
  port with a test database and storage, creates an account and a token, plays the session, then
  downloads the export through its links; it also checks a missing scope and the daily ceiling.
- CI's `mcp` job runs both. A passing A5 means M4 is done.
