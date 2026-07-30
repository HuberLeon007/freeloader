# Summary

What changes, and why. Link the issue or the requirement this addresses.

Closes #

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor with no behaviour change
- [ ] Documentation
- [ ] Build, CI or tooling

## Commit gate

Ran locally, before each commit:

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `pnpm --dir apps/desktop typecheck`

Platform they ran on:

Checks that could not run here, and why:

## Tests

- [ ] New or changed behaviour is covered by an executable test
- [ ] Universal invariants have a property-based test
- [ ] No mock or fixed return value exists only to make an assertion pass

What the tests actually verify:

## Conventions

- [ ] Commit subjects follow Conventional Commits with a crate or package scope
- [ ] Every commit is signed off for the DCO (`git commit -s`)
- [ ] One commit per completed unit of work; no collective commits
- [ ] History is linear; the branch is rebased, not merged
- [ ] New source files carry `SPDX-License-Identifier: GPL-3.0-or-later`

## Scope and privacy

- [ ] No DRM circumvention, and no paywall or login-wall bypass
- [ ] No telemetry, analytics or remote crash reporting added
- [ ] No listening socket or local HTTP server added to a shipped binary
- [ ] No new dependency that pulls in OpenSSL or `native-tls`
- [ ] New dependencies pass `cargo deny check`; any exception has an ADR under `docs/adr/`

## Notes for the reviewer

Anything worth knowing: a deliberate deviation, a follow-up left out on purpose, an area that needs a closer look.
