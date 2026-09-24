# Service — H1, H2, A1

`crates/service` holds the use cases and the ports they need. It knows nothing of HTTP, Tauri or
MCP (invariant 6): the server, the CLI and the desktop app call it with their own adapters. Its
async functions run on tokio; parsing, compiling and rendering go through `spawn_blocking`.

## H1 — Service contract

### Layout

| Path | Content |
|---|---|
| `src/owner.rs` | `Owner` |
| `src/ids.rs` | `ProjectId`, `AnimationId`, `AccountId`: UUID newtypes |
| `src/paging.rs` | `PageRequest`, `Page<T>`, `Cursor` |
| `src/error.rs` | the `Coded` trait and `CODES`, the service's codes |
| `src/ports/` | `clock.rs`, `ids.rs`, `events.rs`, `library_store.rs` |
| `src/library/` | the `Library` use cases, one file per group |
| `src/memory/` | in-memory adapters — feature `testing` |
| `src/testing/` | the contract suites — feature `testing` |

Later modules: `local/` (H2), `animation/` (A1), `accounts/` (H5), `support/` (H9),
`tokens/` (A3). Dependencies: `core`, `tokio`, `async-trait`, `uuid` (v7), `time`, `bytes`,
`serde`, `thiserror`, `tracing`; A1 adds `compiler`, which C2 creates in H1's wave.

### Owners, pages, errors

```rust
pub enum Owner { Account(AccountId), Local }

pub struct PageRequest { pub cursor: Option<Cursor>, pub limit: u16 }
pub struct Page<T> { pub items: Vec<T>, pub next_cursor: Option<Cursor> }
/// Lists run from the most recently updated; the cursor is the last item seen.
pub struct Cursor { pub updated_at: OffsetDateTime, pub id: Uuid }

pub trait Coded {
    fn code(&self) -> &'static str;
    fn params(&self) -> serde_json::Map<String, serde_json::Value>; // camelCase keys
}
/// Any coded error, once its origin no longer matters: what MCP and the desktop return.
pub struct CodedError { pub code: &'static str, pub params: serde_json::Map<String, serde_json::Value> }
```

`Cursor` travels as the base64url of `{"u": <RFC 3339>, "i": <uuid>}`; a cursor that does not
decode is `request.malformed`. `PageRequest` bounds `limit` with `PAGE_SIZE_MAX` and defaults it to
`PAGE_SIZE_DEFAULT`.

### Ports

```rust
pub trait Clock: Send + Sync { fn now(&self) -> OffsetDateTime; }
pub trait IdGenerator: Send + Sync { fn new_id(&self) -> Uuid; } // UUIDv7
/// Records a product event; never blocks and never fails the caller.
pub trait EventSink: Send + Sync { fn record(&self, event: ProductEvent); }

#[async_trait]
pub trait LibraryStore: Send + Sync {
    async fn create_project(&self, owner: &Owner, project: ProjectRecord) -> Result<(), StoreError>;
    async fn get_project(&self, owner: &Owner, id: ProjectId) -> Result<ProjectRecord, StoreError>;
    async fn list_projects(&self, owner: &Owner, page: PageRequest)
        -> Result<Page<ProjectRecord>, StoreError>;
    async fn rename_project(&self, owner: &Owner, id: ProjectId, name: Name, at: OffsetDateTime)
        -> Result<ProjectRecord, StoreError>;
    async fn delete_project(&self, owner: &Owner, id: ProjectId) -> Result<(), StoreError>;
    async fn create_animation(&self, owner: &Owner, new: NewAnimationRecord, quota: Option<u64>)
        -> Result<AnimationRecord, StoreError>;
    async fn get_animation(&self, owner: &Owner, id: AnimationId)
        -> Result<AnimationRecord, StoreError>;
    async fn list_animations(&self, owner: &Owner, filter: AnimationFilter, page: PageRequest)
        -> Result<Page<AnimationRecord>, StoreError>;
    async fn read_document(&self, owner: &Owner, id: AnimationId)
        -> Result<(AnimationRecord, Bytes), StoreError>;
    async fn write_document(&self, owner: &Owner, write: DocumentWrite, quota: Option<u64>)
        -> Result<AnimationRecord, StoreError>;
    async fn move_animation(&self, owner: &Owner, id: AnimationId, to: ProjectId, at: OffsetDateTime)
        -> Result<AnimationRecord, StoreError>;
    async fn delete_animation(&self, owner: &Owner, id: AnimationId) -> Result<(), StoreError>;
    async fn usage(&self, owner: &Owner) -> Result<u64, StoreError>;
    async fn delete_everything(&self, owner: &Owner) -> Result<(), StoreError>;
}

pub struct ProjectRecord { pub id: ProjectId, pub name: Name, pub animation_count: u32,
    pub created_at: OffsetDateTime, pub updated_at: OffsetDateTime }
pub struct AnimationMeta { pub title: Name, pub width: u16, pub height: u16, pub frame_count: u16 }
pub struct NewAnimationRecord { pub id: AnimationId, pub project: ProjectId, pub meta: AnimationMeta,
    pub document: Bytes, pub at: OffsetDateTime }
pub struct AnimationRecord { pub id: AnimationId, pub project: ProjectId, pub meta: AnimationMeta,
    pub document_bytes: u64, pub version: u64,
    pub created_at: OffsetDateTime, pub updated_at: OffsetDateTime }
pub struct DocumentWrite { pub id: AnimationId, pub expected_version: u64, pub meta: AnimationMeta,
    pub document: Bytes, pub at: OffsetDateTime }
pub struct AnimationFilter { pub project: Option<ProjectId>, pub query: Option<String> }
pub enum StoreError {
    ProjectNotFound, AnimationNotFound,
    VersionConflict { current: u64 },
    QuotaExceeded { used: u64, limit: u64, requested: u64 },
    Unavailable(String), // logged, never shown
}
```

