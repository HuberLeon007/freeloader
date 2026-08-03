# Implementation status

The frontend overhaul is being implemented directly on a feature branch, without delegating code generation to an external coding agent. The first vertical slice establishes shared tokens, shell primitives, formatting helpers, paste-many parsing, adapter resolution seams, and queue snapshot persistence.

Verification is incremental. Release gates remain `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo xtask check`; frontend checks use the desktop package's typecheck, build, and Vitest scripts.

## Deferred to a later spec

Multi-connection segmentation, bandwidth limiting, tray integration, checksum verification, outbound update checks, and cookie or credential forwarding. The latter is explicitly refused. No UI or documentation implies deferred features exist.
