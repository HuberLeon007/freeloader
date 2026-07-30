# Contributing to Freeloader

Thanks for taking the time. Freeloader is free software under [GPL-3.0-or-later](LICENSE), and every contribution is expected to keep it that way.

Before you start, please read the [Code of Conduct](CODE_OF_CONDUCT.md) and, for anything security-related, [SECURITY.md](SECURITY.md).

## What belongs here, and what does not

Freeloader is a local-first HTTPS download manager for Windows and Linux. It has no account, no cloud service, no telemetry and no shipped local HTTP server.

Permanently out of scope, and not accepted as a contribution: DRM circumvention, bypassing paywalls or login walls, extraction from streaming sites, YouTube ripping, macOS support.

Features that are deferred rather than rejected are listed in [docs/implementation-status.md](docs/implementation-status.md).

## Setting up

```bash
git clone https://github.com/HuberLeon007/freeloader.git
cd freeloader
rustup toolchain install stable
corepack enable
pnpm install
```

Platform prerequisites (WebView2, Visual Studio C++ tools, WebKitGTK, GTK) are described in [docs/development.md](docs/development.md).

## The commit gate

Four checks must pass before every commit. Not before the pull request: before each individual commit.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir apps/desktop typecheck
```

If any of the four fails, the commit is held back until the cause is fixed. `scripts/verify.sh` and `scripts/verify.ps1` run the same sequence.

Two rules follow from this:

- One commit per completed unit of work. No collective commits that bundle several finished pieces of work.
- No commit that is green only in combination with the next one. Every commit stands on its own.

## Commit messages

Freeloader uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) with a crate or package scope.

```
feat(download-core): add ranged resume with ETag validation
fix(native-host): reject frames above the payload limit
chore(repo): clean tracked build output and add project health files
```

- Types in use: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`.
- The scope is the crate or package the change belongs to: `download-core`, `protocol`, `platform`, `native-host`, `desktop`, `frontend`, `extensions`, `installer`, `ci`, `repo`. Dependency updates use `deps` and `deps-dev`, which is what Dependabot produces.
- The subject is imperative and lower case, no trailing period.
- Breaking changes use `!` after the scope and explain the break in the body.

## Sign your work: the DCO

Every commit must carry a `Signed-off-by` line certifying the [Developer Certificate of Origin](https://developercertificate.org/). By signing off you state that you wrote the patch, or otherwise have the right to submit it under GPL-3.0-or-later.

```bash
git commit -s -m "fix(frontend): restore focus to the dialog trigger"
```

That produces:

```
Signed-off-by: Your Name <your.email@example.org>
```

Use your real name and a reachable address; the name must match your `user.name` and `user.email` git configuration. A commit without a sign-off is not merged. To repair the most recent commit:

```bash
git commit --amend -s --no-edit
```

For several commits, `git rebase --signoff <base>` adds the line to each of them.

## Branches and pull requests

- Work happens on a feature branch. Development for the current vertical slice lives on `feat/freeloader-full-implementation`.
- Publish your branch with `git push -u origin <branch>`.
- Changes reach `main` only through a pull request. There are no direct pushes to `main`.
- History stays linear. Rebase onto the target branch instead of merging it into yours.
- Fill in the [pull request template](.github/PULL_REQUEST_TEMPLATE.md), including which of the four checks you ran and on which platform.

If a check cannot run in your environment (for example a Windows ARM64 bundle on a Linux machine), say so explicitly in the pull request. An honest "not verified here" is useful; a claim that turns out to be untested is not.

## Tests

- New behaviour needs an executable test. Universal invariants get a property-based test; specific cases and edge conditions get unit tests. Both are valuable.
- Tests must exercise real logic. Do not add mocks or fixed return values whose only purpose is to make an assertion pass.
- The download engine is tested through its trait seams (`HttpClient`, `DownloadRepository`, `FileSystem`, `Clock`), so its tests run headless, offline and fast.
- `crates/protocol` is the single source of truth for the wire contract, validation and filename sanitisation. Do not add a second implementation of any of them elsewhere.

## Style

`.editorconfig` is authoritative: UTF-8, LF line endings, a final newline, two-space indentation, four spaces in Rust. `.gitattributes` keeps CRLF for `ps1`, `bat`, `cmd`, `nsi` and `nsh` files because their toolchains expect it.

New source files start with the licence marker in the comment syntax of the language:

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
```

Rust formatting is whatever `cargo fmt` produces. Do not hand-tune it.

## Dependencies

`cargo deny check` enforces the licence policy in [docs/adr/0006-dependency-licence-policy.md](docs/adr/0006-dependency-licence-policy.md). TLS is `rustls` only; nothing may pull in OpenSSL or `native-tls`, see [docs/adr/0002-rustls-only.md](docs/adr/0002-rustls-only.md). If you need an exception, write an ADR under `docs/adr/` first.

## Reporting bugs and proposing features

Use the [issue templates](.github/ISSUE_TEMPLATE). Security vulnerabilities do not belong in a public issue; follow [SECURITY.md](SECURITY.md).
