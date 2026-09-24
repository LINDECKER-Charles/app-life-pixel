# Pricing and quotas

## Principles (accepted)

- **No paywall on usage.** Every editor feature, every export format and the MCP server are
  available on the free plan.
- **Trying needs no account.** Without one, the editor and every export work in the browser, but
  nothing is saved (D37): saving starts with the free plan.
- **Storage bounds the free plan**, at **100 MB**: it is the cost that grows with users (D5).
- **Three paid tiers** above it — Creator, Team, Business — that sell services rather than
  storage (D25).
- **Fair-use ceilings on every plan**, paid ones included, set high enough that only exceptional
  consumers ever reach them (D25).
- **Sold on the web only** (D18): the Android app offers no purchase, so Google Play Billing and
  its commission are not involved; the app signs into an existing account.
- **The desktop app and the self-hosted server are free**, open source, without any licence
  check.

## Why storage alone will not sell

Pixel art is small. A 64×64 animation of 12 frames is 49 KB of raw palette indices and usually
compresses well under 20 KB, so 100 MB holds several thousand animations of that size. A quota
that almost nobody reaches protects our costs, but it gives nobody a reason to pay. The M1
prototype will confirm these sizes on real animations.

## The model (D25): creating is free, services are paid

The paid tiers sell what a free local app cannot do on its own — host, sync, collaborate — which
are also the things that cost us money: bandwidth, storage, support. It is the model of tools
whose app is free and whose sync and publishing are paid.

| Plan | For | What it adds |
|---|---|---|
| Free | everyone | the whole editor, every export, MCP within fair-use limits, 100 MB, a few live embeds to try the feature |
| Creator | individuals | live embeds with a monthly view allowance, sync between devices (desktop included), full version history, higher MCP limits, more storage |
| Team | small teams | shared libraries, roles, per-seat billing, a larger view allowance |
| Business | companies | single sign-on, audit log export, priority support, invoicing, custom limits |

- **Live embeds**: `<life-pixel src="https://embed.lifepixel.tech/<id>.wasm">`, served from a CDN.
  Editing the animation updates every site that embeds it, without redeploying anything. They
  are optional: downloaded exports stay free, unlimited and dependency-free. Views are counted
  without cookies or personal data.
- **Sync** connects the desktop app to a hosted account. The desktop app stays complete and free
  without it; sync is what leads its users to a paid plan.
- **Fair-use ceilings** cover storage, embed views and MCP calls on every plan. They protect the
  service from a runaway consumer, not the price list: set high enough that reaching one is
  exceptional, and a custom Business agreement takes over beyond them.

## Provisional grid (D31)

Starting values, set before any usage data: they are configuration, not code, and are revisited
with the data of the first release. The M1 measurements check the storage allowances.

| | Free | Creator | Team | Business |
|---|---|---|---|---|
| Price | €0 | €6 a month, or €60 a year | €10 per seat and month, or €100 per seat and year | on quote |
| Storage | 100 MB | 10 GB | 10 GB per seat, pooled | custom |
| Live embeds | 3, with 10,000 views a month | unlimited, with 1 million views a month | unlimited, with 5 million views a month, pooled | custom |
| MCP calls | 1,000 a day | 10,000 a day | 10,000 a day per seat | custom |

Paying for a year costs ten months.

## What counts against the storage quota

| Counts | Does not count |
|---|---|
| stored documents, every kept version included | exports — compiled on demand, never stored |
| imported images | downloads, previews, MCP calls |

Usage is shown in the app and returned by the API. `service` enforces the quota on every write,
whatever the client — interface, API or MCP.

## Over quota

- Nothing is ever deleted automatically, and everything stays readable, exportable and
  downloadable.
- A write that would take usage above the quota is refused with `quota.storage_exceeded`;
  deleting always works.
- After a downgrade or a failed payment, a 30-day grace period (D31) precedes the free quota; the
  same read-only rule applies afterwards.

## Billing

- **Provider**: a merchant of record (D30), which sells on our behalf and handles VAT and sales
  taxes in every country. The vendor is chosen at M7.
- The provider hosts checkout, invoices and the customer portal (upgrade, downgrade, cancel). Our
  database keeps the plan, the subscription status and the end of the current period — never
  card data.
