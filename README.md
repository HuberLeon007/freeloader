# Freeloader

**A fast, private, local-first download manager for Windows and Linux.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

Freeloader is a desktop download manager built with Tauri v2, Rust and React. It runs entirely on your machine: no account, no login, no cloud, no subscription, no ads, no telemetry, no tracking, no background web server. The only network traffic Freeloader produces is the downloads you ask for, plus an explicitly opt-in update check.

> Status: **pre-alpha / scaffolding.** The repository is being bootstrapped. Nothing here is production-ready yet.

---

## Why

Most download managers are either abandoned, bundled with adware, wrapped around a local HTTP server, or a 200 MB Electron shell. Freeloader aims to be the opposite: a small native-feeling binary, a calm and precise UI, a Rust core that streams bytes to disk reliably, and browser integration over OS-level Native Messaging instead of a localhost port.

## Design principles

1. **Local-first.** All state lives in SQLite inside the local application data directory.
2. **No hidden network surface.** No localhost HTTP server, no WebSocket bridge, no relay, no telemetry.
3. **Least privilege.** Browser extensions request only the permissions they can justify in writing.
4. **Untrusted input by default.** URLs, filenames, headers and referrers are validated and sanitised before they touch the filesystem.
5. **Efficient.** Streaming I/O, bounded memory, throttled IPC, a virtualised UI, and a release profile tuned for a small binary.
6. **Native feel.** Keyboard-first, WCAG 2.2 AA oriented contrast and focus states, full dark and light themes following the system preference.

## Planned architecture

```
freeloader/
  apps/desktop/           # Tauri v2 shell: React + TypeScript + Vite frontend
  crates/download-core/   # Download engine, state machine, persistence (Rust, no Tauri dependency)
  crates/native-host/     # Native Messaging host / launcher (Rust)
  crates/protocol/        # Versioned message contract shared by app, host and extensions
  extensions/chromium/    # Manifest V3 extension (Chrome, Edge, Brave, Vivaldi, Opera)
  extensions/firefox/     # Firefox WebExtension
  scripts/                # Native-messaging host registration for Windows and Linux
  docs/                   # Architecture, ADRs, security model, release process
```

**Layering rule:** the frontend renders and dispatches intent, nothing more. The Rust core owns networking, the filesystem, the database and all download state. Large files never pass through the webview.

## Tech stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri v2 |
| Core | Rust (stable), Tokio, reqwest with rustls, sqlx + SQLite |
| Observability | tracing / tracing-subscriber, local rotating log files |
| Errors | thiserror in libraries, anyhow at the binary boundary |
| Frontend | React 19, TypeScript (strict), Vite, TanStack Router, Zustand |
| Styling | Tailwind CSS v4 design tokens, selected shadcn/ui primitives, Lucide icons |
| Motion | CSS transitions by default; Motion (LazyMotion) only for layout and exit animations |
| Forms | React Hook Form + Zod |
| Browser bridge | Native Messaging over stdio, no network sockets |

## Feature scope (MVP)

- Add downloads by URL with pre-flight probing: size, content type, suggested filename, resume capability
- Queue with a configurable concurrency limit
- Start, pause, resume, cancel, retry with exponential backoff
- HTTP Range based resume when (and only when) the server supports it
- Streaming writes to `.part` files, atomic rename on completion
- Filename sanitisation and explicit conflict resolution (rename / overwrite / cancel)
- Persistent queue that survives restarts and offers resumption of interrupted downloads
- Categories, search, filtering and sorting
- Guided installer and a first-run setup flow that detects installed browsers
- Browser extensions: context-menu capture and explicit page-link detection, never silent request interception

Explicitly **out of scope**: DRM circumvention, paywall bypass, streaming-site rippers, credential or cookie harvesting.

## Platform support

| Platform | Architecture | Status |
| --- | --- | --- |
| Windows 10 / 11 | x86_64 | Primary target |
| Windows 11 | ARM64 (aarch64) | Primary target, NSIS installer only |
| Linux (Debian / Ubuntu / Fedora based) | x86_64 | Primary target |
| Linux | ARM64 (aarch64) | Primary target |
| macOS | - | Not supported and not planned for now |
| Android | - | Long-term; the core engine is reusable, the UI and OS integration are not |

## Licence

Freeloader is free software licensed under the **[GNU General Public License v3.0 or later](LICENSE)** (`GPL-3.0-or-later`), an OSI-approved open source licence.

In plain terms:

- You may use, study, run, copy, modify, fork and redistribute this software freely, including commercially.
- You **must** preserve the copyright notices and attribution to the original project.
- If you distribute a modified version, you **must** release its complete corresponding source code under the same GPL-3.0-or-later terms. Closed-source forks are not permitted.
- Modified versions must be marked as changed so problems are not misattributed to the original authors.

That is the point of the choice: the project stays open, and anything built on it stays open too.

Contributions are accepted under the same licence, certified via the [Developer Certificate of Origin](https://developercertificate.org/) (`git commit -s`). No copyright assignment is requested.

## Trademarks and references

Freeloader is an independent project. It is not affiliated with, endorsed by, or derived from the source code, branding or assets of any other download manager.
