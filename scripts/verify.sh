#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm install --frozen-lockfile=false
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build

echo "Freeloader portable checks passed. Native packaging still requires the target OS and architecture."
