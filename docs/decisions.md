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
| D28 | The domain is `lifepixel.tech`. Production serves the app, the API (`/api/v1`), the MCP endpoint (`/mcp`) and the catalogues (`/i18n`) from `lifepixel.tech`, and the admin console from `admin.lifepixel.tech`; staging mirrors them on `staging.lifepixel.tech` and `admin.staging.lifepixel.tech`; live embeds will come from `embed.lifepixel.tech`. Domains are configuration: the code names none, so a self-hosted server runs on its own (was O1) | 2026-09-24 |
| D29 | Sign-in with our own accounts — an email address and a password — at first. Social sign-in, GitHub first then Google, comes after the first release as another way into the same account. No magic links: every sign-in would depend on the mailbox, and a link opened from a mail app does not come back to the browser tab where the OAuth flow of the Android app or of an MCP connector waits. A self-hosted server needs no third-party OAuth application (was O2) | 2026-09-24 |
| D30 | Billing goes through a merchant of record, which sells on our behalf and handles VAT and sales taxes in every country: no tax registration or return of our own. The vendor is chosen at M7, behind a port of `service`; Stripe Billing with Stripe Tax is reconsidered if volume ever makes its lower fees worth the tax filings (was O3) | 2026-09-24 |
| D31 | Provisional prices and allowances, detailed in [pricing.md](pricing.md): Creator at €6 a month, Team at €10 per seat and month, Business on quote, and a 30-day grace period after a downgrade or a failed payment. Plan values are configuration, revisited with the usage data of the first release (was O4) | 2026-09-24 |
| D32 | Signing: the Apple Developer Program for the Developer ID certificate and notarisation; on Windows, a certificate whose key stays in a cloud HSM, or Azure Trusted Signing — whichever the publishing entity is eligible for at M5; on Android, Play App Signing, so that we only hold an upload key Google can reset. Signing material lives in a GitHub `release` environment that requires the maintainer's approval; the Tauri updater key and the upload key also have an encrypted offline copy (was O5) | 2026-09-24 |
| D33 | No public gallery or sharing in the first release. Live embeds, when they come, are unlisted — reachable by their URL only — and abuse is reported through the "abuse" category of the support form. A gallery would take a new decision, with its moderation and legal duties (was O6) | 2026-09-24 |
| D34 | Transactional email: Scaleway Transactional Email on the hosted service. The server speaks plain SMTP, so another provider — or a self-hoster's own server — is a configuration change (was O7) | 2026-09-24 |
| D35 | Object storage: Scaleway Object Storage in Paris (`fr-par`), one bucket per environment, versioned in production. Database backups go, encrypted, to a bucket in Amsterdam (`nl-ams`), written by credentials that cannot delete them. Creations and backups stay in the EU (was O8) | 2026-09-24 |
| D36 | Transloco is the client i18n library: its HTTP loader fetches the catalogues of D26 one language at a time, and it handles ICU plurals — a maintained library rather than a home-made loader and message formatter (was P10) | 2026-09-24 |

## Proposed

| ID | Proposal | Why | Affects |
|---|---|---|---|
| P16 | The hosted editor keeps M2's mode without an account: documents stay in the browser until the visitor signs up, then move into their library, where the quota applies | trying the editor needs no sign-up, and M2 builds that mode anyway — without it, M2's browser library is dropped at M3 | [product.md](product.md), [architecture.md](architecture.md), [implementation-plan.md](implementation-plan.md) |

## Open

None at the moment. Three accepted decisions leave a detail to later, within the direction they
set: the final prices (D31, after the first release), the Windows signing service (D32, at M5)
and the billing vendor (D30, at M7).
