# Freeloader architecture

Freeloader is a local-first desktop application. The Rust core owns networking, SQLite, filesystem writes, download state and persistence. The React webview renders state and sends typed intent through Tauri commands; it never downloads file bytes.

## Boundaries

- `crates/protocol`: serde-only message contract and validation. No I/O.
- `crates/download-core`: portable download engine and persistence. No Tauri dependency.
- `crates/platform`: OS-specific paths, browser detection and native-host registration behind traits.
- `crates/native-host`: small Native Messaging stdio launcher. It validates messages and forwards intent; it does not duplicate download logic.
- `apps/desktop`: Tauri adapter plus React UI.

## Portability

Windows x64, Windows ARM64, Linux x64 and Linux ARM64 are release targets. macOS is intentionally not shipped in v0.1. Platform-specific code stays in `platform` and installer adapters. The download core must remain headless and portable.

## Decisions

- GPL-3.0-or-later keeps the project OSI-approved and requires distributed derivatives to remain under GPL with corresponding source.
- sqlx + SQLite is selected for async Tokio integration and migration support.
- Native Messaging is used instead of a localhost server to avoid an additional network surface.
- CSS is preferred for motion; Motion is limited to exit and layout transitions.
- Windows uses NSIS as the common setup format because WiX/MSI does not support ARM64.
