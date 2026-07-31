# Changelog

All notable changes to Freeloader are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Entries are grouped as Added, Changed, Deprecated, Removed, Fixed and Security. Until v0.1.0 is released, everything lands under Unreleased.

## [Unreleased]

### Added

- `CONTRIBUTING.md` with the DCO sign-off requirement, the Conventional Commits convention and the four commit-gate checks.
- `CODE_OF_CONDUCT.md`, `SECURITY.md` and this changelog.
- Issue templates, a pull request template, a Dependabot configuration for Cargo, npm and GitHub Actions, and `CODEOWNERS`.
- `docs/adr/0002-rustls-only.md` and `docs/adr/0006-dependency-licence-policy.md`, the two decisions `deny.toml` refers to.
- A `package.json` for `extensions/chromium` and `extensions/firefox`, so both are real pnpm packages.
- `.gitattributes`, so line endings are normalised for every clone: LF everywhere, CRLF for `ps1`, `bat`, `cmd`, `nsi` and `nsh`.

### Changed

- `docs/implementation-status.md` is valid Markdown again and names the features deferred to a later spec.
- `pnpm-workspace.yaml` lists only directories that contain a `package.json`; the non-existent `extensions/shared` entry is gone.
- `.gitignore` excludes `*.exe`, so build output cannot be committed by accident.
- `deny.toml` states the licence policy explicitly, bans `openssl`, `openssl-sys` and `native-tls`, and points at the two ADRs that explain both decisions.

## Notes

The initial release is not tagged yet. Target platforms are Windows x64, Windows ARM64, Linux x64 and Linux ARM64; treat an artifact as unsupported until its own build and smoke test have run on that platform, as described in [docs/verification.md](docs/verification.md).
