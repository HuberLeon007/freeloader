# Freeloader worktree preview

## Reproduce artifacts

From this worktree:

```bash
pnpm install
```

This installs the workspace dependencies using the repository lockfile and runs the existing icon generation postinstall hook. No environment file is required for the desktop frontend preview.

## Run the server

From this worktree:

```bash
pnpm --filter freeloader-desktop dev -- --host 127.0.0.1
```

The Vite server uses the configured strict default port `1420` from `apps/desktop/vite.config.ts`. Open `http://127.0.0.1:1420/` in the Preview tab. This is the web UI preview; native Tauri commands are unavailable outside the desktop shell, so the app's safe fallbacks are expected in this mode.
