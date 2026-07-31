# Freeloader

**A fast, private, local-first download manager for Windows and Linux.**

## Quick start

**Use branch: `main`**

```bash
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
git switch main
cargo dev
```

`cargo dev` opens the native Tauri desktop window. The localhost address printed by Vite is an internal frontend server and should not be opened manually.

## Build the Windows setup exe

**Use branch: `main`**

```bash
cargo dev build
```

The NSIS installer is written to `target/release/bundle/nsis/`. For a clean GitHub Actions installer, push a version tag such as `v0.1.0` and download the `windows-x86_64-pc-windows-msvc` artifact from the Release workflow.

## First run

The onboarding is shown on first launch and again after this UI update. It asks only for the essentials: download folder, appearance, and optional browser handoff. The folder picker is native; if it is cancelled or unavailable, the path remains editable and the error explains what failed.
