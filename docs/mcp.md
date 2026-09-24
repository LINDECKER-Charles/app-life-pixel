# MCP server

The MCP server turns Life Pixel into a tool an AI agent can use from the developer's own
environment: describe an animation, let the agent draw it, look at it, export it and wire it into
the codebase. Claude is the first client we target; any MCP client works.

## Where it runs

| Distribution | Transport | Authentication | Registering it in Claude Code |
|---|---|---|---|
| Hosted | Streamable HTTP, `https://<api host>/mcp` | personal access token; OAuth 2.1 next | `claude mcp add --transport http life-pixel https://<api host>/mcp --header "Authorization: Bearer <token>"` |
| Self-hosted | the same, on the user's host | the same | the same, with the user's host |
| Desktop and CLI | stdio, `life-pixel mcp` | none: the local user | `claude mcp add life-pixel -- life-pixel mcp` |

- The staging endpoint announces itself as `life-pixel-staging`, so that both environments can
  sit side by side in a client configuration.
- claude.ai and Claude Desktop connectors authenticate remote servers with OAuth: OAuth 2.1, as the
  MCP authorization specification describes it, comes right after token authentication (see
  [product.md](product.md)). Claude Desktop can also launch the local server over stdio.
- The local server works on the desktop app's library folder; the app picks up the changes an
  agent makes.

## Tools

Design rules:

- **Few, composable tools.** Each maps to a `service` use case: the same validation, limits,
  quotas and permissions as the interface.
- **Small, structured answers.** No pixel dump unless asked; lists are paginated.
- **Errors carry the API's stable codes**, so that an agent can react to `quota.storage_exceeded`
  as precisely as the interface does.

| Tool | Purpose |
|---|---|
| `list_animations` | find existing animations: search, pagination |
| `create_animation` | a new document: size, palette, default frame duration |
| `get_animation` | metadata, palette, frames and tags — without pixel data by default |
| `set_palette` | replace or edit the palette |
| `write_frame` | set a frame's pixels from a text grid, one character per palette entry — the representation language models handle best |
| `draw` | a batch of drawing operations (pixel, line, rectangle, fill) on a frame and a layer |
| `edit_frames` | add, duplicate, delete and reorder frames; set their durations |
| `set_tags` | define the named frame ranges and their loop mode |
| `render_preview` | a scaled PNG of a frame, or a contact sheet of all frames, so the agent sees what it drew and iterates |
| `export` | hosted: short-lived download links; local: writes the files into a directory of the user's project |
| `get_embed_snippet` | the integration code for HTML, Angular, React or Vue, and where each file goes |

Resource: `life-pixel://animations/{id}`, a read-only JSON view of an animation.

## Limits and safety

- **Tokens** are scoped (`read`, `write`, `export`), shown once, stored hashed, revocable and
  expiring; their last use is visible to the user.
- **Rate limits** apply per token, and the storage quota applies exactly as in the interface.
- **Previews** are capped in dimensions and bytes.
- **Local writes** stay inside the directories the user allowed, never follow a symbolic link out
  of them, and never overwrite a file unless the call says `overwrite: true`.
- **User content is data.** Names and descriptions returned to the agent come from users; no tool
  ever interprets them as instructions.
- **Every call is measured and audited**: tool, outcome and duration — never the pixels. See
  [admin-console.md](admin-console.md).

## Language

Tool names, descriptions and schemas are in English: they address the model, not the user. What
the user reads comes from the i18n catalogues.
