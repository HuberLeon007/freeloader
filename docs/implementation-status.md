# Implementation status

The quiet instrument overhaul is implemented directly on a feature branch. It establishes shared local-first tokens, shell primitives, typed formatting, paste-many parsing and validation, adapter resolution with direct HTTP fallback, queue state snapshots, settings and density controls, diagnostics and undo surfaces, onboarding primitives, and Chromium and Firefox browser handoff surfaces.

Release gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check`, plus desktop typecheck, build, and Vitest. Deferred features remain explicitly deferred: segmentation, bandwidth limiting, tray, checksums, outbound update checks, and cookies or credentials.
