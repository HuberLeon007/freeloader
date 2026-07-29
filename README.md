# Freeloader

**A fast, private, local-first download manager for Windows and Linux.**

Freeloader is a GPL-3.0-or-later desktop download manager built with Rust, Tauri v2 and React. It has no account, cloud, advertisements, telemetry, tracking or shipped local HTTP server. Downloads are streamed directly to local `.part` files and atomically renamed after completion.

> **Current status:** early development. The repository now contains the Rust protocol, SQLite-backed streaming core, Tauri adapter, React GUI, first-run onboarding, Native Messaging foundations, browser extension packages and release workflow foundations. Treat target-platform artifacts as unverified until the matching CI job is green.

## Clone and build

### Windows x64

```powershell
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
rustup toolchain install stable
corepack enable
pnpm install
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --bundles nsis
```

### Windows ARM64

Install Visual Studio 2022 Desktop development with C++, the MSVC ARM64/ARM64EC tools and the Windows 11 SDK first, then run:

```powershell
rustup target add aarch64-pc-windows-msvc
pnpm install
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target aarch64-pc-windows-msvc --bundles nsis
```

### Linux

Install WebKitGTK 4.1 and GTK development packages for your distribution before running:

```bash
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
pnpm install
cargo test --workspace
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target x86_64-unknown-linux-gnu --bundles deb,rpm,appimage
```

Linux ARM64 packages require a native ARM64 runner or a fully configured cross-compilation toolchain:

```bash
rustup target add aarch64-unknown-linux-gnu
cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml --target aarch64-unknown-linux-gnu --bundles deb,rpm
```

## Development

```bash
pnpm --dir apps/desktop dev
```

For the native desktop shell, use `cargo tauri dev` after installing the current Tauri CLI. The production binary never starts an HTTP server. Test fixtures may use local servers only inside test processes.

## Browser extensions

Firefox and Edge may use their official stores after publication. Chromium browsers use the GitHub Releases ZIP only: download, extract to a stable directory, enable Developer Mode, select **Load unpacked**, and register the exact extension ID in the Native Messaging host manifest. The Chrome Web Store is intentionally not used.

## License

Freeloader is free software under the [GNU GPL-3.0-or-later](LICENSE). Distributed modified versions must preserve attribution, remain under GPL-3.0-or-later and provide corresponding source. See [development.md](docs/development.md) and [extensions.md](docs/extensions.md).
