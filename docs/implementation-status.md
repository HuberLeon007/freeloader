# Implementation status

The frontend overhaul is being implemented directly on a feature branch, without delegating code generation to an external coding agent. The first vertical slice establishes shared tokens, shell primitives, formatting helpers, paste-many parsing, adapter resolution seams, and queue snapshot persistence.

Verification is incremental. The existing Rust gates remain the release gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo xtask check`. Frontend checks use the desktop package's typecheck, build, and Vitest scripts.

## Deferred to a later spec

- Multi-connection segmentation
- Bandwidth limiting
- Tray integration
- Checksum verification
- Outbound update checks
- Cookie and credential forwarding, which is explicitly refused

No UI or documentation implies that deferred features exist.
