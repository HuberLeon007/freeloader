# Contributing

Freeloader uses GitHub Flow. Work on a short-lived branch such as `feat/ranged-resume` and open a pull request into `main`. Keep pull requests focused and explain the test-first sequence.

## Before opening a PR

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint
pnpm typecheck
pnpm test
```

Every commit must use Conventional Commits and include a Developer Certificate of Origin sign-off:

```text
git commit -s -m "feat(core): add ranged resume"
```

By contributing, you certify the DCO and agree that your contribution is licensed under GPL-3.0-or-later. No copyright assignment is requested.
