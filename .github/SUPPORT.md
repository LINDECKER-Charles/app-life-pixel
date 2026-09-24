# Getting help

Life Pixel is free software maintained on limited time. Community support comes without a
response-time commitment. This page says where to ask, and how to ask so that the answer can
help.

## Where to ask

| You have | Go to |
|---|---|
| A question about using Life Pixel | the [documentation](../docs/), then GitHub Discussions if they are open, or an issue |
| A reproducible bug | a **Bug report** issue |
| An idea or a need | a **Feature request** issue |
| A problem with your hosted account, plan or data | the support form in the app (Help → Contact) — never a public issue, which could expose personal data |
| A security vulnerability | **never a public issue** — [SECURITY.md](SECURITY.md) |
| The wish to contribute | [CONTRIBUTING.md](CONTRIBUTING.md) |

## What makes a request actionable

- where it happens: the web app, the Android app, the desktop app (and its operating system), a
  self-hosted server, an export played in your app, or the MCP server;
- the version, and for an export played in your app, the browser or webview, the framework and
  the Content Security Policy of the page;
- what you expected, what happened, and the minimal steps to reproduce it;
- the error messages of the console, with personal data removed.

A minimal animation that reproduces the problem is worth more than a long description.

## What must never be published

In a public issue or discussion, never include an MCP or API token, an email address or account
identifier, a `.env` file, or a creation you do not want to make public.

## Priorities

1. security vulnerabilities;
2. data loss;
3. regressions compared with the previous release;
4. exports that do not play, or play wrong;
5. everything else.

An issue without an answer is not a rejected issue: following up after a few weeks is
legitimate.
