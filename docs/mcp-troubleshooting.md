# MCP troubleshooting

Known problems met when an agent drives the MCP server of [mcp.md](mcp.md), their cause, and how
to fix them. Each entry records the date it was observed: re-check it against the current client
and server before acting on it.

## Claude Code connects but lists no tool

Observed on 2026-09-27, local stack (`http://localhost:8460/mcp`, server `life-pixel-dev` 0.1.0),
Claude Code 2.1.283 on Windows. Fixed the same day.

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

### Cause

Claude Code no longer opens with `initialize`: it first probes with `server/discover` for MCP
revision 2026-07-28, and falls back to `initialize` only when the server does not offer it.
`rmcp` 3.4 answers `server/discover` with every revision it knows, 2026-07-28 included, yet leaves
out of its list results (`tools/list`, `resources/list`…) the cache hints that revision requires:
`ttlMs`, a number, and `cacheScope`, `public` or `private`. The client settled on 2026-07-28 and
rejected every list result. Its MCP log — on Windows, under
`%LOCALAPPDATA%\claude-cli-nodejs\Cache\<project>\mcp-logs-<server>\` — reads
`"protocolEra":"modern","negotiatedProtocolVersion":"2026-07-28"`.

Calling the endpoint with `initialize` does not show it: every revision up to 2025-11-25 is
answered correctly there, and a newer one is negotiated down to 2025-11-25.

### Fix

The server offers only the revisions it implements, up to 2025-11-25: `LATEST_PROTOCOL_VERSION`
in `crates/mcp/src/server.rs`, to which the hosted `HostedMcpHandler` delegates. A
`server/discover` probe for 2026-07-28 is refused with `-32022`, unsupported protocol version,
listing those revisions, and Claude Code falls back to `initialize`: `claude mcp list` reports
`✔ Connected`, and the log reads `"protocolEra":"legacy","negotiatedProtocolVersion":"2025-11-25"`.

A server built before the fix still shows the symptom: rebuild its image and recreate its
container. Offer 2026-07-28 again only once `rmcp` gives list results their cache hints.

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
