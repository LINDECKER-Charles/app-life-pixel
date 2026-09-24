# Decisions

| Status | Meaning |
|---|---|
| **Accepted** | settled by the maintainer; the rest of the documentation relies on it |
| **Proposed** | recommended, waiting for the maintainer; do not build on it without a go-ahead |
| **Open** | to decide; the options are listed where they are known |

When a proposal is accepted or rejected, move it to the right table with the date, and update
the documents it affects in the same commit. An ID is never reused or renumbered.

## Accepted

| ID | Decision | Date |
|---|---|---|
| D1 | Rust for the back-end and for everything compiled to WebAssembly | 2026-09-24 |
| D2 | Angular and Ionic for the application front-end. Capacitor was part of the initial choice; D13 replaced it with Tauri | 2026-09-24 |
| D3 | Three ways to use the product: the hosted service in the browser (free plan and subscriptions, for those who do not want to self-host), a desktop app that runs on the user's own machine (the self-hosted version), and an Android app connected to the hosted service (iOS later, not a priority) | 2026-09-24 |
| D4 | The hosted service runs on the VPS managed by `infra-vps`, with a staging and a production environment, like the other projects | 2026-09-24 |
| D5 | The free plan is bounded by storage, at 100 MB; three paid tiers sit above it — what they sell is D25. No paywall after a few uses | 2026-09-24 |
| D6 | An MCP server lets AI agents — Claude first — create, download and integrate animations, on the hosted service and on the desktop | 2026-09-24 |
| D7 | A separate admin console, front-end and back-end, for the metrics and logs of staging and production, support requests and administration; fine-grained metrics with an ergonomic interface | 2026-09-24 |
| D8 | Open source, with CodeQL, Dependabot and the community documents (contributing guide, security policy…) | 2026-09-24 |
| D9 | The repository is in English; the interface is translated — French and English first, open to more | 2026-09-24 |
| D10 | Pull requests, Dependabot's included, target `dev`. A green `dev` is promoted to `test` and deployed to staging; production is released from `test` through `main` | 2026-09-24 |
| D11 | ~~MIT licence for the whole repository~~ — superseded by D15 | 2026-09-24 |
| D12 | An export is a prebuilt player plus a data payload appended as a custom section: nothing is compiled at export time, and an export needs no dependency (was P1) | 2026-09-24 |
| D13 | Tauri 2 packages the application for the desktop and for mobile — Android now, iOS later — in place of Capacitor; the desktop app embeds the local back-end. Self-hosting uses the same Docker images as the hosted service (was P3) | 2026-09-24 |
| D14 | `dev` is the repository's default branch — changed on GitHub the same day (was P4) | 2026-09-24 |
| D15 | Split licence: MIT for what ships inside users' apps — `crates/format`, `crates/player`, `player-js`, the integration snippets —, AGPL-3.0-only for everything else (was P5) | 2026-09-24 |
| D16 | Creations are kept in external S3-compatible object storage, not on the VPS disk; the database is backed up off the VPS (was P6) | 2026-09-24 |
| D17 | The admin console reads the `infra-vps` observability stack and adds business data; one deployment per environment (was P7) | 2026-09-24 |
| D18 | Subscriptions are sold on the web only; the Android app offers no purchase. The paid tiers need value beyond storage (was P8) | 2026-09-24 |
| D19 | The core and player prototype and the editor are built in parallel, the work being split by an orchestrator from a plan. Measuring an export against GIF, APNG and WebP remains a checkpoint of M1 (P14 proposed to build them one after the other) | 2026-09-24 |
| D20 | One Rust core shared by the editor (through WebAssembly), the server, the MCP tools, the CLI and the player (was P2) | 2026-09-24 |
| D21 | Technical stack: axum and tokio, Postgres with sqlx, `object_store`, `rmcp`, `utoipa` (OpenAPI, from which the TypeScript client is generated), `wasm-bindgen`, Vitest, Playwright (was P13) | 2026-09-24 |
| D22 | Two images: `app` (server and editor) and `admin` (admin server and console) (was P12) | 2026-09-24 |
| D23 | `dev` → `test` → `main` move by fast-forward only; production receives the images built from the exact commit of `main`, promoted by SHA (was P11) | 2026-09-24 |
| D24 | First-party telemetry only; the desktop app and self-hosted servers send nothing without the user's opt-in (was P9) | 2026-09-24 |
| D25 | Pricing: creating stays free; the paid tiers — Creator, Team, Business — sell services: live embeds served from a CDN, sync between devices, full version history, team workspaces, higher MCP limits. Every plan, paid ones included, keeps fair-use ceilings on storage, embed views and MCP calls, set high enough that only exceptional consumers ever reach them (was P15) | 2026-09-24 |
| D26 | Translations are served by an endpoint and loaded at runtime: the app downloads the active language only, fetches another one when the user switches — without reloading —, and reads the list of languages from the same endpoint. No catalogue is compiled into the bundle (P10 described a single bundle) | 2026-09-24 |
| D27 | The product is named Life Pixel: crates `life-pixel-*`, npm package `@life-pixel/player`, custom element `<life-pixel>` (from O1) | 2026-09-24 |

## Proposed

| ID | Proposal | Why | Affects |
|---|---|---|---|
| P10 | Transloco as the client i18n library: its HTTP loader fetches the catalogues of D26 one language at a time, and it handles ICU plurals | a maintained library that already implements D26, rather than a home-made loader and message formatter | [i18n.md](i18n.md) |

## Open

Each open question is scheduled by the plan in the milestone it blocks: O1, O2, O7 and O8 before
M3, O5 before M5 and M6, O3 and O4 before M7, O6 when sharing is considered.

| ID | Question |
|---|---|
| O1 | Domains and branding: application, API and MCP, admin console, embeds, for staging and production. The name is settled (D27). To do early: reserve the npm organisation `life-pixel` and a domain — lifepixel.app, lifepixel.dev, lifepixel.io, life-pixel.com and lifepixel.fr were unregistered on 2026-09-24; lifepixel.com is taken |
| O2 | Sign-in methods: own accounts, social sign-in (GitHub, Google), magic links. Whatever the choice, the Tauri apps and MCP connectors authenticate through OAuth 2.1 |
| O3 | Billing: Stripe Billing with Stripe Tax, or a merchant of record handling EU VAT |
| O4 | Paid tiers: prices, allowances and fair-use ceilings — the structure is D25 |
| O5 | Code signing: Apple notarisation, Windows certificate, custody of the Android upload key |
| O6 | Public sharing or a gallery — and, if so, moderation and the legal duties that come with it |
| O7 | Transactional email provider |
| O8 | Object storage provider and location |