- One port for the library, because its operations are atomic across the index and the bodies: a
  write checks the version and the quota and updates the usage as one change.
- **Quota rule**: a create or a write is refused when it would take the owner's usage above the
  quota *and* above the current usage, so that a write that shrinks a document always passes and
  deleting always works. `quota: None` — the local library — means no quota.
- **Search**: `query` matches titles, case-insensitively, as a substring.
- An owner never sees another owner's projects or animations: they answer not found.

### Library use cases

`Library` holds `Arc<dyn LibraryStore>`, `Arc<dyn Clock>`, `Arc<dyn IdGenerator>`,
`Arc<dyn EventSink>` and `Plans { free_storage_bytes: u64, free_mcp_calls_per_day: u32 }`, the
plan values of configuration; an `Owner::Account` gets the free quota, `Owner::Local` none.

| Use case | Rules |
|---|---|
| `create_project(owner, name)`, `rename_project`, `delete_project` | names checked by `core`; deleting a project deletes its animations |
| `duplicate_project(owner, id, name)` | the whole project's size checked against the quota first, then copied |
| `list_projects(owner, page)`, `list_animations(owner, filter, page)` | |
| `create_animation(owner, project, spec)` | a blank animation from `core::Animation::new`, for MCP |
| `import_animation(owner, project, document)` | from serialized bytes: the app's first save, the desktop |
| `open_document(owner, id)` | the record and the bytes |
| `save_document(owner, id, expected_version, document)` | size against `MAX_DOCUMENT_BYTES`, then parsed and validated by `core` before anything is written |
| `rename_animation(owner, id, expected_version, title)` | the title lives in the document: a save |
| `move_animation`, `duplicate_animation(owner, id, title, project)`, `delete_animation` | a duplicate gets new ids |
| `usage(owner)` | `Usage { used_bytes, limit_bytes }` |
| `delete_everything(owner)` | for account deletion |

Events recorded: `animation_created`, `document_saved` with a size class, `quota_rejected`.

### Product events

```rust
/// A pseudonymous product event: a name, the account it concerns, flat properties.
pub struct ProductEvent {
    pub name: &'static str,
    pub account: Option<AccountId>,
    pub properties: Vec<(&'static str, String)>,
}
```

Events are data rather than an enum, so that each task adds its own without touching a shared
type. The names and properties are fixed here, and nowhere else:

| Name | Properties | Recorded by |
|---|---|---|
| `animation_created` | — | H1 |
| `document_saved` | `size` | H1 |
| `quota_rejected` | — | H1 |
| `signed_up`, `signed_in` | — | H5 |
| `account_deleted` | — | H6 |
| `export_completed` | `format`, `size`, `source` (`app` or `mcp`) | A1 (`mcp`); H13 (`app`, from the app) |
| `mcp_tool_called` | `tool`, `outcome` (`ok` or `error`) | A3 |
| `support_request_created` | `category` | H9 |

