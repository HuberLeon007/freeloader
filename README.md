# Freeloader

**A fast, private, local-first download manager for Windows.**

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
5. **Native feel.** Keyboard-first, WCAG 2.2 AA oriented contrast and focus states, full dark and light themes following the system preference.
6. **Windows-first, not Windows-only.** Platform-specific behaviour is isolated behind traits and adapters so Linux, macOS and later Android remain reachable without a rewrite.

## Planned architecture

```
freeloader/
  apps/desktop/           # Tauri v2 shell: React + TypeScript + Vite frontend
  crates/download-core/   # Download engine, state machine, persistence (Rust)
  crates/native-host/     # Native Messaging host / launcher (Rust)
  crates/protocol/        # Versioned message contract shared by app, host, extensions
  extensions/chromium/    # Manifest V3 extension (Chrome, Edge, Brave, Vivaldi, Opera)
  extensions/firefox/     # Firefox WebExtension
  scripts/                # Windows registration / de-registration for the native host
  docs/                   # Architecture, security model, native messaging, release process
```

**Layering rule:** the frontend renders and dispatches intent, nothing more. The Rust core owns networking, the filesystem, the database and all download state. Large files never pass through the webview.

## Tech stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri v2 |
| Core | Rust (stable, 2021 edition), Tokio, reqwest, sqlx + SQLite |
| Observability | tracing / tracing-subscriber, local rotating log files |
| Errors | thiserror in libraries, anyhow at the binary boundary |
| Frontend | React 19, TypeScript (strict), Vite, TanStack Router, Zustand |
| Styling | Tailwind CSS, a small set of shadcn/ui primitives, Lucide icons |
| Forms | React Hook Form + Zod |
| Browser bridge | Native Messaging (stdio), no network sockets |

## Feature scope (MVP)

- Add downloads by URL with pre-flight probing: size, content type, suggested filename, resume capability
- Queue with a configurable concurrency limit
- Start, pause, resume, cancel, retry with exponential backoff
- HTTP Range based resume when (and only when) the server supports it
- Streaming writes to `.part` files, atomic rename on completion
- Filename sanitisation and explicit conflict resolution (rename / overwrite / cancel)
- Persistent queue that survives restarts and offers resumption of interrupted downloads
- Categories, search, filtering and sorting
- Browser extensions: context-menu capture and explicit page-link detection, never silent request interception

Explicitly **out of scope**: DRM circumvention, paywall bypass, streaming-site rippers, credential or cookie harvesting.

## Platform support

| Platform | Status |
| --- | --- |
| Windows 10/11 (x64) | Primary target |
| Linux | Planned, architecture kept portable |
| macOS | Planned |
| Android | Long-term; core engine is reusable, UI and OS integration are not |

## Licence

Freeloader is **source-available and free for any non-commercial use** under the [PolyForm Noncommercial License 1.0.0](LICENSE).

In plain terms:

- You may read, run, copy, modify, fork and redistribute the code freely for personal, educational, research, charitable and other non-commercial purposes.
- You may **not** use it, or anything derived from it, for commercial advantage or monetary compensation.
- Commercial rights are reserved by the copyright holder. Commercial licences are available on request.

This is a deliberate choice: the project should stay a public good, not a free ingredient for someone else's paid product. Note that this makes Freeloader *source-available*, not OSI-approved open source. The README will not claim otherwise.

Contributions are accepted under the same terms, and contributors grant the maintainer the right to license their contributions commercially. See `CONTRIBUTING.md` (coming soon) once published.

## Trademarks and references

Freeloader is an independent project. It is not affiliated with, endorsed by, or derived from the source code, branding or assets of any other download manager.
