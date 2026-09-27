# MCP troubleshooting

Known problems met when an agent drives the MCP server of [mcp.md](mcp.md), what was checked, and
how to work around them. Each entry records the date it was observed: re-check it against the
current client and server before acting on it.

## Claude Code connects but lists no tool

Observed on 2026-09-27, local stack (`http://localhost:8460/mcp`, server `life-pixel-dev` 0.1.0),
Claude Code on Windows.

### Symptom

The server shows up in Claude Code, its `instructions` reach the agent, but none of its tools do.
`claude mcp list` reports:

```text
life-pixel-dev: http://localhost:8460/mcp (HTTP) - ! Connected · tools fetch failed —
Invalid result for tools/list: [
  { "expected": "number", "code": "invalid_type", "path": [ "ttlMs" ], "message": "Invalid input" },
  { "code": "invalid_value", "values": [ "public", "private" ], "path": [ "cacheScope" ],
    "message": "Invalid input" }
]
```

The agent then has no `list_animations`, `write_frame`, `export`…: the session cannot use Life
Pixel at all.

### What was checked

The server answers correctly when it is called directly, with the same token as the client:

| Request | Result |
|---|---|
| `initialize`, `protocolVersion: 2025-06-18` | accepted, `2025-06-18` echoed, capabilities `tools` and `resources` |
| `initialize`, `protocolVersion: 2025-11-25` | accepted, `2025-11-25` echoed |
| `initialize`, `protocolVersion: 2026-06-18` | negotiated down to `2025-11-25`, as the specification expects |
| `tools/list` (every version above) | `{"tools": [...]}`: the 11 tools of [mcp.md](mcp.md), each with `name`, `description`, `inputSchema` and `annotations` |
| `tools/call` on every tool | works: an animation was created, drawn, tagged and previewed end to end |

Neither `ttlMs` nor `cacheScope` appears anywhere in the repository or in the server's answer.

### Diagnosis

The rejection happens in the client, while it validates the `tools/list` result: that version of
Claude Code expects a cache hint (`ttlMs`, a number, and `cacheScope`, `public` or `private`) that
the MCP revisions the server implements do not define. The server's answer is conforming; the
server was not changed.

Not established: whether those fields come from a newer MCP revision or draft that the server
should adopt, or from a client-side regression. Settle that before adding fields to the server's
answer — an unspecified field is a contract we would then have to keep.

### Workaround

Drive the endpoint over plain JSON-RPC, outside the client's tool loading:

1. `POST /mcp` an `initialize` request, with `Authorization: Bearer <token>`,
   `Content-Type: application/json` and `Accept: application/json, text/event-stream`.
2. Send the `Mcp-Session-Id` the answer returns on every later request, if there is one.
3. Call tools with `tools/call`, `{"name": "<tool>", "arguments": {...}}`. The answer may come as
   a server-sent event: read the JSON after `data: `.

Read the token from the client's configuration rather than pasting it into a command line or a
file of the repository (see [security-model.md](security-model.md)).

## `export` fails with `token.scope`

Observed on 2026-09-27, same setup.

### Symptom

Every tool works except `export`, which answers:

```json
{"code": "token.scope", "params": {"required": "export"}}
```

`get_embed_snippet` still answers, but the `.wasm` it points to is never produced.

### Cause

Tokens are scoped `read`, `write` and `export` ([mcp.md](mcp.md#limits-and-safety)). The token
registered in Claude Code was created without `export`; the server's policy
(`crates/server/src/mcp/policy.rs`) refuses the call, as it should.

### Fix

A scope cannot be added to an existing token:

1. In the app, **Settings → Access tokens** (`/settings/tokens`), create a token with the `read`,
   `write` and `export` scopes. It is shown once.
2. Register it again in Claude Code:
   `claude mcp remove life-pixel-dev` then
   `claude mcp add --transport http life-pixel-dev http://localhost:8460/mcp --header "Authorization: Bearer <token>"`.
3. Revoke the old token from the same page.
