# Product

Life Pixel lets a person — or their AI agent — draw a pixel-art animation and ship it inside an
app as a few kilobytes of dependency-free WebAssembly.

## The problem

Adding a small animation to an app — a mascot, a loader, an empty state, a game sprite — forces
a trade-off today:

| Option | What it costs |
|---|---|
| GIF | 256 colours, all-or-nothing transparency, no control from code |
| APNG, animated WebP | better images, but still a video: no states, no events, no palette swap |
| Sprite sheet + hand-written code | player code to write and maintain in every project |
| Lottie, Rive | vector-oriented, runtimes of tens to hundreds of kilobytes, not pixel-exact |

## What Life Pixel does

1. **Draw and animate** in an editor made for pixel art — in the browser, on the desktop, on
   Android.
2. **Export** a self-contained WebAssembly bundle that plays pixel-exact at any size and can be
   driven from JavaScript (play, pause, seek, switch between named states). The classic formats
   (GIF, APNG, sprite sheet, PNG frames) are there too, for when they fit better.
3. **Integrate** with one custom element, `<life-pixel src="…">`: no framework, no build step,
   no dependency.
4. **Automate** through an MCP server: an AI agent — Claude first — creates, previews, exports and
   integrates an animation from the developer's own environment. Life Pixel becomes a tool that
   plugs into any development ecosystem.

## Who it is for

- web and mobile developers who want lively details without adding a dependency;
- indie game developers who need sprites in a browser or a webview;
- pixel artists who want their work to run, not only to be exported;
- developers working with an AI agent, who describe an animation and get it integrated.

## Principles

1. **Light by default.** An export is measured in kilobytes, and the export dialog shows the size
   of every format side by side, so that the lightest one that fits wins.
2. **Nothing essential behind a paywall.** Every editor feature, every export format and the MCP
   server are free. The free plan is only bounded by storage and by fair-use ceilings that almost
   nobody reaches — see [pricing](pricing.md).
3. **No lock-in.** Open source, documented format, self-hostable, and exports that keep working
   even if Life Pixel disappears.
4. **Humans and agents are peers.** Every editing action of the interface has an MCP equivalent,
   and both edit the same documents.
5. **What you see is what ships.** The editor, the exports and the MCP previews share one Rust
   core — see [architecture](architecture.md).
6. **Respectful.** Accessible, translated, reduced motion honoured, privacy-friendly, no dark
   patterns.

## Feature scope

### MVP

| Area | Content |
|---|---|
| Editor | canvas with zoom, pan and grid; pencil, eraser, fill, line, rectangle, selection and move; indexed palette; layers; frames and timeline with per-frame duration; onion skin; playback preview; undo and redo |
| Import | PNG images and sprite sheets, reduced to the palette |
| States | named frame ranges (tags), switchable from JavaScript |
| Export | WASM bundle and loader, GIF, APNG, sprite sheet with JSON, PNG frames; integration snippets for HTML, Angular, React and Vue |
| Library | projects and animations: search, duplicate, delete |
| Accounts (hosted) | sign-up and sign-in with an email address and a password, storage usage against the 100 MB free quota; without an account, the editor and its exports work, but nothing is saved (D37) |
| MCP | the core tool set, on the hosted service (token authentication) and locally (stdio) |
| Desktop | local library, every export, local MCP |
| Languages | English and French |
| Admin console | health, metrics and logs of staging and production; users; support requests |

### Next

- A real state machine: transitions, events, triggers from JavaScript; palette variants
  switchable at runtime.
- Aseprite import (`.ase`, `.aseprite`).
- Paid plans and billing, sold on the web (D18), with the services they sell (D25): live embeds
  served from a CDN, sync between devices, team workspaces.
- OAuth 2.1 on the MCP endpoint, so that the hosted service plugs into claude.ai as a connector.
- Version history.
- Sharing and team workspaces.
- iOS app.
- Native (non-WASM) players for native mobile apps — only if demand proves it.

### Non-goals

- A general-purpose painting or vector tool.
- Running an AI model ourselves. Users bring their own agent through MCP; we pay for no
  inference.
- Real-time collaborative editing at launch.

## Milestones

| # | Milestone | Done when |
|---|---|---|
| M0 | Foundations | this documentation, the standards, the decisions of [decisions.md](decisions.md) settled |
| M1 | Core and player prototype | a real animation exported as WASM plays in a page, and its size is measured against GIF, APNG and WebP — the checkpoint that proves the product's promise with numbers |
| M2 | Web editor, local | draw → animate → export works end to end in the browser, without an account |
| M3 | Hosted beta | accounts, storage and quota, staging and production on the VPS, admin console v1, backups, legal pages (terms, privacy policy, legal notice) |
| M4 | MCP | hosted endpoint with tokens; local server over stdio through the CLI |
| M5 | Desktop app | Windows, macOS and Linux installers built by Tauri, signed |
| M6 | Android app | the same Tauri shell built for Android, on the Play Store, connected to the hosted service through OAuth 2.1 |
| M7 | Paid plans | billing live, three paid tiers |

M1 and M2 run in parallel (D19): the work is split by an orchestrator from the
[implementation plan](implementation-plan.md), and the following milestones overlap as far as
their dependencies allow.