`size` is a class: `lt_1k`, `lt_10k`, `lt_100k`, `lt_1m`, `ge_1m`; `format` one of `wasm`, `gif`,
`apng`, `sprite_sheet`, `png_frames`. The local adapters record nothing: the desktop and the CLI
send no telemetry.

### Codes

Every error the product can show has a stable code, and one key `errors.<code>` in every catalogue,
added in the same commit as the code by the task that creates it. The codes are the constants of
their crate — `core::error::CODES`, `service::error::CODES`,
`life-pixel-server`'s and `life-pixel-admin-server`'s own —, and each server has a test that fails
when a code lacks its key. The HTTP status is the server's mapping
([server.md](server.md#problems)).

| Code | Params | Status | From |
|---|---|---|---|
| `document.malformed`, `document.reference`, `document.cel` | — | 422 | core |
| `document.unsupported_version` | `version` | 422 | core |
| `document.too_large` | `maxBytes` | 413 | core |
| `document.canvas_size`, `document.frame_duration` | `min`, `max` | 422 | core |
| `document.frame_count`, `document.layer_count`, `document.tag_count`, `document.pixel_budget`, `document.palette`, `document.name` | `max` | 422 | core |
| `document.tag`, `edit.tag_not_found`, `export.tag_not_found` | `name` | 422 | core, compiler |
| `grid.size` | `width`, `height` | 422 | core |
| `grid.character`, `grid.index` | `row`, `column` (and `index`) | 422 | core |
| `edit.out_of_canvas`, `edit.last_layer`, `edit.last_frame`, `edit.layer_not_found`, `edit.frame_not_found`, `import.image_malformed`, `import.sheet_grid` | — | 422 | core |
| `edit.palette_full` | `max` | 422 | core |
| `import.image_too_large` | `maxSide`, `maxBytes` | 422 | core |
| `export.scale` | `min`, `max` | 422 | compiler |
| `export.too_large`, `preview.too_large` | `maxSide` | 422 | compiler, service |
| `draw.too_many_operations` | `max` | 422 | service |
| `edit.palette_in_use` | `index` | 422 | service |
| `library.project_not_found`, `library.animation_not_found` | — | 404 | service |
| `document.version_conflict` | `current` | 412 | service |
| `quota.storage_exceeded` | `used`, `limit`, `requested` | 409 | service |
| `service.unavailable` | — | 503 | service |
| `library.unsupported_version`, `library.unavailable` | `version`; `path` | — | local library |
| `export.directory_not_allowed`, `export.file_exists`, `export.symlink` | `path` | — | CLI |
| `auth.*`, `account.*` | [accounts.md](accounts.md#codes) | | H5, H6 |
| `support.*` | [support-admin.md](support-admin.md#h9--support-requests) | | H9 |
| `token.*`, `mcp.*`, `export.link_invalid` | [mcp-cli.md](mcp-cli.md#a3--hosted-mcp-endpoint) | | A3 |
| `request.*`, `rate_limit.exceeded`, `client.update_required`, `document.version_required`, `internal.error` | [server.md](server.md#problems) | | H3 |

### In-memory adapters and contract suites

Feature `testing`: `memory::InMemoryLibraryStore`, `FixedClock`, `SequentialIds`,
`RecordingEvents`; and `testing::library_store_contract`, a macro that expands into one
`#[tokio::test]` per case for the adapter it is given:

- projects: create, get, list, rename, delete, not found, and invisible to another owner;
- animations: create from a document, read it back byte for byte, list by project, search
  case-insensitively, 120 items paged by 50 in a stable order without gap or duplicate;
- writes: the right version gives a new version, different from the one it replaced —
  `version + 1` in Postgres, a hash locally —; the wrong one `VersionConflict` with the current
  version; two concurrent writes on one version — exactly one wins;
- quota: a create and a write beyond it refused with the exact numbers; a shrinking write accepted
  above it; usage after create, write and delete; deleting a project frees its animations' usage;
- move, delete, `delete_everything`.

**Tests**: every use case against the in-memory adapters, with each error and its code; the
contract suite against `InMemoryLibraryStore`; `CODES` without duplicates.

## H2 — Local library

`service::local::LocalLibrary` implements `LibraryStore` on a folder, for the CLI and the desktop
app, with `Owner::Local` only.

```text
<library>/
  library.json                                   {"format": "life-pixel/library", "version": 1}
  .lock                                          advisory lock, held while writing
  projects/<project-id>/project.json             {"id", "name", "createdAt", "updatedAt"}
  projects/<project-id>/animations/<id>.json     the document, as core serializes it
```

- Opening a folder without `library.json` creates it; a higher version fails with
  `library.unsupported_version`; a missing or unwritable folder with `library.unavailable`.
- **Writes** go to a temporary file beside the target, are flushed, then renamed over it, under the
  `.lock` file lock (`fd-lock`): a reader never sees half a document, and the desktop app, the CLI
  and an agent can share a library.
- **Versions**: an animation's version is the first 6 bytes of the SHA-256 of its file, read as an
  integer — below 2^53, so that a JavaScript number carries it exactly across Tauri's IPC. A write
  expecting another version — the file changed on disk — is a `VersionConflict`.
- **Records**: `updated_at` is the file's modification time, `created_at` its creation time where
  the file system has one, else the modification time; titles and sizes come from parsing the
  document, cached by path, modification time and length.
- Files it does not know are ignored; a document that does not parse is left out of lists, with a
  warning in the log.
- `local::default_library_path(documents: Option<PathBuf>, home: PathBuf) -> PathBuf` returns
  `<documents>/Life Pixel`, or `<home>/Life Pixel` without a documents folder; the CLI and the
  desktop app pass what `dirs` finds.

**Tests**: H1's contract suite on a temporary folder; a file changed behind the adapter's back
gives a conflict; a crash between the temporary file and the rename leaves the old document whole;
unknown and unreadable files; creation and version of `library.json`.

## A1 — Use cases for agents

`service::animation::AnimationEditing` holds `Library` and serves the tools of
[mcp.md](../mcp.md). Frames are addressed by position, from 0; layers by the ids that `describe`
returns; `layer` omitted means the top layer.

| Use case | Arguments | Result |
|---|---|---|
| `describe` | `id`, `pixels: Option<Vec<u16>>` | `AnimationView`: title, size, palette, layers, frames with durations, tags; with `pixels`, those frames' composites as text grids |
| `set_palette` | `id`, `colors` | the view; entry 0 stays transparent, and a colour still used cannot be removed (`edit.palette_in_use`) |
| `write_frame` | `id`, `frame`, `layer`, `grid` | the view; the grid replaces the layer's cel on that frame |
| `draw` | `id`, `frame`, `layer`, `operations` | `pixel`, `line`, `rectangle`, `fill` as `core` operations; at most `DRAW_MAX_OPERATIONS` |
| `edit_frames` | `id`, `edits` | `add`, `duplicate`, `delete`, `move`, `set_duration`, applied in order |
| `set_tags` | `id`, `tags` | `replaceTags` |
| `preview` | `id`, `frame: Option<u16>`, `scale: Option<u8>` | a PNG of one frame, or a contact sheet of all, and its layout |
| `export` | `id`, `format`, `tag`, `scale` | `Vec<ExportFile>`; WASM gives `<stem>.wasm` and `life-pixel.js` |
| `snippet` | `id`, `framework`, `src`, `loader`, `tag`, `alt` | the code from `compiler::render_snippet`, and where each file goes, in English |

- A change reads the document, applies the operations with `core` on a blocking thread, and saves
  with the version it read; on a conflict, it reads again and retries once.
- **Previews**: without `scale`, the largest integer up to 8 that keeps the longest side at most 512
  pixels; the image never exceeds `PREVIEW_MAX_SIDE` nor `PREVIEW_MAX_BYTES`
  (`preview.too_large`). A contact sheet lays the frames out row by row in `ceil(√n)` columns with
  a 2-pixel transparent gap, scaled; its layout says which frame is where.
- Records `export_completed` with the source `mcp`.

**Tests**: each use case on the in-memory adapters — a text grid written then described back, each
drawing operation, frame edits and their effect on tags, previews at the automatic scale and at the
caps, each export format, snippets for every framework —, and a conflict retried once.
