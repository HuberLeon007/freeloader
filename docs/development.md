# Development

## Prerequisites

### Windows

Install Rust stable, Node.js 22, pnpm 10, Visual Studio 2022 with Desktop development with C++, WebView2, and the Tauri CLI. For ARM64 also install the MSVC ARM64/ARM64EC tools and Windows 11 SDK.

### Linux

Use Ubuntu 22.04 or newer with `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, Rust stable, Node.js 22 and pnpm 10. ARM64 builds should run on a native ARM64 runner.

## Commands

```bash
pnpm install
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
pnpm --dir apps/desktop dev
```

The current desktop UI runs in a browser during `pnpm --dir apps/desktop dev`; Tauri commands require `pnpm tauri dev` after the Tauri CLI is installed. The production application never starts an HTTP server. Test fixtures may use local servers only inside test processes.

## Release builds

```powershell
pnpm --dir apps/desktop tauri build -- --target x86_64-pc-windows-msvc --bundles nsis
pnpm --dir apps/desktop tauri build -- --target aarch64-pc-windows-msvc --bundles nsis
```

```bash
pnpm --dir apps/desktop tauri build -- --target x86_64-unknown-linux-gnu --bundles deb,rpm,appimage
pnpm --dir apps/desktop tauri build -- --target aarch64-unknown-linux-gnu --bundles deb,rpm
```

ARM and Linux packaging require the native system libraries and runners described above. Do not claim an artifact is supported until it has been built and smoke-tested on the target architecture.
