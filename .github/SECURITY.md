# Security policy

## Supported versions

Life Pixel has not been released yet. From the first release on, security fixes go to the hosted
service, to the latest release of each distribution — desktop app, Android app, Docker images,
`@life-pixel/player` — and to the latest player of every supported export format version.

## Reporting a vulnerability

**Never open a public issue** for a vulnerability. Use GitHub's private reporting instead:
[report a vulnerability](https://github.com/LINDECKER-Charles/app-life-pixel/security/advisories/new).
If it is unavailable, write to **charles.lindecker@outlook.fr** with a subject starting with
`[security]`.

Include, where possible:

- the component: player or loader, export compiler, web app, API, MCP endpoint, desktop app,
  Android app, Docker image, admin console;
- the version, and how you use it (hosted, desktop, self-hosted, an export in your app);
- the preconditions and the minimal steps to reproduce;
- the impact you expect;
- a suggested fix or a regression test.

An acknowledgement is aimed for within three business days. Fixes are prioritised by
exploitability and impact — anything in the player or the loader first, since they run inside
third-party pages. Disclosure is coordinated with you before anything is published, and you are
credited if you wish.

## Scope

In scope:

- the player and the loader: parsing of an export, anything that reaches the host page;
- the export compiler;
- the web app, the API and the MCP endpoint: authentication, isolation between accounts,
  bypassing a quota with a real impact;
- the desktop app and the local MCP server: access to files outside the allowed directories;
- the admin console, the Docker images, and the CI/CD pipeline of this repository.

Out of scope: volumetric denial of service, social engineering, physical access, issues that
require an already compromised device or browser, missing security headers without a
demonstrated impact, and automated scanner output without a proof of concept.

## Good-faith research

- Test against your own accounts and data only; never access, modify or delete someone else's.
- Do not degrade the service: no load test, no spam.
- Prefer a local or self-hosted instance for intrusive testing.
- Leave us reasonable time to ship a fix before any disclosure.

Research that follows these rules is welcome and treated as made in good faith.
