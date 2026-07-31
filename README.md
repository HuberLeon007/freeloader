# Freeloader

**A fast, private, local-first download manager for Windows and Linux.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

Freeloader is a desktop download manager built with Tauri v2, Rust and React. It runs entirely on your machine: no account, no login, no cloud, no subscription, no ads, no telemetry, no tracking, no background web server. The only network traffic Freeloader produces is the downloads you ask for.

> Status: **alpha.** The vertical slice works end to end. Expect rough edges.

---

## Quick start

```bash
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
cargo dev
```

That is the whole loop. `cargo dev` installs the Node side if it is missing, generates the application icons, builds the Rust core and opens the desktop window. The first build takes a few minutes; after that it is incremental.

Other tasks:

```bash
cargo xtask build    # installers for the current platform
cargo xtask check    # the same four gates CI runs
cargo xtask icons    # regenerate the bundle icons
```

If you would rather drive pnpm yourself, `pnpm install && pnpm --dir apps/desktop app:dev` does the same thing.

### Prerequisites

| Requirement | Notes |
| --- | --- |
| Rust (stable) | `rustup toolchain install stable` |
| Node 22 + pnpm 10 | `corepack enable` is enough |
| Windows | Microsoft Visual Studio C++ Build Tools, WebView2 (preinstalled on Windows 11) |
| Linux | `libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf` |

## First run

Setup asks four questions and stores the answers on this machine only:

1. What Freeloader is, and what it deliberately does not do
2. Where files land: pick from OS-resolved suggestions or the native folder picker, with the target checked for existence and write access before you continue
3. Appearance: system, dark or light
4. Browser handoff: optional detection of browsers on your PATH

Every answer is changeable later in Settings. `Esc` skips the whole thing.

## Repository layout

```
freeloader/
  apps/desktop/           # Tauri v2 shell: React + TypeScript + Vite frontend
  apps/desktop/src-tauri/ # Tauri adapter, commands and events
  crates/protocol/        # Versioned message contract, framing, validation, sanitisation
  crates/download-core/   # Download engine, state machine, SQLite persistence
  crates/platform/        # OS boundaries: data dirs, browser detection, file manager
  crates/native-host/     # Native Messaging host / launcher
  extensions/             # Chromium (MV3) and Firefox WebExtensions
  scripts/                # Icon generation and native-messaging host registration
  xtask/                  # Repository automation behind `cargo dev`
  docs/                   # Architecture, ADRs, security model, release process
```

**Layering rule:** the frontend renders and dispatches intent, nothing more. The Rust core owns networking, the filesystem, the database and all download state. Large files never pass through the webview.

## Design principles

1. **Local-first.** All state lives in SQLite inside the local application data directory.
2. **No hidden network surface.** No localhost HTTP server, no WebSocket bridge, no relay, no telemetry.
3. **Least privilege.** Browser extensions request only the permissions they can justify in writing.
4. **Untrusted input by default.** URLs, filenames, headers and referrers are validated and sanitised before they touch the filesystem.
5. **Efficient.** Streaming I/O, bounded memory, throttled IPC, and a release profile tuned for a small binary.
6. **Native feel.** Keyboard-first, WCAG 2.2 AA oriented contrast and focus states, full dark and light themes following the system preference.

## Tech stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri v2 |
| Core | Rust (stable), Tokio, reqwest with rustls, sqlx + SQLite |
| Observability | tracing / tracing-subscriber, local rotating log files |
| Errors | thiserror in libraries, anyhow at the binary boundary |
| Frontend | React 19, TypeScript (strict), Vite |
| Styling | Hand-written CSS design tokens, Lucide icons |
| Browser bridge | Native Messaging over stdio, no network sockets |

## Keyboard

| Shortcut | Action |
| --- | --- |
| `Ctrl` / `Cmd` + `N` | Focus the URL field |
| `Enter` | Start the download in the URL field, or advance setup |
| `/` | Focus the filter |
| `Esc` | Close settings, or skip setup |

## Feature scope (MVP)

- Add downloads by URL with pre-flight probing: size, content type, suggested filename, resume capability
- Queue with a configurable concurrency limit
- Start, resume, cancel, retry with exponential backoff
- HTTP Range based resume when (and only when) the server supports it
- Streaming writes to `.part` files, atomic rename on completion
- Filename sanitisation and explicit conflict resolution (rename / overwrite / cancel)
- Persistent queue that survives restarts
- Search, filtering and per-state views
- First-run setup flow that detects installed browsers
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

## Continuous integration

`.github/workflows/ci.yml` runs four independent jobs: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the frontend typecheck plus build. They are split so a red badge tells you which one broke without opening a log. `cargo xtask check` runs the same four locally.

`.github/workflows/autofix.yml` runs rustfmt on every non-main branch and pushes the result, so formatting is never a merge blocker.

## Licence

Freeloader is free software licensed under the **[GNU General Public License v3.0 or later](LICENSE)** (`GPL-3.0-or-later`), an OSI-approved open source licence.

In plain terms:

- You may use, study, run, copy, modify, fork and redistribute this software freely, including commercially.
- You **must** preserve the copyright notices and attribution to the original project.
- If you distribute a modified version, you **must** release its complete corresponding source code under the same GPL-3.0-or-later terms. Closed-source forks are not permitted.
- Modified versions must be marked as changed so problems are not misattributed to the original authors.

Contributions are accepted under the same licence, certified via the [Developer Certificate of Origin](https://developercertificate.org/) (`git commit -s`). No copyright assignment is requested.

## Trademarks and references

Freeloader is an independent project. It is not affiliated with, endorsed by, or derived from the source code, branding or assets of any other download manager.
