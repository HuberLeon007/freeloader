# Verification playbook

This document separates commands that can run on any developer machine from commands that require the target operating system and architecture. Do not call an artifact supported until its matching native build and smoke test succeeds.

## 1. Clone

```bash
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
git switch feat/freeloader-full-implementation
```

## 2. Rust checks

```bash
rustup toolchain install stable
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 3. Frontend checks

```bash
corepack enable
pnpm install
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
```

## 4. Development UI

```bash
pnpm --dir apps/desktop dev
```

This runs the browser UI only. To run the Tauri shell, install the current Tauri CLI and use `cargo tauri dev --manifest-path apps/desktop/src-tauri/Cargo.toml` from the repository root. The production binary does not start a local HTTP server.

## 5. Windows x64

Run on Windows 10/11 x64 with WebView2 and Visual Studio C++ build tools:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --bundles nsis
```

Expected primary artifact: `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe`.

## 6. Windows ARM64

Run on Windows 11 ARM64, or a fully configured cross-compilation host with MSVC ARM64/ARM64EC tools and the Windows 11 SDK:

```powershell
rustup target add aarch64-pc-windows-msvc
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target aarch64-pc-windows-msvc --bundles nsis
```

Expected primary artifact: `apps/desktop/src-tauri/target/aarch64-pc-windows-msvc/release/bundle/nsis/*-setup.exe`. MSI is intentionally not produced for ARM64.

## 7. Linux x64

Use Ubuntu 22.04 or a compatible distribution with WebKitGTK 4.1 and GTK development packages installed:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target x86_64-unknown-linux-gnu --bundles deb,rpm,appimage
```

## 8. Linux ARM64

Run on a native ARM64 Linux runner with the matching WebKitGTK and GTK development packages:

```bash
rustup target add aarch64-unknown-linux-gnu
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target aarch64-unknown-linux-gnu --bundles deb,rpm
```

AppImage is intentionally not requested for Linux ARM64 because its dependency bundling is not reliable enough for this release.

## 9. Manual smoke test

1. Install the generated package.
2. Start Freeloader and complete or skip onboarding.
3. Add a direct HTTP(S) URL.
4. Confirm a `.part` file appears while downloading.
5. Confirm progress events update the row.
6. Pause, resume and cancel from the UI once those commands are exposed.
7. Close and relaunch with an interrupted `.part` file, then confirm recovery.
8. Confirm no localhost listener is opened with `netstat -ano` on Windows or `ss -ltnp` on Linux.
9. Open Settings, verify detected browsers, and verify Chromium points only to GitHub Releases.
10. Uninstall and confirm native-host registrations are removed while user data remains unless explicitly deleted.

## Known validation boundary

The repository can define and automate these builds, but a hosted source-editing session cannot truthfully certify a Windows or Linux artifact without executing on the corresponding runner. GitHub Actions is the authoritative place to produce and upload those artifacts; local smoke testing is still required before release.
