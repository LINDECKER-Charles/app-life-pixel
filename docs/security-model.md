# Security model

How to report a vulnerability: [.github/SECURITY.md](../.github/SECURITY.md). This document
explains what the design protects and how.

## What we protect

| Asset | Why it matters |
|---|---|
| Pages of third-party apps that embed an export | our code runs on sites we do not control: a flaw is multiplied by every integration |
| Accounts and creations | the users' work and identity |
| Sessions and tokens | they act on the user's behalf, including from AI agents |
| The shared VPS | other projects live on it |
| Payment data | never stored by us: the billing provider holds it |

## Design decisions that carry security

1. **Exports are data** (D12). The player is prebuilt, reviewed and reproducible; a
   user's content only reaches it as a payload, read by a bounds-checked decoder that refuses
   unknown versions. That decoder is the one parser facing untrusted bytes inside other people's
   pages: it is fuzzed continuously with `cargo fuzz`.
2. **The player imports nothing** beyond what the loader hands it: no network, no DOM, no timer of
   its own. A malicious payload has nothing to reach.
3. **The loader has no dependency** and is small enough to audit in one sitting. It is published
   with provenance (npm trusted publishing, attestations), and integrators can pin it with
   Subresource Integrity.
4. **Uploaded images** are decoded under dimension and size limits — decompression bombs
   included — and re-encoded before storage. Their original bytes are never served back.
5. **The server trusts no client.** `service` validates everything with the limits of `core` and
   enforces quotas and rate limits per account and per token.

## Authentication (hosted and self-hosted)

- Sign-in with an email address and a password (D29 in [decisions.md](decisions.md)); social
  sign-in comes after the first release, as another way into the same account.
- Passwords hashed with Argon2id. Sessions in `HttpOnly`, `Secure`, `SameSite=Lax` cookies with
  the `__Host-` prefix: bound to their host, they never reach a subdomain — the embed host
  included. State-changing requests checked against CSRF (origin check and token).
- MCP: personal access tokens — 256 random bits, shown once, stored hashed, scoped, revocable,
  expiring. OAuth 2.1 with PKCE when claude.ai connectors land.
- Tauri apps (Android, desktop sync): OAuth 2.1 authorization code with PKCE, through the system
  browser and a deep link back — the same authorization server as the MCP connectors. Tokens
  live in the platform's keystore, never in web storage.
- Admin console: separate admin accounts, mandatory second factor, short sessions, audit log.

## Web hardening

- A strict Content Security Policy: `default-src 'self'`; `script-src 'self' 'wasm-unsafe-eval'`;
  no inline script. HSTS, `X-Content-Type-Options`, `Referrer-Policy`, `frame-ancestors 'none'`.
- Rate limits on sign-in, MCP and uploads.
- Errors never expose a stack trace, a query or an internal identifier.

## MCP

- Every tool goes through `service`: the same validation, quotas and permissions as the interface.
- What a tool returns from a document — names, descriptions — is user content. It is returned as
  data, and no tool interprets text from a document as an instruction.
- The local server writes only inside the directories the user allowed, never follows a symbolic
  link out of them, and never overwrites a file unless asked to.

## Supply chain

| Control | How |
|---|---|
| Static analysis | CodeQL on `rust`, `javascript-typescript` and `actions` |
| Dependency updates | Dependabot on `cargo`, `npm`, `github-actions` and `docker` |
| Dependencies added by a pull request | dependency review action, failing on moderate severity and above |
| Advisories, licences, sources | `cargo deny` (RustSec, licence allow-list, crates.io only), `npm audit` |
| Licence boundary | `format`, `player` and `player-js` (MIT) pull no AGPL code and no copyleft dependency (D15) |
| Secrets | GitHub secret scanning with push protection |
| Actions | pinned to a full commit SHA, kept current by Dependabot |
| Toolchains | pinned by `rust-toolchain.toml` and `.nvmrc`; lockfiles committed |
| Releases | provenance attestations for images, installers and the npm package; reproducible player build, hash checked by CI |
| Review | CODEOWNERS on `.github/`, `docker/`, `crates/format`, `crates/player`, `player-js` |

## Privacy

- **Minimisation**: an email address, a password hash, a language, a plan; the creations;
  pseudonymous product events without content.
- **Telemetry**: first-party only; nothing leaves the desktop app or a self-hosted server unless
  the user opts in (D24).
- **Rights**: access (export of the account's data) and erasure (account deletion, object storage
  included), from the app or through a support request.
- **Retention**: product events 13 months; deleted accounts purged within 30 days, then gone from
  backups as they rotate, within six months. The VPS keeps logs, and the edge access log — IP
  addresses and geolocation, which are personal data — 90 days.
- **Processors and hosting locations** are listed in the privacy policy; EU locations are
  preferred. Creations and transactional email are handled by Scaleway in Paris, backups in
  Amsterdam (D34, D35).
