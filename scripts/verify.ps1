# SPDX-License-Identifier: GPL-3.0-or-later
$ErrorActionPreference = "Stop"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install --frozen-lockfile=false
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build

Write-Host "Freeloader portable checks passed. Native packaging still requires the target OS and architecture."
